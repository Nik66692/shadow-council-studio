use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};
use std::{fs, path::Path, process::Command};

use crate::{
    AppError, SourceDocument, read_canon_source_manifest, sha256_file, upsert_source_document,
};

const IMPORTER_VERSION: &str = "canon-docx-importer/1.0.0";
const DOCUMENT_XML_PART: &str = "word/document.xml";

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CanonImportRun {
    pub id: String,
    pub source_document_id: String,
    pub source_version: String,
    pub source_sha256: String,
    pub importer_version: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: String,
    pub raw_block_count: i64,
    pub draft_count: i64,
    pub warning_count: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CanonReviewDraft {
    pub id: String,
    pub raw_block_id: String,
    pub source_anchor: String,
    pub source_part: String,
    pub block_index: i64,
    pub block_kind: String,
    pub style_name: Option<String>,
    pub original_text: String,
    pub text_sha256: String,
    pub review_status: String,
    pub canonical_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CanonImportWarning {
    pub id: String,
    pub source_anchor: Option<String>,
    pub warning_code: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonImportReviewSnapshot {
    pub run: Option<CanonImportRun>,
    pub drafts: Vec<CanonReviewDraft>,
    pub warnings: Vec<CanonImportWarning>,
    pub imported_now: bool,
}

#[derive(Debug, Clone)]
struct ExtractedBlock {
    block_index: i64,
    block_kind: String,
    style_name: Option<String>,
    original_text: String,
}

#[derive(Debug, Clone)]
struct ExtractionWarning {
    block_index: Option<i64>,
    block_kind: Option<String>,
    code: String,
    message: String,
}

#[derive(Debug, Clone)]
struct ExtractionResult {
    blocks: Vec<ExtractedBlock>,
    warnings: Vec<ExtractionWarning>,
}

fn sha256_text(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn deterministic_id(prefix: &str, parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    let hash = hex::encode(digest.finalize());
    format!("{prefix}{}", &hash[..24])
}

fn xml_unescape(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn find_opening_tag(xml: &str, from: usize, tag: &str) -> Option<usize> {
    let needle = format!("<{tag}");
    let mut cursor = from;
    while let Some(relative) = xml[cursor..].find(&needle) {
        let start = cursor + relative;
        let next = xml.as_bytes().get(start + needle.len()).copied();
        if matches!(
            next,
            Some(b'>') | Some(b' ') | Some(b'\t') | Some(b'\r') | Some(b'\n')
        ) {
            return Some(start);
        }
        cursor = start + needle.len();
    }
    None
}

fn extract_text_nodes(fragment: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0;
    while let Some(start) = find_opening_tag(fragment, cursor, "w:t") {
        let Some(relative_open_end) = fragment[start..].find('>') else {
            break;
        };
        let content_start = start + relative_open_end + 1;
        let Some(relative_close) = fragment[content_start..].find("</w:t>") else {
            break;
        };
        let content_end = content_start + relative_close;
        output.push_str(&xml_unescape(&fragment[content_start..content_end]));
        cursor = content_end + "</w:t>".len();
    }
    output
}

fn xml_attribute(tag: &str, attribute: &str) -> Option<String> {
    let needle = format!("{attribute}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(xml_unescape(&tag[start..end]))
}

fn paragraph_style(fragment: &str) -> Option<String> {
    let start = find_opening_tag(fragment, 0, "w:pStyle")?;
    let end = fragment[start..].find('>')? + start;
    let tag = &fragment[start..=end];
    xml_attribute(tag, "w:val").or_else(|| xml_attribute(tag, "val"))
}

fn paragraph_kind(fragment: &str, style: Option<&str>) -> String {
    let normalized_style = style.unwrap_or_default().to_lowercase();
    if normalized_style.contains("heading") || normalized_style.contains("titolo") {
        "HEADING".into()
    } else if fragment.contains("<w:numPr") {
        "LIST_ITEM".into()
    } else {
        "PARAGRAPH".into()
    }
}

fn flatten_table_text(fragment: &str) -> String {
    let mut paragraphs = Vec::new();
    let mut cursor = 0;
    while let Some(start) = find_opening_tag(fragment, cursor, "w:p") {
        let Some(relative_end) = fragment[start..].find("</w:p>") else {
            break;
        };
        let end = start + relative_end + "</w:p>".len();
        let text = extract_text_nodes(&fragment[start..end]);
        if !text.trim().is_empty() {
            paragraphs.push(text);
        }
        cursor = end;
    }
    paragraphs.join("\n")
}

fn parse_document_xml(xml: &str) -> ExtractionResult {
    let mut blocks = Vec::new();
    let mut warnings = Vec::new();
    let mut cursor = xml.find("<w:body").unwrap_or(0);
    let mut block_index = 0_i64;

    loop {
        let paragraph_start = find_opening_tag(xml, cursor, "w:p");
        let table_start = find_opening_tag(xml, cursor, "w:tbl");
        let next = match (paragraph_start, table_start) {
            (None, None) => break,
            (Some(paragraph), None) => ("paragraph", paragraph),
            (None, Some(table)) => ("table", table),
            (Some(paragraph), Some(table)) if table < paragraph => ("table", table),
            (Some(paragraph), Some(_)) => ("paragraph", paragraph),
        };

        if next.0 == "table" {
            let Some(relative_end) = xml[next.1..].find("</w:tbl>") else {
                warnings.push(ExtractionWarning {
                    block_index: None,
                    block_kind: None,
                    code: "MALFORMED_TABLE_XML".into(),
                    message: "A DOCX table did not have a closing tag; import stopped at the malformed structure.".into(),
                });
                break;
            };
            let end = next.1 + relative_end + "</w:tbl>".len();
            let text = flatten_table_text(&xml[next.1..end]);
            if !text.trim().is_empty() {
                blocks.push(ExtractedBlock {
                    block_index,
                    block_kind: "TABLE_TEXT".into(),
                    style_name: None,
                    original_text: text,
                });
                warnings.push(ExtractionWarning {
                    block_index: Some(block_index),
                    block_kind: Some("TABLE_TEXT".into()),
                    code: "UNSUPPORTED_TABLE_STRUCTURE".into(),
                    message: "DOCX table structure was flattened into review text; cell boundaries are not canonical data.".into(),
                });
                block_index += 1;
            }
            cursor = end;
            continue;
        }

        let Some(relative_end) = xml[next.1..].find("</w:p>") else {
            warnings.push(ExtractionWarning {
                block_index: None,
                block_kind: None,
                code: "MALFORMED_PARAGRAPH_XML".into(),
                message: "A DOCX paragraph did not have a closing tag; import stopped at the malformed structure.".into(),
            });
            break;
        };
        let end = next.1 + relative_end + "</w:p>".len();
        let fragment = &xml[next.1..end];
        let text = extract_text_nodes(fragment);
        if !text.trim().is_empty() {
            let style = paragraph_style(fragment);
            blocks.push(ExtractedBlock {
                block_index,
                block_kind: paragraph_kind(fragment, style.as_deref()),
                style_name: style,
                original_text: text,
            });
            block_index += 1;
        }
        cursor = end;
    }

    for (needle, code, message) in [
        (
            "<w:altChunk",
            "UNSUPPORTED_ALT_CHUNK",
            "The DOCX contains an alternative content chunk that was not imported.",
        ),
        (
            "<w:txbxContent",
            "UNSUPPORTED_TEXT_BOX",
            "The DOCX contains text-box content that requires manual source review.",
        ),
        (
            "<w:object",
            "UNSUPPORTED_EMBEDDED_OBJECT",
            "The DOCX contains an embedded object that was not interpreted.",
        ),
    ] {
        if xml.contains(needle) {
            warnings.push(ExtractionWarning {
                block_index: None,
                block_kind: None,
                code: code.into(),
                message: message.into(),
            });
        }
    }

    if blocks.is_empty() {
        warnings.push(ExtractionWarning {
            block_index: None,
            block_kind: None,
            code: "EMPTY_DOCUMENT".into(),
            message: "No supported text blocks were extracted from word/document.xml.".into(),
        });
    }

    ExtractionResult { blocks, warnings }
}

#[cfg(target_os = "windows")]
fn extract_document_xml(source_path: &Path) -> Result<String, AppError> {
    let script = r#"
param([string]$docx)
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::OpenRead($docx)
try {
  $entry = $archive.GetEntry('word/document.xml')
  if ($null -eq $entry) { throw 'word/document.xml not found' }
  $reader = New-Object System.IO.StreamReader($entry.Open(), [System.Text.Encoding]::UTF8, $true)
  try { $reader.ReadToEnd() } finally { $reader.Dispose() }
} finally {
  $archive.Dispose()
}
"#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .arg(source_path)
        .output()?;
    if !output.status.success() {
        return Err(AppError::CanonManifest(format!(
            "PowerShell could not read {DOCUMENT_XML_PART}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| AppError::CanonManifest(format!("invalid UTF-8 in document.xml: {error}")))
}

#[cfg(not(target_os = "windows"))]
fn extract_document_xml(source_path: &Path) -> Result<String, AppError> {
    let output = Command::new("unzip")
        .arg("-p")
        .arg(source_path)
        .arg(DOCUMENT_XML_PART)
        .output()?;
    if !output.status.success() {
        return Err(AppError::CanonManifest(format!(
            "unzip could not read {DOCUMENT_XML_PART}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| AppError::CanonManifest(format!("invalid UTF-8 in document.xml: {error}")))
}

fn source_anchor(
    source_version: &str,
    source_sha256: &str,
    block_kind: &str,
    block_index: i64,
) -> String {
    format!(
        "sc://canon/{source_version}/{source_sha256}/{DOCUMENT_XML_PART}/{}/{block_index:06}",
        block_kind.to_lowercase()
    )
}

async fn existing_import_run(
    pool: &SqlitePool,
    source_sha256: &str,
) -> Result<Option<CanonImportRun>, AppError> {
    Ok(sqlx::query_as::<_, CanonImportRun>(
        "SELECT id,source_document_id,source_version,source_sha256,importer_version,status,started_at,completed_at,raw_block_count,draft_count,warning_count \
         FROM canon_import_runs WHERE source_sha256 = ? AND importer_version = ?",
    )
    .bind(source_sha256)
    .bind(IMPORTER_VERSION)
    .fetch_optional(pool)
    .await?)
}

async fn review_snapshot_for_run(
    pool: &SqlitePool,
    run: Option<CanonImportRun>,
    imported_now: bool,
) -> Result<CanonImportReviewSnapshot, AppError> {
    let Some(run) = run else {
        return Ok(CanonImportReviewSnapshot {
            run: None,
            drafts: Vec::new(),
            warnings: Vec::new(),
            imported_now,
        });
    };

    let drafts = sqlx::query_as::<_, CanonReviewDraft>(
        "SELECT d.id,d.raw_block_id,d.source_anchor,b.source_part,b.block_index,b.block_kind,b.style_name,d.original_text,b.text_sha256,d.review_status,d.canonical_status \
         FROM canon_normalized_drafts d \
         JOIN canon_raw_blocks b ON b.id = d.raw_block_id \
         WHERE d.import_run_id = ? ORDER BY b.block_index",
    )
    .bind(&run.id)
    .fetch_all(pool)
    .await?;

    let warnings = sqlx::query_as::<_, CanonImportWarning>(
        "SELECT id,source_anchor,warning_code,message,created_at \
         FROM canon_import_warnings WHERE import_run_id = ? ORDER BY warning_code,id",
    )
    .bind(&run.id)
    .fetch_all(pool)
    .await?;

    Ok(CanonImportReviewSnapshot {
        run: Some(run),
        drafts,
        warnings,
        imported_now,
    })
}

async fn persist_import(
    pool: &SqlitePool,
    manifest: &crate::CanonSourceManifest,
    source_hash: &str,
    extraction: &ExtractionResult,
) -> Result<CanonImportRun, AppError> {
    if let Some(existing) = existing_import_run(pool, source_hash).await? {
        return Ok(existing);
    }

    let now = chrono::Utc::now().to_rfc3339();
    let source_document_id = format!("source-of-truth-v{}", manifest.current_version);
    let run_id = deterministic_id(
        "canon-import-",
        &[source_hash, IMPORTER_VERSION, &manifest.current_version],
    );

    let mut transaction = pool.begin().await?;
    sqlx::query(
        "INSERT INTO canon_import_runs \
         (id,source_document_id,source_version,source_sha256,importer_version,status,started_at,completed_at,raw_block_count,draft_count,warning_count) \
         VALUES (?,?,?,?,?,'COMPLETED_PENDING_REVIEW',?,?,?,?,?)",
    )
    .bind(&run_id)
    .bind(&source_document_id)
    .bind(&manifest.current_version)
    .bind(source_hash)
    .bind(IMPORTER_VERSION)
    .bind(&now)
    .bind(&now)
    .bind(extraction.blocks.len() as i64)
    .bind(extraction.blocks.len() as i64)
    .bind(extraction.warnings.len() as i64)
    .execute(&mut *transaction)
    .await?;

    for block in &extraction.blocks {
        let anchor = source_anchor(
            &manifest.current_version,
            source_hash,
            &block.block_kind,
            block.block_index,
        );
        let raw_block_id = deterministic_id("canon-block-", &[&run_id, &anchor]);
        let draft_id = deterministic_id("canon-draft-", &[&raw_block_id]);
        let text_hash = sha256_text(&block.original_text);

        sqlx::query(
            "INSERT INTO canon_raw_blocks \
             (id,import_run_id,source_anchor,source_part,block_index,block_kind,style_name,original_text,text_sha256) \
             VALUES (?,?,?,?,?,?,?,?,?)",
        )
        .bind(&raw_block_id)
        .bind(&run_id)
        .bind(&anchor)
        .bind(DOCUMENT_XML_PART)
        .bind(block.block_index)
        .bind(&block.block_kind)
        .bind(block.style_name.as_deref())
        .bind(&block.original_text)
        .bind(&text_hash)
        .execute(&mut *transaction)
        .await?;

        sqlx::query(
            "INSERT INTO canon_normalized_drafts \
             (id,import_run_id,raw_block_id,source_anchor,draft_kind,original_text,review_status,canonical_status,created_at) \
             VALUES (?,?,?,?,?,?,'PENDING_HUMAN_REVIEW',NULL,?)",
        )
        .bind(&draft_id)
        .bind(&run_id)
        .bind(&raw_block_id)
        .bind(&anchor)
        .bind(&block.block_kind)
        .bind(&block.original_text)
        .bind(&now)
        .execute(&mut *transaction)
        .await?;
    }

    for warning in &extraction.warnings {
        let warning_anchor = match (warning.block_index, warning.block_kind.as_deref()) {
            (Some(index), Some(kind)) => Some(source_anchor(
                &manifest.current_version,
                source_hash,
                kind,
                index,
            )),
            _ => None,
        };
        let anchor_part = warning_anchor.as_deref().unwrap_or("document");
        let warning_id = deterministic_id(
            "canon-warning-",
            &[&run_id, &warning.code, anchor_part, &warning.message],
        );
        sqlx::query(
            "INSERT INTO canon_import_warnings \
             (id,import_run_id,source_anchor,warning_code,message,created_at) VALUES (?,?,?,?,?,?)",
        )
        .bind(&warning_id)
        .bind(&run_id)
        .bind(warning_anchor.as_deref())
        .bind(&warning.code)
        .bind(&warning.message)
        .bind(&now)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    existing_import_run(pool, source_hash)
        .await?
        .ok_or_else(|| AppError::CanonManifest("import run was not persisted".into()))
}

pub async fn get_latest_review(pool: &SqlitePool) -> Result<CanonImportReviewSnapshot, AppError> {
    let run = sqlx::query_as::<_, CanonImportRun>(
        "SELECT id,source_document_id,source_version,source_sha256,importer_version,status,started_at,completed_at,raw_block_count,draft_count,warning_count \
         FROM canon_import_runs ORDER BY completed_at DESC,id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    review_snapshot_for_run(pool, run, false).await
}

pub async fn import_source(
    pool: &SqlitePool,
    root: &Path,
) -> Result<CanonImportReviewSnapshot, AppError> {
    let manifest = read_canon_source_manifest(root)?;
    let source_path = root.join(&manifest.current_source);
    if !source_path.exists() {
        return Err(AppError::CanonManifest(format!(
            "canonical source referenced by manifest is missing: {}",
            manifest.current_source
        )));
    }
    let source_hash = sha256_file(&source_path)?;
    if source_hash != manifest.sha256 {
        return Err(AppError::CanonManifest(format!(
            "canonical source hash mismatch for {}: expected {}, got {}",
            manifest.current_source, manifest.sha256, source_hash
        )));
    }

    let source_document = SourceDocument {
        id: format!("source-of-truth-v{}", manifest.current_version),
        title: "Shadow Council Source of Truth".into(),
        version: manifest.current_version.clone(),
        authority_rank: 1,
        original_path: manifest.current_source.clone(),
        sha256: Some(source_hash.clone()),
        imported_at: chrono::Utc::now().to_rfc3339(),
        immutable: 1,
        notes: Some(
            "Phase 1 stores import evidence and review drafts only; no canonical status is inferred."
                .into(),
        ),
    };
    upsert_source_document(pool, &source_document).await?;

    if let Some(existing) = existing_import_run(pool, &source_hash).await? {
        return review_snapshot_for_run(pool, Some(existing), false).await;
    }

    let document_xml = extract_document_xml(&source_path)?;
    let extraction = parse_document_xml(&document_xml);
    let run = persist_import(pool, &manifest, &source_hash, &extraction).await?;
    review_snapshot_for_run(pool, Some(run), true).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connect, run_migrations};
    use tempfile::tempdir;

    fn test_manifest() -> crate::CanonSourceManifest {
        crate::CanonSourceManifest {
            schema_version: 1,
            current_version: "1.3".into(),
            current_source: "docs/canon/source/v1.3/source.docx".into(),
            status: "alpha-provisional".into(),
            approved_by: "Niccolò".into(),
            approved_at: "2026-07-17".into(),
            sha256: "a".repeat(64),
        }
    }

    #[test]
    fn parser_preserves_text_and_warns_when_tables_are_flattened() {
        let xml = r#"
        <w:document><w:body>
          <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Titolo &amp; prova</w:t></w:r></w:p>
          <w:p><w:pPr><w:numPr/></w:pPr><w:r><w:t>Voce elenco</w:t></w:r></w:p>
          <w:tbl><w:tr><w:tc><w:p><w:r><w:t>Cella A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Cella B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
        </w:body></w:document>
        "#;
        let result = parse_document_xml(xml);
        assert_eq!(result.blocks.len(), 3);
        assert_eq!(result.blocks[0].block_kind, "HEADING");
        assert_eq!(result.blocks[0].original_text, "Titolo & prova");
        assert_eq!(result.blocks[1].block_kind, "LIST_ITEM");
        assert_eq!(result.blocks[2].original_text, "Cella A\nCella B");
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.code == "UNSUPPORTED_TABLE_STRUCTURE")
        );
    }

    #[tokio::test]
    async fn persistence_is_idempotent_and_never_assigns_canon_status() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let manifest = test_manifest();
        let document = SourceDocument {
            id: "source-of-truth-v1.3".into(),
            title: "Shadow Council Source of Truth".into(),
            version: "1.3".into(),
            authority_rank: 1,
            original_path: manifest.current_source.clone(),
            sha256: Some(manifest.sha256.clone()),
            imported_at: chrono::Utc::now().to_rfc3339(),
            immutable: 1,
            notes: None,
        };
        upsert_source_document(&pool, &document).await.unwrap();
        let extraction = ExtractionResult {
            blocks: vec![ExtractedBlock {
                block_index: 0,
                block_kind: "HEADING".into(),
                style_name: Some("Heading1".into()),
                original_text: "Testo originale".into(),
            }],
            warnings: Vec::new(),
        };

        let first = persist_import(&pool, &manifest, &manifest.sha256, &extraction)
            .await
            .unwrap();
        let second = persist_import(&pool, &manifest, &manifest.sha256, &extraction)
            .await
            .unwrap();
        assert_eq!(first.id, second.id);

        let snapshot = get_latest_review(&pool).await.unwrap();
        assert_eq!(snapshot.drafts.len(), 1);
        assert_eq!(snapshot.drafts[0].review_status, "PENDING_HUMAN_REVIEW");
        assert_eq!(snapshot.drafts[0].canonical_status, None);
        assert_eq!(snapshot.drafts[0].original_text, "Testo originale");
    }

    #[tokio::test]
    async fn hash_mismatch_stops_before_docx_parsing() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let root = tempdir().unwrap();
        let source_path = "docs/canon/source/v1.3/source.docx";
        let source_file = root.path().join(source_path);
        fs::create_dir_all(source_file.parent().unwrap()).unwrap();
        fs::write(&source_file, "not a docx").unwrap();
        let manifest_path = root.path().join(crate::SOURCE_MANIFEST_RELATIVE_PATH);
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(
            manifest_path,
            format!(
                r#"{{"schemaVersion":1,"currentVersion":"1.3","currentSource":"{source_path}","status":"alpha-provisional","approvedBy":"Niccolò","approvedAt":"2026-07-17","sha256":"{}"}}"#,
                "b".repeat(64)
            ),
        )
        .unwrap();

        let error = import_source(&pool, root.path()).await.unwrap_err();
        assert!(error.to_string().contains("hash mismatch"));
    }

    #[test]
    fn source_anchors_include_version_hash_part_kind_and_index() {
        let anchor = source_anchor("1.3", &"a".repeat(64), "HEADING", 12);
        assert!(anchor.contains("/1.3/"));
        assert!(anchor.contains(&"a".repeat(64)));
        assert!(anchor.contains("/word/document.xml/"));
        assert!(anchor.ends_with("/heading/000012"));
    }
}

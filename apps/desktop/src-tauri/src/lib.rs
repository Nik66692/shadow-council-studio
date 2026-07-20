use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;
use thiserror::Error;

const SOURCE_MANIFEST_RELATIVE_PATH: &str = "docs/canon/source/manifest.json";

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("filesystem error")]
    Io(#[from] std::io::Error),
    #[error("canonical source manifest error: {0}")]
    CanonManifest(String),
}
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SourceDocument {
    pub id: String,
    pub title: String,
    pub version: String,
    pub authority_rank: i64,
    pub original_path: String,
    pub sha256: Option<String>,
    pub imported_at: String,
    pub immutable: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonSourceManifest {
    schema_version: u64,
    current_version: String,
    current_source: String,
    status: String,
    approved_by: String,
    approved_at: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceOfTruthStatus {
    exists: bool,
    filename: String,
    sha256: Option<String>,
    canon_version: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    project_name: String,
    development_stage: String,
    database_connected: bool,
    migrations_applied: bool,
    source_of_truth: SourceOfTruthStatus,
    modules_implemented: Vec<String>,
    next_recommended_phase: String,
    diagnostics: Vec<String>,
}

pub async fn connect(database_url: &str) -> Result<SqlitePool, AppError> {
    Ok(SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(sqlx::Error::from)?;
    Ok(())
}

pub async fn upsert_source_document(
    pool: &SqlitePool,
    doc: &SourceDocument,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO source_documents (id,title,version,authority_rank,original_path,sha256,imported_at,immutable,notes) \
         VALUES (?,?,?,?,?,?,?,?,?) \
         ON CONFLICT(id) DO UPDATE SET title=excluded.title, version=excluded.version, authority_rank=excluded.authority_rank, \
         original_path=excluded.original_path, sha256=excluded.sha256, imported_at=excluded.imported_at, immutable=excluded.immutable, notes=excluded.notes",
    )
    .bind(&doc.id)
    .bind(&doc.title)
    .bind(&doc.version)
    .bind(doc.authority_rank)
    .bind(&doc.original_path)
    .bind(&doc.sha256)
    .bind(&doc.imported_at)
    .bind(doc.immutable)
    .bind(&doc.notes)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_source_document(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<SourceDocument>, AppError> {
    Ok(
        sqlx::query_as::<_, SourceDocument>("SELECT * FROM source_documents WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let bytes = fs::read(path)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn read_canon_source_manifest(root: &Path) -> Result<CanonSourceManifest, AppError> {
    let manifest_path = root.join(SOURCE_MANIFEST_RELATIVE_PATH);
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::CanonManifest(format!(
                "canonical source manifest is missing: {SOURCE_MANIFEST_RELATIVE_PATH}"
            ))
        } else {
            AppError::Io(error)
        }
    })?;
    let manifest: CanonSourceManifest = serde_json::from_str(&manifest_text)
        .map_err(|error| AppError::CanonManifest(format!("invalid manifest JSON: {error}")))?;
    if manifest.schema_version != 1
        || manifest.current_version.is_empty()
        || manifest.current_source.is_empty()
        || manifest.status.is_empty()
        || manifest.approved_by.is_empty()
        || manifest.approved_at.is_empty()
        || manifest.sha256.len() != 64
    {
        return Err(AppError::CanonManifest(
            "canonical source manifest is invalid or incomplete".into(),
        ));
    }
    Ok(manifest)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub async fn build_health(pool: &SqlitePool, root: &Path) -> Result<HealthStatus, AppError> {
    let mut diagnostics = vec![];
    let manifest = match read_canon_source_manifest(root) {
        Ok(manifest) => manifest,
        Err(error) => {
            diagnostics.push(error.to_string());
            diagnostics
                .push("canonical source unavailable; no canonical content was inferred".into());
            return Ok(HealthStatus {
                project_name: "Shadow Council Studio".into(),
                development_stage: "Foundation".into(),
                database_connected: true,
                migrations_applied: true,
                source_of_truth: SourceOfTruthStatus {
                    exists: false,
                    filename: SOURCE_MANIFEST_RELATIVE_PATH.into(),
                    sha256: None,
                    canon_version: None,
                },
                modules_implemented: vec![
                    "Dashboard".into(),
                    "System Status".into(),
                    "SQLite migrations".into(),
                    "Source metadata registry".into(),
                ],
                next_recommended_phase: "Phase 1: canonical data model and deterministic import"
                    .into(),
                diagnostics,
            });
        }
    };

    let source = root.join(&manifest.current_source);
    if !source.exists() {
        diagnostics.push(format!(
            "canonical source referenced by manifest is missing: {}",
            manifest.current_source
        ));
        diagnostics.push("canonical source unavailable; no canonical content was inferred".into());
        return Ok(HealthStatus {
            project_name: "Shadow Council Studio".into(),
            development_stage: "Foundation".into(),
            database_connected: true,
            migrations_applied: true,
            source_of_truth: SourceOfTruthStatus {
                exists: false,
                filename: manifest.current_source,
                sha256: None,
                canon_version: Some(manifest.current_version),
            },
            modules_implemented: vec![
                "Dashboard".into(),
                "System Status".into(),
                "SQLite migrations".into(),
                "Source metadata registry".into(),
            ],
            next_recommended_phase: "Phase 1: canonical data model and deterministic import".into(),
            diagnostics,
        });
    }

    let hash = sha256_file(&source)?;
    if hash != manifest.sha256 {
        diagnostics.push(format!(
            "canonical source hash mismatch for {}: expected {}, got {}",
            manifest.current_source, manifest.sha256, hash
        ));
        diagnostics.push("canonical source unavailable; no canonical content was inferred".into());
        return Ok(HealthStatus {
            project_name: "Shadow Council Studio".into(),
            development_stage: "Foundation".into(),
            database_connected: true,
            migrations_applied: true,
            source_of_truth: SourceOfTruthStatus {
                exists: false,
                filename: manifest.current_source,
                sha256: Some(hash),
                canon_version: Some(manifest.current_version),
            },
            modules_implemented: vec![
                "Dashboard".into(),
                "System Status".into(),
                "SQLite migrations".into(),
                "Source metadata registry".into(),
            ],
            next_recommended_phase: "Phase 1: canonical data model and deterministic import".into(),
            diagnostics,
        });
    }

    let doc = SourceDocument {
        id: format!("source-of-truth-v{}", manifest.current_version),
        title: "Shadow Council Source of Truth".into(),
        version: manifest.current_version.clone(),
        authority_rank: 1,
        original_path: manifest.current_source.clone(),
        sha256: Some(hash.clone()),
        imported_at: chrono::Utc::now().to_rfc3339(),
        immutable: 1,
        notes: Some(format!(
            "Immutable primary game-design source selected by {SOURCE_MANIFEST_RELATIVE_PATH}; semantic extraction not performed in Sprint 0. Status: {}. Approved by {} on {}.",
            manifest.status, manifest.approved_by, manifest.approved_at
        )),
    };
    upsert_source_document(pool, &doc).await?;
    diagnostics.push("Canonical source manifest verified; metadata registered only.".into());

    Ok(HealthStatus {
        project_name: "Shadow Council Studio".into(),
        development_stage: "Foundation".into(),
        database_connected: true,
        migrations_applied: true,
        source_of_truth: SourceOfTruthStatus {
            exists: true,
            filename: manifest.current_source,
            sha256: Some(hash),
            canon_version: Some(manifest.current_version),
        },
        modules_implemented: vec![
            "Dashboard".into(),
            "System Status".into(),
            "SQLite migrations".into(),
            "Source metadata registry".into(),
        ],
        next_recommended_phase: "Phase 1: canonical data model and deterministic import".into(),
        diagnostics,
    })
}

#[tauri::command]
async fn get_system_health(app: tauri::AppHandle) -> Result<HealthStatus, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("shadow-council-studio"));
    fs::create_dir_all(&data_dir)?;
    let db_path = data_dir.join("shadow-council-studio.sqlite");
    let pool = connect(&format!("sqlite://{}?mode=rwc", db_path.display())).await?;
    run_migrations(&pool).await?;
    build_health(&pool, &repo_root()).await
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_system_health])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_manifest(root: &Path, source_path: &str, sha256: &str) {
        let manifest_path = root.join(SOURCE_MANIFEST_RELATIVE_PATH);
        fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        fs::write(
            manifest_path,
            format!(
                r#"{{"schemaVersion":1,"currentVersion":"1.3","currentSource":"{source_path}","status":"alpha-provisional","approvedBy":"Niccolò","approvedAt":"2026-07-17","sha256":"{sha256}"}}"#
            ),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn repository_insert_read_source_document() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let doc = SourceDocument {
            id: "doc".into(),
            title: "Title".into(),
            version: "v1".into(),
            authority_rank: 1,
            original_path: "x.docx".into(),
            sha256: Some("a".repeat(64)),
            imported_at: "2026-07-20T00:00:00Z".into(),
            immutable: 1,
            notes: None,
        };
        upsert_source_document(&pool, &doc).await.unwrap();
        assert_eq!(
            get_source_document(&pool, "doc")
                .await
                .unwrap()
                .unwrap()
                .sha256,
            doc.sha256
        );
    }

    #[tokio::test]
    async fn service_health_handles_missing_manifest() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let dir = tempdir().unwrap();
        let health = build_health(&pool, dir.path()).await.unwrap();
        assert!(!health.source_of_truth.exists);
        assert!(health.diagnostics[0].contains("manifest is missing"));
    }

    #[tokio::test]
    async fn service_health_registers_manifest_source_metadata() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let dir = tempdir().unwrap();
        let source_path = "docs/canon/source/v1.3/Shadow_Council_Source_of_Truth_v1.3.docx";
        let source_file = dir.path().join(source_path);
        fs::create_dir_all(source_file.parent().unwrap()).unwrap();
        fs::write(&source_file, "canon").unwrap();
        let hash = sha256_file(&source_file).unwrap();
        write_manifest(dir.path(), source_path, &hash);

        let health = build_health(&pool, dir.path()).await.unwrap();
        assert!(health.source_of_truth.exists);
        assert_eq!(health.source_of_truth.canon_version, Some("1.3".into()));
        let doc = get_source_document(&pool, "source-of-truth-v1.3")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(doc.original_path, source_path);
        assert_eq!(doc.sha256, Some(hash));
    }

    #[tokio::test]
    async fn service_health_reports_hash_mismatch_without_registering_source() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let dir = tempdir().unwrap();
        let source_path = "docs/canon/source/v1.3/Shadow_Council_Source_of_Truth_v1.3.docx";
        let source_file = dir.path().join(source_path);
        fs::create_dir_all(source_file.parent().unwrap()).unwrap();
        fs::write(&source_file, "canon").unwrap();
        write_manifest(dir.path(), source_path, &"b".repeat(64));

        let health = build_health(&pool, dir.path()).await.unwrap();
        assert!(!health.source_of_truth.exists);
        assert!(health.diagnostics[0].contains("hash mismatch"));
        assert!(
            get_source_document(&pool, "source-of-truth-v1.3")
                .await
                .unwrap()
                .is_none()
        );
    }
}

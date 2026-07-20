use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, QueryBuilder, Row, Sqlite, SqlitePool};
use std::collections::HashSet;

use crate::AppError;

const MAX_REVIEW_DRAFTS: usize = 50;
const MAX_TITLE_LENGTH: usize = 200;
const MAX_TEXT_LENGTH: usize = 20_000;
const MAX_RATIONALE_LENGTH: usize = 2_000;
const MAX_REVIEWER_LENGTH: usize = 100;

const ENTRY_KINDS: [&str; 10] = [
    "RULE",
    "MECHANIC",
    "DEFINITION",
    "COMPONENT",
    "PROCEDURE",
    "DECKBUILDING",
    "VISUAL_SPEC",
    "OPEN_POINT",
    "RISK",
    "OTHER",
];

const CANONICAL_STATUSES: [&str; 7] = [
    "CANONICO",
    "ALPHA_DA_TESTARE",
    "IPOTESI_LINEA_GUIDA",
    "MAYBE",
    "RISCHIO",
    "SCARTATO_SUPERATO",
    "PUNTO_APERTO",
];

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CanonReviewDraftItem {
    pub id: String,
    pub import_run_id: String,
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
pub struct CanonEntrySource {
    pub draft_id: String,
    pub source_order: i64,
    pub source_anchor: String,
    pub block_index: i64,
    pub block_kind: String,
    pub original_text: String,
    pub text_sha256: String,
}

#[derive(Debug, Clone, FromRow)]
struct CanonEntryRow {
    id: String,
    title: String,
    entry_kind: String,
    canonical_status: String,
    normalized_text: String,
    lifecycle_status: String,
    approved_by: String,
    approved_at: String,
    rationale: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonEntry {
    pub id: String,
    pub title: String,
    pub entry_kind: String,
    pub canonical_status: String,
    pub normalized_text: String,
    pub lifecycle_status: String,
    pub approved_by: String,
    pub approved_at: String,
    pub rationale: String,
    pub created_at: String,
    pub updated_at: String,
    pub sources: Vec<CanonEntrySource>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CanonReviewDecision {
    pub id: String,
    pub decision_type: String,
    pub draft_id: String,
    pub entry_id: Option<String>,
    pub reviewer: String,
    pub rationale: String,
    pub decided_at: String,
    pub previous_review_status: String,
    pub resulting_review_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonReviewSummary {
    pub pending_count: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
    pub entry_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonReviewWorkspace {
    pub summary: CanonReviewSummary,
    pub drafts: Vec<CanonReviewDraftItem>,
    pub entries: Vec<CanonEntry>,
    pub recent_decisions: Vec<CanonReviewDecision>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveCanonDraftsRequest {
    pub draft_ids: Vec<String>,
    pub title: String,
    pub entry_kind: String,
    pub canonical_status: String,
    pub normalized_text: String,
    pub reviewer: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectCanonDraftsRequest {
    pub draft_ids: Vec<String>,
    pub reviewer: String,
    pub rationale: String,
}

fn review_error(message: impl Into<String>) -> AppError {
    AppError::CanonReview(message.into())
}

fn normalized_nonempty(value: &str, field: &str, max_length: usize) -> Result<String, AppError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(review_error(format!("{field} is required")));
    }
    if normalized.chars().count() > max_length {
        return Err(review_error(format!(
            "{field} exceeds the maximum length of {max_length} characters"
        )));
    }
    Ok(normalized.to_owned())
}

fn validate_draft_ids(ids: &[String]) -> Result<Vec<String>, AppError> {
    if ids.is_empty() {
        return Err(review_error("select at least one draft"));
    }
    if ids.len() > MAX_REVIEW_DRAFTS {
        return Err(review_error(format!(
            "a single review may contain at most {MAX_REVIEW_DRAFTS} drafts"
        )));
    }
    let mut unique = HashSet::with_capacity(ids.len());
    let mut validated = Vec::with_capacity(ids.len());
    for id in ids {
        let trimmed = id.trim();
        if trimmed.is_empty() || !unique.insert(trimmed.to_owned()) {
            return Err(review_error("draft identifiers must be unique and non-empty"));
        }
        validated.push(trimmed.to_owned());
    }
    Ok(validated)
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

async fn load_drafts_by_ids(
    pool: &SqlitePool,
    draft_ids: &[String],
) -> Result<Vec<CanonReviewDraftItem>, AppError> {
    let mut builder = QueryBuilder::<Sqlite>::new(
        "SELECT d.id,d.import_run_id,d.raw_block_id,d.source_anchor,b.source_part,b.block_index,b.block_kind,b.style_name,d.original_text,b.text_sha256,d.review_status,d.canonical_status \
         FROM canon_normalized_drafts d \
         JOIN canon_raw_blocks b ON b.id = d.raw_block_id \
         WHERE d.id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for draft_id in draft_ids {
            separated.push_bind(draft_id);
        }
    }
    builder.push(") ORDER BY b.block_index,d.id");
    Ok(builder
        .build_query_as::<CanonReviewDraftItem>()
        .fetch_all(pool)
        .await?)
}

fn validate_reviewable_drafts(
    requested_ids: &[String],
    drafts: &[CanonReviewDraftItem],
) -> Result<(), AppError> {
    if drafts.len() != requested_ids.len() {
        return Err(review_error("one or more selected drafts do not exist"));
    }
    if drafts
        .iter()
        .any(|draft| draft.review_status != "PENDING_HUMAN_REVIEW")
    {
        return Err(review_error(
            "one or more selected drafts have already been reviewed",
        ));
    }
    let import_runs: HashSet<&str> = drafts
        .iter()
        .map(|draft| draft.import_run_id.as_str())
        .collect();
    if import_runs.len() != 1 {
        return Err(review_error(
            "drafts from different import runs cannot be merged into one canon entry",
        ));
    }
    Ok(())
}

async fn load_entries(pool: &SqlitePool) -> Result<Vec<CanonEntry>, AppError> {
    let rows = sqlx::query_as::<_, CanonEntryRow>(
        "SELECT id,title,entry_kind,canonical_status,normalized_text,lifecycle_status,approved_by,approved_at,rationale,created_at,updated_at \
         FROM canon_entries ORDER BY approved_at DESC,id DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let sources = sqlx::query_as::<_, CanonEntrySource>(
            "SELECT s.draft_id,s.source_order,d.source_anchor,b.block_index,b.block_kind,d.original_text,b.text_sha256 \
             FROM canon_entry_sources s \
             JOIN canon_normalized_drafts d ON d.id = s.draft_id \
             JOIN canon_raw_blocks b ON b.id = d.raw_block_id \
             WHERE s.entry_id = ? ORDER BY s.source_order",
        )
        .bind(&row.id)
        .fetch_all(pool)
        .await?;
        entries.push(CanonEntry {
            id: row.id,
            title: row.title,
            entry_kind: row.entry_kind,
            canonical_status: row.canonical_status,
            normalized_text: row.normalized_text,
            lifecycle_status: row.lifecycle_status,
            approved_by: row.approved_by,
            approved_at: row.approved_at,
            rationale: row.rationale,
            created_at: row.created_at,
            updated_at: row.updated_at,
            sources,
        });
    }
    Ok(entries)
}

pub async fn get_canon_review_workspace(
    pool: &SqlitePool,
) -> Result<CanonReviewWorkspace, AppError> {
    let counts = sqlx::query(
        "SELECT \
           COALESCE(SUM(CASE WHEN review_status = 'PENDING_HUMAN_REVIEW' THEN 1 ELSE 0 END),0) AS pending_count, \
           COALESCE(SUM(CASE WHEN review_status IN ('APPROVED','MERGED_INTO_ENTRY') THEN 1 ELSE 0 END),0) AS approved_count, \
           COALESCE(SUM(CASE WHEN review_status = 'REJECTED' THEN 1 ELSE 0 END),0) AS rejected_count \
         FROM canon_normalized_drafts",
    )
    .fetch_one(pool)
    .await?;
    let entry_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM canon_entries")
        .fetch_one(pool)
        .await?;
    let summary = CanonReviewSummary {
        pending_count: counts.try_get("pending_count")?,
        approved_count: counts.try_get("approved_count")?,
        rejected_count: counts.try_get("rejected_count")?,
        entry_count,
    };

    let drafts = sqlx::query_as::<_, CanonReviewDraftItem>(
        "SELECT d.id,d.import_run_id,d.raw_block_id,d.source_anchor,b.source_part,b.block_index,b.block_kind,b.style_name,d.original_text,b.text_sha256,d.review_status,d.canonical_status \
         FROM canon_normalized_drafts d \
         JOIN canon_raw_blocks b ON b.id = d.raw_block_id \
         ORDER BY b.block_index,d.id",
    )
    .fetch_all(pool)
    .await?;

    let recent_decisions = sqlx::query_as::<_, CanonReviewDecision>(
        "SELECT id,decision_type,draft_id,entry_id,reviewer,rationale,decided_at,previous_review_status,resulting_review_status \
         FROM canon_review_decisions ORDER BY decided_at DESC,id DESC LIMIT 100",
    )
    .fetch_all(pool)
    .await?;

    Ok(CanonReviewWorkspace {
        summary,
        drafts,
        entries: load_entries(pool).await?,
        recent_decisions,
    })
}

pub async fn approve_canon_drafts(
    pool: &SqlitePool,
    request: ApproveCanonDraftsRequest,
) -> Result<CanonReviewWorkspace, AppError> {
    let draft_ids = validate_draft_ids(&request.draft_ids)?;
    let title = normalized_nonempty(&request.title, "title", MAX_TITLE_LENGTH)?;
    let normalized_text = normalized_nonempty(
        &request.normalized_text,
        "normalized text",
        MAX_TEXT_LENGTH,
    )?;
    let reviewer = normalized_nonempty(&request.reviewer, "reviewer", MAX_REVIEWER_LENGTH)?;
    let rationale = normalized_nonempty(
        &request.rationale,
        "rationale",
        MAX_RATIONALE_LENGTH,
    )?;
    if !ENTRY_KINDS.contains(&request.entry_kind.as_str()) {
        return Err(review_error("unsupported canon entry kind"));
    }
    if !CANONICAL_STATUSES.contains(&request.canonical_status.as_str()) {
        return Err(review_error("unsupported canonical status"));
    }

    let drafts = load_drafts_by_ids(pool, &draft_ids).await?;
    validate_reviewable_drafts(&draft_ids, &drafts)?;
    let now = Utc::now().to_rfc3339();
    let joined_ids = drafts
        .iter()
        .map(|draft| draft.id.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let entry_id = deterministic_id(
        "canon-entry-",
        &[&joined_ids, &title, &normalized_text, &now],
    );
    let resulting_status = if drafts.len() == 1 {
        "APPROVED"
    } else {
        "MERGED_INTO_ENTRY"
    };

    let mut transaction = pool.begin().await?;
    sqlx::query(
        "INSERT INTO canon_entries \
         (id,title,entry_kind,canonical_status,normalized_text,lifecycle_status,approved_by,approved_at,rationale,created_at,updated_at) \
         VALUES (?,?,?,?,?,'ACTIVE',?,?,?,?,?)",
    )
    .bind(&entry_id)
    .bind(&title)
    .bind(&request.entry_kind)
    .bind(&request.canonical_status)
    .bind(&normalized_text)
    .bind(&reviewer)
    .bind(&now)
    .bind(&rationale)
    .bind(&now)
    .bind(&now)
    .execute(&mut *transaction)
    .await?;

    for (source_order, draft) in drafts.iter().enumerate() {
        let update = sqlx::query(
            "UPDATE canon_normalized_drafts \
             SET review_status = ?, canonical_status = ? \
             WHERE id = ? AND review_status = 'PENDING_HUMAN_REVIEW'",
        )
        .bind(resulting_status)
        .bind(&request.canonical_status)
        .bind(&draft.id)
        .execute(&mut *transaction)
        .await?;
        if update.rows_affected() != 1 {
            return Err(review_error(
                "review state changed while approving; no changes were committed",
            ));
        }

        sqlx::query(
            "INSERT INTO canon_entry_sources (entry_id,draft_id,source_order) VALUES (?,?,?)",
        )
        .bind(&entry_id)
        .bind(&draft.id)
        .bind(source_order as i64)
        .execute(&mut *transaction)
        .await?;

        let decision_id = deterministic_id(
            "canon-decision-",
            &["APPROVED", &entry_id, &draft.id, &now],
        );
        sqlx::query(
            "INSERT INTO canon_review_decisions \
             (id,decision_type,draft_id,entry_id,reviewer,rationale,decided_at,previous_review_status,resulting_review_status) \
             VALUES (?,'APPROVED',?,?,?,?,?,'PENDING_HUMAN_REVIEW',?)",
        )
        .bind(&decision_id)
        .bind(&draft.id)
        .bind(&entry_id)
        .bind(&reviewer)
        .bind(&rationale)
        .bind(&now)
        .bind(resulting_status)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    get_canon_review_workspace(pool).await
}

pub async fn reject_canon_drafts(
    pool: &SqlitePool,
    request: RejectCanonDraftsRequest,
) -> Result<CanonReviewWorkspace, AppError> {
    let draft_ids = validate_draft_ids(&request.draft_ids)?;
    let reviewer = normalized_nonempty(&request.reviewer, "reviewer", MAX_REVIEWER_LENGTH)?;
    let rationale = normalized_nonempty(
        &request.rationale,
        "rationale",
        MAX_RATIONALE_LENGTH,
    )?;
    let drafts = load_drafts_by_ids(pool, &draft_ids).await?;
    validate_reviewable_drafts(&draft_ids, &drafts)?;
    let now = Utc::now().to_rfc3339();

    let mut transaction = pool.begin().await?;
    for draft in drafts {
        let update = sqlx::query(
            "UPDATE canon_normalized_drafts \
             SET review_status = 'REJECTED', canonical_status = NULL \
             WHERE id = ? AND review_status = 'PENDING_HUMAN_REVIEW'",
        )
        .bind(&draft.id)
        .execute(&mut *transaction)
        .await?;
        if update.rows_affected() != 1 {
            return Err(review_error(
                "review state changed while rejecting; no changes were committed",
            ));
        }

        let decision_id = deterministic_id(
            "canon-decision-",
            &["REJECTED", &draft.id, &reviewer, &now],
        );
        sqlx::query(
            "INSERT INTO canon_review_decisions \
             (id,decision_type,draft_id,entry_id,reviewer,rationale,decided_at,previous_review_status,resulting_review_status) \
             VALUES (?,'REJECTED',?,NULL,?,?,?,'PENDING_HUMAN_REVIEW','REJECTED')",
        )
        .bind(&decision_id)
        .bind(&draft.id)
        .bind(&reviewer)
        .bind(&rationale)
        .bind(&now)
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    get_canon_review_workspace(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connect, run_migrations};

    async fn seed_draft(pool: &SqlitePool, suffix: &str, index: i64, text: &str) -> String {
        sqlx::query(
            "INSERT OR IGNORE INTO source_documents \
             (id,title,version,authority_rank,original_path,sha256,imported_at,immutable,notes) \
             VALUES ('source','Source','1.3',1,'source.docx',NULL,'2026-07-20T00:00:00Z',1,NULL)",
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT OR IGNORE INTO canon_import_runs \
             (id,source_document_id,source_version,source_sha256,importer_version,status,started_at,completed_at,raw_block_count,draft_count,warning_count) \
             VALUES ('run','source','1.3',?,'test','COMPLETED_PENDING_REVIEW','2026-07-20T00:00:00Z','2026-07-20T00:00:00Z',2,2,0)",
        )
        .bind("a".repeat(64))
        .execute(pool)
        .await
        .unwrap();
        let block_id = format!("block-{suffix}");
        let draft_id = format!("draft-{suffix}");
        sqlx::query(
            "INSERT INTO canon_raw_blocks \
             (id,import_run_id,source_anchor,source_part,block_index,block_kind,style_name,original_text,text_sha256) \
             VALUES (?,'run',?,'word/document.xml',?,'PARAGRAPH',NULL,?,?)",
        )
        .bind(&block_id)
        .bind(format!("sc://test/{suffix}"))
        .bind(index)
        .bind(text)
        .bind(hex::encode(Sha256::digest(text.as_bytes())))
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO canon_normalized_drafts \
             (id,import_run_id,raw_block_id,source_anchor,draft_kind,original_text,review_status,canonical_status,created_at) \
             VALUES (?,'run',?,?, 'PARAGRAPH',?,'PENDING_HUMAN_REVIEW',NULL,'2026-07-20T00:00:00Z')",
        )
        .bind(&draft_id)
        .bind(&block_id)
        .bind(format!("sc://test/{suffix}"))
        .bind(text)
        .execute(pool)
        .await
        .unwrap();
        draft_id
    }

    #[tokio::test]
    async fn approval_merges_sources_and_preserves_evidence() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let first = seed_draft(&pool, "a", 0, "Testo A").await;
        let second = seed_draft(&pool, "b", 1, "Testo B").await;

        let workspace = approve_canon_drafts(
            &pool,
            ApproveCanonDraftsRequest {
                draft_ids: vec![second, first],
                title: "Regola di prova".into(),
                entry_kind: "RULE".into(),
                canonical_status: "ALPHA_DA_TESTARE".into(),
                normalized_text: "Testo A\n\nTesto B".into(),
                reviewer: "Niccolò".into(),
                rationale: "Approvazione test".into(),
            },
        )
        .await
        .unwrap();

        assert_eq!(workspace.summary.pending_count, 0);
        assert_eq!(workspace.summary.approved_count, 2);
        assert_eq!(workspace.entries.len(), 1);
        assert_eq!(workspace.entries[0].sources.len(), 2);
        assert_eq!(workspace.entries[0].sources[0].original_text, "Testo A");
        assert_eq!(workspace.entries[0].sources[1].original_text, "Testo B");
        assert!(workspace
            .drafts
            .iter()
            .all(|draft| draft.review_status == "MERGED_INTO_ENTRY"));
    }

    #[tokio::test]
    async fn rejection_creates_decisions_without_entries() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let draft_id = seed_draft(&pool, "reject", 0, "Rumore editoriale").await;

        let workspace = reject_canon_drafts(
            &pool,
            RejectCanonDraftsRequest {
                draft_ids: vec![draft_id],
                reviewer: "Niccolò".into(),
                rationale: "Non è una regola autonoma".into(),
            },
        )
        .await
        .unwrap();

        assert_eq!(workspace.summary.rejected_count, 1);
        assert!(workspace.entries.is_empty());
        assert_eq!(workspace.recent_decisions[0].decision_type, "REJECTED");
    }

    #[tokio::test]
    async fn reviewed_drafts_cannot_be_reviewed_twice() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let draft_id = seed_draft(&pool, "once", 0, "Una sola volta").await;
        let request = RejectCanonDraftsRequest {
            draft_ids: vec![draft_id],
            reviewer: "Niccolò".into(),
            rationale: "Prima decisione".into(),
        };
        reject_canon_drafts(&pool, request.clone()).await.unwrap();
        let error = reject_canon_drafts(&pool, request).await.unwrap_err();
        assert!(error.to_string().contains("already been reviewed"));
    }
}

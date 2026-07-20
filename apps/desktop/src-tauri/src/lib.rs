use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tauri::Manager;
use thiserror::Error;

const SOURCE_RELATIVE_PATH: &str = "docs/canon/source/Shadow_Council_Source_of_Truth_v1.3.docx";
const SOURCE_FILENAME: &str = "Shadow_Council_Source_of_Truth_v1.3.docx";

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("filesystem error")]
    Io(#[from] std::io::Error),
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
    sqlx::query("INSERT INTO source_documents (id,title,version,authority_rank,original_path,sha256,imported_at,immutable,notes) VALUES (?,?,?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET title=excluded.title, version=excluded.version, authority_rank=excluded.authority_rank, original_path=excluded.original_path, sha256=excluded.sha256, imported_at=excluded.imported_at, immutable=excluded.immutable, notes=excluded.notes") .bind(&doc.id).bind(&doc.title).bind(&doc.version).bind(doc.authority_rank).bind(&doc.original_path).bind(&doc.sha256).bind(&doc.imported_at).bind(doc.immutable).bind(&doc.notes).execute(pool).await?;
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
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
pub async fn build_health(pool: &SqlitePool, root: &Path) -> Result<HealthStatus, AppError> {
    let source = root.join(SOURCE_RELATIVE_PATH);
    let mut diagnostics = vec![];
    let (exists, sha, canon) = if source.exists() {
        let hash = sha256_file(&source)?;
        let doc=SourceDocument{id:"source-of-truth-v1.3".into(),title:"Shadow Council Source of Truth".into(),version:"v1.3".into(),authority_rank:1,original_path:SOURCE_RELATIVE_PATH.into(),sha256:Some(hash.clone()),imported_at:chrono::Utc::now().to_rfc3339(),immutable:1,notes:Some("Immutable primary game-design source; semantic extraction not performed in Sprint 0.".into())};
        upsert_source_document(pool, &doc).await?;
        diagnostics.push("Canonical source registered as immutable metadata only.".into());
        (true, Some(hash), Some("v1.3".into()))
    } else {
        diagnostics.push("canonical source missing; no canonical content was inferred".into());
        (false, None, None)
    };
    Ok(HealthStatus {
        project_name: "Shadow Council Studio".into(),
        development_stage: "Foundation".into(),
        database_connected: true,
        migrations_applied: true,
        source_of_truth: SourceOfTruthStatus {
            exists,
            filename: SOURCE_FILENAME.into(),
            sha256: sha,
            canon_version: canon,
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
        .unwrap_or_else(|| std::env::temp_dir().join("shadow-council-studio"));
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
    async fn service_health_handles_missing_source() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let dir = tempdir().unwrap();
        let health = build_health(&pool, dir.path()).await.unwrap();
        assert!(!health.source_of_truth.exists);
        assert!(health.database_connected);
    }
}

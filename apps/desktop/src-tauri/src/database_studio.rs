use crate::{AppError, sha256_file};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

const MAX_PAGE_SIZE: u32 = 100;
const MAX_EXPORT_ROWS: usize = 50_000;
const EDITABLE_METADATA_KEYS: [&str; 3] = [
    "studio.workspace_name",
    "studio.release_channel",
    "studio.internal_notes",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseColumn {
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key_position: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseIndex {
    pub name: String,
    pub unique: bool,
    pub origin: String,
    pub partial: bool,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRelationship {
    pub id: String,
    pub source_table: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
    pub on_update: String,
    pub on_delete: String,
    pub cardinality: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseTableSchema {
    pub name: String,
    pub row_count: i64,
    pub create_sql: Option<String>,
    pub migration_source: Option<String>,
    pub columns: Vec<DatabaseColumn>,
    pub indexes: Vec<DatabaseIndex>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseIntegrityReport {
    pub ok: bool,
    pub checked_at: String,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseStudioSnapshot {
    pub database_path: String,
    pub tables: Vec<DatabaseTableSchema>,
    pub relationships: Vec<DatabaseRelationship>,
    pub integrity: DatabaseIntegrityReport,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBrowseRequest {
    pub table_name: String,
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub search: Option<String>,
    pub sort_column: Option<String>,
    pub sort_direction: Option<String>,
    pub filter_column: Option<String>,
    pub filter_value: Option<String>,
}

fn default_page_size() -> u32 {
    25
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseTablePage {
    pub table_name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Value>,
    pub total_count: i64,
    pub page: u32,
    pub page_size: u32,
    pub sort_column: String,
    pub sort_direction: String,
    pub filter_column: Option<String>,
    pub filter_value: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseFileResult {
    pub path: String,
    pub file_name: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseAuditEntry {
    pub id: String,
    pub entity_type: String,
    pub record_id: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: String,
    pub changed_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadataUpdate {
    pub key: String,
    pub value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonReviewNoteUpdate {
    pub draft_id: String,
    pub note: String,
    pub reason: String,
}

fn quote_identifier(identifier: &str) -> Result<String, AppError> {
    if identifier.is_empty()
        || !identifier
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(AppError::DatabaseStudio(format!(
            "invalid SQLite identifier: {identifier}"
        )));
    }
    Ok(format!("\"{identifier}\""))
}

async fn user_tables(pool: &SqlitePool) -> Result<Vec<(String, Option<String>)>, AppError> {
    let rows = sqlx::query(
        "SELECT name, sql FROM sqlite_master \
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name <> '_sqlx_migrations' \
         ORDER BY name",
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| Ok((row.try_get("name")?, row.try_get("sql")?)))
        .collect()
}

async fn ensure_user_table(pool: &SqlitePool, table_name: &str) -> Result<(), AppError> {
    quote_identifier(table_name)?;
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master \
         WHERE type = 'table' AND name = ? AND name NOT LIKE 'sqlite_%' AND name <> '_sqlx_migrations'",
    )
    .bind(table_name)
    .fetch_one(pool)
    .await?;
    if exists != 1 {
        return Err(AppError::DatabaseStudio(format!(
            "unknown or unavailable table: {table_name}"
        )));
    }
    Ok(())
}

async fn table_columns(
    pool: &SqlitePool,
    table_name: &str,
) -> Result<Vec<DatabaseColumn>, AppError> {
    ensure_user_table(pool, table_name).await?;
    let sql = format!("PRAGMA table_info({})", quote_identifier(table_name)?);
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    rows.into_iter()
        .map(|row| {
            Ok(DatabaseColumn {
                name: row.try_get("name")?,
                data_type: row.try_get::<String, _>("type")?,
                not_null: row.try_get::<i64, _>("notnull")? == 1,
                default_value: row.try_get("dflt_value")?,
                primary_key_position: row.try_get("pk")?,
            })
        })
        .collect()
}

async fn table_indexes(
    pool: &SqlitePool,
    table_name: &str,
) -> Result<Vec<DatabaseIndex>, AppError> {
    ensure_user_table(pool, table_name).await?;
    let sql = format!("PRAGMA index_list({})", quote_identifier(table_name)?);
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    let mut indexes = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row.try_get("name")?;
        quote_identifier(&name)?;
        let info_sql = format!("PRAGMA index_info({})", quote_identifier(&name)?);
        let info_rows = sqlx::query(&info_sql).fetch_all(pool).await?;
        let columns = info_rows
            .into_iter()
            .map(|info| info.try_get("name"))
            .collect::<Result<Vec<String>, sqlx::Error>>()?;
        indexes.push(DatabaseIndex {
            name,
            unique: row.try_get::<i64, _>("unique")? == 1,
            origin: row.try_get("origin")?,
            partial: row.try_get::<i64, _>("partial")? == 1,
            columns,
        });
    }
    Ok(indexes)
}

async fn table_relationships(
    pool: &SqlitePool,
    table_name: &str,
    columns: &[DatabaseColumn],
    indexes: &[DatabaseIndex],
) -> Result<Vec<DatabaseRelationship>, AppError> {
    ensure_user_table(pool, table_name).await?;
    let unique_single_columns: HashSet<&str> = indexes
        .iter()
        .filter(|index| index.unique && index.columns.len() == 1)
        .map(|index| index.columns[0].as_str())
        .chain(
            columns
                .iter()
                .filter(|column| column.primary_key_position > 0)
                .map(|column| column.name.as_str()),
        )
        .collect();
    let sql = format!("PRAGMA foreign_key_list({})", quote_identifier(table_name)?);
    let rows = sqlx::query(&sql).fetch_all(pool).await?;
    rows.into_iter()
        .map(|row| {
            let source_column: String = row.try_get("from")?;
            let target_table: String = row.try_get("table")?;
            let target_column: String = row.try_get("to")?;
            Ok(DatabaseRelationship {
                id: format!("{table_name}:{source_column}->{target_table}:{target_column}"),
                source_table: table_name.to_owned(),
                source_column: source_column.clone(),
                target_table,
                target_column,
                on_update: row.try_get("on_update")?,
                on_delete: row.try_get("on_delete")?,
                cardinality: if unique_single_columns.contains(source_column.as_str()) {
                    "1:1".into()
                } else {
                    "N:1".into()
                },
            })
        })
        .collect()
}

fn migration_source(table_name: &str) -> Option<String> {
    match table_name {
        "source_documents" | "project_metadata" => Some("0001_foundation.sql".into()),
        "canon_import_runs"
        | "canon_raw_blocks"
        | "canon_normalized_drafts"
        | "canon_import_warnings" => Some("0002_canon_import.sql".into()),
        "database_audit_log" | "canon_review_notes" => Some("0003_database_studio.sql".into()),
        _ => None,
    }
}

pub async fn run_integrity_check(pool: &SqlitePool) -> Result<DatabaseIntegrityReport, AppError> {
    let rows = sqlx::query("PRAGMA integrity_check")
        .fetch_all(pool)
        .await?;
    let messages = rows
        .into_iter()
        .map(|row| row.try_get(0))
        .collect::<Result<Vec<String>, sqlx::Error>>()?;
    Ok(DatabaseIntegrityReport {
        ok: messages.len() == 1 && messages[0].eq_ignore_ascii_case("ok"),
        checked_at: Utc::now().to_rfc3339(),
        messages,
    })
}

pub async fn get_database_snapshot(
    pool: &SqlitePool,
    database_path: &Path,
) -> Result<DatabaseStudioSnapshot, AppError> {
    let mut tables = Vec::new();
    let mut relationships = Vec::new();
    for (name, create_sql) in user_tables(pool).await? {
        let columns = table_columns(pool, &name).await?;
        let indexes = table_indexes(pool, &name).await?;
        relationships.extend(table_relationships(pool, &name, &columns, &indexes).await?);
        let count_sql = format!("SELECT COUNT(*) FROM {}", quote_identifier(&name)?);
        let row_count: i64 = sqlx::query_scalar(&count_sql).fetch_one(pool).await?;
        tables.push(DatabaseTableSchema {
            migration_source: migration_source(&name),
            name,
            row_count,
            create_sql,
            columns,
            indexes,
        });
    }
    Ok(DatabaseStudioSnapshot {
        database_path: database_path.display().to_string(),
        tables,
        relationships,
        integrity: run_integrity_check(pool).await?,
    })
}

fn append_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    columns: &[DatabaseColumn],
    search: Option<String>,
    filter_column: Option<String>,
    filter_value: Option<String>,
) -> Result<(), AppError> {
    let column_names: HashSet<&str> = columns.iter().map(|column| column.name.as_str()).collect();
    let mut has_where = false;

    if let Some(search) = search.filter(|value| !value.trim().is_empty()) {
        builder.push(" WHERE (");
        let pattern = format!("%{}%", search.trim());
        for (index, column) in columns.iter().enumerate() {
            if index > 0 {
                builder.push(" OR ");
            }
            builder
                .push("CAST(")
                .push(quote_identifier(&column.name)?)
                .push(" AS TEXT) LIKE ")
                .push_bind(pattern.clone());
        }
        builder.push(")");
        has_where = true;
    }

    if let (Some(column), Some(value)) = (filter_column, filter_value) {
        if !column_names.contains(column.as_str()) {
            return Err(AppError::DatabaseStudio(format!(
                "unknown filter column: {column}"
            )));
        }
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder
            .push("CAST(")
            .push(quote_identifier(&column)?)
            .push(" AS TEXT) = ")
            .push_bind(value);
    }
    Ok(())
}

fn json_row_expression(columns: &[DatabaseColumn]) -> Result<String, AppError> {
    let mut pairs = Vec::with_capacity(columns.len());
    for column in columns {
        let identifier = quote_identifier(&column.name)?;
        let key = column.name.replace('\'', "''");
        pairs.push(format!(
            "'{key}', CASE WHEN typeof({identifier}) = 'blob' THEN '<BLOB:' || length({identifier}) || ' bytes>' ELSE {identifier} END"
        ));
    }
    Ok(format!("json_object({})", pairs.join(", ")))
}

pub async fn browse_table(
    pool: &SqlitePool,
    request: TableBrowseRequest,
) -> Result<DatabaseTablePage, AppError> {
    ensure_user_table(pool, &request.table_name).await?;
    let columns = table_columns(pool, &request.table_name).await?;
    if columns.is_empty() {
        return Err(AppError::DatabaseStudio(format!(
            "table has no visible columns: {}",
            request.table_name
        )));
    }
    let column_names: HashSet<&str> = columns.iter().map(|column| column.name.as_str()).collect();
    let sort_column = request
        .sort_column
        .as_deref()
        .filter(|column| column_names.contains(*column))
        .map(str::to_owned)
        .unwrap_or_else(|| {
            columns
                .iter()
                .find(|column| column.primary_key_position > 0)
                .unwrap_or(&columns[0])
                .name
                .clone()
        });
    let sort_direction = match request
        .sort_direction
        .as_deref()
        .unwrap_or("ASC")
        .to_ascii_uppercase()
        .as_str()
    {
        "DESC" => "DESC".to_owned(),
        _ => "ASC".to_owned(),
    };
    let page_size = request.page_size.clamp(1, MAX_PAGE_SIZE);
    let table_identifier = quote_identifier(&request.table_name)?;

    let mut count_builder = QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM ");
    count_builder.push(table_identifier.clone());
    append_filters(
        &mut count_builder,
        &columns,
        request.search.clone(),
        request.filter_column.clone(),
        request.filter_value.clone(),
    )?;
    let total_count: i64 = count_builder.build_query_scalar().fetch_one(pool).await?;

    let mut data_builder = QueryBuilder::<Sqlite>::new("SELECT ");
    data_builder
        .push(json_row_expression(&columns)?)
        .push(" AS row_json FROM ")
        .push(table_identifier);
    append_filters(
        &mut data_builder,
        &columns,
        request.search,
        request.filter_column.clone(),
        request.filter_value.clone(),
    )?;
    data_builder
        .push(" ORDER BY ")
        .push(quote_identifier(&sort_column)?)
        .push(" ")
        .push(sort_direction.clone())
        .push(" LIMIT ")
        .push_bind(i64::from(page_size))
        .push(" OFFSET ")
        .push_bind(i64::from(request.page) * i64::from(page_size));

    let rows = data_builder.build().fetch_all(pool).await?;
    let rows = rows
        .into_iter()
        .map(|row| {
            let json: String = row.try_get("row_json")?;
            serde_json::from_str(&json).map_err(|error| sqlx::Error::Decode(Box::new(error)))
        })
        .collect::<Result<Vec<Value>, sqlx::Error>>()?;

    Ok(DatabaseTablePage {
        table_name: request.table_name,
        columns: columns.into_iter().map(|column| column.name).collect(),
        rows,
        total_count,
        page: request.page,
        page_size,
        sort_column,
        sort_direction,
        filter_column: request.filter_column,
        filter_value: request.filter_value,
    })
}

fn value_as_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn csv_cell(value: &Value) -> String {
    let value = value_as_text(value);
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

pub async fn export_table(
    pool: &SqlitePool,
    export_dir: &Path,
    mut request: TableBrowseRequest,
    format: &str,
) -> Result<DatabaseFileResult, AppError> {
    ensure_user_table(pool, &request.table_name).await?;
    let normalized_format = match format.to_ascii_lowercase().as_str() {
        "csv" => "csv",
        "json" => "json",
        other => {
            return Err(AppError::DatabaseStudio(format!(
                "unsupported export format: {other}"
            )));
        }
    };
    fs::create_dir_all(export_dir)?;
    request.page = 0;
    request.page_size = MAX_PAGE_SIZE;
    let mut all_rows = Vec::new();
    let mut columns = Vec::new();
    loop {
        let page = browse_table(pool, request.clone()).await?;
        if columns.is_empty() {
            columns = page.columns;
        }
        let fetched = page.rows.len();
        all_rows.extend(page.rows);
        if fetched < usize::try_from(MAX_PAGE_SIZE).unwrap_or(100)
            || all_rows.len() >= usize::try_from(page.total_count.max(0)).unwrap_or(0)
            || all_rows.len() >= MAX_EXPORT_ROWS
        {
            break;
        }
        request.page += 1;
    }

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    let file_name = format!("{}-{timestamp}.{normalized_format}", request.table_name);
    let path = export_dir.join(&file_name);
    match normalized_format {
        "json" => fs::write(
            &path,
            serde_json::to_vec_pretty(&all_rows).map_err(|error| {
                AppError::DatabaseStudio(format!("could not serialize JSON export: {error}"))
            })?,
        )?,
        "csv" => {
            let mut output = String::new();
            output.push_str(
                &columns
                    .iter()
                    .map(|column| csv_cell(&Value::String(column.clone())))
                    .collect::<Vec<_>>()
                    .join(","),
            );
            output.push('\n');
            for row in all_rows {
                let object = row.as_object().cloned().unwrap_or_default();
                output.push_str(
                    &columns
                        .iter()
                        .map(|column| csv_cell(object.get(column).unwrap_or(&Value::Null)))
                        .collect::<Vec<_>>()
                        .join(","),
                );
                output.push('\n');
            }
            fs::write(&path, output)?;
        }
        _ => unreachable!(),
    }
    file_result(&path)
}

pub async fn create_backup(
    pool: &SqlitePool,
    database_path: &Path,
    backup_dir: &Path,
) -> Result<DatabaseFileResult, AppError> {
    fs::create_dir_all(backup_dir)?;
    sqlx::query("PRAGMA wal_checkpoint(FULL)")
        .execute(pool)
        .await?;
    let file_name = format!(
        "shadow-council-studio-{}.sqlite",
        Utc::now().format("%Y%m%dT%H%M%SZ")
    );
    let backup_path = backup_dir.join(file_name);
    fs::copy(database_path, &backup_path)?;
    file_result(&backup_path)
}

fn file_result(path: &Path) -> Result<DatabaseFileResult, AppError> {
    let metadata = fs::metadata(path)?;
    Ok(DatabaseFileResult {
        path: path.display().to_string(),
        file_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("database-file")
            .to_owned(),
        sha256: sha256_file(path)?,
        size_bytes: metadata.len(),
        created_at: Utc::now().to_rfc3339(),
    })
}

fn validated_reason(reason: &str) -> Result<String, AppError> {
    let reason = reason.trim();
    if reason.len() < 3 {
        return Err(AppError::DatabaseStudio(
            "a change reason of at least three characters is required".into(),
        ));
    }
    Ok(reason.to_owned())
}

fn audit_id(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    format!("audit-{}", hex::encode(hasher.finalize()))
}

async fn insert_audit(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    entry: &DatabaseAuditEntry,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO database_audit_log \
         (id, entity_type, record_id, field_name, old_value, new_value, reason, changed_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&entry.id)
    .bind(&entry.entity_type)
    .bind(&entry.record_id)
    .bind(&entry.field_name)
    .bind(&entry.old_value)
    .bind(&entry.new_value)
    .bind(&entry.reason)
    .bind(&entry.changed_at)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub async fn update_project_metadata(
    pool: &SqlitePool,
    update: ProjectMetadataUpdate,
) -> Result<DatabaseAuditEntry, AppError> {
    if !EDITABLE_METADATA_KEYS.contains(&update.key.as_str()) {
        return Err(AppError::DatabaseStudio(format!(
            "metadata key is not editable: {}",
            update.key
        )));
    }
    let reason = validated_reason(&update.reason)?;
    let mut transaction = pool.begin().await?;
    let old_value: Option<String> =
        sqlx::query_scalar("SELECT value FROM project_metadata WHERE key = ?")
            .bind(&update.key)
            .fetch_optional(&mut *transaction)
            .await?;
    let Some(old_value) = old_value else {
        return Err(AppError::DatabaseStudio(format!(
            "metadata key does not exist: {}",
            update.key
        )));
    };
    let changed_at = Utc::now().to_rfc3339();
    sqlx::query("UPDATE project_metadata SET value = ?, updated_at = ? WHERE key = ?")
        .bind(&update.value)
        .bind(&changed_at)
        .bind(&update.key)
        .execute(&mut *transaction)
        .await?;
    let entry = DatabaseAuditEntry {
        id: audit_id(&[
            "project_metadata",
            &update.key,
            &changed_at,
            &old_value,
            &update.value,
            &reason,
        ]),
        entity_type: "project_metadata".into(),
        record_id: update.key,
        field_name: "value".into(),
        old_value: Some(old_value),
        new_value: Some(update.value),
        reason,
        changed_at,
    };
    insert_audit(&mut transaction, &entry).await?;
    transaction.commit().await?;
    Ok(entry)
}

pub async fn upsert_review_note(
    pool: &SqlitePool,
    update: CanonReviewNoteUpdate,
) -> Result<DatabaseAuditEntry, AppError> {
    let reason = validated_reason(&update.reason)?;
    let mut transaction = pool.begin().await?;
    let draft_exists: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM canon_normalized_drafts WHERE id = ?")
            .bind(&update.draft_id)
            .fetch_one(&mut *transaction)
            .await?;
    if draft_exists != 1 {
        return Err(AppError::DatabaseStudio(format!(
            "review draft does not exist: {}",
            update.draft_id
        )));
    }
    let old_value: Option<String> =
        sqlx::query_scalar("SELECT note FROM canon_review_notes WHERE draft_id = ?")
            .bind(&update.draft_id)
            .fetch_optional(&mut *transaction)
            .await?;
    let changed_at = Utc::now().to_rfc3339();
    let note_id = format!(
        "review-note-{}",
        hex::encode(Sha256::digest(update.draft_id.as_bytes()))
    );
    sqlx::query(
        "INSERT INTO canon_review_notes (id, draft_id, note, updated_at) \
         VALUES (?, ?, ?, ?) \
         ON CONFLICT(draft_id) DO UPDATE SET note = excluded.note, updated_at = excluded.updated_at",
    )
    .bind(note_id)
    .bind(&update.draft_id)
    .bind(&update.note)
    .bind(&changed_at)
    .execute(&mut *transaction)
    .await?;
    let entry = DatabaseAuditEntry {
        id: audit_id(&[
            "canon_review_notes",
            &update.draft_id,
            &changed_at,
            old_value.as_deref().unwrap_or(""),
            &update.note,
            &reason,
        ]),
        entity_type: "canon_review_notes".into(),
        record_id: update.draft_id,
        field_name: "note".into(),
        old_value,
        new_value: Some(update.note),
        reason,
        changed_at,
    };
    insert_audit(&mut transaction, &entry).await?;
    transaction.commit().await?;
    Ok(entry)
}

pub fn database_path(data_dir: &Path) -> PathBuf {
    data_dir.join("shadow-council-studio.sqlite")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connect, run_migrations};
    use tempfile::tempdir;

    async fn memory_pool() -> SqlitePool {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn snapshot_discovers_schema_and_relationships() {
        let pool = memory_pool().await;
        let snapshot = get_database_snapshot(&pool, Path::new("studio.sqlite"))
            .await
            .unwrap();
        assert!(
            snapshot
                .tables
                .iter()
                .any(|table| table.name == "canon_raw_blocks")
        );
        assert!(snapshot.relationships.iter().any(|relationship| {
            relationship.source_table == "canon_raw_blocks"
                && relationship.target_table == "canon_import_runs"
        }));
        assert!(snapshot.integrity.ok);
    }

    #[tokio::test]
    async fn table_browser_is_paginated_and_rejects_unknown_tables() {
        let pool = memory_pool().await;
        let page = browse_table(
            &pool,
            TableBrowseRequest {
                table_name: "project_metadata".into(),
                page: 0,
                page_size: 2,
                search: Some("studio".into()),
                sort_column: Some("key".into()),
                sort_direction: Some("ASC".into()),
                filter_column: None,
                filter_value: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(page.rows.len(), 2);
        assert_eq!(page.total_count, 3);

        let error = browse_table(
            &pool,
            TableBrowseRequest {
                table_name: "project_metadata; DROP TABLE project_metadata".into(),
                page: 0,
                page_size: 25,
                search: None,
                sort_column: None,
                sort_direction: None,
                filter_column: None,
                filter_value: None,
            },
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("invalid SQLite identifier"));
    }

    #[tokio::test]
    async fn metadata_updates_are_allowlisted_and_audited() {
        let pool = memory_pool().await;
        let entry = update_project_metadata(
            &pool,
            ProjectMetadataUpdate {
                key: "studio.workspace_name".into(),
                value: "Shadow Council Alpha".into(),
                reason: "Rename workspace".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(entry.entity_type, "project_metadata");
        let audit_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM database_audit_log")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(audit_count, 1);

        let blocked = update_project_metadata(
            &pool,
            ProjectMetadataUpdate {
                key: "canon.current_version".into(),
                value: "999".into(),
                reason: "Unsafe change".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(blocked.to_string().contains("not editable"));
    }

    #[tokio::test]
    async fn backup_and_export_create_hash_verified_files() {
        let pool = memory_pool().await;
        let root = tempdir().unwrap();
        let database_file = root.path().join("studio.sqlite");
        fs::write(&database_file, "database-placeholder").unwrap();
        let backup = create_backup(&pool, &database_file, &root.path().join("backups"))
            .await
            .unwrap();
        assert!(Path::new(&backup.path).is_file());
        assert_eq!(backup.sha256.len(), 64);

        let export = export_table(
            &pool,
            &root.path().join("exports"),
            TableBrowseRequest {
                table_name: "project_metadata".into(),
                page: 0,
                page_size: 25,
                search: None,
                sort_column: None,
                sort_direction: None,
                filter_column: None,
                filter_value: None,
            },
            "json",
        )
        .await
        .unwrap();
        assert!(Path::new(&export.path).is_file());
        assert_eq!(export.sha256.len(), 64);
    }
}

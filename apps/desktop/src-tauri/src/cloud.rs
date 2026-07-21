use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::AppError;

#[derive(Debug, Clone, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CloudSettings {
    pub supabase_url: Option<String>,
    pub publishable_key: Option<String>,
    pub workspace_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSettingsUpdate {
    pub supabase_url: Option<String>,
    pub publishable_key: Option<String>,
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudStatus {
    pub settings: CloudSettings,
    pub configured: bool,
    pub sync_ready: bool,
    pub pending_outbox_count: i64,
    pub open_conflict_count: i64,
    pub mode: String,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupabaseEndpointKind {
    Local,
    Remote,
}

fn normalized_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn endpoint_kind(url: &str) -> Result<SupabaseEndpointKind, AppError> {
    let normalized = url.trim().trim_end_matches('/').to_ascii_lowercase();
    if normalized.contains(['?', '#', '@']) {
        return Err(invalid_supabase_url());
    }

    if let Some(authority) = normalized.strip_prefix("https://") {
        if authority.contains(['/', ':']) {
            return Err(invalid_supabase_url());
        }
        let Some(project_ref) = authority.strip_suffix(".supabase.co") else {
            return Err(invalid_supabase_url());
        };
        let valid_project_ref = !project_ref.is_empty()
            && project_ref
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
            && project_ref
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphanumeric())
            && project_ref
                .chars()
                .last()
                .is_some_and(|character| character.is_ascii_alphanumeric());
        if valid_project_ref {
            return Ok(SupabaseEndpointKind::Remote);
        }
        return Err(invalid_supabase_url());
    }

    if let Some(authority) = normalized.strip_prefix("http://") {
        if authority.contains('/') {
            return Err(invalid_supabase_url());
        }
        let Some((host, port)) = authority.rsplit_once(':') else {
            return Err(invalid_supabase_url());
        };
        let valid_host = host == "localhost" || host == "127.0.0.1";
        let valid_port = port.parse::<u16>().is_ok_and(|port| port > 0);
        if valid_host && valid_port {
            return Ok(SupabaseEndpointKind::Local);
        }
    }

    Err(invalid_supabase_url())
}

fn invalid_supabase_url() -> AppError {
    AppError::Cloud(
        "Supabase URL must be an exact HTTPS <project-ref>.supabase.co origin or a local http://localhost:<port> CLI origin"
            .into(),
    )
}

fn validate_publishable_key(
    endpoint_kind: SupabaseEndpointKind,
    key: &str,
) -> Result<(), AppError> {
    if key.starts_with("sb_secret_") {
        return Err(AppError::Cloud(
            "Secret Supabase keys are forbidden in the desktop app".into(),
        ));
    }

    if key.starts_with("sb_publishable_")
        || (endpoint_kind == SupabaseEndpointKind::Local && key.split('.').count() == 3)
    {
        Ok(())
    } else {
        Err(AppError::Cloud(
            "Use the Supabase publishable key. Legacy JWT keys are accepted only for local CLI development"
                .into(),
        ))
    }
}

pub async fn get_cloud_status(pool: &SqlitePool) -> Result<CloudStatus, AppError> {
    let settings = sqlx::query_as::<_, CloudSettings>(
        "SELECT supabase_url,publishable_key,workspace_id,updated_at FROM cloud_settings WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;

    let pending_outbox_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cloud_sync_outbox WHERE status IN ('PENDING','FAILED','BLOCKED')",
    )
    .fetch_one(pool)
    .await?;
    let open_conflict_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM cloud_sync_conflicts WHERE status = 'OPEN'")
            .fetch_one(pool)
            .await?;

    let configured = settings.supabase_url.is_some() && settings.publishable_key.is_some();
    let sync_ready = configured && settings.workspace_id.is_some();
    let mut diagnostics = Vec::new();
    if !configured {
        diagnostics
            .push("Cloud mode is disabled; SQLite remains the only active datastore.".into());
    } else if settings.workspace_id.is_none() {
        diagnostics.push(
            "Supabase is configured; authenticate and select a workspace before synchronization."
                .into(),
        );
    } else {
        diagnostics.push("Supabase configuration and workspace selection are present. Automatic synchronization is still disabled in Phase 1.6.".into());
    }

    Ok(CloudStatus {
        settings,
        configured,
        sync_ready,
        pending_outbox_count,
        open_conflict_count,
        mode: if sync_ready {
            "CLOUD_READY"
        } else if configured {
            "CONFIGURED"
        } else {
            "LOCAL_ONLY"
        }
        .into(),
        diagnostics,
    })
}

pub async fn update_cloud_settings(
    pool: &SqlitePool,
    update: CloudSettingsUpdate,
) -> Result<CloudStatus, AppError> {
    let supabase_url = normalized_optional(update.supabase_url);
    let publishable_key = normalized_optional(update.publishable_key);
    let workspace_id = normalized_optional(update.workspace_id);

    match (&supabase_url, &publishable_key) {
        (Some(url), Some(key)) => {
            let endpoint_kind = endpoint_kind(url)?;
            validate_publishable_key(endpoint_kind, key)?;
        }
        (None, None) => {}
        _ => {
            return Err(AppError::Cloud(
                "Supabase URL and publishable key must be saved together".into(),
            ));
        }
    }

    if workspace_id.is_some() && supabase_url.is_none() {
        return Err(AppError::Cloud(
            "A workspace cannot be selected while cloud mode is disabled".into(),
        ));
    }

    sqlx::query(
        "UPDATE cloud_settings SET supabase_url = ?, publishable_key = ?, workspace_id = ?, updated_at = ? WHERE id = 1",
    )
    .bind(supabase_url)
    .bind(publishable_key)
    .bind(workspace_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    get_cloud_status(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{connect, run_migrations};

    #[tokio::test]
    async fn cloud_defaults_to_local_only() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let status = get_cloud_status(&pool).await.unwrap();
        assert_eq!(status.mode, "LOCAL_ONLY");
        assert!(!status.configured);
    }

    #[test]
    fn exact_remote_and_local_supabase_origins_are_accepted() {
        assert_eq!(
            endpoint_kind("https://abcd1234.supabase.co/").unwrap(),
            SupabaseEndpointKind::Remote
        );
        assert_eq!(
            endpoint_kind("http://127.0.0.1:54321").unwrap(),
            SupabaseEndpointKind::Local
        );
        assert_eq!(
            endpoint_kind("http://localhost:54321/").unwrap(),
            SupabaseEndpointKind::Local
        );
    }

    #[test]
    fn lookalike_or_credential_redirect_hosts_are_rejected() {
        for malicious_url in [
            "https://project.supabase.co.evil.com",
            "https://attacker.example/path/.supabase.co",
            "https://project.supabase.co@evil.com",
            "https://supabase.co",
            "https://project.supabase.co:443",
        ] {
            assert!(endpoint_kind(malicious_url).is_err(), "{malicious_url}");
        }
    }

    #[tokio::test]
    async fn secret_keys_are_rejected() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let error = update_cloud_settings(
            &pool,
            CloudSettingsUpdate {
                supabase_url: Some("https://example.supabase.co".into()),
                publishable_key: Some("sb_secret_forbidden".into()),
                workspace_id: None,
            },
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("forbidden"));
    }

    #[tokio::test]
    async fn publishable_key_can_enable_cloud_configuration() {
        let pool = connect("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let status = update_cloud_settings(
            &pool,
            CloudSettingsUpdate {
                supabase_url: Some("https://example.supabase.co".into()),
                publishable_key: Some("sb_publishable_example".into()),
                workspace_id: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(status.mode, "CONFIGURED");
        assert!(status.configured);
        assert!(!status.sync_ready);
    }
}

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS cloud_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  supabase_url TEXT,
  publishable_key TEXT,
  workspace_id TEXT,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO cloud_settings (
  id,
  supabase_url,
  publishable_key,
  workspace_id,
  updated_at
) VALUES (1, NULL, NULL, NULL, '2026-07-21T00:00:00Z');

CREATE TABLE IF NOT EXISTS cloud_sync_outbox (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  operation TEXT NOT NULL CHECK (operation IN ('UPSERT', 'DELETE')),
  payload_json TEXT,
  base_row_version INTEGER,
  mutation_key TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL DEFAULT 'PENDING' CHECK (
    status IN ('PENDING', 'IN_FLIGHT', 'SYNCED', 'FAILED', 'BLOCKED')
  ),
  attempt_count INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cloud_sync_outbox_status_created
  ON cloud_sync_outbox (status, created_at, id);

CREATE TABLE IF NOT EXISTS cloud_sync_state (
  workspace_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  remote_cursor TEXT,
  last_pulled_at TEXT,
  last_pushed_at TEXT,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (workspace_id, entity_type)
);

CREATE TABLE IF NOT EXISTS cloud_sync_conflicts (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  local_payload_json TEXT,
  remote_payload_json TEXT,
  local_row_version INTEGER,
  remote_row_version INTEGER,
  status TEXT NOT NULL DEFAULT 'OPEN' CHECK (
    status IN ('OPEN', 'RESOLVED_LOCAL', 'RESOLVED_REMOTE', 'RESOLVED_MANUAL')
  ),
  resolution_note TEXT,
  detected_at TEXT NOT NULL,
  resolved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_cloud_sync_conflicts_status
  ON cloud_sync_conflicts (workspace_id, status, detected_at DESC);

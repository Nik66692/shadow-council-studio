PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS database_audit_log (
  id TEXT PRIMARY KEY,
  entity_type TEXT NOT NULL,
  record_id TEXT NOT NULL,
  field_name TEXT NOT NULL,
  old_value TEXT,
  new_value TEXT,
  reason TEXT NOT NULL,
  changed_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_database_audit_log_entity
  ON database_audit_log (entity_type, record_id, changed_at DESC);

CREATE TABLE IF NOT EXISTS canon_review_notes (
  id TEXT PRIMARY KEY,
  draft_id TEXT NOT NULL UNIQUE,
  note TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (draft_id) REFERENCES canon_normalized_drafts(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO project_metadata (key, value, updated_at)
VALUES
  ('studio.workspace_name', 'Shadow Council', '2026-07-20T00:00:00Z'),
  ('studio.release_channel', 'internal-preview', '2026-07-20T00:00:00Z'),
  ('studio.internal_notes', '', '2026-07-20T00:00:00Z');

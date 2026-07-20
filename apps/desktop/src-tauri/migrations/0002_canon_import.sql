PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS canon_import_runs (
  id TEXT PRIMARY KEY,
  source_document_id TEXT NOT NULL,
  source_version TEXT NOT NULL,
  source_sha256 TEXT NOT NULL,
  importer_version TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  completed_at TEXT NOT NULL,
  raw_block_count INTEGER NOT NULL,
  draft_count INTEGER NOT NULL,
  warning_count INTEGER NOT NULL,
  FOREIGN KEY (source_document_id) REFERENCES source_documents(id),
  UNIQUE (source_sha256, importer_version)
);

CREATE TABLE IF NOT EXISTS canon_raw_blocks (
  id TEXT PRIMARY KEY,
  import_run_id TEXT NOT NULL,
  source_anchor TEXT NOT NULL,
  source_part TEXT NOT NULL,
  block_index INTEGER NOT NULL,
  block_kind TEXT NOT NULL,
  style_name TEXT,
  original_text TEXT NOT NULL,
  text_sha256 TEXT NOT NULL,
  FOREIGN KEY (import_run_id) REFERENCES canon_import_runs(id) ON DELETE CASCADE,
  UNIQUE (import_run_id, source_anchor)
);

CREATE TABLE IF NOT EXISTS canon_normalized_drafts (
  id TEXT PRIMARY KEY,
  import_run_id TEXT NOT NULL,
  raw_block_id TEXT NOT NULL,
  source_anchor TEXT NOT NULL,
  draft_kind TEXT NOT NULL,
  original_text TEXT NOT NULL,
  review_status TEXT NOT NULL,
  canonical_status TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (import_run_id) REFERENCES canon_import_runs(id) ON DELETE CASCADE,
  FOREIGN KEY (raw_block_id) REFERENCES canon_raw_blocks(id) ON DELETE CASCADE,
  UNIQUE (raw_block_id)
);

CREATE TABLE IF NOT EXISTS canon_import_warnings (
  id TEXT PRIMARY KEY,
  import_run_id TEXT NOT NULL,
  source_anchor TEXT,
  warning_code TEXT NOT NULL,
  message TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (import_run_id) REFERENCES canon_import_runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_canon_raw_blocks_run_index
  ON canon_raw_blocks (import_run_id, block_index);

CREATE INDEX IF NOT EXISTS idx_canon_drafts_run
  ON canon_normalized_drafts (import_run_id);

CREATE INDEX IF NOT EXISTS idx_canon_warnings_run
  ON canon_import_warnings (import_run_id);

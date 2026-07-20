CREATE TABLE IF NOT EXISTS source_documents (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  version TEXT NOT NULL,
  authority_rank INTEGER NOT NULL,
  original_path TEXT NOT NULL,
  sha256 TEXT,
  imported_at TEXT NOT NULL,
  immutable INTEGER NOT NULL,
  notes TEXT
);
CREATE TABLE IF NOT EXISTS project_metadata (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

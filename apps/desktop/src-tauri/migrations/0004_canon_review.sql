PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS canon_entries (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  entry_kind TEXT NOT NULL CHECK (
    entry_kind IN (
      'RULE',
      'MECHANIC',
      'DEFINITION',
      'COMPONENT',
      'PROCEDURE',
      'DECKBUILDING',
      'VISUAL_SPEC',
      'OPEN_POINT',
      'RISK',
      'OTHER'
    )
  ),
  canonical_status TEXT NOT NULL CHECK (
    canonical_status IN (
      'CANONICO',
      'ALPHA_DA_TESTARE',
      'IPOTESI_LINEA_GUIDA',
      'MAYBE',
      'RISCHIO',
      'SCARTATO_SUPERATO',
      'PUNTO_APERTO'
    )
  ),
  normalized_text TEXT NOT NULL,
  lifecycle_status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (
    lifecycle_status IN ('ACTIVE', 'SUPERSEDED', 'RETIRED')
  ),
  approved_by TEXT NOT NULL,
  approved_at TEXT NOT NULL,
  rationale TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS canon_entry_sources (
  entry_id TEXT NOT NULL,
  draft_id TEXT NOT NULL UNIQUE,
  source_order INTEGER NOT NULL,
  PRIMARY KEY (entry_id, draft_id),
  FOREIGN KEY (entry_id) REFERENCES canon_entries(id) ON DELETE RESTRICT,
  FOREIGN KEY (draft_id) REFERENCES canon_normalized_drafts(id) ON DELETE RESTRICT,
  UNIQUE (entry_id, source_order)
);

CREATE TABLE IF NOT EXISTS canon_review_decisions (
  id TEXT PRIMARY KEY,
  decision_type TEXT NOT NULL CHECK (decision_type IN ('APPROVED', 'REJECTED')),
  draft_id TEXT NOT NULL UNIQUE,
  entry_id TEXT,
  reviewer TEXT NOT NULL,
  rationale TEXT NOT NULL,
  decided_at TEXT NOT NULL,
  previous_review_status TEXT NOT NULL,
  resulting_review_status TEXT NOT NULL CHECK (
    resulting_review_status IN ('APPROVED', 'MERGED_INTO_ENTRY', 'REJECTED')
  ),
  FOREIGN KEY (draft_id) REFERENCES canon_normalized_drafts(id) ON DELETE RESTRICT,
  FOREIGN KEY (entry_id) REFERENCES canon_entries(id) ON DELETE RESTRICT,
  CHECK (
    (decision_type = 'APPROVED' AND entry_id IS NOT NULL)
    OR (decision_type = 'REJECTED' AND entry_id IS NULL)
  )
);

CREATE INDEX IF NOT EXISTS idx_canon_entries_status_kind
  ON canon_entries (canonical_status, entry_kind, approved_at DESC);

CREATE INDEX IF NOT EXISTS idx_canon_entry_sources_entry_order
  ON canon_entry_sources (entry_id, source_order);

CREATE INDEX IF NOT EXISTS idx_canon_review_decisions_time
  ON canon_review_decisions (decided_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_canon_drafts_review_status
  ON canon_normalized_drafts (review_status, import_run_id);

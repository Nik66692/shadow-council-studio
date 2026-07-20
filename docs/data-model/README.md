# Data model

## Implemented foundation

- `source_documents`: immutable source metadata, versioned path and SHA-256.
- `project_metadata`: local application metadata.
- `HealthStatus`: desktop diagnostics contract.

## Phase 1 canonical import evidence

Phase 1 stores evidence for review, not approved canon.

### `canon_import_runs`

One deterministic import execution for a source SHA-256 and importer version. Re-importing the same pair returns the existing run instead of duplicating records.

### `canon_raw_blocks`

Document-order blocks extracted from `word/document.xml`. Each record stores:

- deterministic source anchor;
- DOCX part;
- block index and structural kind;
- optional paragraph style metadata;
- original Italian text as extracted;
- SHA-256 of the extracted text.

### `canon_normalized_drafts`

One-to-one read-only review drafts derived from raw blocks. Every draft starts as `PENDING_HUMAN_REVIEW`; `canonical_status` remains `NULL` in Phase 1.

### `canon_import_warnings`

Visible warnings for unsupported, malformed or potentially lossy source structures. Tables are flattened into `TABLE_TEXT` evidence and accompanied by `UNSUPPORTED_TABLE_STRUCTURE` warnings.

## Authority boundary

Raw blocks and normalized drafts do not override the Source of Truth. They are not canonical rules and must not be translated, summarized, interpreted or promoted automatically. Only Niccolò may approve later canonical records.

See `docs/architecture/adr/ADR-0008.md`.

## Planned

Human approval records, CanonEntry, Rule, Ruling, Sphere, Guild, Card, CardVersion, Keyword, Family, Trait, Agenda, Directive, CampaignList, OperationalConfiguration, DeckEntry, RFC, Decision, Risk, Bug, PlaytestSession, PlaytestGame, Metric, Asset and Release.

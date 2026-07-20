# Canon import scaffold

Sprint 0 locates `docs/canon/source/Shadow_Council_Source_of_Truth_v1.3.docx`, computes metadata and SHA-256, and writes a deterministic manifest. It never extracts semantic rules, overwrites curated normalized content, or infers canonical status from formatting.

## Commands

- `pnpm canon:manifest:dry-run`
- `pnpm canon:manifest`

## Phase 1 roadmap

1. Extract paragraphs and tables.
2. Preserve section hierarchy.
3. Produce normalized intermediate JSON.
4. Require explicit human review.
5. Create CanonEntry records.
6. Record source anchors and import version.
7. Generate an import report.
8. Never auto-promote content.

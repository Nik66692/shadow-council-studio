# Shadow Council Studio agent map

Purpose: local-first desktop tooling for the complete Shadow Council design lifecycle. Sprint 0 is complete. Phase 1 adds deterministic canonical-source extraction, import evidence, SQLite review drafts and a read-only inspection UI.

Primary commands: `pnpm install --frozen-lockfile`, `pnpm dev`, `pnpm check`, `pnpm test:rust`, `pnpm canon:manifest:dry-run`.

Source hierarchy: Source of Truth v1.3, approved Decision Records, approved RFC outcomes, derived rulebook, normalized DB records, historical docs, chats/drafts. Lower levels never override higher levels.

Canon rule: never invent, translate, summarize, promote or complete game canon. Status values must stay: CANONICO, ALPHA_DA_TESTARE, IPOTESI_LINEA_GUIDA, MAYBE, RISCHIO, SCARTATO_SUPERATO, PUNTO_APERTO. Only Niccolò can promote canon.

Phase 1 boundary: raw DOCX blocks and normalized drafts are evidence for human review. Imported drafts must remain `PENDING_HUMAN_REVIEW` with no assigned canonical status. Unsupported structures must produce visible warnings rather than silent data loss.

Conventions: code and technical docs in English; UI and canon terminology in Italian; no telemetry, cloud, auth, AI API or secrets.

Commit policy: use a task-specific branch such as `feat/canon-import-phase-1`; use Conventional Commits; inspect diffs before commits; do not commit generated secrets or local databases.

Quality gates: formatting, lint, typecheck, TS tests, Rust fmt, clippy, Rust tests, frontend build and canon manifest dry-run.

Specialized instructions: read `skills/repository-bootstrap/SKILL.md` for tooling/root changes, `skills/canon-governance/SKILL.md` for canon work, `skills/domain-modeling/SKILL.md` for domain contracts, `skills/database-migrations/SKILL.md` for SQLite, `skills/testing-and-ci/SKILL.md` for tests/CI, and `skills/release-management/SKILL.md` for releases.

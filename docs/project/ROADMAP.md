# Roadmap

## Sprint 0: repository foundation — COMPLETE

Goal: testable foundation. Scope: tooling, docs, domain contracts, SQLite metadata, diagnostics and CI. Acceptance completed through merged PR #1 with all quality gates passing.

## Phase 1: canonical data model and deterministic import — COMPLETE

Goal: human-reviewable import pipeline. Scope: manifest-selected DOCX extraction, deterministic source anchors, raw evidence, normalized review drafts, visible warnings, SQLite persistence and read-only review UI. Exclusions: automatic interpretation, canonical promotion and editing. Dependencies: Sprint 0. Acceptance completed through merged PR #3.

## Database Studio 0.2 — ACTIVE

Goal: make the local relational model visible and safely manageable. Scope: dynamic schema inspection, ER-style relationship navigation, paginated data browsing, JSON/CSV exports, integrity checks, backups and allowlisted audited metadata/review-note edits. Exclusions: arbitrary SQL writes, schema editing, canon promotion and card editing. Dependencies: Phase 1.

## Phase 1.5: controlled canon review

Goal: approve, reject, group and annotate imported evidence without losing provenance. Scope: explicit human review workflow and append-only decisions. Exclusions: automatic canonical promotion. Dependencies: Phase 1 and Database Studio safety boundaries.

## Phase 2: read-only living Codex

Goal: browse human-approved canon. Scope: source-linked read-only UI. Exclusions: editing. Dependencies: Phase 1.5. Acceptance: every displayed canonical entry links to approved source evidence.

## Phase 3: card database and card versioning

Goal: track cards and versions. Scope: card/version records and portable JSON import. Exclusions: renderer. Dependencies: Phase 1. Acceptance: append-only history.

## Phase 4: RFC, decisions, risks and playtests

Goal: governance workflows. Scope: RFCs, decisions, risk/bug/playtest records. Exclusions: analytics automation. Dependencies: Phase 3. Acceptance: traceable decisions.

## Phase 5: decklists, Campaign Lists and configurations

Goal: deck/config tracking. Scope: Campaign Lists, operational configurations, deck entries. Exclusions: matchup analytics. Dependencies: Phase 3. Acceptance: validated lists.

## Phase 6: assets and card-rendering pipeline

Goal: asset management and rendering. Scope: prompts, assets, renderer. Exclusions: final art identity if undecided. Dependencies: Phase 3. Acceptance: reproducible preview exports.

## Phase 7: release and print pipeline

Goal: package print/playtest releases. Scope: release notes, artifacts, print export. Exclusions: signed installers. Dependencies: Phase 6. Acceptance: reproducible release bundle.

## Phase 8: optional AI-assisted workflows

Goal: approved local/optional assistance. Scope: opt-in workflows. Exclusions: autonomous design decisions. Dependencies: security decision. Acceptance: no secret leakage and human review.

## Phase 9: generated public website

Goal: publish selected public info. Scope: static website. Exclusions: private canon leakage. Dependencies: release process. Acceptance: explicit content allowlist.

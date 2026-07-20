# Roadmap

## Sprint 0: repository foundation

Goal: testable foundation. Scope: tooling, docs, domain contracts, SQLite metadata, diagnostics, CI. Exclusions: semantic import and CRUD. Dependencies: none. Acceptance: quality gates pass.

## Phase 1: canonical data model and deterministic import

Goal: reviewed import pipeline. Scope: extract structure and CanonEntry drafts. Exclusions: auto-promotion. Dependencies: Sprint 0. Acceptance: human-reviewable import reports.

## Phase 2: read-only living Codex

Goal: browse approved canon. Scope: read-only UI. Exclusions: editing. Dependencies: Phase 1. Acceptance: source-linked entries.

## Phase 3: card database and card versioning

Goal: track cards and versions. Scope: card/version records. Exclusions: renderer. Dependencies: Phase 1. Acceptance: append-only history.

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

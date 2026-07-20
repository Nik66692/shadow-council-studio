# Shadow Council Studio

Shadow Council Studio is a local-first desktop application for managing the lifecycle of the original tabletop card game Shadow Council.

Current maturity: **Phase 1.5 — Controlled Canon Review**.

Sprint 0 established the tested Tauri, React, Rust and SQLite foundation. Phase 1 added deterministic extraction from the manifest-selected Source of Truth. Phase 1.5 adds the explicit human approval boundary required before the Living Codex.

The application is not yet a card editor, published Living Codex, deck builder, playtest tracker, print pipeline, cloud app or AI workflow.

## Current capabilities

- verifies `docs/canon/source/manifest.json`;
- resolves the approved versioned DOCX;
- verifies the source SHA-256 before parsing;
- extracts headings, paragraphs, list items and flattened table text from `word/document.xml`;
- stores deterministic raw blocks and `PENDING_HUMAN_REVIEW` drafts in SQLite;
- persists visible warnings for unsupported or potentially lossy structures;
- prevents duplicate imports for the same source hash and importer version;
- provides a controlled Canon Review queue with filtering and multi-selection;
- approves one or more drafts into one classified canon entry;
- rejects drafts without creating canon entries;
- preserves immutable source text, anchors, hashes and ordered provenance;
- records reviewer, rationale and timestamp for every decision;
- prevents automatic status inference and duplicate review;
- exposes schema, relationships, data browsing, exports, backups and audited safe edits through Database Studio.

## Technology

Tauri 2, Rust stable, SQLite via SQLx, React, TypeScript strict mode, Vite, pnpm workspaces, Vitest, React Testing Library, ESLint and Prettier.

## Repository map

- `apps/desktop`: Tauri and React application.
- `packages/domain`: DTOs, stable IDs, status taxonomy and review contracts.
- `packages/ui`: shared UI constants.
- `docs`: governance, canon, architecture, ADRs and templates.
- `skills`: repository playbooks for future agents.
- `tools/canon-import`: deterministic Source of Truth manifest tooling.

## Prerequisites

Node 22.17.0, pnpm 10.13.1, Rust 1.88.0 with rustfmt/clippy, platform Tauri prerequisites and an available system DOCX archive reader (`PowerShell` on Windows or `unzip` on Unix-like development systems).

## Installation

```sh
pnpm install --frozen-lockfile
```

## Development commands

```sh
pnpm dev
pnpm desktop:dev
```

Inside the desktop app:

1. Open **Import canonico** and select **Esegui import verificato**.
2. Open **Canon Review** to inspect immutable evidence.
3. Select one or more pending drafts.
4. Approve them as one canon entry or reject them with a mandatory rationale.
5. Inspect the approved registry and decision log.

## Testing commands

```sh
pnpm test
pnpm test:rust
pnpm test:all
pnpm check
```

## Build commands

```sh
pnpm build
```

## Source hierarchy

See `docs/project/SOURCE_HIERARCHY.md`. `docs/canon/source/manifest.json` selects the approved current source. The current v1.3 file is `docs/canon/source/v1.3/Shadow_Council_Source_of_Truth_v1.3.docx`.

Imported raw blocks and normalized drafts are immutable review evidence only. Approval creates a separate canon entry linked to every source draft; it never rewrites the Source of Truth or imported evidence.

## Contribution workflow

Use issue-first work for non-trivial changes, task-specific branches, Conventional Commits, PR review, documented tests and no silent canon changes. See `CONTRIBUTING.md` and `AGENTS.md`.

## Data and privacy

Local-first SQLite only. No telemetry, analytics, accounts, remote database, cloud dependency, authentication, AI provider or secret keys.

## Roadmap summary

Sprint 0 foundation, Phase 1 deterministic import and Database Studio are complete. Phase 1.5 establishes controlled human approval. Phase 2 will publish a read-only Living Codex over approved records. Phase 3 adds the card database and card versioning. See `docs/project/ROADMAP.md`.

## Links

- Architecture: `docs/architecture/README.md`
- Canon: `docs/canon/README.md`
- Data model: `docs/data-model/README.md`
- Governance: `docs/project/SOURCE_HIERARCHY.md`

## Windows Preview 0.3

Preview 0.3 packages the deterministic canonical importer, Database Studio and controlled Canon Review as an unsigned local-first Windows NSIS installer. It includes explicit approval and rejection, ordered source provenance, the approved canon registry and immutable decision history. The installed application uses the bundled manifest and Source of Truth v1.3; development builds may use repository resources. Windows may show SmartScreen because internal previews are not code-signed.

## Windows Preview 0.3.1

Hotfix 0.3.1 replaces external PowerShell/unzip DOCX extraction with an in-process Rust ZIP reader and surfaces detailed Tauri import errors. Canonical content remains unchanged.

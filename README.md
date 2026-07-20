# Shadow Council Studio

Shadow Council Studio is a local-first desktop application for managing the lifecycle of the original tabletop card game Shadow Council.

Current maturity: **Phase 1 — Canonical Import** with **Windows Preview 0.1** packaging in progress.

Sprint 0 established the tested Tauri, React, Rust and SQLite foundation. Phase 1 adds deterministic extraction from the manifest-selected Source of Truth, import evidence stored in SQLite and a read-only human-review screen.

The application is not yet a card editor, approved living Codex, deck builder, playtest tracker, print pipeline, cloud app or AI workflow.

## Current capabilities

- verifies `docs/canon/source/manifest.json`;
- resolves the approved versioned DOCX;
- verifies the source SHA-256 before parsing;
- extracts headings, paragraphs, list items and flattened table text from `word/document.xml`;
- stores deterministic raw blocks and `PENDING_HUMAN_REVIEW` drafts in SQLite;
- persists visible warnings for unsupported or potentially lossy structures;
- prevents duplicate imports for the same source hash and importer version;
- exposes a read-only review snapshot through typed Tauri commands;
- never assigns canonical status automatically.

## Windows Preview 0.1

The Windows preview is an unsigned internal build produced by `.github/workflows/windows-preview.yml`.

The installer bundles the immutable canonical manifest and the approved Source of Truth v1.3. Development builds prefer the repository copies; installed builds resolve the same paths from the Tauri resource directory. The SHA-256 check remains mandatory in both modes.

The workflow uploads:

- the NSIS `*-setup.exe` installer;
- `SHA256SUMS.txt` for integrity verification.

Because the preview is not code-signed, Windows SmartScreen may display an unknown-publisher warning. Review the checksum before installation and use the preview only as an internal development build. No updater, telemetry, cloud service, account or signing secret is included.

## Technology

Tauri 2, Rust stable, SQLite via SQLx, React, TypeScript strict mode, Vite, pnpm workspaces, Vitest, React Testing Library, ESLint and Prettier.

## Repository map

- `apps/desktop`: Tauri and React application.
- `packages/domain`: DTOs, stable IDs, status taxonomy and import-review contracts.
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

Inside the desktop app, open **Import canonico** and select **Esegui import verificato**.

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

A local Windows NSIS build additionally requires Tauri CLI 2.7.0:

```sh
cargo install tauri-cli --version 2.7.0 --locked
cd apps/desktop/src-tauri
cargo tauri build --bundles nsis
```

## Source hierarchy

See `docs/project/SOURCE_HIERARCHY.md`. `docs/canon/source/manifest.json` selects the approved current source. The current v1.3 file is `docs/canon/source/v1.3/Shadow_Council_Source_of_Truth_v1.3.docx`.

Imported raw blocks and normalized drafts are review evidence only. They never override the Source of Truth and remain without canonical status until a later explicit approval by Niccolò.

## Contribution workflow

Use issue-first work for non-trivial changes, task-specific branches, Conventional Commits, PR review, documented tests and no silent canon changes. See `CONTRIBUTING.md` and `AGENTS.md`.

## Data and privacy

Local-first SQLite only. No telemetry, analytics, accounts, remote database, cloud dependency, authentication, AI provider or secret keys.

## Roadmap summary

Sprint 0 foundation and Phase 1 deterministic canon import are complete. Preview 0.1 makes the Phase 1 application installable on Windows. Phase 1.5 will add controlled human approval; Phase 2 will add a read-only living Codex over approved records. Phase 3 adds the card database and card versioning. See `docs/project/ROADMAP.md`.

## Links

- Architecture: `docs/architecture/README.md`
- Canon: `docs/canon/README.md`
- Data model: `docs/data-model/README.md`
- Governance: `docs/project/SOURCE_HIERARCHY.md`

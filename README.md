# Shadow Council Studio

Shadow Council Studio is a future local-first desktop application for managing the lifecycle of the original tabletop card game Shadow Council.

Current maturity: **Sprint 0 / Foundation**. It is not yet a card editor, Codex, rule extractor, deck builder, playtest tracker, print pipeline, cloud app or AI workflow.

## Screenshots

Pending. Sprint 0 provides a restrained diagnostic shell only.

## Technology

Tauri 2, Rust stable, SQLite via SQLx, React, TypeScript strict mode, Vite, pnpm workspaces, Vitest, React Testing Library, ESLint and Prettier.

## Repository map

- `apps/desktop`: Tauri and React application.
- `packages/domain`: DTOs, stable IDs and canonical status validation.
- `packages/ui`: small shared UI constants.
- `docs`: governance, canon, architecture, ADRs and templates.
- `skills`: repository playbooks for future agents.
- `tools/canon-import`: deterministic Source of Truth manifest scaffold.

## Prerequisites

Node 22.17.0, pnpm 10.13.1, Rust 1.88.0 with rustfmt/clippy, and platform Tauri prerequisites.

## Installation

```sh
pnpm install --frozen-lockfile
```

## Development commands

```sh
pnpm dev
pnpm desktop:dev
```

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

See `docs/project/SOURCE_HIERARCHY.md`. `docs/canon/source/Shadow_Council_Source_of_Truth_v1.3.docx` is immutable primary source material when present. Sprint 0 records metadata and hashes only.

## Contribution workflow

Use issue-first work for non-trivial changes, Conventional Commits, PR review, documented tests and no silent canon changes. See `CONTRIBUTING.md`.

## Data and privacy

Local-first SQLite only. No telemetry, analytics, accounts, remote database, cloud dependency, authentication, AI provider or secret keys.

## Roadmap summary

Sprint 0 foundation; Phase 1 deterministic canon import; Phase 2 read-only Codex; later phases add cards, decisions, playtests, assets, releases, optional AI and website generation. See `docs/project/ROADMAP.md`.

## Links

- Architecture: `docs/architecture/README.md`
- Canon: `docs/canon/README.md`
- Governance: `docs/project/SOURCE_HIERARCHY.md`

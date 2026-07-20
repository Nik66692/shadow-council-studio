# ADR-0009 — Database Studio safety boundaries

- Status: Accepted for implementation
- Date: 2026-07-20
- Decision owner: Niccolò

## Context

Shadow Council Studio already persists canonical import evidence in SQLite, but the database is not yet inspectable or manageable from inside the application. A general-purpose SQL editor would expose immutable source evidence, hashes, foreign keys and future canon records to accidental corruption.

## Decision

Database Studio is a first-party application module with three explicit boundaries.

### 1. Dynamic inspection

The module discovers user tables, columns, indexes and foreign keys from SQLite metadata (`sqlite_master` and PRAGMA queries). The current schema is not hardcoded into the browser. Known migration provenance may be attached as explanatory metadata, but it does not control discovery.

### 2. Read-only generic browsing

All discovered user tables may be browsed through validated, paginated queries. Table and column identifiers must be selected from discovered metadata before they are interpolated into SQL. Search values, filters and mutation values are always bound parameters.

Generic browsing may sort, search, filter, navigate relationships and export records. It may not issue arbitrary SQL or modify generic rows.

### 3. Allowlisted management

Writes are available only through dedicated service commands with explicit validation and transactions. Version 0.2 permits:

- changing the `value` of existing allowlisted `project_metadata` keys;
- creating or updating a human review note linked to an imported draft.

The following remain immutable through Database Studio:

- source documents and their hashes;
- raw imported text;
- source anchors;
- import-run evidence;
- canonical status and approval state;
- primary keys and foreign keys.

Every successful write records entity, record identifier, field, previous value, new value, reason and timestamp in `database_audit_log`.

## Backup and integrity

Database Studio exposes SQLite integrity checks and local backup creation. Backups are created in the application data directory after a checkpoint and are never uploaded automatically.

## Consequences

- The user can understand and navigate the real relational model without external SQLite tooling.
- Later card, deck and playtest migrations appear automatically in Schema Explorer.
- Schema changes still require reviewed SQL migrations and cannot be authored from the UI.
- Canon approval remains a separate future workflow; Database Studio cannot silently promote imported evidence.

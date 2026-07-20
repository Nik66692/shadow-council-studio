# Contributing

Use branches like `type/short-topic` (example: `bootstrap/foundation`). Non-trivial work should begin with an issue, RFC or decision record. Commits must follow Conventional Commits.

Pull requests must state objective, scope, exclusions, linked issue/RFC, canon impact, migration impact, tests, screenshots if UI changed, risks and rollback.

Definition of Done: code implemented, tests added/updated, docs updated, migrations reviewed, quality gates passing, no secrets, no silent canon changes.

Migrations are forward-only SQLx migrations. Use isolated test databases and document rollback strategy.

Never silently change, reinterpret, translate or promote canon. Only Niccolò may promote content to canonical status.

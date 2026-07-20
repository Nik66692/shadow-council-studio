# testing-and-ci skill

## Purpose

Guide safe testing-and-ci work in Shadow Council Studio.

## When to use it

Use before changing repository areas matching this skill.

## Required inputs

Task objective, affected files, relevant ADRs, source hierarchy impact and expected validation commands.

## Procedure

1. Read root AGENTS.md and local AGENTS.md.
2. Inspect current files and git status.
3. Make the smallest reversible change.
4. Preserve canon authority and status taxonomy.
5. Update documentation when behavior changes.
6. Run relevant checks.

## Expected outputs

Coherent code or documentation, tests where applicable, and notes for review.

## Guardrails

No secrets, telemetry, cloud dependency, AI integration, autonomous canon promotion or silent rule invention.

## Validation checklist

Formatting, lint/typecheck/tests for touched areas, Rust fmt/clippy/tests for Rust, and canon manifest dry-run for canon-source work.

## Failure and rollback behavior

Stop on data-loss risk, revert partial generated output, document failing commands, and never fabricate results.

## Links

- docs/project/SOURCE_HIERARCHY.md
- docs/architecture/README.md
- CONTRIBUTING.md

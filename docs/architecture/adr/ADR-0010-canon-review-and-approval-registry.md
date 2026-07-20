# ADR-0010 — Controlled canon review and approval registry

- Status: Accepted for Phase 1.5 implementation
- Date: 2026-07-20
- Decision owner: Niccolò
- Supersedes: none

## Context

Phase 1 imports the approved Source of Truth as immutable evidence and creates normalized drafts with `PENDING_HUMAN_REVIEW`. Those drafts are not canon. A separate boundary is required before a Living Codex can consume approved material.

The review system must support editorial composition without rewriting history. Several adjacent DOCX blocks may describe one rule, while other blocks may be headings, commentary or duplicated context that should not become standalone entries.

## Decision

Introduce three review-layer entities:

1. `canon_entries` stores the human-approved normalized expression, classification and canonical status.
2. `canon_entry_sources` links one approved entry to one or more immutable imported drafts in explicit source order.
3. `canon_review_decisions` records one immutable approval or rejection decision for every reviewed draft.

Imported evidence remains unchanged. Approval updates only review metadata on `canon_normalized_drafts`; it never changes `original_text`, `source_anchor`, `raw_block_id` or hashes.

Approval of multiple drafts creates one `canon_entry` and links every source draft. Rejection creates decisions but no canon entry. A draft can be reviewed only once in Phase 1.5.

Every write requires a reviewer and rationale and is executed inside one SQLite transaction. Guarded updates prevent double review or partial state if the draft changed concurrently.

## Entry classification

Phase 1.5 uses a deliberately small controlled vocabulary:

- `RULE`
- `MECHANIC`
- `DEFINITION`
- `COMPONENT`
- `PROCEDURE`
- `DECKBUILDING`
- `VISUAL_SPEC`
- `OPEN_POINT`
- `RISK`
- `OTHER`

Canonical status uses the existing governance vocabulary without inference:

- `CANONICO`
- `ALPHA_DA_TESTARE`
- `IPOTESI_LINEA_GUIDA`
- `MAYBE`
- `RISCHIO`
- `SCARTATO_SUPERATO`
- `PUNTO_APERTO`

The reviewer selects both values explicitly.

## Safety boundaries

- No automatic approval, rejection, merging, classification or status inference.
- No mutation of Source of Truth files or imported evidence.
- No arbitrary SQL write surface.
- No deletion or rewriting of review decisions.
- No Living Codex publishing in this phase.
- No card database coupling.

## Consequences

The application gains a reliable approved-canon registry suitable as the input boundary for Phase 2. Editorial normalization is possible while exact source provenance remains inspectable. Reconsideration and supersession workflows are intentionally deferred to the Living Codex/versioning phase rather than implemented as destructive undo operations.

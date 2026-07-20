# Source hierarchy

1. The manifest-selected Source of Truth version in `docs/canon/source/manifest.json`, currently v1.3, and later explicitly approved manifest updates.
2. Approved Decision Records.
3. Approved RFC outcomes.
4. Rulebook derived from canon.
5. Normalized database records.
6. Historical documents.
7. Chats, drafts and brainstorming.

Lower levels must never silently override higher levels. Conflicts require an RFC or decision record and human approval.


Published source versions are immutable and live in versioned directories. The manifest, not the highest numbered directory, selects the approved current version. Only Niccolò can approve a new current version.

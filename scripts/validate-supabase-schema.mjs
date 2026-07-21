import { readFile } from "node:fs/promises";

const migrationPaths = [
  "supabase/migrations/20260721123000_phase_1_6_cloud_foundation.sql",
  "supabase/migrations/20260721124500_harden_evidence_and_sync_identity.sql",
];
const migrations = await Promise.all(
  migrationPaths.map((path) => readFile(path, "utf8")),
);
const migration = migrations.join("\n");

const requiredFragments = [
  "create table public.workspaces",
  "create table public.workspace_members",
  "create table public.source_documents",
  "create table public.canon_entries",
  "create table public.sync_change_log",
  "create or replace function public.create_workspace",
  "alter table public.workspaces enable row level security",
  "public.is_workspace_member(workspace_id)",
  "public.has_workspace_role(workspace_id",
  "revoke all on all tables in schema public from anon",
  "source_documents_update_non_immutable",
  "protect_canon_draft_provenance",
  "revoke delete on public.canon_normalized_drafts",
  "sync_devices_insert_self",
  "database_audit_log_insert_authenticated_actor",
  "sync_change_log_insert_authenticated_actor",
  "Workspace identity fields are immutable",
  "Canonical draft provenance and original text are immutable",
];

const missing = requiredFragments.filter(
  (fragment) => !migration.includes(fragment),
);
if (missing.length > 0) {
  throw new Error(
    `Supabase migrations are missing required safeguards:\n${missing.join("\n")}`,
  );
}

const forbiddenFragments = [
  "grant service_role",
  "to service_role",
  "disable row level security",
];
const forbidden = forbiddenFragments.filter((fragment) =>
  migration.toLowerCase().includes(fragment),
);
if (forbidden.length > 0) {
  throw new Error(
    `Supabase migrations contain forbidden fragments:\n${forbidden.join("\n")}`,
  );
}

console.log(
  `Supabase schema validation passed for ${migrationPaths.join(", ")}: RLS, workspace isolation, immutable evidence and authenticated actor boundaries are present.`,
);

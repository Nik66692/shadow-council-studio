import { readFile } from "node:fs/promises";

const migrationPath =
  "supabase/migrations/20260721123000_phase_1_6_cloud_foundation.sql";
const migration = await readFile(migrationPath, "utf8");

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
];

const missing = requiredFragments.filter(
  (fragment) => !migration.includes(fragment),
);
if (missing.length > 0) {
  throw new Error(
    `Supabase migration is missing required safeguards:\n${missing.join("\n")}`,
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
    `Supabase migration contains forbidden fragments:\n${forbidden.join("\n")}`,
  );
}

console.log(
  `Supabase schema validation passed for ${migrationPath}: RLS, workspace isolation and client-key boundaries are present.`,
);

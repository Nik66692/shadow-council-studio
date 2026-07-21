create extension if not exists pgcrypto with schema extensions;

create or replace function public.set_updated_at_and_version()
returns trigger
language plpgsql
set search_path = public, pg_temp
as $$
begin
  new.updated_at = timezone('utc', now());
  if tg_op = 'UPDATE' then
    new.row_version = old.row_version + 1;
  end if;
  return new;
end;
$$;

create table public.workspaces (
  id uuid primary key default extensions.gen_random_uuid(),
  name text not null check (char_length(trim(name)) between 1 and 120),
  slug text not null unique check (slug ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$'),
  created_by uuid not null references auth.users(id) on delete restrict,
  row_version bigint not null default 1,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now())
);

create table public.workspace_members (
  workspace_id uuid not null references public.workspaces(id) on delete cascade,
  user_id uuid not null references auth.users(id) on delete cascade,
  role text not null check (role in ('OWNER', 'EDITOR', 'VIEWER')),
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, user_id)
);

create or replace function public.is_workspace_member(target_workspace_id uuid)
returns boolean
language sql
stable
security definer
set search_path = public, pg_temp
as $$
  select exists (
    select 1
    from public.workspace_members member
    where member.workspace_id = target_workspace_id
      and member.user_id = auth.uid()
  );
$$;

create or replace function public.has_workspace_role(
  target_workspace_id uuid,
  allowed_roles text[]
)
returns boolean
language sql
stable
security definer
set search_path = public, pg_temp
as $$
  select exists (
    select 1
    from public.workspace_members member
    where member.workspace_id = target_workspace_id
      and member.user_id = auth.uid()
      and member.role = any(allowed_roles)
  );
$$;

create or replace function public.create_workspace(
  workspace_name text,
  workspace_slug text
)
returns uuid
language plpgsql
security definer
set search_path = public, pg_temp
as $$
declare
  new_workspace_id uuid;
begin
  if auth.uid() is null then
    raise exception 'Authentication is required';
  end if;
  if char_length(trim(workspace_name)) = 0 then
    raise exception 'Workspace name is required';
  end if;
  if workspace_slug !~ '^[a-z0-9]+(?:-[a-z0-9]+)*$' then
    raise exception 'Workspace slug is invalid';
  end if;

  insert into public.workspaces (name, slug, created_by)
  values (trim(workspace_name), workspace_slug, auth.uid())
  returning id into new_workspace_id;

  insert into public.workspace_members (workspace_id, user_id, role)
  values (new_workspace_id, auth.uid(), 'OWNER');

  return new_workspace_id;
end;
$$;

create table public.source_documents (
  workspace_id uuid not null references public.workspaces(id) on delete cascade,
  id text not null,
  title text not null,
  version text not null,
  authority_rank integer not null,
  original_path text not null,
  sha256 text check (sha256 is null or sha256 ~ '^[0-9a-f]{64}$'),
  imported_at timestamptz not null,
  immutable boolean not null default true,
  notes text,
  row_version bigint not null default 1,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, id)
);

create table public.canon_import_runs (
  workspace_id uuid not null,
  id text not null,
  source_document_id text not null,
  source_version text not null,
  source_sha256 text not null check (source_sha256 ~ '^[0-9a-f]{64}$'),
  importer_version text not null,
  status text not null,
  started_at timestamptz not null,
  completed_at timestamptz not null,
  raw_block_count integer not null check (raw_block_count >= 0),
  draft_count integer not null check (draft_count >= 0),
  warning_count integer not null check (warning_count >= 0),
  created_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, id),
  foreign key (workspace_id, source_document_id)
    references public.source_documents(workspace_id, id) on delete restrict,
  unique (workspace_id, source_sha256, importer_version)
);

create table public.canon_raw_blocks (
  workspace_id uuid not null,
  id text not null,
  import_run_id text not null,
  source_anchor text not null,
  source_part text not null,
  block_index integer not null check (block_index >= 0),
  block_kind text not null,
  style_name text,
  original_text text not null,
  text_sha256 text not null check (text_sha256 ~ '^[0-9a-f]{64}$'),
  created_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, id),
  foreign key (workspace_id, import_run_id)
    references public.canon_import_runs(workspace_id, id) on delete cascade,
  unique (workspace_id, import_run_id, source_anchor)
);

create table public.canon_normalized_drafts (
  workspace_id uuid not null,
  id text not null,
  import_run_id text not null,
  raw_block_id text not null,
  source_anchor text not null,
  draft_kind text not null,
  original_text text not null,
  review_status text not null,
  canonical_status text,
  row_version bigint not null default 1,
  created_at timestamptz not null,
  updated_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, id),
  foreign key (workspace_id, import_run_id)
    references public.canon_import_runs(workspace_id, id) on delete cascade,
  foreign key (workspace_id, raw_block_id)
    references public.canon_raw_blocks(workspace_id, id) on delete cascade,
  unique (workspace_id, raw_block_id)
);

create table public.canon_import_warnings (
  workspace_id uuid not null,
  id text not null,
  import_run_id text not null,
  source_anchor text,
  warning_code text not null,
  message text not null,
  created_at timestamptz not null,
  primary key (workspace_id, id),
  foreign key (workspace_id, import_run_id)
    references public.canon_import_runs(workspace_id, id) on delete cascade
);

create table public.canon_entries (
  workspace_id uuid not null references public.workspaces(id) on delete cascade,
  id text not null,
  title text not null,
  entry_kind text not null check (
    entry_kind in (
      'RULE', 'MECHANIC', 'DEFINITION', 'COMPONENT', 'PROCEDURE',
      'DECKBUILDING', 'VISUAL_SPEC', 'OPEN_POINT', 'RISK', 'OTHER'
    )
  ),
  canonical_status text not null check (
    canonical_status in (
      'CANONICO', 'ALPHA_DA_TESTARE', 'IPOTESI_LINEA_GUIDA', 'MAYBE',
      'RISCHIO', 'SCARTATO_SUPERATO', 'PUNTO_APERTO'
    )
  ),
  normalized_text text not null,
  lifecycle_status text not null default 'ACTIVE' check (
    lifecycle_status in ('ACTIVE', 'SUPERSEDED', 'RETIRED')
  ),
  approved_by text not null,
  approved_at timestamptz not null,
  rationale text not null,
  row_version bigint not null default 1,
  created_at timestamptz not null,
  updated_at timestamptz not null,
  primary key (workspace_id, id)
);

create table public.canon_entry_sources (
  workspace_id uuid not null,
  entry_id text not null,
  draft_id text not null,
  source_order integer not null check (source_order >= 0),
  created_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, entry_id, draft_id),
  foreign key (workspace_id, entry_id)
    references public.canon_entries(workspace_id, id) on delete restrict,
  foreign key (workspace_id, draft_id)
    references public.canon_normalized_drafts(workspace_id, id) on delete restrict,
  unique (workspace_id, draft_id),
  unique (workspace_id, entry_id, source_order)
);

create table public.canon_review_decisions (
  workspace_id uuid not null,
  id text not null,
  decision_type text not null check (decision_type in ('APPROVED', 'REJECTED')),
  draft_id text not null,
  entry_id text,
  reviewer text not null,
  rationale text not null,
  decided_at timestamptz not null,
  previous_review_status text not null,
  resulting_review_status text not null check (
    resulting_review_status in ('APPROVED', 'MERGED_INTO_ENTRY', 'REJECTED')
  ),
  created_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, id),
  foreign key (workspace_id, draft_id)
    references public.canon_normalized_drafts(workspace_id, id) on delete restrict,
  foreign key (workspace_id, entry_id)
    references public.canon_entries(workspace_id, id) on delete restrict,
  unique (workspace_id, draft_id),
  check (
    (decision_type = 'APPROVED' and entry_id is not null)
    or (decision_type = 'REJECTED' and entry_id is null)
  )
);

create table public.database_audit_log (
  workspace_id uuid not null references public.workspaces(id) on delete cascade,
  id text not null,
  entity_type text not null,
  record_id text not null,
  field_name text not null,
  old_value text,
  new_value text,
  reason text not null,
  changed_by uuid references auth.users(id) on delete set null default auth.uid(),
  changed_at timestamptz not null,
  primary key (workspace_id, id)
);

create table public.canon_review_notes (
  workspace_id uuid not null,
  id text not null,
  draft_id text not null,
  note text not null,
  row_version bigint not null default 1,
  created_at timestamptz not null default timezone('utc', now()),
  updated_at timestamptz not null,
  primary key (workspace_id, id),
  foreign key (workspace_id, draft_id)
    references public.canon_normalized_drafts(workspace_id, id) on delete cascade,
  unique (workspace_id, draft_id)
);

create table public.sync_devices (
  workspace_id uuid not null references public.workspaces(id) on delete cascade,
  id uuid not null default extensions.gen_random_uuid(),
  user_id uuid not null references auth.users(id) on delete cascade default auth.uid(),
  device_name text not null,
  client_version text not null,
  last_seen_at timestamptz not null default timezone('utc', now()),
  created_at timestamptz not null default timezone('utc', now()),
  primary key (workspace_id, id)
);

create table public.sync_change_log (
  id bigint generated always as identity primary key,
  workspace_id uuid not null references public.workspaces(id) on delete cascade,
  entity_type text not null,
  entity_id text not null,
  operation text not null check (operation in ('UPSERT', 'DELETE')),
  payload jsonb,
  row_version bigint,
  mutation_key text not null,
  changed_by uuid references auth.users(id) on delete set null default auth.uid(),
  changed_at timestamptz not null default timezone('utc', now()),
  unique (workspace_id, mutation_key)
);

create index canon_import_runs_source_idx
  on public.canon_import_runs (workspace_id, source_sha256, importer_version);
create index canon_raw_blocks_run_idx
  on public.canon_raw_blocks (workspace_id, import_run_id, block_index);
create index canon_drafts_review_idx
  on public.canon_normalized_drafts (workspace_id, review_status, import_run_id);
create index canon_entries_status_idx
  on public.canon_entries (workspace_id, canonical_status, entry_kind, approved_at desc);
create index canon_decisions_time_idx
  on public.canon_review_decisions (workspace_id, decided_at desc, id desc);
create index audit_entity_idx
  on public.database_audit_log (workspace_id, entity_type, record_id, changed_at desc);
create index sync_change_cursor_idx
  on public.sync_change_log (workspace_id, id);

create trigger workspaces_touch
before update on public.workspaces
for each row execute function public.set_updated_at_and_version();
create trigger source_documents_touch
before update on public.source_documents
for each row execute function public.set_updated_at_and_version();
create trigger canon_drafts_touch
before update on public.canon_normalized_drafts
for each row execute function public.set_updated_at_and_version();
create trigger canon_entries_touch
before update on public.canon_entries
for each row execute function public.set_updated_at_and_version();
create trigger canon_review_notes_touch
before update on public.canon_review_notes
for each row execute function public.set_updated_at_and_version();

alter table public.workspaces enable row level security;
alter table public.workspace_members enable row level security;

create policy workspaces_select_member
on public.workspaces for select
to authenticated
using (public.is_workspace_member(id));

create policy workspaces_update_owner
on public.workspaces for update
to authenticated
using (public.has_workspace_role(id, array['OWNER']))
with check (public.has_workspace_role(id, array['OWNER']));

create policy workspace_members_select_member
on public.workspace_members for select
to authenticated
using (public.is_workspace_member(workspace_id));

create policy workspace_members_insert_owner
on public.workspace_members for insert
to authenticated
with check (public.has_workspace_role(workspace_id, array['OWNER']));

create policy workspace_members_update_owner
on public.workspace_members for update
to authenticated
using (public.has_workspace_role(workspace_id, array['OWNER']))
with check (public.has_workspace_role(workspace_id, array['OWNER']));

create policy workspace_members_delete_owner
on public.workspace_members for delete
to authenticated
using (public.has_workspace_role(workspace_id, array['OWNER']));

do $$
declare
  table_name text;
begin
  foreach table_name in array array[
    'source_documents',
    'canon_import_runs',
    'canon_raw_blocks',
    'canon_normalized_drafts',
    'canon_import_warnings',
    'canon_entries',
    'canon_review_notes',
    'sync_devices'
  ] loop
    execute format('alter table public.%I enable row level security', table_name);
    execute format(
      'create policy %I on public.%I for select to authenticated using (public.is_workspace_member(workspace_id))',
      table_name || '_select_member', table_name
    );
    execute format(
      'create policy %I on public.%I for insert to authenticated with check (public.has_workspace_role(workspace_id, array[''OWNER'',''EDITOR'']))',
      table_name || '_insert_editor', table_name
    );
    execute format(
      'create policy %I on public.%I for update to authenticated using (public.has_workspace_role(workspace_id, array[''OWNER'',''EDITOR''])) with check (public.has_workspace_role(workspace_id, array[''OWNER'',''EDITOR'']))',
      table_name || '_update_editor', table_name
    );
    execute format(
      'create policy %I on public.%I for delete to authenticated using (public.has_workspace_role(workspace_id, array[''OWNER'',''EDITOR'']))',
      table_name || '_delete_editor', table_name
    );
  end loop;
end;
$$;

do $$
declare
  table_name text;
begin
  foreach table_name in array array[
    'canon_entry_sources',
    'canon_review_decisions',
    'database_audit_log',
    'sync_change_log'
  ] loop
    execute format('alter table public.%I enable row level security', table_name);
    execute format(
      'create policy %I on public.%I for select to authenticated using (public.is_workspace_member(workspace_id))',
      table_name || '_select_member', table_name
    );
    execute format(
      'create policy %I on public.%I for insert to authenticated with check (public.has_workspace_role(workspace_id, array[''OWNER'',''EDITOR'']))',
      table_name || '_insert_editor', table_name
    );
  end loop;
end;
$$;

revoke all on all tables in schema public from anon;
revoke all on all sequences in schema public from anon;

grant usage on schema public to authenticated;
grant select, update on public.workspaces to authenticated;
grant select, insert, update, delete on public.workspace_members to authenticated;
grant select, insert, update, delete on public.source_documents to authenticated;
grant select, insert, update, delete on public.canon_import_runs to authenticated;
grant select, insert, update, delete on public.canon_raw_blocks to authenticated;
grant select, insert, update, delete on public.canon_normalized_drafts to authenticated;
grant select, insert, update, delete on public.canon_import_warnings to authenticated;
grant select, insert, update, delete on public.canon_entries to authenticated;
grant select, insert on public.canon_entry_sources to authenticated;
grant select, insert on public.canon_review_decisions to authenticated;
grant select, insert on public.database_audit_log to authenticated;
grant select, insert, update, delete on public.canon_review_notes to authenticated;
grant select, insert, update, delete on public.sync_devices to authenticated;
grant select, insert on public.sync_change_log to authenticated;
grant usage, select on sequence public.sync_change_log_id_seq to authenticated;
grant execute on function public.create_workspace(text, text) to authenticated;
grant execute on function public.is_workspace_member(uuid) to authenticated;
grant execute on function public.has_workspace_role(uuid, text[]) to authenticated;

create or replace function public.prevent_workspace_identity_change()
returns trigger
language plpgsql
set search_path = public, pg_temp
as $$
begin
  if new.id is distinct from old.id
     or new.created_by is distinct from old.created_by
     or new.created_at is distinct from old.created_at then
    raise exception 'Workspace identity fields are immutable';
  end if;
  return new;
end;
$$;

create trigger workspaces_protect_identity
before update on public.workspaces
for each row execute function public.prevent_workspace_identity_change();

-- Immutable source documents can be inserted but cannot be rewritten or deleted.
drop policy if exists source_documents_update_editor on public.source_documents;
drop policy if exists source_documents_delete_editor on public.source_documents;

create policy source_documents_update_non_immutable
on public.source_documents for update
to authenticated
using (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and not immutable
)
with check (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and not immutable
);

create policy source_documents_delete_non_immutable
on public.source_documents for delete
to authenticated
using (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and not immutable
);

-- Import runs, raw blocks and warnings are evidence: append-only after insertion.
drop policy if exists canon_import_runs_update_editor on public.canon_import_runs;
drop policy if exists canon_import_runs_delete_editor on public.canon_import_runs;
drop policy if exists canon_raw_blocks_update_editor on public.canon_raw_blocks;
drop policy if exists canon_raw_blocks_delete_editor on public.canon_raw_blocks;
drop policy if exists canon_import_warnings_update_editor on public.canon_import_warnings;
drop policy if exists canon_import_warnings_delete_editor on public.canon_import_warnings;

revoke update, delete on public.canon_import_runs from authenticated;
revoke update, delete on public.canon_raw_blocks from authenticated;
revoke update, delete on public.canon_import_warnings from authenticated;

-- A device registration can only represent the authenticated user.
drop policy if exists sync_devices_insert_editor on public.sync_devices;
drop policy if exists sync_devices_update_editor on public.sync_devices;
drop policy if exists sync_devices_delete_editor on public.sync_devices;

create policy sync_devices_insert_self
on public.sync_devices for insert
to authenticated
with check (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and user_id = auth.uid()
);

create policy sync_devices_update_self
on public.sync_devices for update
to authenticated
using (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and user_id = auth.uid()
)
with check (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and user_id = auth.uid()
);

create policy sync_devices_delete_self
on public.sync_devices for delete
to authenticated
using (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and user_id = auth.uid()
);

-- Audit and change records must identify the authenticated actor.
drop policy if exists database_audit_log_insert_editor on public.database_audit_log;
drop policy if exists sync_change_log_insert_editor on public.sync_change_log;

create policy database_audit_log_insert_authenticated_actor
on public.database_audit_log for insert
to authenticated
with check (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and changed_by = auth.uid()
);

create policy sync_change_log_insert_authenticated_actor
on public.sync_change_log for insert
to authenticated
with check (
  public.has_workspace_role(workspace_id, array['OWNER', 'EDITOR'])
  and changed_by = auth.uid()
);

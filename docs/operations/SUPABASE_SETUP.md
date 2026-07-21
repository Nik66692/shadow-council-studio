# Supabase setup for Shadow Council Studio

This runbook provisions the optional remote PostgreSQL backend. The desktop app continues to work with SQLite when Supabase is not configured.

## 1. Create the project

Create a Supabase project in the desired organization and region. Record only:

- the project URL, such as `https://project-ref.supabase.co`;
- the publishable key, beginning with `sb_publishable_`.

Never place a secret key or legacy service-role key in the desktop application, SQLite database, source control or screenshots.

## 2. Link the repository

Install the official Supabase CLI, authenticate, then from the repository root run:

```bash
supabase login
supabase link --project-ref <project-ref>
supabase migration list
```

The project reference and access token remain local and must not be committed.

## 3. Validate locally

The Supabase CLI local stack requires Docker.

```bash
supabase start
supabase db reset
supabase status
```

The migration creates the workspace model, canonical data tables, audit tables, synchronization metadata, helper functions and RLS policies.

No canon records are seeded. `supabase/seed.sql` is intentionally empty.

## 4. Deploy the schema

Review the migration diff before applying it:

```bash
supabase db diff --linked
supabase db push --dry-run
supabase db push
supabase migration list
```

## 5. Create the first user

Create or invite Niccolò through Supabase Auth. Email/password login is sufficient for the Phase 1.6 preview.

The first workspace is created from **Cloud & Sync** in the desktop app. The `create_workspace` RPC atomically creates:

1. the workspace;
2. the authenticated user's `OWNER` membership.

Direct anonymous access is revoked. All project tables require an authenticated workspace member through Row Level Security.

## 6. Configure the desktop app

Open **Cloud & Sync** and enter:

- Project URL;
- publishable key.

Then authenticate and create or select the `Shadow Council` workspace.

At the end of Phase 1.6 the app will report `CLOUD_READY`, but automatic upload and download remain disabled. Existing SQLite records stay local until a later migration/synchronization command is explicitly approved.

## 7. Recovery

Clearing the cloud configuration does not delete local SQLite data or remote Supabase data. It only returns the desktop app to `LOCAL_ONLY` mode.

Before enabling future synchronization:

- create a local SQLite backup from Database Studio;
- verify the remote migration history;
- verify the selected workspace ID;
- test with non-canonical records first.

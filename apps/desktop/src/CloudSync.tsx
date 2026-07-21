import { createClient, type Session, type SupabaseClient } from "@supabase/supabase-js";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useMemo, useState } from "react";

interface CloudSettings {
  supabaseUrl: string | null;
  publishableKey: string | null;
  workspaceId: string | null;
  updatedAt: string;
}

interface CloudStatus {
  settings: CloudSettings;
  configured: boolean;
  syncReady: boolean;
  pendingOutboxCount: number;
  openConflictCount: number;
  mode: "LOCAL_ONLY" | "CONFIGURED" | "CLOUD_READY";
  diagnostics: string[];
}

interface WorkspaceRow {
  id: string;
  name: string;
  slug: string;
  created_at: string;
}

const environmentUrl = import.meta.env.VITE_SUPABASE_URL ?? "";
const environmentKey = import.meta.env.VITE_SUPABASE_PUBLISHABLE_KEY ?? "";

function createConfiguredClient(settings: CloudSettings | null): SupabaseClient | null {
  if (!settings?.supabaseUrl || !settings.publishableKey) return null;
  return createClient(settings.supabaseUrl, settings.publishableKey, {
    auth: {
      persistSession: true,
      autoRefreshToken: true,
      detectSessionInUrl: false,
      flowType: "pkce",
    },
  });
}

export function CloudSync() {
  const [status, setStatus] = useState<CloudStatus | null>(null);
  const [url, setUrl] = useState(environmentUrl);
  const [publishableKey, setPublishableKey] = useState(environmentKey);
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [session, setSession] = useState<Session | null>(null);
  const [workspaces, setWorkspaces] = useState<WorkspaceRow[]>([]);
  const [workspaceName, setWorkspaceName] = useState("Shadow Council");
  const [workspaceSlug, setWorkspaceSlug] = useState("shadow-council");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const client = useMemo(
    () => createConfiguredClient(status?.settings ?? null),
    [status?.settings],
  );

  const refreshLocalStatus = useCallback(async () => {
    const next = await invoke<CloudStatus>("get_cloud_status");
    setStatus(next);
    setUrl(next.settings.supabaseUrl ?? environmentUrl);
    setPublishableKey(next.settings.publishableKey ?? environmentKey);
    return next;
  }, []);

  const loadWorkspaces = useCallback(async (supabase: SupabaseClient) => {
    const { data, error: queryError } = await supabase
      .from("workspaces")
      .select("id,name,slug,created_at")
      .order("created_at", { ascending: true });
    if (queryError) throw queryError;
    setWorkspaces((data ?? []) as WorkspaceRow[]);
  }, []);

  useEffect(() => {
    refreshLocalStatus().catch((cause: unknown) => {
      console.error(cause);
      setError("Configurazione cloud locale non disponibile.");
    });
  }, [refreshLocalStatus]);

  useEffect(() => {
    if (!client) {
      setSession(null);
      setWorkspaces([]);
      return;
    }

    let mounted = true;
    client.auth.getSession().then(({ data }) => {
      if (!mounted) return;
      setSession(data.session);
      if (data.session) {
        loadWorkspaces(client).catch((cause: unknown) => {
          console.error(cause);
          setError("Impossibile leggere i workspace Supabase autorizzati.");
        });
      }
    });

    const {
      data: { subscription },
    } = client.auth.onAuthStateChange((_event, nextSession) => {
      setSession(nextSession);
      if (nextSession) {
        void loadWorkspaces(client);
      } else {
        setWorkspaces([]);
      }
    });

    return () => {
      mounted = false;
      subscription.unsubscribe();
    };
  }, [client, loadWorkspaces]);

  async function saveConfiguration(workspaceId = status?.settings.workspaceId ?? null) {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const next = await invoke<CloudStatus>("update_cloud_settings", {
        update: {
          supabaseUrl: url,
          publishableKey,
          workspaceId,
        },
      });
      setStatus(next);
      setNotice(
        "Configurazione salvata localmente. Nessun dato di progetto è stato caricato.",
      );
    } catch (cause: unknown) {
      console.error(cause);
      setError(String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function disableCloud() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      if (client) await client.auth.signOut();
      const next = await invoke<CloudStatus>("update_cloud_settings", {
        update: {
          supabaseUrl: null,
          publishableKey: null,
          workspaceId: null,
        },
      });
      setStatus(next);
      setUrl("");
      setPublishableKey("");
      setSession(null);
      setWorkspaces([]);
      setNotice("Cloud disattivato. SQLite resta pienamente operativo.");
    } catch (cause: unknown) {
      console.error(cause);
      setError(String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function signIn() {
    if (!client) {
      setError("Salva prima URL e publishable key.");
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const { data, error: authError } = await client.auth.signInWithPassword({
        email,
        password,
      });
      if (authError) throw authError;
      setSession(data.session);
      setPassword("");
      await loadWorkspaces(client);
      setNotice("Autenticazione Supabase completata.");
    } catch (cause: unknown) {
      console.error(cause);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function signOut() {
    if (!client) return;
    setBusy(true);
    setError(null);
    try {
      const { error: authError } = await client.auth.signOut();
      if (authError) throw authError;
      setSession(null);
      setWorkspaces([]);
      setNotice("Sessione Supabase chiusa.");
    } catch (cause: unknown) {
      console.error(cause);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function createWorkspace() {
    if (!client || !session) return;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const { data, error: rpcError } = await client.rpc("create_workspace", {
        workspace_name: workspaceName,
        workspace_slug: workspaceSlug,
      });
      if (rpcError) throw rpcError;
      await loadWorkspaces(client);
      await saveConfiguration(String(data));
      setNotice("Workspace remoto creato e selezionato. La sincronizzazione resta disattivata.");
    } catch (cause: unknown) {
      console.error(cause);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function selectWorkspace(workspaceId: string) {
    setBusy(true);
    setError(null);
    try {
      await saveConfiguration(workspaceId);
      setNotice("Workspace selezionato. Nessun upload è stato eseguito.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <article className="cloud-sync">
      <header className="section-header">
        <div>
          <p className="eyebrow">PHASE 1.6 · CLOUD FOUNDATION</p>
          <h2>Cloud &amp; Sync</h2>
          <p>
            Supabase è opzionale. SQLite continua a funzionare offline e nessun dato
            viene caricato senza un comando esplicito.
          </p>
        </div>
        <span className={`cloud-mode cloud-mode-${status?.mode ?? "LOCAL_ONLY"}`}>
          {status?.mode ?? "CARICAMENTO"}
        </span>
      </header>

      {error && <p className="warning" role="alert">{error}</p>}
      {notice && <p className="success-message" role="status">{notice}</p>}

      <section className="summary-grid" aria-label="Stato cloud">
        <CloudMetric label="Configurazione" value={status?.configured ? "Presente" : "Locale"} />
        <CloudMetric label="Sessione" value={session ? "Autenticata" : "Disconnessa"} />
        <CloudMetric label="Outbox" value={String(status?.pendingOutboxCount ?? 0)} />
        <CloudMetric label="Conflitti" value={String(status?.openConflictCount ?? 0)} />
      </section>

      <section className="cloud-grid">
        <form
          className="maintenance-card cloud-card"
          onSubmit={(event) => {
            event.preventDefault();
            void saveConfiguration();
          }}
        >
          <h3>1. Progetto Supabase</h3>
          <p className="definition-meta">
            Inserisci esclusivamente URL del progetto e publishable key. Le secret key
            vengono rifiutate dall'app.
          </p>
          <label>
            Project URL
            <input
              type="url"
              value={url}
              placeholder="https://project-ref.supabase.co"
              onChange={(event) => setUrl(event.target.value)}
              disabled={busy}
              required
            />
          </label>
          <label>
            Publishable key
            <input
              type="password"
              value={publishableKey}
              placeholder="sb_publishable_…"
              onChange={(event) => setPublishableKey(event.target.value)}
              disabled={busy}
              autoComplete="off"
              required
            />
          </label>
          <div className="cloud-actions">
            <button type="submit" disabled={busy}>Salva configurazione</button>
            <button type="button" disabled={busy || !status?.configured} onClick={() => void disableCloud()}>
              Disattiva cloud
            </button>
          </div>
        </form>

        <form
          className="maintenance-card cloud-card"
          onSubmit={(event) => {
            event.preventDefault();
            void signIn();
          }}
        >
          <h3>2. Autenticazione</h3>
          {session ? (
            <>
              <p>Connesso come <strong>{session.user.email ?? session.user.id}</strong>.</p>
              <button type="button" disabled={busy} onClick={() => void signOut()}>
                Esci da Supabase
              </button>
            </>
          ) : (
            <>
              <label>
                Email
                <input
                  type="email"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  disabled={busy || !status?.configured}
                  autoComplete="username"
                  required
                />
              </label>
              <label>
                Password
                <input
                  type="password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  disabled={busy || !status?.configured}
                  autoComplete="current-password"
                  required
                />
              </label>
              <button type="submit" disabled={busy || !status?.configured}>
                Accedi
              </button>
            </>
          )}
        </form>

        <section className="maintenance-card cloud-card cloud-workspaces">
          <h3>3. Workspace remoto</h3>
          {!session && <p>Autenticati per vedere i workspace autorizzati.</p>}
          {session && workspaces.length === 0 && (
            <div className="cloud-create-workspace">
              <p>Nessun workspace disponibile. Creane uno in modo esplicito.</p>
              <label>
                Nome
                <input value={workspaceName} onChange={(event) => setWorkspaceName(event.target.value)} />
              </label>
              <label>
                Slug
                <input value={workspaceSlug} onChange={(event) => setWorkspaceSlug(event.target.value)} />
              </label>
              <button type="button" disabled={busy} onClick={() => void createWorkspace()}>
                Crea workspace
              </button>
            </div>
          )}
          {workspaces.length > 0 && (
            <ul className="workspace-list">
              {workspaces.map((workspace) => (
                <li key={workspace.id}>
                  <div>
                    <strong>{workspace.name}</strong>
                    <small>{workspace.slug}</small>
                  </div>
                  <button
                    type="button"
                    className={status?.settings.workspaceId === workspace.id ? "active" : ""}
                    disabled={busy}
                    onClick={() => void selectWorkspace(workspace.id)}
                  >
                    {status?.settings.workspaceId === workspace.id ? "Selezionato" : "Seleziona"}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="maintenance-card cloud-card protected-card">
          <h3>4. Sincronizzazione</h3>
          <p>
            La struttura outbox, cursori e conflitti è pronta. Il push/pull automatico
            rimane intenzionalmente disabilitato finché non approviamo le regole di
            conflitto per ogni entità.
          </p>
          <button type="button" disabled>
            Sincronizzazione non ancora abilitata
          </button>
        </section>
      </section>

      <details>
        <summary>Diagnostica cloud</summary>
        <ul>
          {(status?.diagnostics ?? []).map((diagnostic) => (
            <li key={diagnostic}>{diagnostic}</li>
          ))}
        </ul>
      </details>
    </article>
  );
}

function CloudMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="summary-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

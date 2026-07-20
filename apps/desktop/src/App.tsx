import type { HealthStatus } from "@shadow-council/domain";
import { sprint0Sections } from "@shadow-council/ui";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

const fallback: HealthStatus = {
  projectName: "Shadow Council Studio",
  developmentStage: "Foundation",
  databaseConnected: false,
  migrationsApplied: false,
  sourceOfTruth: {
    exists: false,
    filename: "docs/canon/source/manifest.json",
    sha256: null,
    canonVersion: null,
  },
  modulesImplemented: ["Dashboard", "System Status"],
  nextRecommendedPhase:
    "Phase 1: canonical data model and deterministic import",
  diagnostics: [
    "Esecuzione fuori da Tauri: diagnostica dimostrativa senza accesso SQLite.",
  ],
};
export function App() {
  const [active, setActive] = useState("Dashboard");
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    invoke<HealthStatus>("get_system_health")
      .then(setHealth)
      .catch((e: unknown) => {
        setError("Diagnostica Tauri non disponibile in questa sessione.");
        setHealth(fallback);
        console.error(e);
      });
  }, []);
  return (
    <main className="app">
      <aside>
        <h1>Shadow Council Studio</h1>
        <nav aria-label="Sezioni">
          <ul>
            {sprint0Sections.map((s) => (
              <li key={s}>
                <button
                  className={active === s ? "active" : ""}
                  onClick={() => setActive(s)}
                >
                  {s}
                </button>
              </li>
            ))}
          </ul>
        </nav>
      </aside>
      <section className="panel">
        {!health && <p role="status">Caricamento diagnostica…</p>}
        {error && (
          <p role="alert" className="warning">
            {error}
          </p>
        )}
        {health && active === "Dashboard" && <Dashboard health={health} />}{" "}
        {health && active !== "Dashboard" && <NotImplemented title={active} />}
      </section>
    </main>
  );
}
function Dashboard({ health }: { health: HealthStatus }) {
  return (
    <article>
      <h2>Dashboard</h2>
      <dl>
        <dt>Fase</dt>
        <dd>{health.developmentStage}</dd>
        <dt>Database</dt>
        <dd>{health.databaseConnected ? "connesso" : "non connesso"}</dd>
        <dt>Migrazioni</dt>
        <dd>{health.migrationsApplied ? "applicate" : "non applicate"}</dd>
        <dt>Source of Truth</dt>
        <dd>
          {health.sourceOfTruth.exists
            ? "presente"
            : "sorgente canonica mancante"}
        </dd>
        <dt>File</dt>
        <dd>{health.sourceOfTruth.filename}</dd>
        <dt>SHA-256</dt>
        <dd>{health.sourceOfTruth.sha256 ?? "non disponibile"}</dd>
        <dt>Versione canon</dt>
        <dd>{health.sourceOfTruth.canonVersion ?? "non disponibile"}</dd>
      </dl>
      <h3>Moduli implementati</h3>
      <ul>
        {health.modulesImplemented.map((m) => (
          <li key={m}>{m}</li>
        ))}
      </ul>
      <p>
        <strong>Prossima fase consigliata:</strong>{" "}
        {health.nextRecommendedPhase}
      </p>
      <details>
        <summary>Diagnostica sviluppatore</summary>
        <ul>
          {health.diagnostics.map((d) => (
            <li key={d}>{d}</li>
          ))}
        </ul>
      </details>
    </article>
  );
}
function NotImplemented({ title }: { title: string }) {
  return (
    <article>
      <h2>{title}</h2>
      <p>Sezione non ancora implementata in Sprint 0.</p>
    </article>
  );
}

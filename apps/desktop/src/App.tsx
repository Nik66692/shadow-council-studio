import type {
  CanonImportReviewSnapshot,
  HealthStatus,
} from "@shadow-council/domain";
import { sprint0Sections } from "@shadow-council/ui";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { DatabaseStudio } from "./DatabaseStudio";

const fallback: HealthStatus = {
  projectName: "Shadow Council Studio",
  developmentStage: "Phase 1",
  databaseConnected: false,
  migrationsApplied: false,
  sourceOfTruth: {
    exists: false,
    filename: "docs/canon/source/manifest.json",
    sha256: null,
    canonVersion: null,
  },
  modulesImplemented: ["Dashboard", "Import canonico", "Database Studio"],
  nextRecommendedPhase:
    "Esegui l'app desktop Tauri per accedere al database SQLite locale.",
  diagnostics: [
    "Esecuzione fuori da Tauri: diagnostica dimostrativa senza accesso SQLite.",
  ],
};

const emptyReview: CanonImportReviewSnapshot = {
  run: null,
  drafts: [],
  warnings: [],
  importedNow: false,
};

export function App() {
  const [active, setActive] = useState("Dashboard");
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [review, setReview] = useState<CanonImportReviewSnapshot | null>(null);
  const [reviewLoading, setReviewLoading] = useState(false);
  const [importRunning, setImportRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadReview = useCallback(async () => {
    setReviewLoading(true);
    try {
      setReview(
        await invoke<CanonImportReviewSnapshot>("get_canon_import_review"),
      );
    } catch (cause: unknown) {
      console.error(cause);
      setReview(emptyReview);
      setError(
        "Revisione canonica non disponibile fuori dall'app desktop Tauri.",
      );
    } finally {
      setReviewLoading(false);
    }
  }, []);

  const runImport = useCallback(async () => {
    setImportRunning(true);
    setError(null);
    try {
      const snapshot =
        await invoke<CanonImportReviewSnapshot>("run_canon_import");
      setReview(snapshot);
      setHealth(await invoke<HealthStatus>("get_system_health"));
    } catch (cause: unknown) {
      console.error(cause);
      setError(
        cause instanceof Error
          ? cause.message
          : "Importazione canonica non riuscita.",
      );
    } finally {
      setImportRunning(false);
    }
  }, []);

  useEffect(() => {
    invoke<HealthStatus>("get_system_health")
      .then(setHealth)
      .catch((cause: unknown) => {
        setError("Diagnostica Tauri non disponibile in questa sessione.");
        setHealth(fallback);
        console.error(cause);
      });
  }, []);

  useEffect(() => {
    if (active === "Import canonico" && review === null) {
      void loadReview();
    }
  }, [active, loadReview, review]);

  return (
    <main className="app">
      <aside>
        <h1>Shadow Council Studio</h1>
        <p className="phase-badge">Phase 1 · Database Studio 0.2</p>
        <nav aria-label="Sezioni">
          <ul>
            {sprint0Sections.map((section) => (
              <li key={section}>
                <button
                  className={active === section ? "active" : ""}
                  onClick={() => setActive(section)}
                >
                  {section}
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
        {health && active === "Dashboard" && <Dashboard health={health} />}
        {health && active === "Import canonico" && (
          <CanonImportReview
            review={review}
            loading={reviewLoading}
            importRunning={importRunning}
            onImport={runImport}
          />
        )}
        {health && active === "Database Studio" && <DatabaseStudio />}
        {health &&
          active !== "Dashboard" &&
          active !== "Import canonico" &&
          active !== "Database Studio" && <NotImplemented title={active} />}
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
            ? "presente e verificata"
            : "sorgente canonica mancante"}
        </dd>
        <dt>File</dt>
        <dd>{health.sourceOfTruth.filename}</dd>
        <dt>SHA-256</dt>
        <dd className="hash">
          {health.sourceOfTruth.sha256 ?? "non disponibile"}
        </dd>
        <dt>Versione canon</dt>
        <dd>{health.sourceOfTruth.canonVersion ?? "non disponibile"}</dd>
      </dl>
      <h3>Moduli implementati</h3>
      <ul>
        {health.modulesImplemented.map((module) => (
          <li key={module}>{module}</li>
        ))}
      </ul>
      <p>
        <strong>Prossima fase consigliata:</strong>{" "}
        {health.nextRecommendedPhase}
      </p>
      <details>
        <summary>Diagnostica sviluppatore</summary>
        <ul>
          {health.diagnostics.map((diagnostic) => (
            <li key={diagnostic}>{diagnostic}</li>
          ))}
        </ul>
      </details>
    </article>
  );
}

function CanonImportReview({
  review,
  loading,
  importRunning,
  onImport,
}: {
  review: CanonImportReviewSnapshot | null;
  loading: boolean;
  importRunning: boolean;
  onImport: () => Promise<void>;
}) {
  return (
    <article>
      <header className="section-header">
        <div>
          <p className="eyebrow">EVIDENZA DI IMPORTAZIONE</p>
          <h2>Import canonico</h2>
          <p>
            Lettura strutturale e sola consultazione. Nessun elemento importato
            diventa canonico senza approvazione esplicita di Niccolò.
          </p>
        </div>
        <button
          className="primary-action"
          disabled={importRunning}
          onClick={() => void onImport()}
        >
          {importRunning ? "Importazione…" : "Esegui import verificato"}
        </button>
      </header>

      {loading && <p role="status">Caricamento revisione…</p>}
      {!loading && review?.run === null && (
        <section className="empty-state">
          <h3>Nessuna importazione disponibile</h3>
          <p>
            Il database è pronto. Avvia l'import per verificare il manifest,
            controllare lo SHA-256 ed estrarre il testo della Source of Truth.
          </p>
        </section>
      )}

      {review?.run && (
        <>
          <section className="summary-grid" aria-label="Riepilogo importazione">
            <SummaryCard label="Versione" value={review.run.sourceVersion} />
            <SummaryCard
              label="Blocchi grezzi"
              value={String(review.run.rawBlockCount)}
            />
            <SummaryCard
              label="Bozze da revisionare"
              value={String(review.run.draftCount)}
            />
            <SummaryCard
              label="Avvisi"
              value={String(review.run.warningCount)}
            />
          </section>

          <section className="evidence-panel">
            <h3>Provenienza</h3>
            <dl>
              <dt>Run ID</dt>
              <dd>{review.run.id}</dd>
              <dt>Importer</dt>
              <dd>{review.run.importerVersion}</dd>
              <dt>SHA-256 sorgente</dt>
              <dd className="hash">{review.run.sourceSha256}</dd>
              <dt>Stato</dt>
              <dd>{review.run.status}</dd>
            </dl>
          </section>

          {review.warnings.length > 0 && (
            <section>
              <h3>Avvisi di importazione</h3>
              <ul className="warning-list">
                {review.warnings.map((warning) => (
                  <li key={warning.id}>
                    <strong>{warning.warningCode}</strong>
                    <span>{warning.message}</span>
                    {warning.sourceAnchor && (
                      <code>{warning.sourceAnchor}</code>
                    )}
                  </li>
                ))}
              </ul>
            </section>
          )}

          <section>
            <h3>Bozze in attesa di revisione umana</h3>
            <p>
              Testo italiano conservato come estratto. Stato canonico non
              assegnato.
            </p>
            <ol className="draft-list">
              {review.drafts.map((draft) => (
                <li key={draft.id}>
                  <div className="draft-meta">
                    <span>{draft.blockKind}</span>
                    <span>#{draft.blockIndex}</span>
                    <span>{draft.reviewStatus}</span>
                  </div>
                  <p>{draft.originalText}</p>
                  <details>
                    <summary>Provenienza tecnica</summary>
                    <code>{draft.sourceAnchor}</code>
                    <code>{draft.textSha256}</code>
                  </details>
                </li>
              ))}
            </ol>
          </section>
        </>
      )}
    </article>
  );
}

function SummaryCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="summary-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function NotImplemented({ title }: { title: string }) {
  return (
    <article>
      <h2>{title}</h2>
      <p>Sezione non ancora implementata nella Phase 1.</p>
    </article>
  );
}

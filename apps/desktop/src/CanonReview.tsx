import type {
  ApproveCanonDraftsRequest,
  CanonDraftReviewStatus,
  CanonEntryKind,
  CanonReviewDraftItem,
  CanonReviewWorkspace,
  CanonicalStatus,
  RejectCanonDraftsRequest,
} from "@shadow-council/domain";
import { canonEntryKinds, canonicalStatuses } from "@shadow-council/domain";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useMemo, useState } from "react";
import "./CanonReview.css";

type ReviewTab = "queue" | "entries" | "decisions";
type ReviewStatusFilter = CanonDraftReviewStatus | "ALL";

const statusLabels: Record<CanonDraftReviewStatus, string> = {
  PENDING_HUMAN_REVIEW: "Da revisionare",
  APPROVED: "Approvata",
  MERGED_INTO_ENTRY: "Unita in voce canonica",
  REJECTED: "Rifiutata",
};

function messageFromError(cause: unknown): string {
  if (typeof cause === "string") return cause;
  if (cause instanceof Error) return cause.message;
  return "Operazione di revisione canonica non riuscita.";
}

export function CanonReview() {
  const [workspace, setWorkspace] = useState<CanonReviewWorkspace | null>(null);
  const [tab, setTab] = useState<ReviewTab>("queue");
  const [statusFilter, setStatusFilter] = useState<ReviewStatusFilter>(
    "PENDING_HUMAN_REVIEW",
  );
  const [blockKindFilter, setBlockKindFilter] = useState("ALL");
  const [search, setSearch] = useState("");
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [title, setTitle] = useState("");
  const [entryKind, setEntryKind] = useState<CanonEntryKind | "">("");
  const [canonicalStatus, setCanonicalStatus] = useState<CanonicalStatus | "">(
    "",
  );
  const [normalizedText, setNormalizedText] = useState("");
  const [reviewer, setReviewer] = useState("Niccolò");
  const [rationale, setRationale] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const loadWorkspace = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setWorkspace(
        await invoke<CanonReviewWorkspace>("get_canon_review_workspace"),
      );
    } catch (cause: unknown) {
      setError(messageFromError(cause));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadWorkspace();
  }, [loadWorkspace]);

  const blockKinds = useMemo(
    () =>
      Array.from(
        new Set(workspace?.drafts.map((draft) => draft.blockKind) ?? []),
      ).sort(),
    [workspace],
  );

  const filteredDrafts = useMemo(() => {
    const needle = search.trim().toLocaleLowerCase("it");
    return (workspace?.drafts ?? []).filter((draft) => {
      if (statusFilter !== "ALL" && draft.reviewStatus !== statusFilter)
        return false;
      if (blockKindFilter !== "ALL" && draft.blockKind !== blockKindFilter)
        return false;
      if (!needle) return true;
      return [draft.originalText, draft.sourceAnchor, draft.blockKind]
        .join("\n")
        .toLocaleLowerCase("it")
        .includes(needle);
    });
  }, [blockKindFilter, search, statusFilter, workspace]);

  const selectedDrafts = useMemo(
    () =>
      (workspace?.drafts ?? [])
        .filter((draft) => selectedIds.has(draft.id))
        .sort((left, right) => left.blockIndex - right.blockIndex),
    [selectedIds, workspace],
  );

  useEffect(() => {
    setNormalizedText(
      selectedDrafts.map((draft) => draft.originalText).join("\n\n"),
    );
  }, [selectedDrafts]);

  const resetDecisionForm = () => {
    setSelectedIds(new Set<string>());
    setTitle("");
    setEntryKind("");
    setCanonicalStatus("");
    setNormalizedText("");
    setRationale("");
  };

  const toggleDraft = (draft: CanonReviewDraftItem) => {
    if (draft.reviewStatus !== "PENDING_HUMAN_REVIEW") return;
    setSelectedIds((current) => {
      const next = new Set(current);
      if (next.has(draft.id)) next.delete(draft.id);
      else next.add(draft.id);
      return next;
    });
  };

  const selectVisiblePending = () => {
    setSelectedIds(
      new Set(
        filteredDrafts
          .filter((draft) => draft.reviewStatus === "PENDING_HUMAN_REVIEW")
          .slice(0, 50)
          .map((draft) => draft.id),
      ),
    );
  };

  const approve = async () => {
    if (!entryKind || !canonicalStatus) {
      setError("Seleziona esplicitamente categoria e stato canonico.");
      return;
    }
    const request: ApproveCanonDraftsRequest = {
      draftIds: selectedDrafts.map((draft) => draft.id),
      title,
      entryKind,
      canonicalStatus,
      normalizedText,
      reviewer,
      rationale,
    };
    setSaving(true);
    setError(null);
    setMessage(null);
    try {
      setWorkspace(
        await invoke<CanonReviewWorkspace>("approve_canon_drafts", {
          request,
        }),
      );
      setMessage(
        selectedDrafts.length === 1
          ? "Bozza approvata e registrata nel canon."
          : `${selectedDrafts.length} bozze unite in una voce canonica.`,
      );
      resetDecisionForm();
    } catch (cause: unknown) {
      setError(messageFromError(cause));
    } finally {
      setSaving(false);
    }
  };

  const reject = async () => {
    const request: RejectCanonDraftsRequest = {
      draftIds: selectedDrafts.map((draft) => draft.id),
      reviewer,
      rationale,
    };
    setSaving(true);
    setError(null);
    setMessage(null);
    try {
      setWorkspace(
        await invoke<CanonReviewWorkspace>("reject_canon_drafts", {
          request,
        }),
      );
      setMessage(
        selectedDrafts.length === 1
          ? "Bozza rifiutata con decisione registrata."
          : `${selectedDrafts.length} bozze rifiutate con decisioni registrate.`,
      );
      resetDecisionForm();
    } catch (cause: unknown) {
      setError(messageFromError(cause));
    } finally {
      setSaving(false);
    }
  };

  if (loading) return <p role="status">Caricamento Canon Review…</p>;

  return (
    <article className="canon-review">
      <header className="section-header">
        <div>
          <p className="eyebrow">APPROVAZIONE UMANA CONTROLLATA</p>
          <h2>Canon Review</h2>
          <p>
            Il testo importato resta immutabile. Solo una decisione esplicita
            crea una voce canonica o rifiuta una bozza, mantenendo provenienza e
            storico.
          </p>
        </div>
        <button className="primary-action" onClick={() => void loadWorkspace()}>
          Aggiorna revisione
        </button>
      </header>

      {error && (
        <p className="warning" role="alert">
          {error}
        </p>
      )}
      {message && (
        <p className="success-message" role="status">
          {message}
        </p>
      )}

      {workspace && (
        <>
          <section className="summary-grid" aria-label="Riepilogo revisione">
            <Summary
              label="Da revisionare"
              value={workspace.summary.pendingCount}
            />
            <Summary
              label="Fonti approvate"
              value={workspace.summary.approvedCount}
            />
            <Summary
              label="Rifiutate"
              value={workspace.summary.rejectedCount}
            />
            <Summary
              label="Voci canoniche"
              value={workspace.summary.entryCount}
            />
          </section>

          <div className="studio-tabs" role="tablist" aria-label="Canon Review">
            <button
              className={tab === "queue" ? "active" : ""}
              onClick={() => setTab("queue")}
            >
              Coda
            </button>
            <button
              className={tab === "entries" ? "active" : ""}
              onClick={() => setTab("entries")}
            >
              Registro canonico
            </button>
            <button
              className={tab === "decisions" ? "active" : ""}
              onClick={() => setTab("decisions")}
            >
              Decisioni
            </button>
          </div>

          {tab === "queue" && (
            <div className="canon-review-layout">
              <section className="review-queue">
                <div className="review-filters">
                  <label>
                    Cerca
                    <input
                      value={search}
                      onChange={(event) => setSearch(event.target.value)}
                      placeholder="Testo, anchor o tipo blocco"
                    />
                  </label>
                  <label>
                    Stato
                    <select
                      value={statusFilter}
                      onChange={(event) =>
                        setStatusFilter(
                          event.target.value as ReviewStatusFilter,
                        )
                      }
                    >
                      <option value="ALL">Tutti</option>
                      {Object.entries(statusLabels).map(([value, label]) => (
                        <option key={value} value={value}>
                          {label}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Tipo blocco
                    <select
                      value={blockKindFilter}
                      onChange={(event) =>
                        setBlockKindFilter(event.target.value)
                      }
                    >
                      <option value="ALL">Tutti</option>
                      {blockKinds.map((kind) => (
                        <option key={kind}>{kind}</option>
                      ))}
                    </select>
                  </label>
                </div>

                <div className="review-selection-toolbar">
                  <span>{filteredDrafts.length} elementi visibili</span>
                  <button onClick={selectVisiblePending}>
                    Seleziona fino a 50 pending
                  </button>
                  <button onClick={() => setSelectedIds(new Set<string>())}>
                    Deseleziona
                  </button>
                </div>

                {filteredDrafts.length === 0 ? (
                  <div className="empty-state">
                    <h3>Nessuna bozza corrispondente</h3>
                    <p>
                      Esegui prima l'import canonico oppure modifica i filtri.
                    </p>
                  </div>
                ) : (
                  <ol className="review-draft-list">
                    {filteredDrafts.map((draft) => (
                      <li
                        key={draft.id}
                        className={selectedIds.has(draft.id) ? "selected" : ""}
                      >
                        <label className="review-draft-select">
                          <input
                            type="checkbox"
                            checked={selectedIds.has(draft.id)}
                            disabled={
                              draft.reviewStatus !== "PENDING_HUMAN_REVIEW"
                            }
                            onChange={() => toggleDraft(draft)}
                          />
                          <span>#{draft.blockIndex}</span>
                          <strong>{draft.blockKind}</strong>
                          <span>{statusLabels[draft.reviewStatus]}</span>
                        </label>
                        <p>{draft.originalText}</p>
                        <details>
                          <summary>Provenienza immutabile</summary>
                          <code>{draft.sourceAnchor}</code>
                          <code>{draft.textSha256}</code>
                          {draft.styleName && (
                            <code>Stile: {draft.styleName}</code>
                          )}
                        </details>
                      </li>
                    ))}
                  </ol>
                )}
              </section>

              <aside className="review-decision-panel">
                <p className="eyebrow">DECISIONE</p>
                <h3>{selectedDrafts.length} bozze selezionate</h3>
                <p>
                  Approvando più bozze verrà creata una sola voce con tutte le
                  fonti ordinate. Il testo originale non viene modificato.
                </p>

                <label>
                  Titolo della voce
                  <input
                    value={title}
                    onChange={(event) => setTitle(event.target.value)}
                  />
                </label>
                <div className="review-form-row">
                  <label>
                    Categoria
                    <select
                      value={entryKind}
                      onChange={(event) =>
                        setEntryKind(event.target.value as CanonEntryKind)
                      }
                    >
                      <option value="" disabled>
                        Seleziona categoria
                      </option>
                      {canonEntryKinds.map((kind) => (
                        <option key={kind}>{kind}</option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Stato canonico
                    <select
                      value={canonicalStatus}
                      onChange={(event) =>
                        setCanonicalStatus(
                          event.target.value as CanonicalStatus,
                        )
                      }
                    >
                      <option value="" disabled>
                        Seleziona stato
                      </option>
                      {canonicalStatuses.map((status) => (
                        <option key={status}>{status}</option>
                      ))}
                    </select>
                  </label>
                </div>
                <label>
                  Testo normalizzato approvato
                  <textarea
                    value={normalizedText}
                    onChange={(event) => setNormalizedText(event.target.value)}
                    placeholder="Seleziona una o più bozze"
                  />
                </label>
                <label>
                  Revisore
                  <input
                    value={reviewer}
                    onChange={(event) => setReviewer(event.target.value)}
                  />
                </label>
                <label>
                  Motivazione obbligatoria
                  <textarea
                    value={rationale}
                    onChange={(event) => setRationale(event.target.value)}
                    placeholder="Perché questa decisione è corretta?"
                  />
                </label>
                <div className="review-actions">
                  <button
                    className="approve-action"
                    disabled={
                      saving ||
                      selectedDrafts.length === 0 ||
                      !title.trim() ||
                      !entryKind ||
                      !canonicalStatus ||
                      !normalizedText.trim() ||
                      !reviewer.trim() ||
                      !rationale.trim()
                    }
                    onClick={() => void approve()}
                  >
                    {saving ? "Salvataggio…" : "Approva nel canon"}
                  </button>
                  <button
                    className="reject-action"
                    disabled={
                      saving ||
                      selectedDrafts.length === 0 ||
                      !reviewer.trim() ||
                      !rationale.trim()
                    }
                    onClick={() => void reject()}
                  >
                    Rifiuta selezione
                  </button>
                </div>
              </aside>
            </div>
          )}

          {tab === "entries" && (
            <section className="canon-entry-list">
              {workspace.entries.length === 0 ? (
                <div className="empty-state">
                  <h3>Registro ancora vuoto</h3>
                  <p>Le voci appariranno dopo la prima approvazione umana.</p>
                </div>
              ) : (
                workspace.entries.map((entry) => (
                  <article key={entry.id} className="canon-entry-card">
                    <div className="canon-entry-heading">
                      <div>
                        <p className="eyebrow">{entry.entryKind}</p>
                        <h3>{entry.title}</h3>
                      </div>
                      <span>{entry.canonicalStatus}</span>
                    </div>
                    <p className="canon-entry-text">{entry.normalizedText}</p>
                    <dl>
                      <dt>Approvata da</dt>
                      <dd>{entry.approvedBy}</dd>
                      <dt>Data</dt>
                      <dd>{entry.approvedAt}</dd>
                      <dt>Motivazione</dt>
                      <dd>{entry.rationale}</dd>
                      <dt>ID</dt>
                      <dd>
                        <code>{entry.id}</code>
                      </dd>
                    </dl>
                    <details>
                      <summary>{entry.sources.length} fonti collegate</summary>
                      <ol>
                        {entry.sources.map((source) => (
                          <li key={source.draftId}>
                            <strong>
                              #{source.blockIndex} · {source.blockKind}
                            </strong>
                            <p>{source.originalText}</p>
                            <code>{source.sourceAnchor}</code>
                          </li>
                        ))}
                      </ol>
                    </details>
                  </article>
                ))
              )}
            </section>
          )}

          {tab === "decisions" && (
            <section className="decision-log">
              {workspace.recentDecisions.length === 0 ? (
                <div className="empty-state">
                  <h3>Nessuna decisione registrata</h3>
                </div>
              ) : (
                <table>
                  <thead>
                    <tr>
                      <th>Quando</th>
                      <th>Decisione</th>
                      <th>Bozza</th>
                      <th>Revisore</th>
                      <th>Motivazione</th>
                    </tr>
                  </thead>
                  <tbody>
                    {workspace.recentDecisions.map((decision) => (
                      <tr key={decision.id}>
                        <td>{decision.decidedAt}</td>
                        <td>{decision.resultingReviewStatus}</td>
                        <td>
                          <code>{decision.draftId}</code>
                        </td>
                        <td>{decision.reviewer}</td>
                        <td>{decision.rationale}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </section>
          )}
        </>
      )}
    </article>
  );
}

function Summary({ label, value }: { label: string; value: number }) {
  return (
    <div className="summary-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

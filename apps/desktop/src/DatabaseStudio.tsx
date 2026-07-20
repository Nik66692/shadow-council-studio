import type {
  CanonReviewNoteUpdate,
  DatabaseAuditEntry,
  DatabaseFileResult,
  DatabaseIntegrityReport,
  DatabaseRelationship,
  DatabaseStudioSnapshot,
  DatabaseTable,
  DatabaseTablePage,
  ProjectMetadataUpdate,
  TableBrowseRequest,
} from "@shadow-council/domain";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useMemo, useState } from "react";

type StudioTab = "schema" | "data" | "maintenance";
type DatabaseRow = Record<string, unknown>;

type ActiveFilter = {
  column: string;
  value: string;
  label: string;
} | null;

const defaultRequest = (tableName: string): TableBrowseRequest => ({
  tableName,
  page: 0,
  pageSize: 25,
  search: null,
  sortColumn: null,
  sortDirection: "ASC",
  filterColumn: null,
  filterValue: null,
});

function errorMessage(cause: unknown): string {
  if (typeof cause === "string") return cause;
  if (cause instanceof Error) return cause.message;
  return "Operazione Database Studio non riuscita.";
}

export function formatDatabaseValue(value: unknown): string {
  if (value === null || value === undefined) return "NULL";
  if (typeof value === "string") return value;
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

export function DatabaseStudio() {
  const [snapshot, setSnapshot] = useState<DatabaseStudioSnapshot | null>(null);
  const [selectedTableName, setSelectedTableName] = useState<string | null>(null);
  const [tab, setTab] = useState<StudioTab>("schema");
  const [page, setPage] = useState<DatabaseTablePage | null>(null);
  const [selectedRow, setSelectedRow] = useState<DatabaseRow | null>(null);
  const [search, setSearch] = useState("");
  const [sortColumn, setSortColumn] = useState<string | null>(null);
  const [sortDirection, setSortDirection] = useState<"ASC" | "DESC">("ASC");
  const [activeFilter, setActiveFilter] = useState<ActiveFilter>(null);
  const [pageIndex, setPageIndex] = useState(0);
  const [loading, setLoading] = useState(true);
  const [tableLoading, setTableLoading] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectedTable = useMemo(
    () =>
      snapshot?.tables.find((table) => table.name === selectedTableName) ??
      null,
    [selectedTableName, snapshot],
  );

  const loadSnapshot = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DatabaseStudioSnapshot>(
        "get_database_studio_snapshot",
      );
      setSnapshot(result);
      setSelectedTableName((current) => current ?? result.tables[0]?.name ?? null);
    } catch (cause: unknown) {
      setError(errorMessage(cause));
    } finally {
      setLoading(false);
    }
  }, []);

  const buildRequest = useCallback((): TableBrowseRequest | null => {
    if (!selectedTableName) return null;
    return {
      ...defaultRequest(selectedTableName),
      page: pageIndex,
      search: search.trim() || null,
      sortColumn,
      sortDirection,
      filterColumn: activeFilter?.column ?? null,
      filterValue: activeFilter?.value ?? null,
    };
  }, [activeFilter, pageIndex, search, selectedTableName, sortColumn, sortDirection]);

  const loadTable = useCallback(async () => {
    const request = buildRequest();
    if (!request) return;
    setTableLoading(true);
    setError(null);
    try {
      setPage(
        await invoke<DatabaseTablePage>("browse_database_table", { request }),
      );
    } catch (cause: unknown) {
      setError(errorMessage(cause));
      setPage(null);
    } finally {
      setTableLoading(false);
    }
  }, [buildRequest]);

  useEffect(() => {
    void loadSnapshot();
  }, [loadSnapshot]);

  useEffect(() => {
    if (tab === "data") void loadTable();
  }, [loadTable, tab]);

  const selectTable = (tableName: string, nextTab: StudioTab = tab) => {
    setSelectedTableName(tableName);
    setSelectedRow(null);
    setSearch("");
    setSortColumn(null);
    setSortDirection("ASC");
    setActiveFilter(null);
    setPageIndex(0);
    setTab(nextTab);
  };

  const navigateRelationship = (
    relationship: DatabaseRelationship,
    direction: "outgoing" | "incoming",
  ) => {
    if (!selectedRow) return;
    const sourceValue =
      direction === "outgoing"
        ? selectedRow[relationship.sourceColumn]
        : selectedRow[relationship.targetColumn];
    if (sourceValue === null || sourceValue === undefined) return;
    const nextTable =
      direction === "outgoing"
        ? relationship.targetTable
        : relationship.sourceTable;
    const nextColumn =
      direction === "outgoing"
        ? relationship.targetColumn
        : relationship.sourceColumn;
    setSelectedTableName(nextTable);
    setSelectedRow(null);
    setSearch("");
    setSortColumn(null);
    setSortDirection("ASC");
    setPageIndex(0);
    setActiveFilter({
      column: nextColumn,
      value: String(sourceValue),
      label: `${relationship.sourceTable}.${relationship.sourceColumn} → ${relationship.targetTable}.${relationship.targetColumn}`,
    });
    setTab("data");
  };

  const exportTable = async (format: "json" | "csv") => {
    const request = buildRequest();
    if (!request) return;
    setMessage(null);
    setError(null);
    try {
      const result = await invoke<DatabaseFileResult>("export_database_table", {
        request,
        format,
      });
      setMessage(`Esportato ${result.fileName} · SHA-256 ${result.sha256}`);
    } catch (cause: unknown) {
      setError(errorMessage(cause));
    }
  };

  const createBackup = async () => {
    setMessage(null);
    setError(null);
    try {
      const result = await invoke<DatabaseFileResult>("create_database_backup");
      setMessage(`Backup creato: ${result.path} · SHA-256 ${result.sha256}`);
    } catch (cause: unknown) {
      setError(errorMessage(cause));
    }
  };

  const runIntegrity = async () => {
    setMessage(null);
    setError(null);
    try {
      const integrity = await invoke<DatabaseIntegrityReport>(
        "run_database_integrity_check",
      );
      setSnapshot((current) => (current ? { ...current, integrity } : current));
      setMessage(
        integrity.ok
          ? "Controllo integrità completato: database integro."
          : `Problemi rilevati: ${integrity.messages.join(" · ")}`,
      );
    } catch (cause: unknown) {
      setError(errorMessage(cause));
    }
  };

  const outgoing =
    snapshot?.relationships.filter(
      (relationship) => relationship.sourceTable === selectedTableName,
    ) ?? [];
  const incoming =
    snapshot?.relationships.filter(
      (relationship) => relationship.targetTable === selectedTableName,
    ) ?? [];

  return (
    <article className="database-studio">
      <header className="section-header">
        <div>
          <p className="eyebrow">SCHEMA · DATI · RELAZIONI</p>
          <h2>Database Studio</h2>
          <p>
            Consulta il modello SQLite reale e gestisci soltanto i campi
            esplicitamente autorizzati. Fonti, hash, anchor e stato canonico
            restano protetti.
          </p>
        </div>
        <button className="primary-action" onClick={() => void loadSnapshot()}>
          Aggiorna schema
        </button>
      </header>

      {loading && <p role="status">Analisi del database…</p>}
      {error && (
        <p role="alert" className="warning">
          {error}
        </p>
      )}
      {message && <p className="success-message">{message}</p>}

      {snapshot && (
        <>
          <section className="summary-grid database-summary">
            <Summary label="Tabelle" value={String(snapshot.tables.length)} />
            <Summary
              label="Relazioni"
              value={String(snapshot.relationships.length)}
            />
            <Summary
              label="Integrità"
              value={snapshot.integrity.ok ? "OK" : "ATTENZIONE"}
            />
            <Summary
              label="Record totali"
              value={String(
                snapshot.tables.reduce((sum, table) => sum + table.rowCount, 0),
              )}
            />
          </section>

          <section className="database-location">
            <strong>Database locale</strong>
            <code>{snapshot.databasePath}</code>
          </section>

          <div className="database-layout">
            <aside className="table-sidebar" aria-label="Tabelle SQLite">
              <h3>Tabelle</h3>
              <ul>
                {snapshot.tables.map((table) => (
                  <li key={table.name}>
                    <button
                      className={table.name === selectedTableName ? "active" : ""}
                      onClick={() => selectTable(table.name)}
                    >
                      <span>{table.name}</span>
                      <small>{table.rowCount} record</small>
                    </button>
                  </li>
                ))}
              </ul>
            </aside>

            <div className="database-workspace">
              <div className="studio-tabs" role="tablist">
                <button
                  role="tab"
                  aria-selected={tab === "schema"}
                  className={tab === "schema" ? "active" : ""}
                  onClick={() => setTab("schema")}
                >
                  Schema e relazioni
                </button>
                <button
                  role="tab"
                  aria-selected={tab === "data"}
                  className={tab === "data" ? "active" : ""}
                  onClick={() => setTab("data")}
                >
                  Dati
                </button>
                <button
                  role="tab"
                  aria-selected={tab === "maintenance"}
                  className={tab === "maintenance" ? "active" : ""}
                  onClick={() => setTab("maintenance")}
                >
                  Integrità e backup
                </button>
              </div>

              {tab === "schema" && selectedTable && (
                <SchemaView
                  table={selectedTable}
                  incoming={incoming}
                  outgoing={outgoing}
                  allTables={snapshot.tables}
                  onSelectTable={(name) => selectTable(name, "schema")}
                />
              )}

              {tab === "data" && selectedTable && (
                <DataView
                  table={selectedTable}
                  page={page}
                  loading={tableLoading}
                  selectedRow={selectedRow}
                  search={search}
                  activeFilter={activeFilter}
                  incoming={incoming}
                  outgoing={outgoing}
                  onSearch={(value) => {
                    setSearch(value);
                    setPageIndex(0);
                  }}
                  onSort={(column) => {
                    if (sortColumn === column) {
                      setSortDirection((current) =>
                        current === "ASC" ? "DESC" : "ASC",
                      );
                    } else {
                      setSortColumn(column);
                      setSortDirection("ASC");
                    }
                    setPageIndex(0);
                  }}
                  onSelectRow={setSelectedRow}
                  onPrevious={() => setPageIndex((current) => Math.max(0, current - 1))}
                  onNext={() => setPageIndex((current) => current + 1)}
                  onClearFilter={() => {
                    setActiveFilter(null);
                    setPageIndex(0);
                  }}
                  onNavigate={navigateRelationship}
                  onExport={exportTable}
                  onRefresh={loadTable}
                  onMutationComplete={async (entry) => {
                    setMessage(
                      `Modifica registrata nell’audit log: ${entry.entityType}.${entry.fieldName}`,
                    );
                    await loadTable();
                    await loadSnapshot();
                  }}
                />
              )}

              {tab === "maintenance" && (
                <MaintenanceView
                  snapshot={snapshot}
                  onIntegrity={runIntegrity}
                  onBackup={createBackup}
                />
              )}
            </div>
          </div>
        </>
      )}
    </article>
  );
}

function Summary({ label, value }: { label: string; value: string }) {
  return (
    <div className="summary-card">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function SchemaView({
  table,
  incoming,
  outgoing,
  allTables,
  onSelectTable,
}: {
  table: DatabaseTable;
  incoming: DatabaseRelationship[];
  outgoing: DatabaseRelationship[];
  allTables: DatabaseTable[];
  onSelectTable: (name: string) => void;
}) {
  return (
    <section className="schema-view">
      <div className="table-definition">
        <div className="definition-heading">
          <div>
            <p className="eyebrow">TABELLA SELEZIONATA</p>
            <h3>{table.name}</h3>
          </div>
          <div className="definition-meta">
            <span>{table.rowCount} record</span>
            <span>{table.migrationSource ?? "migration non mappata"}</span>
          </div>
        </div>
        <table>
          <thead>
            <tr>
              <th>Colonna</th>
              <th>Tipo</th>
              <th>Vincoli</th>
              <th>Default</th>
            </tr>
          </thead>
          <tbody>
            {table.columns.map((column) => (
              <tr key={column.name}>
                <td>
                  <code>{column.name}</code>
                </td>
                <td>{column.dataType || "ANY"}</td>
                <td>
                  {[
                    column.primaryKeyPosition > 0 ? "PK" : null,
                    column.notNull ? "NOT NULL" : null,
                  ]
                    .filter(Boolean)
                    .join(" · ") || "—"}
                </td>
                <td>{column.defaultValue ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>

        <details>
          <summary>Indici e SQL di creazione</summary>
          <ul className="index-list">
            {table.indexes.map((index) => (
              <li key={index.name}>
                <code>{index.name}</code> · {index.unique ? "UNIQUE" : "INDEX"} ·{" "}
                {index.columns.join(", ")}
              </li>
            ))}
          </ul>
          <pre>{table.createSql ?? "SQL non disponibile"}</pre>
        </details>
      </div>

      <div className="relationship-map">
        <h3>Mappa relazionale</h3>
        <div className="relation-columns">
          <RelationshipColumn
            title="Relazioni in ingresso"
            relationships={incoming}
            direction="incoming"
            onSelectTable={onSelectTable}
          />
          <div className="selected-table-node">
            <strong>{table.name}</strong>
            <span>{table.columns.length} colonne</span>
          </div>
          <RelationshipColumn
            title="Relazioni in uscita"
            relationships={outgoing}
            direction="outgoing"
            onSelectTable={onSelectTable}
          />
        </div>
        <details>
          <summary>Panoramica di tutte le tabelle</summary>
          <div className="all-table-nodes">
            {allTables.map((candidate) => (
              <button
                key={candidate.name}
                className={candidate.name === table.name ? "active" : ""}
                onClick={() => onSelectTable(candidate.name)}
              >
                <strong>{candidate.name}</strong>
                <span>{candidate.rowCount} record</span>
              </button>
            ))}
          </div>
        </details>
      </div>
    </section>
  );
}

function RelationshipColumn({
  title,
  relationships,
  direction,
  onSelectTable,
}: {
  title: string;
  relationships: DatabaseRelationship[];
  direction: "incoming" | "outgoing";
  onSelectTable: (name: string) => void;
}) {
  return (
    <div className="relationship-column">
      <h4>{title}</h4>
      {relationships.length === 0 && <p>Nessuna relazione.</p>}
      {relationships.map((relationship) => {
        const linkedTable =
          direction === "incoming"
            ? relationship.sourceTable
            : relationship.targetTable;
        return (
          <button
            key={relationship.id}
            onClick={() => onSelectTable(linkedTable)}
          >
            <strong>{linkedTable}</strong>
            <span>
              {relationship.sourceColumn} → {relationship.targetColumn}
            </span>
            <small>
              {relationship.cardinality} · DELETE {relationship.onDelete}
            </small>
          </button>
        );
      })}
    </div>
  );
}

function DataView({
  table,
  page,
  loading,
  selectedRow,
  search,
  activeFilter,
  incoming,
  outgoing,
  onSearch,
  onSort,
  onSelectRow,
  onPrevious,
  onNext,
  onClearFilter,
  onNavigate,
  onExport,
  onRefresh,
  onMutationComplete,
}: {
  table: DatabaseTable;
  page: DatabaseTablePage | null;
  loading: boolean;
  selectedRow: DatabaseRow | null;
  search: string;
  activeFilter: ActiveFilter;
  incoming: DatabaseRelationship[];
  outgoing: DatabaseRelationship[];
  onSearch: (value: string) => void;
  onSort: (column: string) => void;
  onSelectRow: (row: DatabaseRow) => void;
  onPrevious: () => void;
  onNext: () => void;
  onClearFilter: () => void;
  onNavigate: (
    relationship: DatabaseRelationship,
    direction: "outgoing" | "incoming",
  ) => void;
  onExport: (format: "json" | "csv") => Promise<void>;
  onRefresh: () => Promise<void>;
  onMutationComplete: (entry: DatabaseAuditEntry) => Promise<void>;
}) {
  const maxPage = page
    ? Math.max(0, Math.ceil(page.totalCount / page.pageSize) - 1)
    : 0;
  return (
    <section className="data-view">
      <div className="data-toolbar">
        <label>
          Cerca nella tabella
          <input
            value={search}
            onChange={(event) => onSearch(event.target.value)}
            placeholder={`Cerca in ${table.name}`}
          />
        </label>
        <div className="toolbar-actions">
          <button onClick={() => void onRefresh()}>Aggiorna</button>
          <button onClick={() => void onExport("json")}>Esporta JSON</button>
          <button onClick={() => void onExport("csv")}>Esporta CSV</button>
        </div>
      </div>

      {activeFilter && (
        <div className="active-filter">
          <span>
            Filtro relazione: <strong>{activeFilter.label}</strong> ·{" "}
            {activeFilter.column} = {activeFilter.value}
          </span>
          <button onClick={onClearFilter}>Rimuovi filtro</button>
        </div>
      )}

      {loading && <p role="status">Caricamento record…</p>}
      {page && (
        <>
          <div className="data-grid-wrapper">
            <table className="data-grid">
              <thead>
                <tr>
                  {page.columns.map((column) => (
                    <th key={column}>
                      <button onClick={() => onSort(column)}>{column}</button>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {page.rows.map((row, rowIndex) => (
                  <tr
                    key={`${page.page}-${rowIndex}-${JSON.stringify(row)}`}
                    className={selectedRow === row ? "selected" : ""}
                    onClick={() => onSelectRow(row)}
                  >
                    {page.columns.map((column) => (
                      <td key={column} title={formatDatabaseValue(row[column])}>
                        {formatDatabaseValue(row[column])}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {page.rows.length === 0 && <p>Nessun record trovato.</p>}
          <div className="pagination">
            <button disabled={page.page === 0} onClick={onPrevious}>
              Precedente
            </button>
            <span>
              Pagina {page.page + 1} di {maxPage + 1} · {page.totalCount} record
            </span>
            <button disabled={page.page >= maxPage} onClick={onNext}>
              Successiva
            </button>
          </div>
        </>
      )}

      {selectedRow && (
        <RecordInspector
          table={table}
          row={selectedRow}
          incoming={incoming}
          outgoing={outgoing}
          onNavigate={onNavigate}
          onMutationComplete={onMutationComplete}
        />
      )}
    </section>
  );
}

function RecordInspector({
  table,
  row,
  incoming,
  outgoing,
  onNavigate,
  onMutationComplete,
}: {
  table: DatabaseTable;
  row: DatabaseRow;
  incoming: DatabaseRelationship[];
  outgoing: DatabaseRelationship[];
  onNavigate: (
    relationship: DatabaseRelationship,
    direction: "outgoing" | "incoming",
  ) => void;
  onMutationComplete: (entry: DatabaseAuditEntry) => Promise<void>;
}) {
  return (
    <section className="record-inspector">
      <h3>Dettaglio record</h3>
      <dl>
        {table.columns.map((column) => (
          <div key={column.name}>
            <dt>{column.name}</dt>
            <dd>
              <code>{formatDatabaseValue(row[column.name])}</code>
            </dd>
          </div>
        ))}
      </dl>

      {(outgoing.length > 0 || incoming.length > 0) && (
        <div className="record-relations">
          <h4>Naviga record collegati</h4>
          {outgoing.map((relationship) => (
            <button
              key={`out-${relationship.id}`}
              disabled={row[relationship.sourceColumn] == null}
              onClick={() => onNavigate(relationship, "outgoing")}
            >
              Apri {relationship.targetTable} tramite {relationship.sourceColumn}
            </button>
          ))}
          {incoming.map((relationship) => (
            <button
              key={`in-${relationship.id}`}
              disabled={row[relationship.targetColumn] == null}
              onClick={() => onNavigate(relationship, "incoming")}
            >
              Vedi {relationship.sourceTable} collegati
            </button>
          ))}
        </div>
      )}

      {table.name === "project_metadata" && (
        <MetadataEditor row={row} onComplete={onMutationComplete} />
      )}
      {table.name === "canon_normalized_drafts" && (
        <ReviewNoteEditor row={row} onComplete={onMutationComplete} />
      )}
    </section>
  );
}

function MetadataEditor({
  row,
  onComplete,
}: {
  row: DatabaseRow;
  onComplete: (entry: DatabaseAuditEntry) => Promise<void>;
}) {
  const key = String(row.key ?? "");
  const [value, setValue] = useState(String(row.value ?? ""));
  const [reason, setReason] = useState("");
  const [saving, setSaving] = useState(false);
  const editable = [
    "studio.workspace_name",
    "studio.release_channel",
    "studio.internal_notes",
  ].includes(key);

  if (!editable) {
    return <p className="protected-field">Questo metadato è protetto.</p>;
  }

  const save = async () => {
    setSaving(true);
    try {
      const update: ProjectMetadataUpdate = { key, value, reason } as ProjectMetadataUpdate;
      const entry = await invoke<DatabaseAuditEntry>(
        "update_database_project_metadata",
        { update },
      );
      setReason("");
      await onComplete(entry);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="safe-editor">
      <p className="eyebrow">MODIFICA CONTROLLATA</p>
      <h4>Valore metadato</h4>
      <textarea value={value} onChange={(event) => setValue(event.target.value)} />
      <input
        value={reason}
        onChange={(event) => setReason(event.target.value)}
        placeholder="Motivazione obbligatoria"
      />
      <button disabled={saving || reason.trim().length < 3} onClick={() => void save()}>
        {saving ? "Salvataggio…" : "Salva e registra audit"}
      </button>
    </div>
  );
}

function ReviewNoteEditor({
  row,
  onComplete,
}: {
  row: DatabaseRow;
  onComplete: (entry: DatabaseAuditEntry) => Promise<void>;
}) {
  const draftId = String(row.id ?? "");
  const [note, setNote] = useState("");
  const [reason, setReason] = useState("");
  const [saving, setSaving] = useState(false);

  const save = async () => {
    setSaving(true);
    try {
      const update: CanonReviewNoteUpdate = { draftId, note, reason };
      const entry = await invoke<DatabaseAuditEntry>(
        "upsert_database_review_note",
        { update },
      );
      setReason("");
      await onComplete(entry);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="safe-editor">
      <p className="eyebrow">NOTA DI REVISIONE</p>
      <p>
        La nota non modifica testo, provenienza, hash o stato canonico della
        bozza.
      </p>
      <textarea
        value={note}
        onChange={(event) => setNote(event.target.value)}
        placeholder="Annotazione umana"
      />
      <input
        value={reason}
        onChange={(event) => setReason(event.target.value)}
        placeholder="Motivazione obbligatoria"
      />
      <button disabled={saving || reason.trim().length < 3} onClick={() => void save()}>
        {saving ? "Salvataggio…" : "Salva nota e audit"}
      </button>
    </div>
  );
}

function MaintenanceView({
  snapshot,
  onIntegrity,
  onBackup,
}: {
  snapshot: DatabaseStudioSnapshot;
  onIntegrity: () => Promise<void>;
  onBackup: () => Promise<void>;
}) {
  return (
    <section className="maintenance-view">
      <div className="maintenance-card">
        <p className="eyebrow">INTEGRITÀ SQLITE</p>
        <h3>{snapshot.integrity.ok ? "Database integro" : "Verifica richiesta"}</h3>
        <p>Ultimo controllo: {snapshot.integrity.checkedAt}</p>
        <ul>
          {snapshot.integrity.messages.map((message) => (
            <li key={message}>{message}</li>
          ))}
        </ul>
        <button onClick={() => void onIntegrity()}>Esegui integrity_check</button>
      </div>
      <div className="maintenance-card">
        <p className="eyebrow">BACKUP LOCALE</p>
        <h3>Crea punto di ripristino</h3>
        <p>
          Il backup viene salvato nella cartella dati locale dopo un checkpoint.
          Non viene caricato su cloud.
        </p>
        <button onClick={() => void onBackup()}>Crea backup verificato</button>
      </div>
      <div className="maintenance-card protected-card">
        <p className="eyebrow">PROTEZIONI ATTIVE</p>
        <h3>Scritture limitate</h3>
        <ul>
          <li>Nessuna console SQL di scrittura.</li>
          <li>Nessuna modifica a PK, FK, hash o source anchor.</li>
          <li>Nessuna promozione automatica del canon.</li>
          <li>Ogni modifica consentita crea un audit record.</li>
        </ul>
      </div>
    </section>
  );
}

import { render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { DatabaseStudio, formatDatabaseValue } from "./DatabaseStudio";

it("formats database values without losing null state", () => {
  expect(formatDatabaseValue(null)).toBe("NULL");
  expect(formatDatabaseValue({ id: 1 })).toBe('{"id":1}');
});

it("renders discovered tables and relationships", async () => {
  invoke.mockImplementation((command: string) => {
    if (command === "get_database_studio_snapshot") {
      return Promise.resolve({
        databasePath: "C:/data/shadow-council-studio.sqlite",
        integrity: {
          ok: true,
          checkedAt: "2026-07-20T00:00:00Z",
          messages: ["ok"],
        },
        tables: [
          {
            name: "source_documents",
            rowCount: 1,
            createSql: "CREATE TABLE source_documents (...) ",
            migrationSource: "0001_foundation.sql",
            columns: [
              {
                name: "id",
                dataType: "TEXT",
                notNull: false,
                defaultValue: null,
                primaryKeyPosition: 1,
              },
            ],
            indexes: [],
          },
          {
            name: "canon_import_runs",
            rowCount: 1,
            createSql: "CREATE TABLE canon_import_runs (...) ",
            migrationSource: "0002_canon_import.sql",
            columns: [
              {
                name: "source_document_id",
                dataType: "TEXT",
                notNull: true,
                defaultValue: null,
                primaryKeyPosition: 0,
              },
            ],
            indexes: [],
          },
        ],
        relationships: [
          {
            id: "canon_import_runs:source_document_id->source_documents:id",
            sourceTable: "canon_import_runs",
            sourceColumn: "source_document_id",
            targetTable: "source_documents",
            targetColumn: "id",
            onUpdate: "NO ACTION",
            onDelete: "NO ACTION",
            cardinality: "N:1",
          },
        ],
      });
    }
    return Promise.reject(new Error(`Unexpected command: ${command}`));
  });

  render(<DatabaseStudio />);
  expect(await screen.findByText("Database Studio")).toBeInTheDocument();
  expect((await screen.findAllByText("source_documents")).length).toBeGreaterThan(0);
  expect(screen.getByText("Relazioni")).toBeInTheDocument();
  expect(screen.getByText("OK")).toBeInTheDocument();
});

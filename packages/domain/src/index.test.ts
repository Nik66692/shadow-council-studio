import { describe, expect, it } from "vitest";
import {
  CanonImportReviewSnapshotSchema,
  CanonicalStatusSchema,
  HealthStatusSchema,
  parseStableId,
  reservedIdExamples,
} from "./index";

describe("domain foundation", () => {
  it("parses reserved stable id families", () => {
    for (const id of reservedIdExamples) expect(parseStableId(id)).toBe(id);
  });
  it("rejects unknown id families", () =>
    expect(() => parseStableId("SC-UNKNOWN-0001")).toThrow());
  it("preserves canonical status taxonomy", () =>
    expect(CanonicalStatusSchema.options).toContain("PUNTO_APERTO"));
  it("validates health contracts", () => {
    expect(
      HealthStatusSchema.parse({
        projectName: "Shadow Council Studio",
        developmentStage: "Phase 1",
        databaseConnected: true,
        migrationsApplied: true,
        sourceOfTruth: {
          exists: false,
          filename: "docs/canon/source/manifest.json",
          sha256: null,
          canonVersion: null,
        },
        modulesImplemented: ["Dashboard"],
        nextRecommendedPhase: "Phase 1",
        diagnostics: [],
      }).databaseConnected,
    ).toBe(true);
  });
  it("keeps imported drafts pending and without canon status", () => {
    const snapshot = CanonImportReviewSnapshotSchema.parse({
      run: {
        id: "canon-import-123",
        sourceDocumentId: "source-of-truth-v1.3",
        sourceVersion: "1.3",
        sourceSha256: "a".repeat(64),
        importerVersion: "canon-docx-importer/1.0.0",
        status: "COMPLETED_PENDING_REVIEW",
        startedAt: "2026-07-20T00:00:00.000Z",
        completedAt: "2026-07-20T00:00:00.000Z",
        rawBlockCount: 1,
        draftCount: 1,
        warningCount: 0,
      },
      drafts: [
        {
          id: "canon-draft-123",
          rawBlockId: "canon-block-123",
          sourceAnchor: "sc://canon/1.3/hash/word/document.xml/heading/000000",
          sourcePart: "word/document.xml",
          blockIndex: 0,
          blockKind: "HEADING",
          styleName: "Heading1",
          originalText: "Testo originale",
          textSha256: "b".repeat(64),
          reviewStatus: "PENDING_HUMAN_REVIEW",
          canonicalStatus: null,
        },
      ],
      warnings: [],
      importedNow: true,
    });
    expect(snapshot.drafts[0]?.canonicalStatus).toBeNull();
  });
});

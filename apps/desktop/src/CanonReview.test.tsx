import { render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { CanonReview } from "./CanonReview";

const workspace = {
  summary: {
    pendingCount: 1,
    approvedCount: 0,
    rejectedCount: 0,
    entryCount: 0,
  },
  drafts: [
    {
      id: "draft-1",
      importRunId: "run-1",
      rawBlockId: "block-1",
      sourceAnchor:
        "sc://canon/1.3/hash/word/document.xml/paragraph/000001",
      sourcePart: "word/document.xml",
      blockIndex: 1,
      blockKind: "PARAGRAPH",
      styleName: null,
      originalText: "Regola estratta",
      textSha256: "a".repeat(64),
      reviewStatus: "PENDING_HUMAN_REVIEW",
      canonicalStatus: null,
    },
  ],
  entries: [],
  recentDecisions: [],
};

beforeEach(() => {
  vi.mocked(invoke).mockResolvedValue(workspace);
});

it("loads the controlled review queue", async () => {
  render(<CanonReview />);
  expect(
    await screen.findByRole("heading", { name: "Canon Review" }),
  ).toBeInTheDocument();
  expect(screen.getByText("Regola estratta")).toBeInTheDocument();
  expect(screen.getByText("1 elementi visibili")).toBeInTheDocument();
});

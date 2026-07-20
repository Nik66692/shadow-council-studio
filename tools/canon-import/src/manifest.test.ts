import { mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { mkdtempSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  canonicalManifestRelativePath,
  createManifest,
  sha256File,
} from "./manifest";

function makeRoot(): string {
  return mkdtempSync(path.join(tmpdir(), "sc-canon-"));
}

function writeManifest(root: string, sourcePath: string, sha256: string): void {
  const manifestPath = path.join(root, canonicalManifestRelativePath);
  mkdirSync(path.dirname(manifestPath), { recursive: true });
  writeFileSync(
    manifestPath,
    JSON.stringify({
      schemaVersion: 1,
      currentVersion: "1.3",
      currentSource: sourcePath,
      status: "alpha-provisional",
      approvedBy: "Niccolò",
      approvedAt: "2026-07-17",
      sha256,
    }),
  );
}

describe("canon import manifest", () => {
  it("creates a deterministic manifest for a valid source", () => {
    const root = makeRoot();
    const source =
      "docs/canon/source/v1.3/Shadow_Council_Source_of_Truth_v1.3.docx";
    const sourceFile = path.join(root, source);
    mkdirSync(path.dirname(sourceFile), { recursive: true });
    writeFileSync(sourceFile, "canon");
    writeManifest(root, source, sha256File(sourceFile));

    expect(createManifest(root)).toMatchObject({
      currentVersion: "1.3",
      sourcePath: source,
      exists: true,
      sha256: sha256File(sourceFile),
      semanticExtraction: "not-implemented-in-sprint-0",
    });
  });

  it("fails when the source manifest is missing", () => {
    expect(() => createManifest(makeRoot())).toThrow(/manifest is missing/);
  });

  it("fails when the manifest source is missing", () => {
    const root = makeRoot();
    writeManifest(root, "docs/canon/source/v1.3/missing.docx", "a".repeat(64));
    expect(() => createManifest(root)).toThrow(
      /source referenced by manifest is missing/,
    );
  });

  it("fails when the computed hash does not match the manifest hash", () => {
    const root = makeRoot();
    const source =
      "docs/canon/source/v1.3/Shadow_Council_Source_of_Truth_v1.3.docx";
    const sourceFile = path.join(root, source);
    mkdirSync(path.dirname(sourceFile), { recursive: true });
    writeFileSync(sourceFile, "canon");
    writeManifest(root, source, "b".repeat(64));
    expect(() => createManifest(root)).toThrow(/hash mismatch/);
  });
});

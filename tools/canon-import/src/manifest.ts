import { createHash } from 'node:crypto';
import { existsSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';

export const canonicalManifestRelativePath = 'docs/canon/source/manifest.json';

export interface SourceManifest {
  schemaVersion: number;
  currentVersion: string;
  currentSource: string;
  status: string;
  approvedBy: string;
  approvedAt: string;
  sha256: string;
}

export interface ImportManifest {
  schemaVersion: 1;
  sourceManifest: string;
  currentVersion: string | null;
  sourcePath: string | null;
  exists: boolean;
  sha256: string | null;
  sizeBytes: number | null;
  generatedAt: string;
  warnings: string[];
  semanticExtraction: 'not-implemented-in-sprint-0';
}

export class CanonManifestError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CanonManifestError';
  }
}

export function sha256File(filePath: string): string {
  return createHash('sha256').update(readFileSync(filePath)).digest('hex');
}

export function readSourceManifest(rootDir: string): SourceManifest {
  const manifestPath = path.join(rootDir, canonicalManifestRelativePath);
  if (!existsSync(manifestPath)) {
    throw new CanonManifestError(`Canonical source manifest is missing: ${canonicalManifestRelativePath}`);
  }

  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8')) as Partial<SourceManifest>;
  if (
    manifest.schemaVersion !== 1 ||
    typeof manifest.currentVersion !== 'string' ||
    typeof manifest.currentSource !== 'string' ||
    typeof manifest.status !== 'string' ||
    typeof manifest.approvedBy !== 'string' ||
    typeof manifest.approvedAt !== 'string' ||
    typeof manifest.sha256 !== 'string'
  ) {
    throw new CanonManifestError('Canonical source manifest is invalid or incomplete.');
  }

  return manifest as SourceManifest;
}

export function createManifest(rootDir: string, generatedAt = '1970-01-01T00:00:00.000Z'): ImportManifest {
  const sourceManifest = readSourceManifest(rootDir);
  const sourcePath = path.join(rootDir, sourceManifest.currentSource);
  if (!existsSync(sourcePath)) {
    throw new CanonManifestError(`Canonical source referenced by manifest is missing: ${sourceManifest.currentSource}`);
  }

  const computedHash = sha256File(sourcePath);
  if (computedHash !== sourceManifest.sha256) {
    throw new CanonManifestError(
      `Canonical source hash mismatch for ${sourceManifest.currentSource}: expected ${sourceManifest.sha256}, got ${computedHash}`,
    );
  }

  const stat = statSync(sourcePath);
  return {
    schemaVersion: 1,
    sourceManifest: canonicalManifestRelativePath,
    currentVersion: sourceManifest.currentVersion,
    sourcePath: sourceManifest.currentSource,
    exists: true,
    sha256: computedHash,
    sizeBytes: stat.size,
    generatedAt,
    warnings: ['semantic DOCX extraction is intentionally not implemented in Sprint 0'],
    semanticExtraction: 'not-implemented-in-sprint-0',
  };
}

export function writeManifest(rootDir: string, manifest: ImportManifest): string {
  const out = path.join(rootDir, 'docs/canon/registry/source-manifest.json');
  writeFileSync(out, `${JSON.stringify(manifest, null, 2)}\n`);
  return out;
}

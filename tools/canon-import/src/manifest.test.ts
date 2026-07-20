import { mkdtempSync, mkdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { canonicalSourceRelativePath, createManifest, sha256File } from './manifest';
describe('canon import manifest', () => {
  it('warns when source is missing', () => { const dir = mkdtempSync(path.join(tmpdir(), 'sc-missing-')); expect(createManifest(dir).exists).toBe(false); });
  it('hashes source deterministically', () => { const dir = mkdtempSync(path.join(tmpdir(), 'sc-source-')); const file = path.join(dir, canonicalSourceRelativePath); mkdirSync(path.dirname(file), { recursive:true }); writeFileSync(file, 'canon'); expect(sha256File(file)).toHaveLength(64); expect(createManifest(dir).sha256).toBe(sha256File(file)); });
});

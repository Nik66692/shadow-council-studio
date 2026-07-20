import { describe, expect, it } from 'vitest';
import { CanonicalStatusSchema, HealthStatusSchema, parseStableId, reservedIdExamples } from './index';

describe('domain foundation', () => {
  it('parses reserved stable id families', () => {
    for (const id of reservedIdExamples) expect(parseStableId(id)).toBe(id);
  });
  it('rejects unknown id families', () => expect(() => parseStableId('SC-UNKNOWN-0001')).toThrow());
  it('preserves canonical status taxonomy', () => expect(CanonicalStatusSchema.options).toContain('PUNTO_APERTO'));
  it('validates health contracts', () => {
    expect(HealthStatusSchema.parse({ projectName:'Shadow Council Studio', developmentStage:'Foundation', databaseConnected:true, migrationsApplied:true, sourceOfTruth:{ exists:false, filename:'Shadow_Council_Source_of_Truth_v1.3.docx', sha256:null, canonVersion:null }, modulesImplemented:['Dashboard'], nextRecommendedPhase:'Phase 1', diagnostics:[]}).databaseConnected).toBe(true);
  });
});

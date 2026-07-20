import { z } from "zod";

export const canonicalStatuses = [
  "CANONICO",
  "ALPHA_DA_TESTARE",
  "IPOTESI_LINEA_GUIDA",
  "MAYBE",
  "RISCHIO",
  "SCARTATO_SUPERATO",
  "PUNTO_APERTO",
] as const;
export const CanonicalStatusSchema = z.enum(canonicalStatuses);
export type CanonicalStatus = z.infer<typeof CanonicalStatusSchema>;

const idPattern =
  /^SC-(RULE|MECH|KW|CARD-MIL|AGEN|DIR|RFC|DEC|PT|RISK|BUG|ASSET|REL)-\d{4}$/;
export type StableId = string & { readonly __brand: "StableId" };
export const reservedIdExamples = [
  "SC-RULE-0001",
  "SC-MECH-0001",
  "SC-KW-0001",
  "SC-CARD-MIL-0001",
  "SC-AGEN-0001",
  "SC-DIR-0001",
  "SC-RFC-0001",
  "SC-DEC-0001",
  "SC-PT-0001",
  "SC-RISK-0001",
  "SC-BUG-0001",
  "SC-ASSET-0001",
  "SC-REL-0001",
] as const;
export function parseStableId(value: string): StableId {
  if (!idPattern.test(value))
    throw new Error(`Invalid Shadow Council stable id: ${value}`);
  return value as StableId;
}

export const SourceDocumentMetadataSchema = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  version: z.string().min(1),
  authorityRank: z.number().int().positive(),
  originalPath: z.string().min(1),
  sha256: z
    .string()
    .regex(/^[a-f0-9]{64}$/)
    .nullable(),
  importedAt: z.string().datetime(),
  immutable: z.boolean(),
  notes: z.string().nullable(),
});
export type SourceDocumentMetadata = z.infer<
  typeof SourceDocumentMetadataSchema
>;

export const ProjectMetadataSchema = z.object({
  key: z.string().min(1),
  value: z.string(),
  updatedAt: z.string().datetime(),
});
export type ProjectMetadata = z.infer<typeof ProjectMetadataSchema>;

export const HealthStatusSchema = z.object({
  projectName: z.literal("Shadow Council Studio"),
  developmentStage: z.literal("Foundation"),
  databaseConnected: z.boolean(),
  migrationsApplied: z.boolean(),
  sourceOfTruth: z.object({
    exists: z.boolean(),
    filename: z.string(),
    sha256: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    canonVersion: z.string().nullable(),
  }),
  modulesImplemented: z.array(z.string()),
  nextRecommendedPhase: z.string(),
  diagnostics: z.array(z.string()),
});
export type HealthStatus = z.infer<typeof HealthStatusSchema>;

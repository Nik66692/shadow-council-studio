import { z } from "zod";

const CanonEntryCanonicalStatusSchema = z.enum([
  "CANONICO",
  "ALPHA_DA_TESTARE",
  "IPOTESI_LINEA_GUIDA",
  "MAYBE",
  "RISCHIO",
  "SCARTATO_SUPERATO",
  "PUNTO_APERTO",
]);

export const canonEntryKinds = [
  "RULE",
  "MECHANIC",
  "DEFINITION",
  "COMPONENT",
  "PROCEDURE",
  "DECKBUILDING",
  "VISUAL_SPEC",
  "OPEN_POINT",
  "RISK",
  "OTHER",
] as const;

export const CanonEntryKindSchema = z.enum(canonEntryKinds);
export type CanonEntryKind = z.infer<typeof CanonEntryKindSchema>;

export const CanonDraftReviewStatusSchema = z.enum([
  "PENDING_HUMAN_REVIEW",
  "APPROVED",
  "MERGED_INTO_ENTRY",
  "REJECTED",
]);
export type CanonDraftReviewStatus = z.infer<
  typeof CanonDraftReviewStatusSchema
>;

export const CanonReviewDraftItemSchema = z.object({
  id: z.string().min(1),
  importRunId: z.string().min(1),
  rawBlockId: z.string().min(1),
  sourceAnchor: z.string().min(1),
  sourcePart: z.literal("word/document.xml"),
  blockIndex: z.number().int().nonnegative(),
  blockKind: z.string().min(1),
  styleName: z.string().nullable(),
  originalText: z.string(),
  textSha256: z.string().regex(/^[a-f0-9]{64}$/),
  reviewStatus: CanonDraftReviewStatusSchema,
  canonicalStatus: CanonEntryCanonicalStatusSchema.nullable(),
});
export type CanonReviewDraftItem = z.infer<typeof CanonReviewDraftItemSchema>;

export const CanonEntrySourceSchema = z.object({
  draftId: z.string().min(1),
  sourceOrder: z.number().int().nonnegative(),
  sourceAnchor: z.string().min(1),
  blockIndex: z.number().int().nonnegative(),
  blockKind: z.string().min(1),
  originalText: z.string(),
  textSha256: z.string().regex(/^[a-f0-9]{64}$/),
});
export type CanonEntrySource = z.infer<typeof CanonEntrySourceSchema>;

export const CanonEntrySchema = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  entryKind: CanonEntryKindSchema,
  canonicalStatus: CanonEntryCanonicalStatusSchema,
  normalizedText: z.string().min(1),
  lifecycleStatus: z.enum(["ACTIVE", "SUPERSEDED", "RETIRED"]),
  approvedBy: z.string().min(1),
  approvedAt: z.string().datetime(),
  rationale: z.string().min(1),
  createdAt: z.string().datetime(),
  updatedAt: z.string().datetime(),
  sources: z.array(CanonEntrySourceSchema).min(1),
});
export type CanonEntry = z.infer<typeof CanonEntrySchema>;

export const CanonReviewDecisionSchema = z.object({
  id: z.string().min(1),
  decisionType: z.enum(["APPROVED", "REJECTED"]),
  draftId: z.string().min(1),
  entryId: z.string().nullable(),
  reviewer: z.string().min(1),
  rationale: z.string().min(1),
  decidedAt: z.string().datetime(),
  previousReviewStatus: z.literal("PENDING_HUMAN_REVIEW"),
  resultingReviewStatus: z.enum(["APPROVED", "MERGED_INTO_ENTRY", "REJECTED"]),
});
export type CanonReviewDecision = z.infer<typeof CanonReviewDecisionSchema>;

export const CanonReviewSummarySchema = z.object({
  pendingCount: z.number().int().nonnegative(),
  approvedCount: z.number().int().nonnegative(),
  rejectedCount: z.number().int().nonnegative(),
  entryCount: z.number().int().nonnegative(),
});
export type CanonReviewSummary = z.infer<typeof CanonReviewSummarySchema>;

export const CanonReviewWorkspaceSchema = z.object({
  summary: CanonReviewSummarySchema,
  drafts: z.array(CanonReviewDraftItemSchema),
  entries: z.array(CanonEntrySchema),
  recentDecisions: z.array(CanonReviewDecisionSchema),
});
export type CanonReviewWorkspace = z.infer<typeof CanonReviewWorkspaceSchema>;

export const ApproveCanonDraftsRequestSchema = z.object({
  draftIds: z.array(z.string().min(1)).min(1).max(50),
  title: z.string().trim().min(1).max(200),
  entryKind: CanonEntryKindSchema,
  canonicalStatus: CanonEntryCanonicalStatusSchema,
  normalizedText: z.string().trim().min(1).max(20_000),
  reviewer: z.string().trim().min(1).max(100),
  rationale: z.string().trim().min(1).max(2_000),
});
export type ApproveCanonDraftsRequest = z.infer<
  typeof ApproveCanonDraftsRequestSchema
>;

export const RejectCanonDraftsRequestSchema = z.object({
  draftIds: z.array(z.string().min(1)).min(1).max(50),
  reviewer: z.string().trim().min(1).max(100),
  rationale: z.string().trim().min(1).max(2_000),
});
export type RejectCanonDraftsRequest = z.infer<
  typeof RejectCanonDraftsRequestSchema
>;

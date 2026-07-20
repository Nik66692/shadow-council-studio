import { z } from "zod";

export const DatabaseColumnSchema = z.object({
  name: z.string().min(1),
  dataType: z.string(),
  notNull: z.boolean(),
  defaultValue: z.string().nullable(),
  primaryKeyPosition: z.number().int().nonnegative(),
});
export type DatabaseColumn = z.infer<typeof DatabaseColumnSchema>;

export const DatabaseIndexSchema = z.object({
  name: z.string().min(1),
  unique: z.boolean(),
  origin: z.string(),
  partial: z.boolean(),
  columns: z.array(z.string()),
});
export type DatabaseIndex = z.infer<typeof DatabaseIndexSchema>;

export const DatabaseRelationshipSchema = z.object({
  id: z.string().min(1),
  sourceTable: z.string().min(1),
  sourceColumn: z.string().min(1),
  targetTable: z.string().min(1),
  targetColumn: z.string().min(1),
  onUpdate: z.string(),
  onDelete: z.string(),
  cardinality: z.enum(["1:1", "N:1"]),
});
export type DatabaseRelationship = z.infer<typeof DatabaseRelationshipSchema>;

export const DatabaseTableSchema = z.object({
  name: z.string().min(1),
  rowCount: z.number().int().nonnegative(),
  createSql: z.string().nullable(),
  migrationSource: z.string().nullable(),
  columns: z.array(DatabaseColumnSchema),
  indexes: z.array(DatabaseIndexSchema),
});
export type DatabaseTable = z.infer<typeof DatabaseTableSchema>;

export const DatabaseIntegrityReportSchema = z.object({
  ok: z.boolean(),
  checkedAt: z.string().datetime(),
  messages: z.array(z.string()),
});
export type DatabaseIntegrityReport = z.infer<
  typeof DatabaseIntegrityReportSchema
>;

export const DatabaseStudioSnapshotSchema = z.object({
  databasePath: z.string().min(1),
  tables: z.array(DatabaseTableSchema),
  relationships: z.array(DatabaseRelationshipSchema),
  integrity: DatabaseIntegrityReportSchema,
});
export type DatabaseStudioSnapshot = z.infer<
  typeof DatabaseStudioSnapshotSchema
>;

export const TableBrowseRequestSchema = z.object({
  tableName: z.string().min(1),
  page: z.number().int().nonnegative(),
  pageSize: z.number().int().min(1).max(100),
  search: z.string().nullable(),
  sortColumn: z.string().nullable(),
  sortDirection: z.enum(["ASC", "DESC"]).nullable(),
  filterColumn: z.string().nullable(),
  filterValue: z.string().nullable(),
});
export type TableBrowseRequest = z.infer<typeof TableBrowseRequestSchema>;

export const DatabaseTablePageSchema = z.object({
  tableName: z.string().min(1),
  columns: z.array(z.string()),
  rows: z.array(z.record(z.string(), z.unknown())),
  totalCount: z.number().int().nonnegative(),
  page: z.number().int().nonnegative(),
  pageSize: z.number().int().positive(),
  sortColumn: z.string().min(1),
  sortDirection: z.enum(["ASC", "DESC"]),
  filterColumn: z.string().nullable(),
  filterValue: z.string().nullable(),
});
export type DatabaseTablePage = z.infer<typeof DatabaseTablePageSchema>;

export const DatabaseFileResultSchema = z.object({
  path: z.string().min(1),
  fileName: z.string().min(1),
  sha256: z.string().regex(/^[a-f0-9]{64}$/),
  sizeBytes: z.number().int().nonnegative(),
  createdAt: z.string().datetime(),
});
export type DatabaseFileResult = z.infer<typeof DatabaseFileResultSchema>;

export const DatabaseAuditEntrySchema = z.object({
  id: z.string().min(1),
  entityType: z.string().min(1),
  recordId: z.string().min(1),
  fieldName: z.string().min(1),
  oldValue: z.string().nullable(),
  newValue: z.string().nullable(),
  reason: z.string().min(3),
  changedAt: z.string().datetime(),
});
export type DatabaseAuditEntry = z.infer<typeof DatabaseAuditEntrySchema>;

export const ProjectMetadataUpdateSchema = z.object({
  key: z.enum([
    "studio.workspace_name",
    "studio.release_channel",
    "studio.internal_notes",
  ]),
  value: z.string(),
  reason: z.string().min(3),
});
export type ProjectMetadataUpdate = z.infer<typeof ProjectMetadataUpdateSchema>;

export const CanonReviewNoteUpdateSchema = z.object({
  draftId: z.string().min(1),
  note: z.string(),
  reason: z.string().min(3),
});
export type CanonReviewNoteUpdate = z.infer<typeof CanonReviewNoteUpdateSchema>;

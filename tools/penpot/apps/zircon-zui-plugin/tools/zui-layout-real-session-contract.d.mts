export function sha256(value: string | Uint8Array): string;
export function requireRealSessionDirectory(path: string, resolvePhysical?: (path: string) => string): string;
export function workspaceIdentity(url: string): { origin: string; teamId: string; fileId: string; pageId: string };
export function realSessionExecutionPlan(options: unknown, context: {
  workspaceUrl?: string; authenticatedStorage: boolean; uiBuildReceipt: boolean; workerBuildReceipt: boolean;
}): { mode: 'reuse-existing' | 'managed-bootstrap'; bootstrapAllowed: boolean };
export function verifySaveResponse(response: unknown, identity: { fileId: string }, previousRevision: number,
  saveRequest: { id: string; revn: number; 'session-id': string; changes: unknown[] }): number;
export function verifyReopenedFile(before: unknown, after: unknown, identity: { fileId: string },
  acknowledgedRevision: number, metadataBefore: unknown[], metadataAfter: unknown[]): boolean;
export function verifySourceOwnerAppendAdmission(file: unknown, identity: { fileId: string; pageId: string },
  append: { fileId: string; pageId: string; revn: number; vern: number },
  context: { sourceClosure: boolean; existingPlugin: boolean; authenticatedStorage: boolean;
    previousReceipt: boolean; editRequested: boolean; fullWorkbench: boolean }): boolean;
export function assertExistingPenpotContentPreserved(before: unknown, after: unknown): void;
export function sourceColorRevisionSnapshot(file: unknown, linkedFiles: unknown[], pageId: string,
  boardId: string, document: unknown): {
  schema: string; consumer: { id: string; revn: number; vern: number }; pageId: string; boardId: string;
  libraries: Array<{ id: string; revn: number; vern: number }>;
  references: Array<{ shapeId: string; path: string; kind: string; refId: string; refFile: string;
    color: string; opacity: number; sourcePath: string; sourceSha256: string; token: string; styleSha256: string }>;
};
export function assertSourceColorRevisionPreserved(before: unknown, after: unknown): void;
export function assertNativeInstanceGeometryPreserved(before: unknown, after: unknown): {
  copies: Array<{ pageId: string; rootId: string }>; shapesChecked: number;
  schema: string; fileId: string;
  before: { revn: number | null; vern: number | null };
  after: { revn: number | null; vern: number | null };
  hashFormat: string; nodeVersion: string; v8Version: string;
  nodes: Array<{ pageId: string; rootId: string; id: string; shapeRef: string | null;
    beforeSha256: string; afterSha256: string | null; protectedStatePreserved: boolean }>;
  changedNodes: Array<{ pageId: string; rootId: string; id: string; shapeRef: string | null;
    beforeSha256: string; afterSha256: string | null; protectedStatePreserved: boolean;
    fields: Record<string, { before: { present: boolean; sha256?: string; value?: unknown; valueOmitted?: string };
      after: { present: boolean; sha256?: string; value?: unknown; valueOmitted?: string } }> }>;
  protectedStatePreserved: boolean;
};
export function verifyReviewScrollCleanupAdmission(
  acknowledgedMetadata: Array<{ boardId: string; metadata: Record<string, unknown> }>,
  currentMetadata: Array<{ boardId: string; metadata: Record<string, unknown> }>,
): Array<{ boardId: string; key: string }>;
export function persistedZuiMetadata(file: unknown, pageId: string): Array<{
  boardId: string; boardName: string; metadata: { document: string; [key: string]: unknown };
}>;
export function persistenceClaim(evidence: unknown): boolean;
export function persistedViewportToolbarGroupMetadata(file: unknown, pageId: string, assetBoardId: string,
  contract: { row: string; popup: string; groups: readonly string[] }): Array<{
  nodeId: string; boardId: string; parentBoardId: string; parentNodeId: string; metadata: Record<string, unknown>;
}>;

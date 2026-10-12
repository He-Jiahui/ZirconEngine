export function sha256(value: string | Uint8Array): string;
export function requireRealSessionDirectory(path: string, resolvePhysical?: (path: string) => string): string;
export function workspaceIdentity(url: string): { origin: string; teamId: string; fileId: string; pageId: string };
export function verifySaveResponse(response: unknown, identity: { fileId: string }, previousRevision: number,
  saveRequest: { id: string; revn: number; 'session-id': string; changes: unknown[] }): number;
export function verifyReopenedFile(before: unknown, after: unknown, identity: { fileId: string },
  acknowledgedRevision: number, metadataBefore: unknown[], metadataAfter: unknown[]): boolean;
export function persistedZuiMetadata(file: unknown, pageId: string): Array<{
  boardId: string; boardName: string; metadata: { document: string; [key: string]: unknown };
}>;
export function persistenceClaim(evidence: unknown): boolean;
export function persistedViewportToolbarGroupMetadata(file: unknown, pageId: string, assetBoardId: string,
  contract: { row: string; popup: string; groups: readonly string[] }): Array<{
  nodeId: string; boardId: string; parentBoardId: string; parentNodeId: string; metadata: Record<string, unknown>;
}>;

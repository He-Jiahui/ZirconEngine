import type { ZuiDocument } from '../../zircon-zui-plugin/src/bridge/zui-document';
export function sha256(bytes: string | Uint8Array): string;
export function serializeDocument(document: ZuiDocument): string;
export function compileBundle(bundle: { manifest: unknown; entries: Record<string, Uint8Array> }, contractSource: string,
  translations?: Record<string, unknown>, options?: { includePreview?: boolean }): {
  files: Record<string, Uint8Array>; documents: Record<string, ZuiDocument>;
  lock: { outputs: Record<string, string> };
};

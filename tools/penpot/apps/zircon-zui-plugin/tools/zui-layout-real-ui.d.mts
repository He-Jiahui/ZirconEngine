export function verifyUiBytes(expected: string, bytes: Uint8Array, label: string): void;
export function verifyUiSourceHashes(expected: Record<string, string>, current: Record<string, string>): void;
export function uiIndexAssetNames(html: string, outputs: { relative: string }[]): string[];
export function verifyCurrentPluginUi(options: {
  buildReceiptPath: string; repositoryRoot: string; dist: string; host: string; indexBytes: Buffer;
  get(url: string): Promise<{ status(): number; body(): Promise<Buffer> }>;
}): Promise<Record<string, unknown>>;

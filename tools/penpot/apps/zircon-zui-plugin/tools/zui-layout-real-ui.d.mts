export function verifyUiBytes(expected: string, bytes: Uint8Array, label: string): void;
export function verifyUiSourceHashes(expected: Record<string, string>, current: Record<string, string>): void;
export function uiIndexAssetNames(html: string, outputs: { relative: string }[]): string[];
export function verifyCurrentPluginUi(options: {
  buildReceiptPath: string; repositoryRoot: string; dist: string; host: string; indexBytes: Buffer;
  get(url: string): Promise<{ status(): number; body(): Promise<Buffer> }>;
}): Promise<Record<string, unknown>>;

export function verifyPluginWorkerBuildReceipt(build: unknown, options: {
  repositoryRoot: string; bundleSha256: string; currentInputs: Record<string, string>;
}): { inputCount: number; sourceUnchanged: boolean; workerSha256: string };
export function verifyCurrentPluginWorkerBuild(options: {
  buildReceiptPath: string; buildReceiptSha256: string; repositoryRoot: string; bundleBytes: Uint8Array;
}): Promise<Record<string, unknown>>;
export function verifyCurrentPluginWorker(options: {
  host: string; bundleBytes: Uint8Array; distributionWorkerBytes: Uint8Array;
  get(url: string): Promise<{ status(): number; url(): string; body(): Promise<Buffer> }>;
}): Promise<Record<string, unknown>>;

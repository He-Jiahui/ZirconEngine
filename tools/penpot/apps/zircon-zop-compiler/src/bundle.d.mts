export interface BundleFile { id: string; revision: number }
export function createBundle(entries: Record<string, Uint8Array>, files: BundleFile[], digest: (bytes: Uint8Array) => Promise<string>): Promise<{
  manifest: { schema_version: number; files: BundleFile[]; entries: Record<string, string> };
  entries: Record<string, Uint8Array>;
}>;
export function zipBundle(entries: Record<string, Uint8Array>): Uint8Array;

import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { mkdir, writeFile } from 'node:fs/promises';
import { relative, resolve } from 'node:path';
import type { Page, Response } from 'playwright';
import { bytesSha256 } from './zui-layout-evidence';

export interface FontResourceRecord {
  /** The exact browser response URL named by the loaded CSS font face. */
  url: string;
  /** Repository-relative copy used for the dependency/runtime asset gate. */
  path: string;
  sha256: string;
}

export function collectFontResources(
  page: Page,
  repoRoot: string,
  catalogRoot: string,
  write: boolean,
) {
  const pending = new Set<Promise<void>>();
  const resources = new Map<string, FontResourceRecord>();
  const failures: string[] = [];
  const capture = async (response: Response): Promise<void> => {
    if (!response.ok())
      throw new Error(`Font response ${response.status()}: ${response.url()}`);
    const bytes = await response.body();
    const sha256 = bytesSha256(bytes);
    const signature = bytes.subarray(0, 4).toString('ascii');
    const extension =
      { wOF2: 'woff2', wOFF: 'woff', OTTO: 'otf', ttcf: 'ttc' }[signature] ??
      'ttf';
    const directory = resolveLayoutCatalogPath(catalogRoot, 'resources/fonts');
    const path = resolve(directory, `${sha256}.${extension}`);
    if (write) {
      await mkdir(directory, { recursive: true });
      await writeFile(path, bytes);
    }
    resources.set(response.url(), {
      path: relative(repoRoot, path).replaceAll('\\', '/'),
      sha256,
      url: response.url(),
    });
  };
  const listener = (response: Response): void => {
    if (response.request().resourceType() !== 'font') return;
    const task = capture(response)
      .catch((error: unknown) => {
        failures.push(error instanceof Error ? error.message : String(error));
      })
      .finally(() => pending.delete(task));
    pending.add(task);
  };
  page.on('response', listener);
  return {
    async snapshotRecords(): Promise<FontResourceRecord[]> {
      while (pending.size) await Promise.all([...pending]);
      if (failures.length) throw new Error(failures.join('\n'));
      const records = [...resources.values()].sort((a, b) =>
        a.url.localeCompare(b.url),
      );
      if (write)
        await writeFile(
          resolveLayoutCatalogPath(catalogRoot, 'resources/fonts/manifest.json'),
          `${JSON.stringify(records, null, 2)}\n`,
        );
      return records;
    },
    async snapshot(): Promise<Array<[string, string]>> {
      const records = await this.snapshotRecords();
      return [
        ...new Map(
          records.map(({ path, sha256 }) => [path, sha256]),
        ).entries(),
      ];
    },
    stop(): void {
      page.off('response', listener);
    },
  };
}

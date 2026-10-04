import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

import {
  acquireLayoutCatalogLock,
  withLayoutCatalogLock,
} from '../tools/zui-layout-catalog-lock';

describe('layout catalog capture lock', () => {
  const roots: string[] = [];

  async function catalogRoot(): Promise<string> {
    const root = await mkdtemp(join(tmpdir(), 'zircon-layout-catalog-lock-'));
    roots.push(root);
    return root;
  }

  afterEach(async () => {
    await Promise.all(
      roots.splice(0).map((root) => rm(root, { recursive: true, force: true })),
    );
  });

  it('publishes an owner receipt and removes only its own lock', async () => {
    const root = await catalogRoot();
    const lock = await acquireLayoutCatalogLock(root, {
      command: 'vitest owner receipt',
      processId: 42001,
    });

    const owner = JSON.parse(
      await readFile(join(lock.path, 'owner.json'), 'utf8'),
    ) as { processId: number; command: string; token: string };
    expect(owner).toMatchObject({
      processId: 42001,
      command: 'vitest owner receipt',
    });
    expect(owner.token).not.toEqual('');

    await lock.release();
    expect(existsSync(lock.path)).toBe(false);
  });

  it('rejects a concurrent writer before it can alter the catalog', async () => {
    const root = await catalogRoot();
    const lock = await acquireLayoutCatalogLock(root, {
      command: 'first capture',
      processId: 42002,
    });

    await expect(
      acquireLayoutCatalogLock(root, {
        command: 'second capture',
        processId: 42003,
        isProcessAlive: (processId) => processId === 42002,
      }),
    ).rejects.toThrow(/already held.*42002.*first capture/i);

    await lock.release();
  });

  it('reclaims a receipt only after its owner is confirmed dead', async () => {
    const root = await catalogRoot();
    await acquireLayoutCatalogLock(root, {
      command: 'abandoned capture',
      processId: 42004,
    });

    const replacement = await acquireLayoutCatalogLock(root, {
      command: 'replacement capture',
      processId: 42005,
      isProcessAlive: () => false,
    });

    expect(replacement.owner).toMatchObject({
      processId: 42005,
      command: 'replacement capture',
    });
    await replacement.release();
  });

  it('releases the receipt when a capture throws', async () => {
    const root = await catalogRoot();

    await expect(
      withLayoutCatalogLock(
        root,
        { command: 'failing capture', processId: 42006 },
        async () => {
          throw new Error('capture failed');
        },
      ),
    ).rejects.toThrow('capture failed');

    expect(existsSync(join(root, '.zui-layout-catalog.lock'))).toBe(false);
  });
});

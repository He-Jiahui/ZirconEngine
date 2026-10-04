import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { randomUUID } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

const lockDirectoryName = '.zui-layout-catalog.lock';
const ownerFileName = 'owner.json';

export interface LayoutCatalogLockOwner {
  schema: 'dev.zircon.zui.layout-catalog-lock';
  version: 1;
  processId: number;
  command: string;
  acquiredAt: string;
  token: string;
}

export interface LayoutCatalogLockOptions {
  command: string;
  /** Override only for deterministic tests. */
  processId?: number;
  /** Override only for deterministic stale-owner tests. */
  isProcessAlive?: (processId: number) => boolean;
  /** Override only for deterministic tests. */
  now?: () => Date;
  /** Override only for deterministic tests. */
  token?: string;
}

export interface LayoutCatalogLock {
  path: string;
  owner: LayoutCatalogLockOwner;
  release(): Promise<void>;
}

/**
 * Rejects concurrent catalog writers before a Penpot capture starts.  A
 * directory is used as the atomic primitive so the owner receipt can live
 * beside the lock and make contention actionable to another Codex session.
 */
export async function acquireLayoutCatalogLock(
  catalogRoot: string,
  options: LayoutCatalogLockOptions,
): Promise<LayoutCatalogLock> {
  await mkdir(catalogRoot, { recursive: true });
  const path = resolveLayoutCatalogPath(catalogRoot, lockDirectoryName);
  try {
    await mkdir(path);
  } catch (error) {
    if (errorCode(error) !== 'EEXIST') throw error;
    const owner = await readOwner(path);
    if (owner) {
      if (!(options.isProcessAlive ?? processIsAlive)(owner.processId)) {
        await reclaimStaleLock(path, owner);
        return acquireLayoutCatalogLock(catalogRoot, options);
      }
      throw new Error(
        `Layout catalog lock is already held at ${path} by pid ${owner.processId} (${owner.command}) since ${owner.acquiredAt}. Wait for that capture to finish or verify a stale owner before removing the lock.`,
        { cause: error },
      );
    }
    throw new Error(
      `Layout catalog lock is already held at ${path}, but its owner receipt is not readable yet. Do not remove it while another capture may be starting.`,
      { cause: error },
    );
  }

  const owner: LayoutCatalogLockOwner = {
    schema: 'dev.zircon.zui.layout-catalog-lock',
    version: 1,
    processId: options.processId ?? process.pid,
    command: options.command,
    acquiredAt: (options.now ?? (() => new Date()))().toISOString(),
    token: options.token ?? randomUUID(),
  };
  try {
    await writeFile(
      join(path, ownerFileName),
      `${JSON.stringify(owner, null, 2)}\n`,
      { encoding: 'utf8', flag: 'wx' },
    );
  } catch (error) {
    await rm(path, { recursive: true, force: true });
    throw error;
  }

  let released = false;
  return {
    path,
    owner,
    async release(): Promise<void> {
      if (released) return;
      const currentOwner = await readOwner(path);
      if (!currentOwner || currentOwner.token !== owner.token) {
        throw new Error(
          `Refusing to remove layout catalog lock at ${path}: its owner receipt no longer belongs to this capture.`,
        );
      }
      await rm(path, { recursive: true, force: false });
      released = true;
    },
  };
}

export async function withLayoutCatalogLock<T>(
  catalogRoot: string,
  options: LayoutCatalogLockOptions,
  operation: () => Promise<T>,
): Promise<T> {
  const lock = await acquireLayoutCatalogLock(catalogRoot, options);
  try {
    return await operation();
  } finally {
    await lock.release();
  }
}

async function readOwner(
  path: string,
): Promise<LayoutCatalogLockOwner | undefined> {
  try {
    const parsed: unknown = JSON.parse(
      await readFile(join(path, ownerFileName), 'utf8'),
    );
    if (!isOwner(parsed)) return undefined;
    return parsed;
  } catch {
    return undefined;
  }
}

function isOwner(value: unknown): value is LayoutCatalogLockOwner {
  if (!value || typeof value !== 'object') return false;
  const owner = value as Partial<LayoutCatalogLockOwner>;
  return (
    owner.schema === 'dev.zircon.zui.layout-catalog-lock' &&
    owner.version === 1 &&
    typeof owner.processId === 'number' &&
    Number.isInteger(owner.processId) &&
    owner.processId > 0 &&
    typeof owner.command === 'string' &&
    typeof owner.acquiredAt === 'string' &&
    typeof owner.token === 'string' &&
    owner.token.length > 0
  );
}

async function reclaimStaleLock(
  path: string,
  expectedOwner: LayoutCatalogLockOwner,
): Promise<void> {
  const currentOwner = await readOwner(path);
  if (!currentOwner || currentOwner.token !== expectedOwner.token) {
    throw new Error(
      `Layout catalog lock at ${path} changed while its stale owner was being verified; refusing to remove it.`,
    );
  }
  await rm(path, { recursive: true, force: false });
}

function processIsAlive(processId: number): boolean {
  try {
    process.kill(processId, 0);
    return true;
  } catch (error) {
    return errorCode(error) === 'EPERM';
  }
}

function errorCode(error: unknown): string | undefined {
  if (!error || typeof error !== 'object' || !('code' in error))
    return undefined;
  const code = (error as { code?: unknown }).code;
  return typeof code === 'string' ? code : undefined;
}

import { extname, resolve, sep } from 'node:path';

/** Catalog evidence stays in docs/_data/layout; ZUI inputs live in docs/ui/zui. */
export function layoutSourceRoot(catalogRoot: string): string {
  const root = resolve(catalogRoot);
  const marker = `${sep}docs${sep}_data${sep}layout`;
  const index = root.indexOf(marker);
  if (index < 0 || (root.length !== index + marker.length && root[index + marker.length] !== sep))
    return root;
  return root.slice(0, index) + `${sep}docs${sep}ui${sep}zui` + root.slice(index + marker.length);
}

export function resolveLayoutCatalogPath(...paths: string[]): string {
  const candidate = resolve(...paths);
  return extname(candidate) === '.zui' ? layoutSourceRoot(candidate) : candidate;
}

import { existsSync, lstatSync, realpathSync, statSync } from 'node:fs';
import { dirname, isAbsolute, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';

export interface LayoutCurrentContext {
  readonly role: 'current';
  readonly repoRoot: string;
  readonly outputRoot: string;
}

const currentContexts = new WeakMap<LayoutCurrentContext, { repoRoot: string; outputRoot: string }>();
const comparable = (path: string): string =>
  process.platform === 'win32' ? path.toLowerCase() : path;
const physical = (path: string): string => realpathSync.native(path);
const inside = (root: string, path: string): boolean => {
  const suffix = relative(root, path);
  return !isAbsolute(suffix) && suffix !== '..' && !/^\.\.[/\\]/.test(suffix);
};

/** Current review roots are independent of the retired catalog directory. */
export function createLayoutCurrentContext(
  options: { repoRoot?: string; outputRoot?: string } = {},
): LayoutCurrentContext {
  const repoRoot = resolve(
    options.repoRoot ?? process.env['ZUI_LAYOUT_REPO_ROOT'] ??
      resolve(dirname(fileURLToPath(import.meta.url)), '../../../../..'),
  );
  for (const marker of ['Cargo.toml', 'zircon_runtime/src/lib.rs']) {
    const path = resolve(repoRoot, marker);
    if (!existsSync(path) || !statSync(path).isFile() ||
        !inside(physical(repoRoot), physical(path)))
      throw new Error('Current layout repository is missing its source markers');
  }
  const output = options.outputRoot ?? process.env['ZUI_LAYOUT_OUTPUT_ROOT'];
  if (!output?.trim()) throw new Error('Current layout review requires explicit ZUI_LAYOUT_OUTPUT_ROOT');
  const outputRoot = requireApprovedPenpotArtifactDirectory(output, 'Layout review output');
  const repo = physical(repoRoot);
  const out = physical(outputRoot);
  if (inside(repo, out) || inside(out, repo))
    throw new Error('Layout review output must not overlap current source');
  const archive = process.env['ZIRCON_DOCS_HISTORY_ARCHIVE_ROOT'];
  if (archive?.trim()) {
    const history = existsSync(archive) ? physical(archive) : resolve(archive);
    if (inside(history, out) || inside(out, history))
      throw new Error('Layout review output must not overlap historical archive');
  }
  const context: LayoutCurrentContext = Object.freeze({ role: 'current', repoRoot, outputRoot });
  currentContexts.set(context, { repoRoot: comparable(repo), outputRoot: comparable(out) });
  return context;
}

export function assertLayoutCurrentRoots(
  context: LayoutCurrentContext,
  repoRoot: string,
  outputRoot: string,
): void {
  const roots = currentContexts.get(context);
  if (!roots || context.role !== 'current')
    throw new Error('Historical or package roots cannot be used for current review');
  if (comparable(physical(repoRoot)) !== roots.repoRoot ||
      comparable(physical(outputRoot)) !== roots.outputRoot)
    throw new Error('Current layout root identity differs');
  requireApprovedPenpotArtifactDirectory(context.outputRoot, 'Layout review output');
}

/** Resolve without extension-based rewriting, checking existing ancestors physically. */
function rolePath(root: string, path: string): string {
  if (!path.trim()) throw new Error('Missing layout role path');
  const candidate = resolve(root, path);
  if (!inside(root, candidate)) throw new Error('Layout path escapes role root');
  let ancestor = candidate;
  while (true) {
    try {
      lstatSync(ancestor);
      break;
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      const parent = dirname(ancestor);
      if (parent === ancestor) throw new Error('Layout path has no existing parent');
      ancestor = parent;
    }
  }
  if (!inside(physical(root), physical(ancestor)))
    throw new Error('Layout path physically escapes role root');
  return candidate;
}

export function layoutCurrentSourcePath(context: LayoutCurrentContext, path: string): string {
  assertLayoutCurrentRoots(context, context.repoRoot, context.outputRoot);
  const candidate = rolePath(context.repoRoot, path);
  const suffix = relative(context.repoRoot, candidate).replaceAll('\\', '/');
  const physicalSuffix = existsSync(candidate)
    ? relative(physical(context.repoRoot), physical(candidate)).replaceAll('\\', '/') : suffix;
  if ([suffix, physicalSuffix].some((value) => /^docs\/_data(?:\/|$)/i.test(value)))
    throw new Error('Historical catalog data is not current source');
  return candidate;
}

export function layoutCurrentOutputPath(context: LayoutCurrentContext, path: string): string {
  assertLayoutCurrentRoots(context, context.repoRoot, context.outputRoot);
  return rolePath(context.outputRoot, path);
}

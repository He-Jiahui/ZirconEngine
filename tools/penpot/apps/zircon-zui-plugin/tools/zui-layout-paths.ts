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

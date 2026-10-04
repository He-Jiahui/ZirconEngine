import { readFile, realpath } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import {
  resolve,
  relative,
  isAbsolute,
  sep,
  extname,
  basename,
} from 'node:path';
import type { ZuiDocument } from '../src/bridge/zui-document';

export async function embedLayoutImages(
  document: ZuiDocument,
  sourcePath: string,
  repoRoot: string,
): Promise<boolean> {
  if (!sourcePath.includes('/assets/')) return false;
  const root = resolve(repoRoot, sourcePath.split('/assets/')[0], 'assets');
  let embedded = false;
  for (const node of Object.values(document.nodes ?? {})) {
    const uri =
      node.props?.['background_image'] ??
      (node.component.toLowerCase() === 'image'
        ? (node.props?.['source'] ??
          node.props?.['image'] ??
          node.props?.['value'])
        : undefined);
    if (
      typeof uri !== 'string' ||
      (uri.includes('://') && !uri.startsWith('res://'))
    )
      continue;
    const path = resolve(root, uri.startsWith('res://') ? uri.slice(6) : uri);
    const inside = (base: string, target: string) => {
      const suffix = relative(base, target);
      return (
        !isAbsolute(suffix) && suffix !== '..' && !suffix.startsWith(`..${sep}`)
      );
    };
    if (!inside(root, path))
      throw new Error(`Image escapes asset root: ${uri}`);
    if (!existsSync(path)) continue;
    if (!inside(await realpath(root), await realpath(path)))
      throw new Error(`Image symlink escapes asset root: ${uri}`);
    if (extname(path).toLowerCase() === '.svg') {
      node['penpot_image_svg'] = await readFile(path, 'utf8');
      embedded = true;
      continue;
    }
    const mime = (
      {
        '.png': 'image/png',
        '.jpg': 'image/jpeg',
        '.jpeg': 'image/jpeg',
        '.webp': 'image/webp',
      } as Record<string, string>
    )[extname(path).toLowerCase()];
    if (!mime) continue;
    node['penpot_image'] = {
      uri,
      name: basename(path),
      mime,
      base64: (await readFile(path)).toString('base64'),
    };
    embedded = true;
  }
  return embedded;
}

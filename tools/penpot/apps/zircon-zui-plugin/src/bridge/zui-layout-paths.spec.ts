import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { containedPath } from '../../tools/zui-layout-evidence';
import { resolveLayoutCatalogPath } from '../../tools/zui-layout-paths';

describe('migrated layout catalog inputs', () => {
  const root = resolve('fixture/docs/_data/layout');
  it('reads mirrored sources and prepared hosts from docs/ui/zui', () => {
    expect(containedPath(root, 'editor/window/evidence/penpot-input.zui'))
      .toBe(resolve('fixture/docs/ui/zui/editor/window/evidence/penpot-input.zui'));
  });
  it('keeps evidence and resource manifests in the data directory', () => {
    expect(resolveLayoutCatalogPath(root, 'editor/window/evidence/penpot.png'))
      .toBe(resolve(root, 'editor/window/evidence/penpot.png'));
    expect(containedPath(root, 'resources/source-manifest.json'))
      .toBe(resolve(root, 'resources/source-manifest.json'));
  });
  it('keeps isolated catalog fixtures and engine source paths unchanged', () => {
    expect(containedPath(resolve('temporary-catalog'), 'input.zui'))
      .toBe(resolve('temporary-catalog/input.zui'));
    expect(containedPath(resolve('fixture'), 'zircon_editor/assets/window.zui'))
      .toBe(resolve('fixture/zircon_editor/assets/window.zui'));
  });
  it('rejects catalog path traversal before selecting the ZUI source root', () => {
    expect(() => containedPath(root, '../foreign.zui')).toThrow('Path escapes root');
  });
});

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import { editorHostColors } from '../../tools/zui-layout-host-theme';
import { cloneZuiDocument, parseZuiDocument } from './zui-document';

describe('explicit review host theme dependencies', () => {
  const root = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
  const assetPath =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui';
  const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const read = (path: string) =>
    parseZuiDocument(readFileSync(`${root}/${path}`, 'utf8')).document;
  const dependencies = new LayoutDependencies();
  beforeAll(async () => dependencies.load(root, [assetPath, themePath]));

  it('resolves standard theme names and editor aliases from the same source palette', () => {
    const source = read(assetPath);
    const original = cloneZuiDocument(source);
    dependencies.embed(source, assetPath, themePath);
    const theme = read(themePath);
    expect(source['penpot_dependency_sources']).toEqual([themePath]);
    expect(source.tokens?.['theme.palette.text.primary']).toBe(
      theme['palette']['text_primary'],
    );
    expect(source.tokens?.['editor.text.primary']).toBe(
      theme['palette']['text_primary'],
    );
    expect(source.tokens?.['theme.palette.surface.2']).toBe(
      theme['palette']['surface'][2],
    );
    expect(source['penpot_host_theme_source']).toBe(themePath);
    expect(source['penpot_original_imports']).toEqual(original.imports);
    expect(source.nodes).toEqual(original.nodes);
  });

  it('refuses a missing host theme instead of substituting a palette', () => {
    expect(() =>
      dependencies.embed(read(assetPath), assetPath, 'missing-theme.zui'),
    ).toThrow('Missing review host theme');
  });

  it('does not confuse widget color variants with generic retained-host color roles', () => {
    const colors = editorHostColors(read(themePath));
    expect(colors['material.primary']).toBe('#60aeff');
    expect(colors['material.surface']).toBe('#2f2f2f');
    expect(colors['secondary']).toBeUndefined();
    expect(colors['material.accent_soft']).toBeUndefined();
  });
});

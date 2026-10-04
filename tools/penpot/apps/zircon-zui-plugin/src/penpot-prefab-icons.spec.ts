import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { Shape } from '@penpot/plugin-types';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import { parseZuiDocument } from './bridge/zui-document';
import { previewIconTint, recolorIconShape } from './penpot-prefab-icons';

describe('native IconButton design projection', () => {
  const root = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url)));
  const assetPath =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui';
  const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const dependencies = new LayoutDependencies();

  beforeAll(async () => dependencies.load(root, [assetPath, themePath]));

  it.each([
    [{}, '#b3b3b3'],
    [{ hovered: true }, '#e8e8e8'],
    [{ pressed: true }, '#60aeff'],
    [{ focused: true }, '#b3b3b3'],
    [{ selected: true }, '#60aeff'],
    [{ disabled: true, selected: true }, '#737373'],
  ])('tints the authored SVG like the retained host for %j', (state, color) => {
    const document = parseZuiDocument(
      readFileSync(`${root}/${assetPath}`, 'utf8'),
    ).document;
    dependencies.embed(document, assetPath, themePath);
    const node = document.nodes!.root;
    node.state = state;
    expect(previewIconTint(document, node)).toBe(color);
  });

  it('recolors both authored fills and strokes without painting transparent paths', () => {
    const shape = {
      fills: [{ fillColor: '#26d8d1' }, { fillColor: 'transparent' }],
      strokes: [{ strokeColor: '#bafff8' }],
      children: [{ fills: [{ fillColor: '#ffffff' }], strokes: [] }],
    } as unknown as Shape;
    recolorIconShape(shape, '#60aeff');
    expect(shape.fills?.map((fill) => fill.fillColor)).toEqual([
      '#60aeff',
      'transparent',
    ]);
    expect(shape.strokes.map((stroke) => stroke.strokeColor)).toEqual([
      '#60aeff',
    ]);
    expect(
      (shape as unknown as { children: Shape[] }).children[0].fills?.[0]
        .fillColor,
    ).toBe('#60aeff');
  });
});

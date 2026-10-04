import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import { cloneZuiDocument, parseZuiDocument } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';

describe('Editor retained Workbench button projection', () => {
  const root = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
  const assetPath =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui';
  const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const showcasePath =
    'zircon_editor/assets/ui/editor/components/showcase/showcase_input_section.zui';
  const dependencies = new LayoutDependencies();
  beforeAll(async () =>
    dependencies.load(root, [assetPath, themePath, showcasePath]),
  );
  const source = () => {
    const document = parseZuiDocument(
      readFileSync(`${root}/${assetPath}`, 'utf8'),
    ).document;
    dependencies.embed(document, assetPath, themePath);
    const node = Object.values(document.nodes!)[0];
    node.props = {
      ...node.props,
      text: 'Bake Atlas',
      button_variant: 'filled',
    };
    return document;
  };

  it.each([
    [{}, '#60aeff', '#151515', '#484848'],
    [{ hovered: true }, '#66b2ff', '#151515', '#484848'],
    [{ pressed: true, focused: true }, '#243f5a', '#e8e8e8', '#484848'],
    [{ focused: true }, '#60aeff', '#151515', '#66b2ff'],
    [{ selected: true }, '#66b2ff', '#151515', '#484848'],
    [{ disabled: true, pressed: true }, '#2b2b2b', '#737373', '#363636'],
  ])(
    'uses native palette precedence for state %j',
    (state, fill, text, border) => {
      const document = source();
      Object.values(document.nodes!)[0].state = state;
      const original = cloneZuiDocument(document);
      const projection = projectZuiDocument(document);
      expect(projection.rootNodes[0].paint.fillColor).toBe(fill);
      expect(projection.rootNodes[0].paint.strokeColor).toBe(border);
      expect(projection.rootNodes[0].text?.color).toBe(text);
      expect(
        reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
          .document,
      ).toEqual(original);
    },
  );

  it('keeps Runtime authored palettes when no Editor host is present', () => {
    const document = source();
    delete document['penpot_host_theme_source'];
    const shape = projectZuiDocument(document).rootNodes[0];
    expect(shape.paint.fillColor).toBe('#242424');
    expect(shape.text?.color).toBe('#e8e8e8');
  });

  it('rejects a primary surface edit that the native selector cannot consume', () => {
    const document = source();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.paint.fillColor = '#ff00ff';
    expect(() => reconcileZuiDocument(document, snapshot)).toThrow(
      /native.*button.*fillColor/i,
    );
  });

  it('writes a secondary resting surface through its authored token property', () => {
    const document = source();
    Object.values(document.nodes!)[0].props!['button_variant'] = 'secondary';
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.paint.fillColor = '#123456';
    const result = reconcileZuiDocument(document, snapshot);
    expect(
      projectZuiDocument(result.document).rootNodes[0].paint.fillColor,
    ).toBe('#123456');
  });

  it('projects actual showcase primary, danger and disabled controls with native contrast', () => {
    const document = parseZuiDocument(
      readFileSync(`${root}/${showcasePath}`, 'utf8'),
    ).document;
    dependencies.embed(document, showcasePath, themePath);
    const shapes = new Map(
      projectZuiDocument(document).shapes.map((shape) => [shape.nodeId, shape]),
    );
    expect(shapes.get('button_demo')?.text?.color).toBe('#151515');
    expect(shapes.get('button_danger_demo')?.paint.fillColor).toBe('#383838');
    expect(shapes.get('button_danger_demo')?.text?.color).toBe('#eb605c');
    expect(shapes.get('button_disabled_demo')?.paint.fillColor).toBe('#2b2b2b');
    expect(shapes.get('button_disabled_demo')?.text?.color).toBe('#737373');
    expect(shapes.get('button_outlined_demo')?.paint.strokeColor).toBe(
      '#66b2ff',
    );
    expect(shapes.get('button_text_demo')?.paint.fillColor).toBe('#454545');
    expect(
      reconcileZuiDocument(
        document,
        cloneProjectionSnapshot(projectZuiDocument(document)),
      ).changes,
    ).toEqual([]);
  });

  it('resolves command and tab paint from existing event routes', () => {
    const document = source();
    const node = Object.values(document.nodes!)[0];
    node.props!['button_variant'] = 'outlined';
    node.events = [
      {
        id: 'review-import',
        event: 'Click',
        route: 'workbench.module.assets.import.invoke',
      },
    ];
    let shape = projectZuiDocument(document).rootNodes[0];
    expect(shape.paint.fillColor).toBe('#60aeff');
    expect(shape.paint.strokeColor).toBe('#60aeff');
    expect(shape.text?.color).toBe('#151515');
    node.events = [
      {
        id: 'review-module',
        event: 'Click',
        route: 'workbench.module.material.select',
      },
    ];
    shape = projectZuiDocument(document).rootNodes[0];
    expect(shape.paint.fillOpacity).toBe(0);
    expect(shape.paint.strokeWidth).toBe(0);
    node.state = { selected: true };
    shape = projectZuiDocument(document).rootNodes[0];
    expect(shape.paint.fillColor).toBe('#454545');
    expect(shape.text?.color).toBe('#e8e8e8');
  });

  it('keeps pointer focus quiet and gives drag precedence over a pressed focus state', () => {
    const document = source();
    const node = Object.values(document.nodes!)[0];
    node.state = { focused: true, focus_visible: false };
    expect(projectZuiDocument(document).rootNodes[0].paint.strokeColor).toBe(
      '#484848',
    );
    node.state = { focused: true, pressed: true, dragging: true };
    const shape = projectZuiDocument(document).rootNodes[0];
    expect(shape.paint.fillColor).toBe('#66b2ff');
    expect(shape.paint.strokeColor).toBe('#484848');
  });
});

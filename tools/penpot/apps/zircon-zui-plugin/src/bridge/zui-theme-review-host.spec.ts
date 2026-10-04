import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { prepareThemeReviewHost } from '../../tools/zui-layout-theme-hosts';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import { defaultReviewCases } from '../../tools/zui-layout-review-contract';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
} from './penpot-projection';
import {
  normalizeZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
} from './zui-document';
import { reviewDocumentForCase } from './zui-review-case';
import { reconcileReviewSource, reviewHost } from './zui-review-host';

const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
const shell = 'examples/woc/assets/ui/shell/shell_theme.zui';
const hud = 'examples/woc/assets/ui/hud/hud_theme.zui';
const read = async (path: string) =>
  parseZuiDocument(await readFile(`${repo}/${path}`, 'utf8')).document;

describe('theme product consumer host', () => {
  it('mounts Editor token consumers from current primitives without replacing their properties', async () => {
    const editorTheme =
      'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
    const theme = await read(editorTheme);
    const dependencies = new LayoutDependencies();
    await dependencies.load(repo, [editorTheme]);
    dependencies.embed(theme, editorTheme);
    const before = normalizeZuiDocument(theme);
    const result = (await prepareThemeReviewHost(repo, editorTheme, theme))!;
    const host = parseZuiDocument(result.source).document;
    expect(host.imports?.['styles']).toEqual([theme.asset.id]);
    expect(result.consumers).toHaveLength(5);
    for (const [index, consumer] of result.consumers.entries()) {
      const product = (await read(consumer.sourcePath)).nodes![consumer.nodeId];
      const node = host.nodes![`review_theme_component_${index}`];
      expect({ ...node, layout: undefined, control_id: undefined }).toEqual({
        ...product,
        layout: undefined,
        control_id: undefined,
      });
    }
    const original = { ...theme, penpot_review_host: result.projection };
    for (const reviewCase of defaultReviewCases(
      editorTheme,
      theme.asset.kind,
    )) {
      const projection = projectZuiDocument(
        reviewDocumentForCase(result.projection, reviewCase),
      );
      expect(projection.shapes).toHaveLength(12);
      expect(
        normalizeZuiDocument(
          reconcileReviewSource(original, cloneProjectionSnapshot(projection))
            .document,
        ),
      ).toEqual(normalizeZuiDocument(original));
    }
    expect(normalizeZuiDocument(theme)).toEqual(before);
  });

  it.each([
    'editor_base',
    'editor_material',
    'editor_unreal_dark',
    'editor_workbench_spatial',
    'editor_workbench_strict',
  ])(
    'mounts actual selector consumers for %s with a 32px narrow-screen margin',
    async (name) => {
      const sourcePath = `zircon_editor/assets/ui/theme/${name}.zui`;
      const theme = await read(sourcePath);
      const dependencies = new LayoutDependencies();
      await dependencies.load(repo, [
        sourcePath,
        'zircon_editor/assets/ui/editor/theme/editor_tokens.zui',
        'zircon_editor/assets/ui/theme/editor_material.zui',
        'zircon_editor/assets/ui/theme/editor_workbench_strict.zui',
      ]);
      dependencies.embed(
        theme,
        sourcePath,
        'zircon_editor/assets/ui/editor/theme/editor_tokens.zui',
      );
      const result = (await prepareThemeReviewHost(repo, sourcePath, theme))!;
      const host = parseZuiDocument(result.source).document;
      for (const [index, consumer] of result.consumers.entries()) {
        const product = (await read(consumer.sourcePath)).nodes![
          consumer.nodeId
        ];
        const node = host.nodes![`review_theme_component_${index}`];
        expect({ ...node, layout: undefined, control_id: undefined }).toEqual({
          ...product,
          layout: undefined,
          control_id: undefined,
        });
        const layout = node.layout as {
          position: { x: number };
          width: { preferred: number };
        };
        expect(layout.position.x + layout.width.preferred).toBeLessThanOrEqual(
          608,
        );
      }
      for (const reviewCase of defaultReviewCases(sourcePath, 'style')) {
        const projection = projectZuiDocument(
          reviewDocumentForCase(result.projection, reviewCase),
        );
        expect(projection.shapes.length).toBeGreaterThan(10);
        const button = projection.shapes.find(
          (shape) => shape.nodeId === 'review_theme_component_0',
        )!;
        expect(button.text?.fontSize).toBe(14);
        expect(button.paint.borderRadius).toBe(4);
        const original = { ...theme, penpot_review_host: result.projection };
        expect(
          normalizeZuiDocument(
            reconcileReviewSource(original, cloneProjectionSnapshot(projection))
              .document,
          ),
        ).toEqual(normalizeZuiDocument(original));
      }
    },
    30000,
  );
  let source: ZuiDocument;
  beforeAll(async () => {
    source = await read(shell);
    const dependencies = new LayoutDependencies();
    await dependencies.load(repo, [shell, hud]);
    dependencies.embed(source, shell);
  });

  it('mounts actual leaf controls and preserves authored content, events and classes', async () => {
    const before = normalizeZuiDocument(source);
    const result = (await prepareThemeReviewHost(repo, shell, source))!;
    const native = parseZuiDocument(result.source).document;
    expect(native.imports?.['styles']).toEqual([source.asset.id]);
    expect(native.nodes?.['sample_0'].props?.['text']).toBe('Log In');
    for (const [index, consumer] of result.consumers.entries()) {
      const product = (await read(consumer.sourcePath)).nodes![consumer.nodeId];
      const node = native.nodes![`sample_${index}`];
      expect({ ...node, layout: undefined }).toEqual({
        ...product,
        layout: undefined,
      });
    }
    expect(normalizeZuiDocument(source)).toEqual(before);
    expect(
      parseZuiDocument(serializeZuiDocument(result.projection)).document.root
        ?.node,
    ).toBe('review_theme_host');
  });

  it('changes real semantic controls for each state and keeps the theme roundtrip exact', async () => {
    const host = (await prepareThemeReviewHost(repo, shell, source))!
      .projection;
    const original = { ...source, penpot_review_host: host };
    const before = normalizeZuiDocument(original);
    for (const reviewCase of defaultReviewCases(shell, 'style')) {
      const projected = projectZuiDocument(
        reviewDocumentForCase(host, reviewCase),
      );
      expect(projected.shapes.length).toBe(14);
      const snapshot = cloneProjectionSnapshot(projected);
      expect(
        normalizeZuiDocument(
          reconcileReviewSource(original, snapshot).document,
        ),
      ).toEqual(before);
      if (reviewCase.state === 'focused')
        expect(
          projected.shapes.find((node) => node.nodeId === 'sample_2')?.paint
            .strokeColor,
        ).toBe('#ffd100');
      if (reviewCase.state === 'disabled')
        expect(
          projected.shapes.find((node) => node.nodeId === 'sample_0')?.paint
            .fillColor,
        ).toBe('#242431');
    }
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(host));
    snapshot.shapes.find(
      (item) => item.nodeId === 'sample_0',
    )!.current.text!.characters = 'Replaced';
    expect(() => reconcileReviewSource(original, snapshot)).toThrow(
      'explicit source mapping',
    );
  });

  it('refuses unsupported and empty hosts instead of accepting swatch-only images', async () => {
    expect(
      await prepareThemeReviewHost(repo, 'unknown/theme.zui', source),
    ).toBeNull();
    expect(() => reviewHost({ ...source, penpot_review_host: {} })).toThrow(
      'valid nonempty',
    );
    expect(() =>
      reviewDocumentForCase(
        source,
        defaultReviewCases(shell, 'style').find(
          (item) => item.state === 'hover',
        )!,
      ),
    ).toThrow('component consumer host');
  });
});

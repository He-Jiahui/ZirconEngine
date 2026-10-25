import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { LayoutDependencies } from './zui-layout-dependencies';
import { trackedZuiPaths } from './zui-layout-catalog';
import { parseZuiDocument, type ZuiDocument, type ZuiTable } from '../src/bridge/zui-document';

const maskedAsset = vi.hoisted(() => ({ path: '' }));
vi.mock('node:fs', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:fs')>();
  return { ...actual, existsSync: (path: Parameters<typeof actual.existsSync>[0]) =>
    !(maskedAsset.path && String(path).replaceAll('\\', '/').endsWith(maskedAsset.path)) && actual.existsSync(path) };
});

describe('original PropertyEditorRow height variants and typed value slot', () => {
  const root = process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url));
  const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const property = 'zircon_editor/assets/ui/editor/workbench/inputs/property_editor_row.zui';
  const dependencies = new LayoutDependencies();
  const imports = [
    'res://ui/editor/workbench/inputs/property_editor_row.zui#WorkbenchPropertyEditorRow',
    'res://ui/editor/components/workbench/primitives/inputs/field.zui#WorkbenchField',
    'res://ui/editor/components/workbench/primitives/data/caption.zui#WorkbenchCaption',
    'res://ui/editor/components/workbench/primitives/data/label.zui#WorkbenchLabel',
    'res://ui/editor/components/workbench/primitives/inputs/button.zui#WorkbenchButton',
  ];
  beforeAll(async () => dependencies.load(root, [theme,
    ...imports.map(reference => 'zircon_editor/assets/' + reference.split('#')[0].slice('res://'.length)),
  ]), 120_000);
  const readonly = {
    inspector_property_editable: false, read_only: true, editable_text: false,
    input_focusable: false, input_clickable: false, focusable: false, clickable: false,
  };
  const caller = (params: ZuiTable = {}): ZuiDocument => ({
    asset: { id: 'unit-property-height-variants', kind: 'view', version: 2 },
    imports: { widgets: imports }, root: { node: 'row' },
    nodes: {
      row: { component: 'WorkbenchPropertyEditorRow', control_id: 'UnitPropertyRow', params,
        props: { text: '项目设置中的长中文属性名称' },
        children: [{ node: 'value', slot: { name: 'value' } }] },
      value: { component: 'WorkbenchField', control_id: 'UnitPropertyValue',
        props: { value: '0.125', ...readonly } },
    },
  });
  const embed = (document: ZuiDocument): void => {
    dependencies.embed(document, 'zircon_editor/assets/ui/editor/unit-property-height-variants.zui', theme);
  };

  it('materializes canonical defaults without changing the existing row or support heights', () => {
    const document = parseZuiDocument(readFileSync(root + '/' + property, 'utf8')).document;
    dependencies.embed(document, property, theme);
    expect(document.nodes!['root'].layout?.['height']).toEqual({
      min: '$editor.control.height.dense', preferred: '$editor.control.height.compact', stretch: 'Fixed',
    });
    for (const id of ['description_slot', 'source_slot'])
      expect(document.nodes![id].layout?.['height']).toEqual({ stretch: 'Fixed' });
    expect(document.components!['WorkbenchPropertyEditorRow']['slots']).toMatchObject({
      value: { required: true, multiple: false, accepts: [
        'WorkbenchCheckbox', 'WorkbenchDropdown', 'WorkbenchField', 'WorkbenchNumberField',
        'WorkbenchRangeSlider', 'WorkbenchSlider', 'WorkbenchToggle',
      ] },
    });
  });

  it('retains the actual theme token owner and byte fingerprint through component expansion', () => {
    const document = caller(); embed(document);
    const owners = document['penpot_token_source_owners'] as Record<string, unknown>;
    expect(owners['editor.surface.1']).toEqual(dependencies.sourceFingerprint(theme));
    expect(owners['editor.text.primary']).toEqual(dependencies.sourceFingerprint(theme));
    expect(document.nodes!['value'].props).toMatchObject({ value: '0.125', ...readonly });
  });

  it('keeps a same-value caller token override owned by that caller rather than the shared theme', () => {
    const document = caller();
    const themeDocument = parseZuiDocument(readFileSync(root + '/' + theme, 'utf8')).document;
    document.tokens = { 'editor.surface.1': (themeDocument['palette'] as { surface: string[] }).surface[1] };
    embed(document);
    expect((document['penpot_token_source_owners'] as Record<string, unknown>)['editor.surface.1'])
      .toEqual({ sourcePath: 'zircon_editor/assets/ui/editor/unit-property-height-variants.zui' });
  });

  it('retains inherited token ownership across repeated embedding of the same prepared host', () => {
    const document = caller(); embed(document); embed(document);
    expect((document['penpot_token_source_owners'] as Record<string, unknown>)['editor.surface.1'])
      .toEqual(dependencies.sourceFingerprint(theme));
    document.tokens!['editor.surface.1'] = '#010203';
    embed(document);
    expect((document['penpot_token_source_owners'] as Record<string, unknown>)['editor.surface.1'])
      .toEqual({ sourcePath: 'zircon_editor/assets/ui/editor/unit-property-height-variants.zui' });
  });

  it('does not trust caller-supplied token provenance on a fresh document', () => {
    const document = caller();
    const themeDocument = parseZuiDocument(readFileSync(root + '/' + theme, 'utf8')).document;
    document.tokens = { 'editor.surface.1': (themeDocument['palette'] as { surface: string[] }).surface[1] };
    document['penpot_token_source_owners'] = { 'editor.surface.1': dependencies.sourceFingerprint(theme) };
    embed(document);
    expect((document['penpot_token_source_owners'] as Record<string, unknown>)['editor.surface.1'])
      .toEqual({ sourcePath: 'zircon_editor/assets/ui/editor/unit-property-height-variants.zui' });
  });

  it('expands a content-driven variant with original supporting slots and typed readonly value props', () => {
    const supportHeight = { min: '$editor.control.height.dense',
      preferred: '$editor.control.height.large', max: '$editor.control.height.large', stretch: 'ContentDriven' };
    const document = caller({ row_height: { min: '$editor.control.height.dense', stretch: 'ContentDriven' },
      description_height: supportHeight, source_height: supportHeight });
    document.nodes!['row'].children!.push(
      { node: 'description', slot: { name: 'description' } },
      { node: 'source', slot: { name: 'source' } },
      { node: 'reset', slot: { name: 'reset' } },
    );
    document.nodes!['description'] = { component: 'WorkbenchCaption', props: { text: '支持换行的说明' } };
    document.nodes!['source'] = { component: 'WorkbenchLabel', props: { text: 'Project' } };
    document.nodes!['reset'] = { component: 'WorkbenchButton', props: { text: 'Reset' } };
    embed(document);
    expect(document.nodes!['row'].layout?.['height']).toEqual({
      min: '$editor.control.height.dense', stretch: 'ContentDriven',
    });
    expect(document.nodes!['row'].control_id).toBe('UnitPropertyRow');
    expect(document.nodes!['value'].props).toMatchObject({ value: '0.125', ...readonly });
    expect(document.nodes!['value'].control_id).toBe('UnitPropertyValue');
    for (const id of ['description', 'source', 'reset']) expect(document.nodes![id]).toBeDefined();
    for (const id of ['description', 'source'])
      expect(document.nodes![id].layout?.['height']).toEqual(supportHeight);
  });

  it('continues to reject an empty required value slot', () => {
    const document = caller(); document.nodes!['row'].children = [];
    expect(() => embed(document)).toThrow(/required|missing/i);
  });

  it('continues to reject two values in the single value slot', () => {
    const document = caller();
    document.nodes!['second'] = { component: 'WorkbenchField', props: { value: '2' } };
    document.nodes!['row'].children!.push({ node: 'second', slot: { name: 'value' } });
    expect(() => embed(document)).toThrow(/single|multiple|one/i);
  });
});

describe('actual missing semantic icons dependency closure', () => {
  const root = process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url));
  const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const details = 'zircon_editor/assets/ui/editor/unreal/components/details.zui';
  const assets = ['zircon_editor_shell/controls/add.svg', 'zircon_editor_shell/inspector/transform.svg',
    'zircon_editor_shell/inspector/mesh-renderer.svg'];
  const dependencies = new LayoutDependencies();
  beforeAll(async () => dependencies.load(root, await trackedZuiPaths()), 120_000);
  afterEach(() => { maskedAsset.path = ''; });

  it('embeds the real bounded native SVG dependencies without authoring icon props or changing protected Source', () => {
    const document = parseZuiDocument(readFileSync(root + '/' + details, 'utf8')).document;
    const before = structuredClone(document);
    dependencies.embed(document, details, theme);
    const svgs = document['penpot_native_icon_svgs'] as Record<string, string>;
    expect(Object.keys(svgs).sort()).toEqual([...assets].sort());
    for (const path of assets) {
      expect(svgs[path]).toBe(readFileSync(root + '/zircon_editor/assets/icons/' + path, 'utf8'));
      expect(document['penpot_dependency_sources']).toContain('zircon_editor/assets/icons/' + path);
    }
    for (const id of ['transform_title', 'component_title', 'name']) {
      expect(document.nodes![id].props).toEqual(before.nodes![id].props);
      expect(document.nodes![id].layout).toEqual(before.nodes![id].layout);
    }
    const add = Object.values(document.nodes!).find((node) => node.control_id === 'WorkbenchAddComponent')!;
    expect(add.props?.['icon']).toBeUndefined();
    expect(add['penpot_review_source_node_id']).toBe('root');
    expect(JSON.parse(String(add['penpot_review_instance_path']))).toEqual([{ sourcePath: details, sourceNodeId: 'add' }]);
  });

  it.each(['ionicons/terminal-outline.svg', 'ionicons/grid-outline.svg', 'ionicons/share-outline.svg'])(
    'retains the actual Header projection asset %s as a dependency', (assetPath) => {
      const document: ZuiDocument = { asset: { id: 'unit-header-dependencies', kind: 'view', version: 2 },
        root: { node: 'tab' }, nodes: { tab: { component: 'Label', props: { text: 'Unit tab' },
          penpot_review_native_tab_icon: { assetPath } } } };
      dependencies.embed(document, 'unit-header.zui', theme);
      expect((document['penpot_native_icon_svgs'] as Record<string, string>)[assetPath])
        .toBe(readFileSync(root + '/zircon_editor/assets/icons/' + assetPath, 'utf8'));
      expect(document.nodes!['tab'].props).toEqual({ text: 'Unit tab' });
    },
  );

  it('rejects unsupported projection paths without reading outside the bounded asset set', () => {
    const document: ZuiDocument = { asset: { id: 'unit-invalid-glyph', kind: 'view', version: 2 },
      root: { node: 'tab' }, nodes: { tab: { component: 'Label', penpot_review_native_tab_icon: { assetPath: '../../private.svg' } } } };
    expect(() => dependencies.embed(document, 'unit-header.zui', theme)).toThrow(/Missing actual native semantic SVG/);
  });

  it('rejects a requested actual SVG that was unavailable during dependency loading', async () => {
    maskedAsset.path = '/zircon_editor_shell/controls/add.svg';
    const unavailable = new LayoutDependencies();
    await unavailable.load(root, [theme]);
    const document: ZuiDocument = { asset: { id: 'unit-missing-glyph', kind: 'view', version: 2 },
      root: { node: 'add' }, nodes: { add: { component: 'Button', control_id: 'WorkbenchAddComponent' } } };
    expect(() => unavailable.embed(document, 'unit-add.zui', theme)).toThrow(/Missing actual native semantic SVG/);
  });
});

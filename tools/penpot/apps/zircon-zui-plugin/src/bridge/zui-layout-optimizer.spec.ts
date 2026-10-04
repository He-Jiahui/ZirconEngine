import { describe, expect, it } from 'vitest';
import {
  catalogEntryForSource,
  optimizeV2Document,
  prepareZuiLayoutAsset,
} from './zui-layout-optimizer';
import { projectZuiDocument } from './penpot-projection';

describe('zui layout optimizer', () => {
  it('fits an anchored overlay viewport around its centered panel', () => {
    const { document } = prepareZuiLayoutAsset(
      `[asset]
kind="view"
id="res://overlay"
version=2
[root]
node="root"
[nodes.root]
component="Overlay"
layout={clip=true,width={stretch="Stretch"},height={stretch="Stretch"}}
children=[{node="panel"}]
[nodes.panel]
component="VerticalBox"
layout={anchor={x=0.5,y=0.5},pivot={x=0.5,y=0.5},width={preferred=1100,stretch="Stretch"},height={preferred=720,stretch="Stretch"}}
`,
      'overlay.zui',
    );
    expect(document.nodes!['root'].layout?.['width']).toMatchObject({
      preferred: 1100,
    });
    expect(document.nodes!['root'].layout?.['height']).toMatchObject({
      preferred: 720,
    });
  });
  it('maps source paths to stable Penpot catalog entries', () => {
    expect(
      catalogEntryForSource('zircon_editor/assets/ui/editor/welcome.zui'),
    ).toEqual({
      category: 'editor-ui',
      name: 'welcome-ad1e835e',
    });
  });

  it('migrates an inline v1 layout into a valid v2 view', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "layout"\nid = "legacy.layout"\nversion = 1\ndisplay_name = "Legacy"\n\n[root]\nnode_id = "root"\nkind = "native"\ntype = "VerticalBox"\nchildren = []\n',
      'zircon_editor/src/tests/fixtures/ui_zui/editor/legacy.zui',
    );

    expect(prepared.document.asset.kind).toBe('view');
    expect(prepared.document.asset.version).toBe(2);
    expect(prepared.document.root?.['node']).toBe('root');
    expect(prepared.document.nodes?.['root']?.['component']).toBe(
      'VerticalBox',
    );
    expect(prepared.sourceFormat).toBe('legacy');
    expect(prepared.changes).toContain('migrate-legacy-document');
  });

  it('keeps a legacy style asset as a v2 style document', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "style"\nid = "legacy.theme"\nversion = 1\ndisplay_name = "Legacy theme"\n\n[tokens]\naccent = "#ABC"\n',
      'zircon_editor/src/tests/fixtures/ui_zui/theme/legacy.zui',
    );

    expect(prepared.document.asset.kind).toBe('style');
    expect(prepared.document.asset.version).toBe(2);
    expect(prepared.document.tokens?.['accent']).toBe('#aabbcc');
    expect(prepared.sourceFormat).toBe('legacy');
  });

  it('normalizes Penpot-editable geometry and paint boundaries', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "v"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "Button"\nprops = { opacity = 2.5, corner_radius = -4.0, border_width = -1.0, background_color = "#abc" }\nlayout = { width = { min = 40.0, preferred = 12.0, max = 20.0 }, height = { preferred = -3.0 } }\n',
      'zircon_plugins/sample/editor/button.zui',
    );

    const node = prepared.document.nodes?.['root'];
    expect(node?.props?.['opacity']).toBe(1);
    expect(node?.props?.['corner_radius']).toBe(0);
    expect(node?.props?.['border_width']).toBe(0);
    expect(node?.props?.['background_color']).toBe('#aabbcc');
    expect(node?.layout?.['width']).toEqual({
      min: 40,
      preferred: 40,
      max: 40,
    });
    expect(node?.layout?.['height']).toEqual({ preferred: 0 });
    expect(prepared.changes).toEqual(
      expect.arrayContaining(['normalize-paint', 'normalize-geometry']),
    );
  });

  it('embeds a stable Penpot prefab foundation without discarding authored classes', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "prefab-test"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "VerticalBox"\nclasses = ["authored-shell"]\nchildren = [{ node = "title" }, { node = "submit" }, { node = "search" }]\n\n[nodes.title]\ncomponent = "Label"\nprops = { text = "Project Settings", font_weight = 600 }\n\n[nodes.submit]\ncomponent = "WorkbenchButton"\nprops = { text = "Apply", button_variant = "filled", button_color = "accent" }\n\n[nodes.search]\ncomponent = "WorkbenchSearchInput"\nprops = { placeholder = "Search settings" }\n',
      'zircon_editor/assets/ui/editor/prefab_test.zui',
    );

    expect(prepared.document.asset['design_profile']).toBe(
      'zircon.penpot.prefabs.v1',
    );
    expect(prepared.document.nodes?.['root']?.classes).toEqual(
      expect.arrayContaining(['authored-shell', 'zr-prefab-root']),
    );
    expect(prepared.document.nodes?.['title']?.classes).toContain(
      'zr-prefab-title',
    );
    expect(prepared.document.nodes?.['submit']?.classes).toContain(
      'zr-prefab-button',
    );
    expect(prepared.document.nodes?.['search']?.classes).toContain(
      'zr-prefab-field',
    );
    expect(
      prepared.document.stylesheets?.some(
        (sheet) => sheet['id'] === 'zircon_penpot_prefabs_v1',
      ),
    ).toBe(true);
    expect(prepared.changes).toEqual(
      expect.arrayContaining([
        'assign-penpot-design-profile',
        'apply-penpot-prefab-classes',
        'embed-penpot-prefab-styles',
      ]),
    );
  });

  it('applies the prefab foundation idempotently', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "idempotent"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "Container"\n',
      'zircon_plugins/sample/editor/idempotent.zui',
    );
    const second = optimizeV2Document(prepared.document);

    expect(second.document).toEqual(prepared.document);
    expect(second.changes).not.toEqual(
      expect.arrayContaining([
        'assign-penpot-design-profile',
        'apply-penpot-prefab-classes',
        'embed-penpot-prefab-styles',
      ]),
    );
  });

  it('replaces stale prefab role classes while preserving authored classes', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "role-cutover"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "Container"\nclasses = ["authored-shell", "zr-prefab-layout"]\n',
      'zircon_editor/assets/ui/editor/role_cutover.zui',
    );

    expect(prepared.document.nodes?.['root']?.classes).toEqual([
      'authored-shell',
      'zr-prefab-root',
    ]);
  });

  it('styles component roots as reusable prefabs instead of full-page roots', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "component"\nid = "button-component"\nversion = 2\n\n[components.Button]\nroot = "root"\n\n[nodes.root]\ncomponent = "WorkbenchButton"\nprops = { text = "Run" }\n',
      'zircon_editor/assets/ui/editor/components/button.zui',
    );

    expect(prepared.document.nodes?.['root']?.classes).toContain(
      'zr-prefab-button',
    );
    expect(prepared.document.nodes?.['root']?.classes).not.toContain(
      'zr-prefab-root',
    );
  });

  it('separates bottom overlay rails from corner actions and side panels', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "overlay-actions"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "Overlay"\nlayout = { container = { kind = "Overlay" }, width = { preferred = 960.0 }, height = { preferred = 640.0 } }\nchildren = [{ node = "details" }, { node = "rail" }, { node = "back" }, { node = "confirm" }]\n\n[nodes.details]\ncomponent = "ScrollBox"\nlayout = { anchor = { x = 1.0, y = 0.0 }, pivot = { x = 1.0, y = 0.0 }, position = { x = -26.0, y = 110.0 }, width = { preferred = 360.0 }, height = { min = 360.0, preferred = 500.0, max = 560.0 } }\n\n[nodes.rail]\ncomponent = "HorizontalBox"\nlayout = { container = { kind = "HorizontalBox", gap = 14.0 }, anchor = { x = 0.5, y = 1.0 }, pivot = { x = 0.5, y = 1.0 }, position = { x = 0.0, y = -26.0 }, width = { preferred = 832.0 }, height = { preferred = 68.0 } }\n\n[nodes.back]\ncomponent = "Button"\nlayout = { anchor = { x = 0.0, y = 1.0 }, pivot = { x = 0.0, y = 1.0 }, position = { x = 26.0, y = -30.0 }, width = { preferred = 140.0 }, height = { preferred = 44.0 } }\n\n[nodes.confirm]\ncomponent = "Button"\nlayout = { anchor = { x = 1.0, y = 1.0 }, pivot = { x = 1.0, y = 1.0 }, position = { x = -26.0, y = -30.0 }, width = { preferred = 180.0 }, height = { preferred = 44.0 } }\n',
      'examples/woc/assets/ui/shell/overlay_actions.zui',
    );

    expect(prepared.document.nodes?.['rail']?.layout?.['position']).toEqual({
      x: 0,
      y: -86,
    });
    expect(prepared.document.nodes?.['details']?.layout?.['height']).toEqual({
      min: 360,
      preferred: 364,
      max: 364,
    });
    expect(prepared.changes).toEqual(
      expect.arrayContaining([
        'separate-overlay-bottom-actions',
        'constrain-overlay-side-panel',
      ]),
    );
  });

  it('derives a standalone component root size from its horizontal content', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "component"\nid = "toolbar-component"\nversion = 2\n\n[components.Toolbar]\nroot = "root"\n\n[nodes.root]\ncomponent = "HorizontalBox"\nlayout = { container = { kind = "HorizontalBox", gap = 12.0 }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }\nchildren = [{ node = "left" }, { node = "right" }]\n\n[nodes.left]\ncomponent = "Button"\nprops = { text = "Left" }\nlayout = { width = { preferred = 80.0 }, height = { preferred = 32.0 } }\n\n[nodes.right]\ncomponent = "Button"\nprops = { text = "Right" }\nlayout = { width = { preferred = 90.0 }, height = { preferred = 32.0 } }\n',
      'zircon_editor/assets/ui/editor/components/toolbar.zui',
    );

    expect(prepared.document.nodes?.['root']?.layout?.['width']).toEqual({
      stretch: 'Stretch',
      min: 182,
      preferred: 182,
    });
    expect(prepared.document.nodes?.['root']?.layout?.['height']).toEqual({
      stretch: 'Stretch',
      min: 32,
      preferred: 32,
    });
    expect(prepared.changes).toContain('fit-auto-layout-to-content');
  });

  it('keeps an authored component root size when it is already explicit', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "component"\nid = "fixed-component"\nversion = 2\n\n[components.Fixed]\nroot = "root"\n\n[nodes.root]\ncomponent = "VerticalBox"\nlayout = { width = { min = 240.0, preferred = 320.0, max = 480.0, stretch = "Fixed" }, height = { min = 120.0, preferred = 180.0, max = 240.0, stretch = "Fixed" } }\n',
      'zircon_editor/assets/ui/editor/components/fixed.zui',
    );

    expect(prepared.document.nodes?.['root']?.layout?.['width']).toEqual({
      min: 240,
      preferred: 320,
      max: 480,
      stretch: 'Fixed',
    });
    expect(prepared.document.nodes?.['root']?.layout?.['height']).toEqual({
      min: 120,
      preferred: 180,
      max: 240,
      stretch: 'Fixed',
    });
  });

  it('resolves a missing default control-height token to a compact preview size', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "default-height"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "VerticalBox"\nchildren = [{ node = "row" }]\n\n[nodes.row]\ncomponent = "HorizontalBox"\nlayout = { height = { min = "$editor.control.height.default", preferred = "$editor.control.height.default", max = "$editor.control.height.default", stretch = "Fixed" } }\n',
      'zircon_editor/assets/ui/editor/default_height.zui',
    );
    const row = projectZuiDocument(prepared.document).shapes.find(
      ({ nodeId }) => nodeId === 'row',
    );

    expect(row?.geometry.height).toBe(32);
  });

  it('keeps missing dense and compact control tokens on the shared 28/32 scale', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "control-heights"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "VerticalBox"\nchildren = [{ node = "dense" }, { node = "compact" }]\n\n[nodes.dense]\ncomponent = "HorizontalBox"\nlayout = { height = { preferred = "$editor.control.height.dense" } }\n\n[nodes.compact]\ncomponent = "HorizontalBox"\nlayout = { height = { preferred = "$editor.control.height.compact" } }\n',
      'zircon_editor/assets/ui/editor/control_heights.zui',
    );
    const projection = projectZuiDocument(prepared.document);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'dense')?.geometry
        .height,
    ).toBe(28);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'compact')?.geometry
        .height,
    ).toBe(32);
  });

  it('expands nested auto-layout containers from visible content', () => {
    const prepared = prepareZuiLayoutAsset(
      '[asset]\nkind = "view"\nid = "nested-fit"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "VerticalBox"\nlayout = { container = { kind = "VerticalBox", gap = 0.0 }, width = { preferred = 320.0 }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } }\nchildren = [{ node = "stack" }]\n\n[nodes.stack]\ncomponent = "VerticalBox"\nlayout = { container = { kind = "VerticalBox", gap = 8.0 }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }\nchildren = [{ node = "first" }, { node = "second" }, { node = "hidden" }]\n\n[nodes.first]\ncomponent = "Label"\nprops = { text = "First" }\nlayout = { height = { preferred = 20.0 } }\n\n[nodes.second]\ncomponent = "Label"\nprops = { text = "Second" }\nlayout = { height = { preferred = 20.0 } }\n\n[nodes.hidden]\ncomponent = "Label"\nprops = { text = "Hidden", visibility = "collapsed" }\nlayout = { height = { preferred = 200.0 } }\n',
      'zircon_editor/assets/ui/editor/nested_fit.zui',
    );

    expect(prepared.document.nodes?.['stack']?.layout?.['height']).toEqual({
      stretch: 'Stretch',
      min: 48,
      preferred: 48,
    });
    expect(prepared.document.nodes?.['root']?.layout?.['height']).toEqual({
      min: 48,
      preferred: 48,
      max: 48,
      stretch: 'Fixed',
    });
    expect(prepared.changes).toContain('fit-auto-layout-to-content');
  });
});

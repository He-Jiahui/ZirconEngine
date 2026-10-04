import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { cloneZuiDocument, parseZuiDocument, type ZuiDocument } from './bridge/zui-document';
import {
  DYNAMIC_REVIEW_STATE_KEY,
  prepareDynamicReviewState,
} from '../tools/zui-layout-dynamic-hosts';
import { reviewDocumentForCase } from './bridge/zui-review-case';

function componentDocument(
  root: string,
  nodes: ZuiDocument['nodes'],
): ZuiDocument {
  return {
    asset: {
      kind: 'component',
      version: 2,
      id: `res://${root}`,
      display_name: root,
    },
    components: { Dynamic: { root } },
    nodes,
  };
}

describe('dynamic layout review host projection', () => {
  it('selects one authored additional workspace and preserves every branch', () => {
    const source = componentDocument('additional_module_workspaces', {
      additional_module_workspaces: {
        component: 'Overlay',
        children: [
          { node: 'ability_workspace_body' },
          { node: 'tags_workspace_body' },
        ],
      },
      ability_workspace_body: {
        component: 'WorkbenchAbilityWorkspace',
        props: { visibility: 'collapsed', business_key: 'ability' },
      },
      tags_workspace_body: {
        component: 'WorkbenchTagsWorkspace',
        props: { visibility: 'collapsed', business_key: 'tags' },
      },
    });
    const before = cloneZuiDocument(source);
    const result = prepareDynamicReviewState(
      source,
      'zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui',
    );

    expect(result.changes).toEqual(['inject-deterministic-authored-host-state']);
    expect(source.nodes?.ability_workspace_body.props).toMatchObject({
      visibility: 'visible',
      business_key: 'ability',
    });
    expect(source.nodes?.tags_workspace_body.props?.visibility).toBe('collapsed');
    expect(source.nodes?.tags_workspace_body.component).toBe(
      before.nodes?.tags_workspace_body.component,
    );
    expect(source[DYNAMIC_REVIEW_STATE_KEY]).toMatchObject({
      selectedNodeIds: ['ability_workspace_body'],
    });
  });

  it('aligns qualified nested instances while selecting the authored module branch', () => {
    const source = componentDocument('module_workspace', {
      module_workspace: {
        component: 'Overlay',
        props: { visibility: 'visible' },
      },
      effect_workspace_body: {
        component: 'Overlay',
        props: { visibility: 'collapsed' },
      },
      material_workspace_body: {
        component: 'Overlay',
        props: { visibility: 'collapsed' },
      },
      additional_module_workspaces_body__ability_workspace_body: {
        component: 'Overlay',
        props: { visibility: 'visible' },
      },
    });
    const result = prepareDynamicReviewState(
      source,
      'zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui',
    );

    expect(result.state?.selectedNodeIds).toEqual(['effect_workspace_body']);
    expect(source.nodes?.effect_workspace_body.props?.visibility).toBe('visible');
    expect(source.nodes?.material_workspace_body.props?.visibility).toBe('collapsed');
    expect(
      source.nodes?.additional_module_workspaces_body__ability_workspace_body.props
        ?.visibility,
    ).toBe('visible');
  });

  it('makes a direct authored workspace root visible without replacing its content', () => {
    const source = componentDocument('blend_space_workspace', {
      blend_space_workspace: {
        component: 'Overlay',
        classes: ['workbench-module-body', 'workbench-extension-module-body'],
        props: { visibility: 'collapsed', business_key: 'blend-space' },
        children: [{ node: 'authored_canvas' }],
      },
      authored_canvas: {
        component: 'WorkbenchSampleGrid',
        props: { x_axis_label: 'Direction', y_axis_label: 'Speed' },
      },
    });
    const before = cloneZuiDocument(source);

    const result = prepareDynamicReviewState(
      source,
      'zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui',
    );

    expect(result.changes).toEqual([
      'inject-deterministic-authored-host-state',
    ]);
    expect(result.state).toMatchObject({
      selectedNodeIds: ['blend_space_workspace'],
      selectionPolicy: 'authored-visible-root',
    });
    expect(source.nodes?.blend_space_workspace.props).toMatchObject({
      visibility: 'visible',
      business_key: 'blend-space',
    });
    expect(source.nodes?.authored_canvas).toEqual(before.nodes?.authored_canvas);
  });

  it('does not mutate unrelated product documents or add synthetic nodes', () => {
    const source = componentDocument('plain', {
      plain: { component: 'Panel', children: [] },
    });
    const before = cloneZuiDocument(source);
    expect(
      prepareDynamicReviewState(source, 'zircon_editor/assets/ui/editor/plain.zui'),
    ).toEqual({ changes: [] });
    expect(source).toEqual(before);
  });

  it('opens the authored component drawer branch used by the retained host', () => {
    const source = componentDocument('component_drawer', {
      component_drawer: {
        component: 'VerticalGroup',
        children: [
          { node: 'drawer_tabs' },
          { node: 'component_drawer_content' },
        ],
      },
      drawer_tabs: {
        component: 'HorizontalGroup',
        children: [{ node: 'retired_showcase_tabs' }],
      },
      component_drawer_content: {
        component: 'VerticalGroup',
        children: [{ node: 'retired_showcase_content' }],
      },
      retired_showcase_tabs: {
        component: 'HorizontalGroup',
        props: { visibility: 'hidden' },
        children: [{ node: 'drawer_tab_components' }, { node: 'drawer_tab_console' }],
      },
      retired_showcase_content: {
        component: 'VerticalGroup',
        props: { visibility: 'hidden' },
        children: [{ node: 'component_body' }, { node: 'console_body' }],
      },
      drawer_tab_components: { component: 'WorkbenchTab', props: { selected: false } },
      drawer_tab_console: { component: 'WorkbenchTab', props: { selected: true } },
      component_body: { component: 'ScrollableBox', props: { visibility: 'hidden' } },
      console_body: { component: 'VerticalGroup', props: { visibility: 'visible' } },
    });

    const result = prepareDynamicReviewState(
      source,
      'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui',
    );

    expect(result.changes).toContain('inject-deterministic-workbench-host-state');
    expect(source.nodes?.retired_showcase_tabs.props?.visibility).toBe('visible');
    expect(source.nodes?.retired_showcase_content.props?.visibility).toBe('visible');
    expect(source.nodes?.drawer_tab_components.props?.selected).toBe(true);
    expect(source.nodes?.drawer_tab_console.props?.selected).toBe(false);
    expect(source.nodes?.console_body.props?.visibility).toBe('collapsed');
  });

  it('selects the authored scene default and retains the inactive module branch', () => {
    const source = componentDocument('main_band', {
      main_band: {
        component: 'Overlay',
        children: [{ node: 'scene_workspace' }, { node: 'module_workspace' }],
      },
      scene_workspace: { component: 'HorizontalGroup' },
      module_workspace: {
        component: 'WorkbenchModuleWorkspace',
        props: { visibility: 'visible' },
        children: [{ node: 'module_body' }],
      },
      module_body: { component: 'VerticalGroup', children: [{ node: 'module_label' }] },
      module_label: { component: 'Label', props: { text: 'Module' } },
    });

    const result = prepareDynamicReviewState(
      source,
      'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui',
    );

    expect(result.changes).toContain('select-retained-scene-workspace');
    expect(result.changes).toContain('retain-inactive-workspace-branch');
    expect(source.nodes?.main_band.children).toEqual([
      { node: 'scene_workspace' },
      { node: 'module_workspace' },
    ]);
    expect(source.nodes?.scene_workspace.props?.['visibility']).toBe('visible');
    expect(source.nodes?.module_workspace).toMatchObject({
      props: { visibility: 'collapsed' },
      children: [{ node: 'module_body' }],
    });
    expect(source.nodes?.module_body).toBeDefined();
    expect(source.nodes?.module_label?.props?.['text']).toBe('Module');
  });

  it('keeps the main-band drawer breakpoint contract aligned with the native host', () => {
    const sourcePath = fileURLToPath(
      new URL(
        '../../../../../zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui',
        import.meta.url,
      ),
    );
    const source = parseZuiDocument(readFileSync(sourcePath, 'utf8')).document;
    const rightDrawer = source.nodes?.right_drawer_shell;
    expect(rightDrawer?.layout?.['width']).toMatchObject({
      max: '$editor.density.right_drawer_width',
    });
    expect(rightDrawer?.props?.['responsive_min_tier']).toBe('regular');

    const narrow = reviewDocumentForCase(source, {
      id: 'main-band-narrow-contract',
      sourcePath:
        'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui',
      host: 'editor',
      viewport: { width: 640, height: 520 },
      dpi: 1,
      locale: 'en-US',
      state: 'default',
      data: {},
    });
    expect(narrow.nodes?.right_drawer_shell.props?.['visibility']).toBe(
      'collapsed',
    );

    const regular = reviewDocumentForCase(source, {
      id: 'main-band-regular-contract',
      sourcePath:
        'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui',
      host: 'editor',
      viewport: { width: 900, height: 620 },
      dpi: 1,
      locale: 'en-US',
      state: 'default',
      data: {},
    });
    expect(regular.nodes?.right_drawer_shell.layout?.['width']).toMatchObject({
      preferred: '$editor.density.compact_right_drawer_max_width',
      max: '$editor.density.compact_right_drawer_max_width',
    });

    const wide = reviewDocumentForCase(source, {
      id: 'main-band-wide-contract',
      sourcePath:
        'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui',
      host: 'editor',
      viewport: { width: 1280, height: 800 },
      dpi: 1,
      locale: 'en-US',
      state: 'default',
      data: {},
    });
    expect(wide.nodes?.right_drawer_shell.layout?.['width']).toMatchObject({
      preferred: '$editor.density.right_drawer_width',
      max: '$editor.density.right_drawer_width',
    });
  });

  it('gives the authored activity rail an explicit vertical scroll contract', () => {
    const sourcePath = fileURLToPath(
      new URL(
        '../../../../../zircon_editor/assets/ui/editor/components/workbench/shell/workbench_activity_rail.zui',
        import.meta.url,
      ),
    );
    const source = parseZuiDocument(readFileSync(sourcePath, 'utf8')).document;
    const rail = source.nodes?.activity_rail;
    expect(rail?.component).toBe('VerticalGroup');
    expect(rail?.layout).toMatchObject({
      clip: true,
      container: {
        kind: 'ScrollableBox',
        axis: 'Vertical',
        scrollbar_visibility: 'Auto',
      },
    });
  });

  it('keeps fixed toolbar groups intact inside the authored horizontal scroller', () => {
    const sourcePath = fileURLToPath(
      new URL(
        '../../../../../zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui',
        import.meta.url,
      ),
    );
    const source = parseZuiDocument(readFileSync(sourcePath, 'utf8')).document;
    const commandRow = source.nodes?.toolbar_command_row;
    expect(commandRow?.component).toBe('ScrollableBox');
    expect(commandRow?.layout).toMatchObject({
      clip: true,
      container: {
        kind: 'ScrollableBox',
        axis: 'Horizontal',
      },
    });

    for (const group of [
      'toolbar_file_group',
      'toolbar_module_commands',
      'toolbar_tool_group',
      'toolbar_run_group',
      'toolbar_layout_group',
    ]) {
      const slot = commandRow?.children?.find((child) => child.node === group)?.slot;
      expect(slot?.layout?.['linear_size']).toMatchObject({
        rule: 'StretchContent',
        shrink_value: 0,
      });
    }
  });
});

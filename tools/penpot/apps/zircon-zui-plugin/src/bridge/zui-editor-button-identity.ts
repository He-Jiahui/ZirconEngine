import type { ZuiNode } from './zui-document';

// Matches the retained host's template_buttons/identity and workbench_button/tab_like.
export function usesWorkbenchLanguage(node: ZuiNode): boolean {
  return (
    (node.control_id ?? '').startsWith('Workbench') ||
    ['component_variant', 'surface_variant', 'button_variant'].some((key) =>
      String(node.props?.[key] ?? '').includes('workbench'),
    )
  );
}

export function editorButtonKind(
  node: ZuiNode,
): 'primary' | 'secondary' | 'tertiary' | 'danger' {
  const identity = [
    node.control_id,
    ...[
      'text',
      'value_text',
      'button_variant',
      'surface_variant',
      'validation_level',
    ].map((key) => node.props?.[key]),
  ].map((value) => String(value ?? '').toLowerCase());
  const has = (needles: string[]) =>
    identity.some((value) => needles.some((needle) => value.includes(needle)));
  return has(['danger', 'delete', 'trash'])
    ? 'danger'
    : has(['primary', 'filled', 'accent'])
      ? 'primary'
      : has(['tertiary', 'text'])
        ? 'tertiary'
        : 'secondary';
}

export function buttonAction(node: ZuiNode): string {
  // workbench_window_projection prefers Click, Toggle and then Change routes.
  for (const event of ['Click', 'Toggle', 'Change']) {
    const binding = node.events?.find((value) => value['event'] === event);
    if (typeof binding?.['route'] === 'string') return binding['route'];
  }
  return '';
}

export function editorButtonTabKind(
  node: ZuiNode,
): 'chip' | 'utility' | 'asset' | 'module' | 'tab' | null {
  const id = node.control_id ?? '';
  const action = buttonAction(node);
  if (
    /^(AssetBrowserKind|AssetsActivityKind).*(Chip|Button)$/.test(id) ||
    /^(AssetBrowserViewMode|AssetsActivityViewMode)/.test(id) ||
    [
      'workbench.asset.kind_filter.set',
      'workbench.asset.view_mode.set',
    ].includes(action)
  )
    return 'chip';
  if (
    /^(AssetBrowser|AssetsActivity)(Preview|References|Metadata|Plugins)TabButton$/.test(
      id,
    ) ||
    id === 'AssetsActivityPreviewButton' ||
    action === 'workbench.asset.utility_tab.set'
  )
    return 'utility';
  if (/^(AssetBrowser|AssetsActivity).*TabButton$/.test(id)) return 'asset';
  if (
    /^WorkbenchModule(Scene|Effect|Ability|Tags|Perception|Material|Behavior|Render|Assets|Vfx|Hud)$/.test(
      id,
    ) ||
    /^workbench\.module\.(scene|effect|ability|tags|perception|material|behavior|render|assets|vfx|hud)(\.select)?$/.test(
      action,
    )
  )
    return 'module';
  return /^(PageTab|DockTab)/.test(id) ? 'tab' : null;
}

export function isEditorWorkbenchButton(node: ZuiNode): boolean {
  const id = node.control_id ?? '';
  return (
    ['Button', 'ToggleButton'].includes(node.component) &&
    !/^(WorkbenchDrawerTab|WorkbenchTool|WorkbenchToolbar|WorkbenchRail|WorkbenchStatus|WorkbenchMini)/.test(
      id,
    ) &&
    !id.includes('IconButton') &&
    (usesWorkbenchLanguage(node) ||
      editorButtonTabKind(node) !== null ||
      [
        'primary',
        'secondary',
        'tertiary',
        'filled',
        'outlined',
        'text',
        'ghost',
        'danger',
      ].includes(String(node.props?.['button_variant'] ?? '')))
  );
}

export function editorButtonCommand(node: ZuiNode): 'primary' | 'muted' | null {
  const id = node.control_id ?? '';
  const action = buttonAction(node);
  if (
    ['ImportModel', 'WorkbenchAssetsImportButton'].includes(id) ||
    [
      'workbench.asset.import_model',
      'workbench.module.assets.import.invoke',
      'workbench.module.assets.import_now',
    ].includes(action)
  )
    return 'primary';
  return ['WorkbenchModuleCompile', 'WorkbenchToolbarCompile'].includes(id) ||
    ['workbench.module.compile', 'workbench.toolbar.compile'].includes(action)
    ? 'muted'
    : null;
}

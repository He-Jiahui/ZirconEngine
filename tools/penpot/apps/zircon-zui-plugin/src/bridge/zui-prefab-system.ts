import type { ZuiDocument, ZuiNode, ZuiTable, ZuiValue } from './zui-document';

export const PENPOT_DESIGN_PROFILE = 'zircon.penpot.prefabs.v1';
export const PENPOT_PREFAB_STYLESHEET_ID = 'zircon_penpot_prefabs_v1';

export const PENPOT_PALETTE = {
  canvas: '#151719',
  surface: '#202326',
  surfaceRaised: '#292d30',
  surfaceInset: '#111315',
  surfaceSelected: '#183e38',
  border: '#3f4648',
  borderStrong: '#485052',
  text: '#f1f4f2',
  textMuted: '#a9b1ae',
  textSubtle: '#757f7b',
  accent: '#2fbc9a',
  accentStrong: '#16836d',
  info: '#5f8fd3',
  warning: '#d8a657',
  danger: '#d76067',
} as const;

export type ZuiPrefabRole =
  | 'root'
  | 'layout'
  | 'panel'
  | 'label'
  | 'title'
  | 'caption'
  | 'button'
  | 'icon-button'
  | 'field'
  | 'toggle'
  | 'tab'
  | 'row'
  | 'chip'
  | 'badge'
  | 'divider'
  | 'progress'
  | 'canvas'
  | 'space';

/** Runtime-owned painters that must remain visible as editable owner boards in Penpot. */
export const NATIVE_PAINTER_COMPONENTS = new Set([
  'AgentChat',
  'AgentPlan',
  'AgentApproval',
  'AIUsage',
  'ChatComposer',
  'DataGrid',
  'TreeView',
  'CommandPalette',
  'ConfirmDialog',
  'Dialog',
  'DragOverlay',
  'NotificationCenter',
  'ToolCalls',
  'WorkbenchToast',
]);

export interface PrefabVisualDefaults {
  fillColor: string | null;
  strokeColor: string | null;
  strokeWidth: number;
  borderRadius: number;
  textColor: string;
  fontSize: number;
  fontWeight: string;
  textAlign: 'left' | 'center' | 'right';
}

const LAYOUT_COMPONENTS = new Set([
  'flowbox',
  'gridbox',
  'gridgroup',
  'horizontalbox',
  'horizontalgroup',
  'overlay',
  'slot',
  'stack',
  'verticalbox',
  'verticalgroup',
  'view',
  'wrapbox',
]);

const PREFAB_CLASS: Record<ZuiPrefabRole, string> = {
  root: 'zr-prefab-root',
  layout: 'zr-prefab-layout',
  panel: 'zr-prefab-panel',
  label: 'zr-prefab-label',
  title: 'zr-prefab-title',
  caption: 'zr-prefab-caption',
  button: 'zr-prefab-button',
  'icon-button': 'zr-prefab-icon-button',
  field: 'zr-prefab-field',
  toggle: 'zr-prefab-toggle',
  tab: 'zr-prefab-tab',
  row: 'zr-prefab-row',
  chip: 'zr-prefab-chip',
  badge: 'zr-prefab-badge',
  divider: 'zr-prefab-divider',
  progress: 'zr-prefab-progress',
  canvas: 'zr-prefab-canvas',
  space: 'zr-prefab-space',
};

const PREFAB_TOKENS: Record<string, string | number> = {
  'zr.penpot.color.canvas': PENPOT_PALETTE.canvas,
  'zr.penpot.color.surface': PENPOT_PALETTE.surface,
  'zr.penpot.color.surface_raised': PENPOT_PALETTE.surfaceRaised,
  'zr.penpot.color.surface_inset': PENPOT_PALETTE.surfaceInset,
  'zr.penpot.color.surface_selected': PENPOT_PALETTE.surfaceSelected,
  'zr.penpot.color.border': PENPOT_PALETTE.border,
  'zr.penpot.color.border_strong': PENPOT_PALETTE.borderStrong,
  'zr.penpot.color.text': PENPOT_PALETTE.text,
  'zr.penpot.color.text_muted': PENPOT_PALETTE.textMuted,
  'zr.penpot.color.accent': PENPOT_PALETTE.accent,
  'zr.penpot.color.accent_strong': PENPOT_PALETTE.accentStrong,
  'zr.penpot.color.danger': PENPOT_PALETTE.danger,
  'zr.penpot.radius.control': 4,
  'zr.penpot.radius.panel': 0,
  'zr.penpot.space.xs': 4,
  'zr.penpot.space.sm': 8,
  'zr.penpot.space.md': 12,
  'zr.penpot.space.lg': 16,
};

export function prefabRoleForNode(
  node: ZuiNode,
  nodeId: string,
  isRoot: boolean,
): ZuiPrefabRole {
  if (isRoot) return 'root';
  const component = node.component.toLowerCase();
  if (NATIVE_PAINTER_COMPONENTS.has(node.component)) return 'panel';
  const authoredClasses = (node.classes ?? []).filter(
    (className) => !className.startsWith('zr-prefab-'),
  );
  const classIdentity = authoredClasses.join(' ').toLowerCase();
  const identity = `${component} ${nodeId.toLowerCase()} ${classIdentity}`;
  const surface =
    stringValue(node.props?.['surface_variant'])?.toLowerCase() ?? '';
  if (
    LAYOUT_COMPONENTS.has(component) &&
    !node.children?.length &&
    !/(?:panel|surface|canvas|preview|viewport|scrim|drawer)/.test(
      classIdentity,
    ) &&
    !surface
  )
    return 'layout';
  if (isTextComponent(component)) {
    if (/(?:chip|badge)/.test(classIdentity)) return 'chip';
    if (
      /(?:^|[_-])(title|heading)(?:$|[_-])/.test(nodeId) ||
      /(?:title|heading)/.test(classIdentity)
    )
      return 'title';
    if (
      node.props?.['text_tone'] === 'muted' ||
      /(?:muted|caption|subtitle|hint|meta)/.test(classIdentity)
    )
      return 'caption';
    return 'label';
  }
  if (LAYOUT_COMPONENTS.has(component) && (node.children?.length ?? 0) > 0) {
    return /(?:panel|surface|pane|drawer|section|tracker|unit-frame)/.test(
      classIdentity,
    ) || ['panel', 'inset'].includes(surface)
      ? 'panel'
      : 'layout';
  }
  if (/(?:^|\s)workbench-tab(?:\s|$)/.test(classIdentity)) return 'tab';
  if (
    component === 'togglebutton' &&
    node.props?.['selection_state'] === 'exclusive'
  )
    return 'button';

  if (
    component.includes('divider') ||
    surface === 'divider' ||
    /(?:divider|separator)/.test(classIdentity) ||
    /(?:^|[_-])(divider|separator)(?:$|[_-])/.test(nodeId)
  ) {
    return 'divider';
  }
  if (
    component.includes('progress') ||
    /(?:progress|meter|bar-rail|health-fill|resource-fill|cast-fill|xp-fill|timer-fill)/.test(
      classIdentity,
    )
  ) {
    return 'progress';
  }
  if (
    component.includes('iconbutton') ||
    component.includes('railbutton') ||
    /(?:icon-button|rail-button)/.test(classIdentity)
  ) {
    return 'icon-button';
  }
  if (
    component.includes('toggle') ||
    component.includes('checkbox') ||
    component.includes('radio') ||
    component.includes('switch') ||
    component.includes('segmented') ||
    /(?:toggle|checkbox|radio|switch|segmented)/.test(classIdentity)
  ) {
    return 'toggle';
  }
  if (
    component.includes('button') ||
    /(?:button|action|command)/.test(classIdentity)
  ) {
    return 'button';
  }
  if (
    component.includes('textfield') ||
    component.includes('numberfield') ||
    component.includes('rangefield') ||
    component.includes('textarea') ||
    component.includes('searchinput') ||
    component.includes('inputfield') ||
    component.includes('dropdown') ||
    component === 'select' ||
    component.includes('composer') ||
    component.endsWith('field') ||
    /(?:field|input|dropdown|select|textarea)/.test(classIdentity)
  ) {
    return 'field';
  }
  if (
    component.endsWith('row') ||
    component.includes('listrow') ||
    component.includes('treerow') ||
    component.includes('statusitem') ||
    component.includes('propertyrow') ||
    /(?:table-row|list-row|tree-item|property-row|status-item|nav-item|side-row|meta-strip)/.test(
      classIdentity,
    )
  ) {
    return 'row';
  }
  if (component.includes('tab') || /(?:^|\s)[^\s]*tab/.test(classIdentity)) {
    return 'tab';
  }
  if (component.includes('chip') || /(?:chip|pill)/.test(classIdentity)) {
    return 'chip';
  }
  if (component.includes('badge') || classIdentity.includes('badge')) {
    return 'badge';
  }
  if (
    component.includes('canvas') ||
    surface.includes('preview') ||
    surface.includes('placeholder') ||
    identity.includes('preview-visual') ||
    identity.includes('preview_visual') ||
    /(?:canvas|viewport|preview|thumbnail|portrait|minimap|board-host)/.test(
      classIdentity,
    )
  ) {
    return 'canvas';
  }
  if (
    component.includes('sectiontitle') ||
    (isTextComponent(component) &&
      (/(?:^|[_-])(title|heading)(?:$|[_-])/.test(nodeId) ||
        /(?:title|heading)/.test(classIdentity)))
  ) {
    return 'title';
  }
  if (
    component.includes('caption') ||
    (isTextComponent(component) &&
      (stringValue(node.props?.['text_tone']) === 'muted' ||
        /(?:muted|caption|subtitle|hint|meta)/.test(classIdentity)))
  ) {
    return 'caption';
  }
  if (isTextComponent(component)) return 'label';
  if (
    component.includes('panel') ||
    component.includes('container') ||
    component.includes('scroll') ||
    component.includes('dialog') ||
    component.includes('drawer') ||
    component.includes('popup') ||
    component.includes('menu') ||
    component.includes('inspector') ||
    surface === 'panel' ||
    surface === 'inset' ||
    /(?:panel|card|surface|pane|shell|window|drawer|scrim|tracker|unit-frame|toolbar-group|module-body|module-side|module-center)/.test(
      classIdentity,
    )
  ) {
    return 'panel';
  }
  if (LAYOUT_COMPONENTS.has(component)) return 'layout';
  if (
    component === 'space' ||
    component === 'icon' ||
    component === 'skeleton'
  ) {
    return 'space';
  }
  return node.children && node.children.length > 0 ? 'panel' : 'label';
}

export function prefabClassForRole(role: ZuiPrefabRole): string {
  return PREFAB_CLASS[role];
}

function prefabVisualDefaultsFallback(
  role: ZuiPrefabRole,
  node: ZuiNode,
): PrefabVisualDefaults {
  const selected =
    booleanValue(node.props?.['checked']) ||
    booleanValue(node.props?.['selected']) ||
    booleanValue(node.state?.['selected']) ||
    hasWord(node, 'selected');
  const danger =
    hasWord(node, 'danger') ||
    hasWord(node, 'error') ||
    hasWord(node, 'destructive');
  const primary =
    hasWord(node, 'primary') ||
    stringValue(node.props?.['button_color'])?.toLowerCase() === 'accent' ||
    ['contained', 'filled'].includes(
      stringValue(node.props?.['button_variant'])?.toLowerCase() ?? '',
    );
  const disabled =
    node.props?.['disabled'] === true || node.state?.['disabled'] === true;
  const variant = String(node.props?.['button_variant'] ?? '').toLowerCase();
  const outlined = variant === 'outlined' || variant === 'outline';
  const textButton = variant === 'text' || variant === 'ghost';

  switch (role) {
    case 'root':
      return visual(
        PENPOT_PALETTE.canvas,
        null,
        0,
        0,
        PENPOT_PALETTE.text,
        12,
        'regular',
        'left',
      );
    case 'layout':
    case 'space':
      return visual(
        null,
        null,
        0,
        0,
        PENPOT_PALETTE.text,
        12,
        'regular',
        'left',
      );
    case 'panel':
      return visual(
        stringValue(node.props?.['surface_variant']) === 'inset'
          ? PENPOT_PALETTE.surfaceInset
          : PENPOT_PALETTE.surface,
        null,
        0,
        0,
        PENPOT_PALETTE.text,
        12,
        'regular',
        'left',
      );
    case 'title':
      return visual(null, null, 0, 0, PENPOT_PALETTE.text, 14, '600', 'left');
    case 'caption':
      return visual(
        null,
        null,
        0,
        0,
        PENPOT_PALETTE.textMuted,
        10,
        'regular',
        'left',
      );
    case 'label':
      return visual(
        null,
        null,
        0,
        0,
        PENPOT_PALETTE.text,
        12,
        'regular',
        'left',
      );
    case 'button':
      return visual(
        disabled
          ? PENPOT_PALETTE.surfaceRaised
          : outlined || textButton
            ? null
            : danger
              ? PENPOT_PALETTE.danger
              : primary
                ? PENPOT_PALETTE.accentStrong
                : PENPOT_PALETTE.surfaceRaised,
        disabled
          ? PENPOT_PALETTE.border
          : outlined
            ? danger
              ? PENPOT_PALETTE.danger
              : PENPOT_PALETTE.accent
            : textButton || danger || primary
              ? null
              : PENPOT_PALETTE.borderStrong,
        !disabled && (textButton || (!outlined && (danger || primary))) ? 0 : 1,
        6,
        disabled
          ? PENPOT_PALETTE.textSubtle
          : outlined || textButton
            ? danger
              ? PENPOT_PALETTE.danger
              : PENPOT_PALETTE.accent
            : PENPOT_PALETTE.text,
        11,
        '600',
        'center',
      );
    case 'icon-button':
      return visual(
        selected
          ? PENPOT_PALETTE.surfaceSelected
          : PENPOT_PALETTE.surfaceRaised,
        selected ? PENPOT_PALETTE.accent : PENPOT_PALETTE.border,
        1,
        6,
        selected ? PENPOT_PALETTE.accent : PENPOT_PALETTE.textMuted,
        10,
        '600',
        'center',
      );
    case 'field':
      return visual(
        PENPOT_PALETTE.surfaceInset,
        PENPOT_PALETTE.borderStrong,
        1,
        6,
        PENPOT_PALETTE.textMuted,
        11,
        'regular',
        'left',
      );
    case 'toggle':
    case 'tab':
      return visual(
        selected
          ? PENPOT_PALETTE.surfaceSelected
          : PENPOT_PALETTE.surfaceRaised,
        selected ? PENPOT_PALETTE.accent : PENPOT_PALETTE.border,
        1,
        6,
        selected ? PENPOT_PALETTE.accent : PENPOT_PALETTE.text,
        11,
        '600',
        'center',
      );
    case 'row':
      return visual(
        selected ? PENPOT_PALETTE.surfaceSelected : null,
        null,
        0,
        4,
        selected ? PENPOT_PALETTE.accent : PENPOT_PALETTE.text,
        11,
        selected ? '600' : 'regular',
        'left',
      );
    case 'chip':
    case 'badge':
      return visual(
        selected
          ? PENPOT_PALETTE.surfaceSelected
          : PENPOT_PALETTE.surfaceRaised,
        selected ? PENPOT_PALETTE.accent : PENPOT_PALETTE.border,
        1,
        role === 'badge' ? 10 : 6,
        selected ? PENPOT_PALETTE.accent : PENPOT_PALETTE.textMuted,
        10,
        '600',
        'center',
      );
    case 'divider':
      return visual(
        PENPOT_PALETTE.border,
        null,
        0,
        0,
        PENPOT_PALETTE.textSubtle,
        10,
        'regular',
        'left',
      );
    case 'progress':
      return visual(
        PENPOT_PALETTE.surfaceRaised,
        PENPOT_PALETTE.border,
        1,
        6,
        PENPOT_PALETTE.accent,
        10,
        '600',
        'left',
      );
    case 'canvas':
      return visual(
        PENPOT_PALETTE.surfaceInset,
        PENPOT_PALETTE.border,
        1,
        6,
        PENPOT_PALETTE.textMuted,
        11,
        'regular',
        'center',
      );
  }
}

/** Resolve prefab defaults from the owning document's semantic tokens first. */
export function prefabVisualDefaults(
  role: ZuiPrefabRole,
  node: ZuiNode,
  document?: ZuiDocument,
): PrefabVisualDefaults {
  const defaults = prefabVisualDefaultsFallback(role, node);
  if (
    !document ||
    !Object.keys(document.tokens ?? {}).some((key) => key.startsWith('editor.'))
  )
    return defaults;
  // Only an authored token may override a prefab fallback. The public resolvers
  // intentionally accept semantic names such as `border` for legacy assets;
  // using that heuristic here would silently apply Editor tokens to WoC and
  // fixture documents which do not import the Editor theme.
  const color = (key: string, fallback: string | null): string | null =>
    explicitDesignColor(document, key) ?? fallback;
  const number = (key: string, fallback: number): number =>
    explicitDesignNumber(document, key) ?? fallback;
  const text =
    color('editor.text.primary', defaults.textColor) ?? defaults.textColor;
  const muted =
    color('editor.text.secondary', defaults.textColor) ?? defaults.textColor;
  const subtle = color('editor.text.disabled', muted) ?? muted;
  const accent =
    color('editor.accent', PENPOT_PALETTE.accent) ?? PENPOT_PALETTE.accent;
  const border =
    color('editor.border', defaults.strokeColor) ?? defaults.strokeColor;
  const strongBorder = color('editor.separator.strong', border) ?? border;
  const raised =
    color('editor.surface.3', defaults.fillColor) ?? defaults.fillColor;
  const inset =
    color('editor.surface.recessed', defaults.fillColor) ?? defaults.fillColor;
  const selected =
    color('editor.surface.selected', defaults.fillColor) ?? defaults.fillColor;
  const bodySize = number('editor.typography.body.size', 14);
  const captionSize = number('editor.typography.caption.size', 12);
  const titleSize = number('editor.typography.title.size', 20);
  const controlRadius = number('editor.control.radius.control', 4);
  const panelRadius = number('editor.control.radius.panel', 0);
  const danger =
    color('editor.semantic.error', PENPOT_PALETTE.danger) ??
    PENPOT_PALETTE.danger;
  switch (role) {
    case 'root':
      return {
        ...defaults,
        fillColor: color('editor.surface.0', defaults.fillColor),
        textColor: text,
        fontSize: bodySize,
      };
    case 'panel':
      return {
        ...defaults,
        fillColor:
          defaults.fillColor === null
            ? null
            : color('editor.surface.1', defaults.fillColor),
        strokeColor: border,
        borderRadius: panelRadius,
        textColor: text,
        fontSize: bodySize,
      };
    case 'title':
      return { ...defaults, textColor: text, fontSize: titleSize };
    case 'caption':
      return { ...defaults, textColor: muted, fontSize: captionSize };
    case 'label':
      return { ...defaults, textColor: text, fontSize: bodySize };
    case 'button':
      return {
        ...defaults,
        fillColor: defaults.fillColor
          ? defaults.fillColor === PENPOT_PALETTE.accentStrong ||
            defaults.fillColor === PENPOT_PALETTE.accent
            ? accent
            : defaults.fillColor === PENPOT_PALETTE.danger
              ? danger
              : raised
          : null,
        strokeColor: defaults.strokeColor
          ? defaults.strokeColor === PENPOT_PALETTE.accent
            ? accent
            : strongBorder
          : null,
        borderRadius: controlRadius,
        textColor:
          defaults.textColor === PENPOT_PALETTE.textSubtle
            ? subtle
            : defaults.textColor === PENPOT_PALETTE.danger
              ? danger
              : defaults.textColor === PENPOT_PALETTE.accent
                ? accent
                : text,
        fontSize: bodySize,
      };
    case 'icon-button':
    case 'toggle':
    case 'tab':
      return {
        ...defaults,
        fillColor: defaults.fillColor
          ? defaults.fillColor === PENPOT_PALETTE.surfaceSelected
            ? selected
            : raised
          : null,
        strokeColor: defaults.strokeColor ? border : null,
        borderRadius: controlRadius,
        textColor: defaults.textColor === PENPOT_PALETTE.accent ? accent : text,
        fontSize: bodySize,
      };
    case 'field':
      return {
        ...defaults,
        fillColor: inset,
        strokeColor: strongBorder,
        borderRadius: controlRadius,
        textColor: muted,
        fontSize: bodySize,
      };
    case 'row':
      return {
        ...defaults,
        fillColor: defaults.fillColor ? selected : null,
        textColor: defaults.textColor === PENPOT_PALETTE.accent ? accent : text,
        fontSize: bodySize,
      };
    case 'chip':
    case 'badge':
      return {
        ...defaults,
        fillColor: defaults.fillColor ? raised : null,
        strokeColor: defaults.strokeColor ? border : null,
        borderRadius: controlRadius,
        textColor: muted,
        fontSize: captionSize,
      };
    case 'divider':
      return {
        ...defaults,
        fillColor: border,
        textColor: subtle,
        fontSize: captionSize,
      };
    case 'progress':
      return {
        ...defaults,
        fillColor: raised,
        strokeColor: border,
        borderRadius: controlRadius,
        textColor: accent,
        fontSize: captionSize,
      };
    default:
      return defaults;
  }
}

export function applyPenpotPrefabFoundation(document: ZuiDocument): string[] {
  const nodes = document.nodes;
  if (!nodes || Object.keys(nodes).length === 0) return [];
  const changes = new Set<string>();
  if (document.asset['design_profile'] !== PENPOT_DESIGN_PROFILE) {
    document.asset['design_profile'] = PENPOT_DESIGN_PROFILE;
    changes.add('assign-penpot-design-profile');
  }

  const tokens = (document.tokens ??= {});
  for (const [key, value] of Object.entries(PREFAB_TOKENS)) {
    if (tokens[key] !== value) {
      tokens[key] = value;
      changes.add('embed-penpot-prefab-tokens');
    }
  }

  const viewportRoots = new Set<string>();
  const rootNode = stringValue(document.root?.['node']);
  if (document.asset.kind === 'view' && rootNode) viewportRoots.add(rootNode);
  for (const [nodeId, node] of Object.entries(nodes)) {
    const prefabClass = prefabClassForRole(
      prefabRoleForNode(node, nodeId, viewportRoots.has(nodeId)),
    );
    const originalClasses = [...(node.classes ?? [])];
    const classes = originalClasses.filter(
      (className) => !className.startsWith('zr-prefab-'),
    );
    classes.push(prefabClass);
    if (!sameValue(originalClasses, classes)) {
      node.classes = classes;
      changes.add('apply-penpot-prefab-classes');
    }
  }

  const stylesheet = penpotPrefabStylesheet();
  const stylesheets = [...(document.stylesheets ?? [])];
  const existingIndex = stylesheets.findIndex(
    (candidate) => candidate['id'] === PENPOT_PREFAB_STYLESHEET_ID,
  );
  if (existingIndex === -1) {
    stylesheets.push(stylesheet);
    document.stylesheets = stylesheets;
    changes.add('embed-penpot-prefab-styles');
  } else if (!sameValue(stylesheets[existingIndex], stylesheet)) {
    stylesheets[existingIndex] = stylesheet;
    document.stylesheets = stylesheets;
    changes.add('embed-penpot-prefab-styles');
  }
  return [...changes];
}

export function resolveDesignNumber(
  document: ZuiDocument,
  value: unknown,
): number | null {
  return resolveNumber(document, value, new Set<string>());
}

function explicitDesignNumber(
  document: ZuiDocument,
  key: string,
): number | null {
  let value: unknown = document.tokens?.[key];
  const seen = new Set<string>();
  while (typeof value === 'string' && value.startsWith('$')) {
    const reference = value.slice(1);
    if (seen.has(reference)) return null;
    seen.add(reference);
    value = document.tokens?.[reference];
  }
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value === 'string' && value.trim() !== '') {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) return parsed;
  }
  return null;
}

function explicitDesignColor(
  document: ZuiDocument,
  key: string,
): string | null {
  let value: unknown = document.tokens?.[key];
  const seen = new Set<string>();
  while (typeof value === 'string' && value.startsWith('$')) {
    const reference = value.slice(1);
    if (seen.has(reference)) return null;
    seen.add(reference);
    value = document.tokens?.[reference];
  }
  if (
    typeof value === 'string' &&
    (/^#[0-9a-fA-F]{3,8}$/.test(value.trim()) ||
      value.trim().toLowerCase() === 'transparent')
  ) {
    return value.trim();
  }
  return null;
}

export function resolveDesignColor(
  document: ZuiDocument,
  value: unknown,
): string | null {
  if (typeof value !== 'string' || value.trim() === '') return null;
  const normalized = value.trim();
  if (
    /^#[0-9a-fA-F]{3,8}$/.test(normalized) ||
    normalized.toLowerCase() === 'transparent'
  ) {
    return normalized;
  }
  if (normalized.startsWith('$')) {
    const key = normalized.slice(1);
    const token = document.tokens?.[key];
    if (token !== undefined && token !== value) {
      const resolved = resolveDesignColor(document, token);
      if (resolved) return resolved;
    }
  }
  const key = normalized.toLowerCase();
  if (/(danger|error|destructive)/.test(key)) return PENPOT_PALETTE.danger;
  if (/warning/.test(key)) return PENPOT_PALETTE.warning;
  if (/(accent|primary)/.test(key)) return PENPOT_PALETTE.accent;
  if (/(info|link)/.test(key)) return PENPOT_PALETTE.info;
  if (/(text.*disabled|text.*subtle)/.test(key))
    return PENPOT_PALETTE.textSubtle;
  if (/(text.*secondary|text.*muted|foreground.*muted)/.test(key))
    return PENPOT_PALETTE.textMuted;
  if (/(text|foreground)/.test(key)) return PENPOT_PALETTE.text;
  if (/(separator|border|outline)/.test(key)) return PENPOT_PALETTE.border;
  if (/(selected|selection)/.test(key)) return PENPOT_PALETTE.surfaceSelected;
  if (/(surface\.0|inset)/.test(key)) return PENPOT_PALETTE.surfaceInset;
  if (/(surface\.1|shell|canvas|background)/.test(key))
    return PENPOT_PALETTE.canvas;
  if (/(surface\.3|raised|hover)/.test(key))
    return PENPOT_PALETTE.surfaceRaised;
  if (/(surface|panel)/.test(key)) return PENPOT_PALETTE.surface;
  return null;
}

export function responsiveDirection(value: unknown): 'row' | 'column' | null {
  const candidate = responsivePreviewValue(value);
  return candidate === 'row' || candidate === 'column' ? candidate : null;
}

export function responsivePreviewValue(value: unknown): unknown {
  if (!isTable(value)) return value;
  for (const breakpoint of ['md', 'lg', 'xl', 'sm', 'xs']) {
    if (Object.hasOwn(value, breakpoint)) return value[breakpoint];
  }
  return undefined;
}

function penpotPrefabStylesheet(): ZuiTable {
  const rule = (
    role: ZuiPrefabRole,
    background: string | null,
    foreground: string,
    border: string | null,
    radius: number,
  ): ZuiTable => {
    const self: ZuiTable = {
      foreground: { color: foreground },
    };
    if (background) self['background'] = { color: background };
    if (border) self['border'] = { color: border, width: 1, radius };
    return { selector: `.${PREFAB_CLASS[role]}`, set: { self } };
  };
  return {
    id: PENPOT_PREFAB_STYLESHEET_ID,
    rules: [
      rule('root', '$zr.penpot.color.canvas', '$zr.penpot.color.text', null, 0),
      rule(
        'panel',
        '$zr.penpot.color.surface',
        '$zr.penpot.color.text',
        '$zr.penpot.color.border',
        8,
      ),
      rule(
        'button',
        '$zr.penpot.color.surface_raised',
        '$zr.penpot.color.text',
        '$zr.penpot.color.border_strong',
        6,
      ),
      rule(
        'icon-button',
        '$zr.penpot.color.surface_raised',
        '$zr.penpot.color.text_muted',
        '$zr.penpot.color.border',
        6,
      ),
      rule(
        'field',
        '$zr.penpot.color.surface_inset',
        '$zr.penpot.color.text_muted',
        '$zr.penpot.color.border_strong',
        6,
      ),
      rule(
        'toggle',
        '$zr.penpot.color.surface_raised',
        '$zr.penpot.color.text',
        '$zr.penpot.color.border',
        6,
      ),
      rule(
        'tab',
        '$zr.penpot.color.surface_raised',
        '$zr.penpot.color.text',
        '$zr.penpot.color.border',
        6,
      ),
      rule('row', null, '$zr.penpot.color.text', null, 4),
      rule(
        'chip',
        '$zr.penpot.color.surface_raised',
        '$zr.penpot.color.text_muted',
        '$zr.penpot.color.border',
        6,
      ),
      rule(
        'badge',
        '$zr.penpot.color.surface_raised',
        '$zr.penpot.color.text_muted',
        '$zr.penpot.color.border',
        10,
      ),
      rule(
        'canvas',
        '$zr.penpot.color.surface_inset',
        '$zr.penpot.color.text_muted',
        '$zr.penpot.color.border',
        6,
      ),
      rule('title', null, '$zr.penpot.color.text', null, 0),
      rule('label', null, '$zr.penpot.color.text', null, 0),
      rule('caption', null, '$zr.penpot.color.text_muted', null, 0),
      rule(
        'divider',
        '$zr.penpot.color.border',
        '$zr.penpot.color.text_muted',
        null,
        0,
      ),
      rule(
        'progress',
        '$zr.penpot.color.surface_raised',
        '$zr.penpot.color.accent',
        '$zr.penpot.color.border',
        6,
      ),
    ],
  };
}

function resolveNumber(
  document: ZuiDocument,
  value: unknown,
  seen: Set<string>,
): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value !== 'string' || value.trim() === '') return null;
  const normalized = value.trim();
  const parsed = Number(normalized);
  if (Number.isFinite(parsed)) return parsed;
  if (normalized.startsWith('$')) {
    const key = normalized.slice(1);
    if (!seen.has(key)) {
      seen.add(key);
      const token = document.tokens?.[key];
      if (token !== undefined) {
        const resolved = resolveNumber(document, token, seen);
        if (resolved !== null) return resolved;
      }
    }
  }
  const key = normalized.toLowerCase();
  if (/(border[_.]width|separator[_.]width)/.test(key)) return 1;
  if (/(radius.*small|corner.*small)/.test(key)) return 4;
  if (/(radius.*panel|corner.*panel)/.test(key)) return 8;
  if (/(radius|corner)/.test(key)) return 6;
  if (/(gap|space|spacing).*(xxs|tiny)/.test(key)) return 2;
  if (/(gap|space|spacing).*(xs|small)/.test(key)) return 4;
  if (/(gap|space|spacing).*(medium|md)/.test(key)) return 8;
  if (/(gap|space|spacing).*(large|lg)/.test(key)) return 12;
  if (/(padding).*(small|compact)/.test(key)) return 8;
  if (/(padding).*(medium|panel)/.test(key)) return 12;
  if (/(height).*(dense|small)/.test(key)) return 28;
  if (/(height).*(compact|medium)/.test(key)) return 32;
  if (/(height).*default/.test(key)) return 32;
  if (/(height).*(standard|large)/.test(key)) return 36;
  if (/(row[_.]height)/.test(key)) return 28;
  if (/(font|typography).*(caption|meta|small).*size/.test(key)) return 10;
  if (/(font|typography).*(title|heading).*size/.test(key)) return 14;
  if (/(font|typography).*size/.test(key)) return 12;
  if (/(font|typography).*(strong|emphasis|semibold).*weight/.test(key))
    return 600;
  if (/(font|typography).*weight/.test(key)) return 400;
  return null;
}

function visual(
  fillColor: string | null,
  strokeColor: string | null,
  strokeWidth: number,
  borderRadius: number,
  textColor: string,
  fontSize: number,
  fontWeight: string,
  textAlign: 'left' | 'center' | 'right',
): PrefabVisualDefaults {
  return {
    fillColor,
    strokeColor,
    strokeWidth,
    borderRadius,
    textColor,
    fontSize,
    fontWeight,
    textAlign,
  };
}

function isTextComponent(component: string): boolean {
  return component.includes('label') || component === 'text';
}

function hasWord(node: ZuiNode, word: string): boolean {
  const values = [
    node.component,
    ...(node.classes ?? []),
    stringValue(node.props?.['button_variant']) ?? '',
    stringValue(node.props?.['button_color']) ?? '',
    stringValue(node.props?.['text_tone']) ?? '',
  ];
  return values.some((value) => value.toLowerCase().includes(word));
}

function stringValue(value: unknown): string | null {
  return typeof value === 'string' ? value : null;
}

function booleanValue(value: unknown): boolean {
  return value === true;
}

function isTable(value: unknown): value is ZuiTable {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function sameValue(
  left: ZuiValue | ZuiTable,
  right: ZuiValue | ZuiTable,
): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import {
  cloneZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
  type ZuiNode,
} from '../src/bridge/zui-document';

interface Consumer {
  sourcePath: string;
  nodeId: string;
  label: string;
  width: number;
  height: number;
}

const hud = 'examples/woc/assets/ui/hud/in_world_hud.zui';
const auth = 'examples/woc/assets/ui/shell/auth_form.zui';
const character = 'examples/woc/assets/ui/shell/character_create.zui';
const lockpick = 'examples/woc/assets/ui/hud/lockpick_window.zui';
const editorTheme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
interface EditorConsumer {
  sourcePath: string;
  nodeId?: string;
  label: string;
  x: number;
  y: number;
}

const editorThemeComponents: EditorConsumer[] = [
  {
    sourcePath:
      'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui',
    label: 'Button',
    x: 32,
    y: 120,
  },
  {
    sourcePath:
      'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_field.zui',
    label: 'Field',
    x: 328,
    y: 120,
  },
  {
    sourcePath:
      'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_dropdown.zui',
    label: 'Dropdown',
    x: 32,
    y: 240,
  },
  {
    sourcePath:
      'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_tab.zui',
    label: 'Tab',
    x: 328,
    y: 240,
  },
  {
    sourcePath:
      'zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_divider.zui',
    label: 'Divider',
    x: 32,
    y: 360,
  },
];
const showcaseSource =
  'zircon_editor/assets/ui/editor/components/showcase/showcase_input_section.zui';
const showcaseThemeComponents: EditorConsumer[] = [
  ['button_demo', 'Primary action'],
  ['button_outlined_demo', 'Outlined action'],
  ['checkbox_demo', 'Checkbox'],
  ['text_field_demo', 'Text field'],
  ['toggle_button_demo', 'Toggle action'],
  ['number_field_demo', 'Number field'],
].map(([nodeId, label], index) => ({
  sourcePath: showcaseSource,
  nodeId,
  label,
  x: index % 2 === 0 ? 32 : 328,
  y: 120 + Math.floor(index / 2) * 120,
}));

const EDITOR_THEME_CONSUMERS: Record<string, EditorConsumer[]> = {
  [editorTheme]: editorThemeComponents,
  'zircon_editor/assets/ui/theme/editor_base.zui': showcaseThemeComponents,
  'zircon_editor/assets/ui/theme/editor_material.zui': showcaseThemeComponents,
  'zircon_editor/assets/ui/theme/editor_unreal_dark.zui':
    showcaseThemeComponents,
  'zircon_editor/assets/ui/theme/editor_workbench_spatial.zui':
    editorThemeComponents,
  'zircon_editor/assets/ui/theme/editor_workbench_strict.zui':
    editorThemeComponents,
};

// These are explicit acceptance specimens. Text and control properties are read
// from the product source; only the mounting geometry belongs to this host.
const CONSUMERS: Record<string, Consumer[]> = {
  'examples/woc/assets/ui/hud/hud_theme.zui': [
    {
      sourcePath: hud,
      nodeId: 'action_slot_0',
      label: 'Action slot',
      width: 46,
      height: 46,
    },
    {
      sourcePath: hud,
      nodeId: 'touch_jump',
      label: 'Touch action',
      width: 58,
      height: 48,
    },
    {
      sourcePath: hud,
      nodeId: 'pause_keybinds',
      label: 'Pause action',
      width: 280,
      height: 42,
    },
    {
      sourcePath: lockpick,
      nodeId: 'lockpick_ante_premium',
      label: 'Lockpick ante',
      width: 240,
      height: 100,
    },
  ],
  'examples/woc/assets/ui/shell/shell_theme.zui': [
    {
      sourcePath: auth,
      nodeId: 'submit',
      label: 'Primary action',
      width: 280,
      height: 44,
    },
    {
      sourcePath: auth,
      nodeId: 'back',
      label: 'Secondary action',
      width: 280,
      height: 44,
    },
    {
      sourcePath: auth,
      nodeId: 'username',
      label: 'Text field',
      width: 320,
      height: 44,
    },
    {
      sourcePath: auth,
      nodeId: 'toggle_mode',
      label: 'Text action',
      width: 280,
      height: 40,
    },
    {
      sourcePath: character,
      nodeId: 'class_warrior',
      label: 'Class selection',
      width: 112,
      height: 56,
    },
    {
      sourcePath: character,
      nodeId: 'skin_1',
      label: 'Appearance selection',
      width: 56,
      height: 56,
    },
  ],
};

async function prepareEditorThemeReviewHost(
  repoRoot: string,
  theme: ZuiDocument,
  specimens: EditorConsumer[],
): Promise<{
  source: string;
  projection: ZuiDocument;
  consumers: Consumer[];
}> {
  const nodes: Record<string, ZuiNode> = {
    review_theme_host: {
      component: 'Panel',
      control_id: 'EditorThemeReviewHost',
      classes: ['workbench-panel'],
      style: {
        self: {
          background: { color: '$editor.surface.0' },
          foreground: { color: '$editor.text.primary' },
        },
      },
      layout: {
        container: { kind: 'Overlay' },
        width: { stretch: 'Stretch' },
        height: { stretch: 'Stretch' },
      },
      children: [{ node: 'review_theme_title' }],
    },
    review_theme_title: {
      component: 'Label',
      control_id: 'EditorThemeReviewTitle',
      props: {
        text: 'Editor component states',
        font_size: '$editor.typography.title.size',
        font_weight: '$editor.typography.strong.weight',
      },
      layout: {
        position: { x: 32, y: 32 },
        width: { preferred: 560, stretch: 'Fixed' },
        height: { preferred: 32, stretch: 'Fixed' },
      },
    },
  };
  const consumers: Consumer[] = [];
  for (const [index, item] of specimens.entries()) {
    const source = parseZuiDocument(
      await readFile(resolve(repoRoot, item.sourcePath), 'utf8'),
    ).document;
    const definition = Object.values(source.components ?? {})[0];
    const rootId = item.nodeId ?? definition?.root;
    const root = rootId ? source.nodes?.[rootId] : undefined;
    if (!root || root.children?.length)
      throw new Error(`Editor theme specimen is missing ${item.sourcePath}`);
    const labelId = `review_theme_label_${index}`;
    const nodeId = `review_theme_component_${index}`;
    nodes[labelId] = {
      component: 'Label',
      control_id: `${nodeId}Label`,
      props: {
        text: item.label,
        font_size: '$editor.typography.caption.size',
        font_weight: '$editor.typography.medium.weight',
        text_tone: 'secondary',
      },
      layout: {
        position: { x: item.x, y: item.y },
        width: { preferred: 280, stretch: 'Fixed' },
        height: { preferred: 20, stretch: 'Fixed' },
      },
    };
    nodes[nodeId] = {
      ...root,
      control_id: `${nodeId}Control`,
      layout: {
        ...root.layout,
        position: { x: item.x, y: item.y + 28 },
        width: {
          min: item.label === 'Divider' ? 240 : 280,
          preferred: item.label === 'Divider' ? 240 : 280,
          max: 280,
          stretch: 'Fixed',
        },
      },
    };
    nodes['review_theme_host'].children!.push(
      { node: labelId },
      { node: nodeId },
    );
    consumers.push({
      sourcePath: item.sourcePath,
      nodeId: rootId,
      label: item.label,
      width: item.label === 'Divider' ? 240 : 280,
      height: item.label === 'Divider' ? 4 : 32,
    });
  }
  const document: ZuiDocument = {
    asset: {
      kind: 'view',
      version: 2,
      id: `${theme.asset.id}#review-host`,
      display_name: theme.asset.display_name,
    },
    imports: { widgets: [], styles: [theme.asset.id] },
    root: { node: 'review_theme_host' },
    nodes,
  };
  const source = serializeZuiDocument(document);
  const projection = cloneZuiDocument(document);
  projection.imports = { widgets: [], styles: [] };
  projection.tokens = theme.tokens;
  projection.stylesheets = theme.stylesheets;
  return { source, projection, consumers };
}

export async function prepareThemeReviewHost(
  repoRoot: string,
  sourcePath: string,
  theme: ZuiDocument,
): Promise<{
  source: string;
  projection: ZuiDocument;
  consumers: Consumer[];
} | null> {
  const editorConsumers = EDITOR_THEME_CONSUMERS[sourcePath];
  if (editorConsumers) {
    if (
      !['style', 'theme_tokens'].includes(theme.asset.kind) ||
      Object.keys(theme.nodes ?? {}).length
    )
      throw new Error(
        `Editor theme consumer host does not match ${sourcePath}`,
      );
    return prepareEditorThemeReviewHost(repoRoot, theme, editorConsumers);
  }
  const consumers = CONSUMERS[sourcePath];
  if (!consumers) return null;
  if (theme.asset.kind !== 'style' || Object.keys(theme.nodes ?? {}).length)
    throw new Error(`Theme consumer host does not match ${sourcePath}`);
  const nodes: Record<string, ZuiNode> = {
    review_theme_host: {
      component: 'Overlay',
      classes: ['woc-shell'],
      layout: {
        container: { kind: 'Overlay' },
        width: { stretch: 'Stretch' },
        height: { stretch: 'Stretch' },
      },
      children: [{ node: 'review_theme_title' }],
    },
    review_theme_title: {
      component: 'Label',
      classes: ['woc-screen-title'],
      props: { text: theme.asset.display_name ?? theme.asset.id },
      layout: {
        position: { x: 32, y: 24 },
        width: { preferred: 800, stretch: 'Fixed' },
        height: { preferred: 40, stretch: 'Fixed' },
      },
    },
  };
  for (const [index, consumer] of consumers.entries()) {
    const document = parseZuiDocument(
      await readFile(resolve(repoRoot, consumer.sourcePath), 'utf8'),
    ).document;
    const node = cloneZuiDocument(document).nodes?.[consumer.nodeId];
    if (
      !node ||
      node.children?.length ||
      !['Button', 'ToggleButton', 'TextField'].includes(node.component)
    )
      throw new Error(
        `Theme specimen requires an explicit leaf control: ${consumer.sourcePath}#${consumer.nodeId}`,
      );
    if (!(node.classes ?? []).some((value) => value.startsWith('woc-')))
      throw new Error(
        `Theme specimen has no WoC style class: ${consumer.nodeId}`,
      );
    const x = 32 + (index % 2) * 400;
    const y = 104 + Math.floor(index / 2) * 184;
    const id = `sample_${index}`;
    nodes[`${id}_label`] = {
      component: 'Label',
      classes: ['woc-section-title'],
      props: { text: consumer.label },
      layout: {
        position: { x, y },
        width: { preferred: 344, stretch: 'Fixed' },
        height: { preferred: 24, stretch: 'Fixed' },
      },
    };
    nodes[id] = {
      ...node,
      layout: {
        ...node.layout,
        anchor: { x: 0, y: 0 },
        pivot: { x: 0, y: 0 },
        position: { x, y: y + 36 },
        width: {
          min: consumer.width,
          preferred: consumer.width,
          max: consumer.width,
          stretch: 'Fixed',
        },
        height: {
          min: consumer.height,
          preferred: consumer.height,
          max: consumer.height,
          stretch: 'Fixed',
        },
      },
    };
    nodes['review_theme_host'].children!.push(
      { node: `${id}_label` },
      { node: id },
    );
  }
  const document: ZuiDocument = {
    asset: {
      kind: 'view',
      version: 2,
      id: `${theme.asset.id}#review-host`,
      display_name: theme.asset.display_name,
    },
    imports: { widgets: [], styles: [theme.asset.id] },
    root: { node: 'review_theme_host' },
    nodes,
  };
  const source = serializeZuiDocument(document);
  const projection = cloneZuiDocument(document);
  projection.imports = { widgets: [], styles: [] };
  projection.tokens = theme.tokens;
  projection.stylesheets = theme.stylesheets;
  return { source, projection, consumers };
}

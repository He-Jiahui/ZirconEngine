import {
  ZuiDocumentError,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
  type ZuiValue,
} from './zui-document';
import { isSelectionControl } from './zui-input-projection';

type PropertyKind = 'color' | 'number' | 'weight' | 'align' | 'family';
interface PaintProperty {
  kind: PropertyKind;
  candidates: readonly (readonly [string, string?])[];
}

// Candidate order mirrors surface/render/resolve.rs, including nested-first precedence.
const PAINT_PROPERTIES = {
  selected_foreground_color: {
    kind: 'color',
    candidates: [['selected_foreground_color'], ['selected_text_color']],
  },
  idle_text_color: {
    kind: 'color',
    candidates: [['foreground_color'], ['idle_text_color']],
  },
  tab_font_size: { kind: 'number', candidates: [['tab_font_size']] },
  tab_line_height: { kind: 'number', candidates: [['tab_line_height']] },
  label_color: { kind: 'color', candidates: [['label_color']] },
  disabled_foreground_color: {
    kind: 'color',
    candidates: [['disabled_foreground_color']],
  },
  background_color: {
    kind: 'color',
    candidates: [['background'], ['background_color']],
  },
  foreground_color: {
    kind: 'color',
    candidates: [['foreground'], ['foreground_color'], ['fg'], ['color']],
  },
  border_color: {
    kind: 'color',
    candidates: [['border'], ['border_color'], ['outline']],
  },
  border_width: {
    kind: 'number',
    candidates: [['border', 'width'], ['border_width']],
  },
  corner_radius: {
    kind: 'number',
    candidates: [['border', 'radius'], ['radius'], ['corner_radius']],
  },
  font_size: { kind: 'number', candidates: [['font', 'size'], ['font_size']] },
  font_weight: {
    kind: 'weight',
    candidates: [['font', 'weight'], ['font_weight'], ['text_font_weight']],
  },
  font_family: {
    kind: 'family',
    candidates: [['font', 'family'], ['font_family']],
  },
  line_height: {
    kind: 'number',
    candidates: [['font', 'line_height'], ['line_height']],
  },
  line_height_ratio: {
    kind: 'number',
    candidates: [['font', 'line_height_ratio'], ['line_height_ratio']],
  },
  text_align: {
    kind: 'align',
    candidates: [['font', 'align'], ['text_align']],
  },
  opacity: { kind: 'number', candidates: [['opacity']] },
} as const satisfies Record<string, PaintProperty>;

export type AuthoredPaintProperty = keyof typeof PAINT_PROPERTIES;
interface PropertySource {
  value: ZuiValue;
  path: string[];
}

function table(value: unknown): value is ZuiTable {
  return (
    value !== null &&
    typeof value === 'object' &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}

function sourceForProperty(
  node: ZuiNode,
  property: AuthoredPaintProperty,
): PropertySource | null {
  const descriptor: PaintProperty = PAINT_PROPERTIES[property];
  const layers: Array<{ value: ZuiTable | undefined; path: string[] }> = [
    {
      value: table(node.style?.['self']) ? node.style['self'] : undefined,
      path: ['style', 'self'],
    },
    { value: node.state, path: ['state'] },
    { value: node.props, path: ['props'] },
  ];
  const candidates: PaintProperty['candidates'] =
    property === 'foreground_color' && isSelectionControl(node)
      ? [['label_color'], ...descriptor.candidates]
      : descriptor.candidates;
  for (const [key, member] of candidates) {
    const layer = layers.find(
      ({ value }) => value && Object.hasOwn(value, key),
    );
    if (!layer?.value) continue;
    const value = layer.value[key];
    const path = [...layer.path, key];
    if (member) {
      if (!table(value) || value[member] === undefined) continue;
      return { value: value[member], path: [...path, member] };
    }
    if (descriptor.kind === 'color' && table(value)) {
      if (value['color'] === undefined) continue;
      return { value: value['color'], path: [...path, 'color'] };
    }
    return { value, path };
  }
  return null;
}

/** Flatten only the preview copy; source tables remain the roundtrip authority. */
export function authoredPaintPreviewNode(
  document: ZuiDocument,
  node: ZuiNode,
  nodeId: string,
): ZuiNode {
  const props: ZuiTable = { ...node.props };
  // styledPreviewNode has already merged metadata and stylesheet values by key.
  const preview = { ...node, state: undefined, style: undefined };
  for (const name of ['background', 'foreground', 'border', 'font']) {
    const value = props[name];
    if (value === undefined) continue;
    if (!table(value) && typeof value !== 'string')
      throw new ZuiDocumentError(
        `Unsupported structured paint nodes.${nodeId}.props.${name}: expected a string or table.`,
      );
    if (
      table(value) &&
      ['background', 'foreground'].includes(name) &&
      value['color'] === undefined
    )
      throw new ZuiDocumentError(
        `Unsupported structured paint nodes.${nodeId}.props.${name}: missing color.`,
      );
  }
  for (const property of Object.keys(
    PAINT_PROPERTIES,
  ) as AuthoredPaintProperty[]) {
    const source = sourceForProperty(preview, property);
    if (!source) continue;
    const newlyMapped =
      source.path.length > 2 || source.path.at(-1) !== property;
    props[property] = newlyMapped
      ? validatedValue(
          document,
          source,
          PAINT_PROPERTIES[property].kind,
          property,
          nodeId,
        )
      : source.value;
  }
  return { ...node, props };
}

function validatedValue(
  document: ZuiDocument,
  source: PropertySource,
  kind: PropertyKind,
  property: AuthoredPaintProperty,
  nodeId: string,
): ZuiValue {
  let value: ZuiValue | undefined = source.value;
  const seen = new Set<string>();
  while (typeof value === 'string' && value.startsWith('$')) {
    const token = value.slice(1);
    if (seen.has(token)) break;
    seen.add(token);
    value = document.tokens?.[token];
  }
  const fail = (): never => {
    throw new ZuiDocumentError(
      `Unsupported structured paint nodes.${nodeId}.${source.path.join('.')}: cannot map ${property} to Penpot.`,
    );
  };
  if (kind === 'color') {
    const hostColors = document['penpot_host_colors'];
    if (
      typeof value === 'string' &&
      table(hostColors) &&
      Object.hasOwn(hostColors, value)
    )
      value = hostColors[value];
    if (
      typeof value !== 'string' ||
      !/^(?:#[\da-f]{3}|#[\da-f]{4}|#[\da-f]{6}|#[\da-f]{8}|transparent)$/i.test(
        value.trim(),
      )
    )
      return fail();
    return value.trim();
  }
  if (kind === 'align') {
    if (typeof value !== 'string') return fail();
    const normalized = value.trim().toLowerCase();
    if (normalized === 'middle') return 'center';
    if (normalized === 'justified') return 'justify';
    if (!['left', 'center', 'right', 'justify'].includes(normalized))
      return fail();
    return normalized;
  }
  if (kind === 'family') {
    if (typeof value !== 'string' || !value.trim()) return fail();
    return value;
  }
  if (typeof value !== 'number' || !Number.isFinite(value) || value < 0)
    return fail();
  if (property === 'font_size' && value <= 0) return fail();
  if (kind === 'weight' && (value < 100 || value > 900 || value % 100 !== 0))
    return fail();
  return value;
}

/** Write through the original winning property, preserving sibling fields and tokens. */
export function writeAuthoredPaintProperty(
  node: ZuiNode,
  nodeId: string,
  property: AuthoredPaintProperty,
  value: ZuiValue,
  selfStyle?: ZuiTable,
): string {
  const effective = selfStyle ? { ...node, style: { self: selfStyle } } : node;
  const path = sourceForProperty(effective, property)?.path ?? [
    'props',
    property,
  ];
  let current = node as unknown as ZuiTable;
  for (const key of path.slice(0, -1)) {
    if (current[key] === undefined) {
      const original =
        path[0] === 'style' && path[1] === 'self'
          ? selfStyle?.[key]
          : undefined;
      current[key] = table(original) ? { ...original } : {};
    }
    if (!table(current[key]))
      throw new ZuiDocumentError(
        `Cannot write nodes.${nodeId}.${path.join('.')}: ${key} is not a table.`,
      );
    current = current[key];
  }
  current[path.at(-1)!] = value;
  return `nodes.${nodeId}.${path.join('.')}`;
}

export function writeAuthoredLineHeight(
  node: ZuiNode,
  nodeId: string,
  ratio: number,
  size: number,
  selfStyle?: ZuiTable,
): string {
  const effective = selfStyle ? { ...node, style: { self: selfStyle } } : node;
  return sourceForProperty(effective, 'line_height')
    ? writeAuthoredPaintProperty(
        node,
        nodeId,
        'line_height',
        ratio * size,
        selfStyle,
      )
    : writeAuthoredPaintProperty(
        node,
        nodeId,
        'line_height_ratio',
        ratio,
        selfStyle,
      );
}

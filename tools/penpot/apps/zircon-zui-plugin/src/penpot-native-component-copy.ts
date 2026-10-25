import type { Board, Shape } from '@penpot/plugin-types';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { applySlotPadding } from './penpot-slot-padding';
import { nativeLayoutAllocationForCopy } from './penpot-layout-allocation';

export function shapeChildren(shape: Shape): Shape[] {
  return 'children' in shape ? [...shape.children] : [];
}

export function nativeComponentStructure(shape: Shape): unknown {
  return [
    shape.type,
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'role'),
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-property'),
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-fragment'),
    nativeSizeConstraints(shape),
    shapeChildren(shape).map(nativeComponentStructure),
  ];
}

export function supportsNativeComponentCopy(shape: Shape): boolean {
  return (
    ['board', 'text', 'rect', 'ellipse'].includes(shape.type) &&
    shapeChildren(shape).every(supportsNativeComponentCopy)
  );
}

// Instance overrides retain the native component link and the source node IDs.
export function copyNativeComponentAppearance(
  source: Shape,
  target: Shape,
): void {
  const ids = new Map<string, string>();
  const targetIds = new Set<string>();
  const collect = (original: Shape, copy: Shape): void => {
    if (original.type !== copy.type || shapeChildren(original).length !== shapeChildren(copy).length)
      throw new Error(`Native component structure differs: ${original.name}`);
    if (original.id && copy.id) {
      if (ids.has(original.id)) throw new Error('Native component source has duplicate shape IDs.');
      if (targetIds.has(copy.id)) throw new Error('Native component target has duplicate shape IDs.');
      ids.set(original.id, copy.id);
      targetIds.add(copy.id);
    }
    shapeChildren(original).forEach((child, index) => collect(child, shapeChildren(copy)[index]));
  };
  collect(source, target);
  if (source.parent?.id && target.parent?.id && !ids.has(source.parent.id))
    ids.set(source.parent.id, target.parent.id);
  // Validate every derived reference before applying any appearance or metadata.
  const plans = new Map<string | Shape, Map<string, string>>();
  const prepare = (original: Shape, copy: Shape): void => {
    const values = new Map<string, string>();
    for (const key of new Set([
      ...copy.getSharedPluginDataKeys(ZUI_METADATA_NAMESPACE),
      ...original.getSharedPluginDataKeys(ZUI_METADATA_NAMESPACE),
    ])) {
      const raw = original.getSharedPluginData(ZUI_METADATA_NAMESPACE, key) ?? '';
      values.set(key, copiedMetadata(key, raw, original, copy, ids));
    }
    plans.set(copy.id || copy, values);
    shapeChildren(original).forEach((child, index) => prepare(child, shapeChildren(copy)[index]));
  };
  prepare(source, target);
  copyAppearance(source, target, plans);
}

/** Interpret inherited painter IDs through the SDK link without copying appearance. */
export function inheritedTreeRowPainterForCopy(source: Shape, target: Shape): string {
  const component = target.component?.();
  const originalComponent = source.component?.();
  if (!component || !target.isComponentCopyInstance?.() ||
      !originalComponent || originalComponent.id !== component.id ||
      originalComponent.libraryId !== component.libraryId ||
      !source.isComponentMainInstance?.() || component.mainInstance()?.id !== source.id)
    throw new Error('TreeRow requires its actual native component main and copy.');
  const raw = source.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'tree-row-authoring-painter');
  if (!raw || target.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'tree-row-authoring-painter') !== raw)
    throw new Error('TreeRow inherited painter differs from its canonical main.');
  const state = JSON.parse(raw) as Record<string, unknown>;
  const record = (value: unknown): value is Record<string, unknown> =>
    !!value && typeof value === 'object' && !Array.isArray(value);
  if (!record(state) || state['version'] !== 1 || !Array.isArray(state['parts']) ||
      !['generated', 'originalPaint', 'originalGeometry', 'originalBounds', 'originalWidths']
        .every((key) => record(state[key])))
    throw new Error('TreeRow linked copy requires the immutable original painter baseline.');

  const originals = new Map<string, Shape>(), copies = new Map<string, Shape>();
  const ids = new Map<string, string>(), targetIds = new Set<string>();
  const collectSource = (shape: Shape): void => {
    if (!shape.id || originals.has(shape.id))
      throw new Error('TreeRow canonical main has duplicate or missing shape IDs.');
    originals.set(shape.id, shape);
    for (const child of shapeChildren(shape)) collectSource(child);
  };
  collectSource(source);
  const collectCopy = (shape: Shape): void => {
    const reference = shape.componentRefShape?.(), owner = shape.component?.();
    const original = reference && originals.get(reference.id);
    if (!shape.id || targetIds.has(shape.id) || originals.has(shape.id) || !original || ids.has(original.id) ||
        !owner || owner.id !== component.id || owner.libraryId !== component.libraryId ||
        shape.type !== original.type)
      throw new Error('TreeRow native source references are missing, duplicated or outside its main.');
    ids.set(original.id, shape.id);
    targetIds.add(shape.id);
    copies.set(shape.id, shape);
    for (const child of shapeChildren(shape)) collectCopy(child);
  };
  collectCopy(target);
  if (ids.size !== originals.size || ids.get(source.id) !== target.id ||
      !source.parent?.id || !target.parent?.id || originals.has(source.parent.id) ||
      targetIds.has(target.parent.id))
    throw new Error('TreeRow native source references do not cover its complete main.');
  ids.set(source.parent.id, target.parent.id);
  for (const original of originals.values()) {
    const copy = copies.get(ids.get(original.id)!)!;
    if (copy.parent?.id !== ids.get(original.parent?.id ?? '') ||
        JSON.stringify(shapeChildren(copy).map((child) => child.id)) !==
          JSON.stringify(shapeChildren(original).map((child) => ids.get(child.id))))
      throw new Error('TreeRow native parent or child order changed without a source mapping.');
  }
  const generated = state['generated'] as Record<string, unknown>;
  if (Object.keys(generated).length !== originals.size ||
      Object.keys(generated).some((id) => !originals.has(id)))
    throw new Error('TreeRow immutable generated baseline does not cover its complete main.');
  // Only reference keys, parents and child IDs change. Captured geometry,
  // paint, path content and strokes never come from the copy's readback.
  return copiedMetadata('tree-row-authoring-painter', raw, source, target, ids);
}

function copiedMetadata(
  key: string, raw: string, source: Shape, target: Shape, ids: ReadonlyMap<string, string>,
): string {
  if (!raw) return raw;
  const reference = (id: unknown): string => {
    if (typeof id !== 'string' || !ids.has(id))
      throw new Error(`Native component ${key} refers to a shape outside its source copy.`);
    return ids.get(id)!;
  };
  const keyed = (value: unknown, entry: (value: unknown) => unknown = (item) => item): unknown => {
    if (!value || typeof value !== 'object' || Array.isArray(value))
      throw new Error(`Invalid native component ${key} reference map.`);
    return Object.fromEntries(Object.entries(value).map(([id, item]) => [reference(id), entry(item)]));
  };
  if (key === 'native-layout-allocation') {
    if (source.type !== 'board' || target.type !== 'board')
      throw new Error('Native layout allocation requires a board copy.');
    return nativeLayoutAllocationForCopy(source, target);
  }
  if (key === 'control-part-list') {
    const parts: unknown = JSON.parse(raw);
    if (!Array.isArray(parts)) throw new Error('Invalid native component painter part list.');
    return JSON.stringify(parts.map(reference).sort());
  }
  if (key !== 'tree-row-authoring-painter' && key !== 'field-icon-paint') return raw;
  const state = JSON.parse(raw) as Record<string, unknown>;
  if (!state || typeof state !== 'object' || Array.isArray(state) || state['version'] !== 1)
    throw new Error(`Invalid native component ${key} state.`);
  if (key === 'tree-row-authoring-painter') {
    if (!Array.isArray(state['parts'])) throw new Error('Invalid native TreeRow part references.');
    state['parts'] = state['parts'].map(reference);
    if (state['generated']) state['generated'] = keyed(state['generated'], (item) => {
      if (!item || typeof item !== 'object' || Array.isArray(item))
        throw new Error('Invalid native TreeRow generated part.');
      const part = { ...item } as Record<string, unknown>;
      if (part['parent'] !== null) part['parent'] = reference(part['parent']);
      if (!Array.isArray(part['children'])) throw new Error('Invalid native TreeRow children.');
      part['children'] = part['children'].map(reference);
      return part;
    });
    for (const name of ['originalPaint', 'originalGeometry', 'originalBounds', 'originalWidths'])
      if (state[name]) state[name] = keyed(state[name]);
  } else {
    for (const name of ['generated', 'originalWidths'])
      if (state[name]) state[name] = keyed(state[name]);
  }
  return JSON.stringify(state);
}

function copyAppearance(
  source: Shape, target: Shape, plans: ReadonlyMap<string | Shape, ReadonlyMap<string, string>>,
): void {
  if (source.type !== target.type)
    throw new Error(`Native component shape type differs: ${source.name}`);
  const originals = shapeChildren(source),
    copies = shapeChildren(target);
  if (originals.length !== copies.length)
    throw new Error(`Native component structure differs: ${source.name}`);
  for (const [key, value] of plans.get(target.id || target)!) {
    if (target.getSharedPluginData(ZUI_METADATA_NAMESPACE, key) !== value)
      target.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
  }
  copyProperties(source, target, [
    'name',
    'hidden',
    'opacity',
    'fills',
    'strokes',
    'rotation',
    'constraintsHorizontal',
    'constraintsVertical',
  ]);
  if (source.width !== target.width || source.height !== target.height)
    target.resize(source.width, source.height);
  if (source.type === 'board' && target.type === 'board') {
    copyProperties(source, target, ['borderRadius', 'clipContent']);
  }
  if (source.type === 'text' && target.type === 'text') {
    copyProperties(source, target, [
      'characters',
      'fontId',
      'fontFamily',
      'fontVariantId',
      'fontWeight',
      'fontSize',
      'lineHeight',
      'letterSpacing',
      'align',
      'verticalAlign',
      'growType',
    ]);
  }
  if (source.layoutChild && target.layoutChild) {
    copyProperties(source.layoutChild, target.layoutChild, [
      'absolute',
      'horizontalSizing',
      'verticalSizing',
      'alignSelf',
      'minWidth',
      'maxWidth',
      'minHeight',
      'maxHeight',
    ]);
    if (
      JSON.stringify(nativeSizeConstraints(source)) !==
      JSON.stringify(nativeSizeConstraints(target))
    )
      throw new Error(
        `Penpot did not preserve native instance size constraints: ${source.name}`,
      );
    if (
      ['topMargin', 'rightMargin', 'bottomMargin', 'leftMargin'].some(
        (key) =>
          (source.layoutChild as unknown as Record<string, unknown>)[key] !==
          (target.layoutChild as unknown as Record<string, unknown>)[key],
      )
    )
      applySlotPadding(target as Board, {
        top: source.layoutChild.topMargin ?? 0,
        right: source.layoutChild.rightMargin ?? 0,
        bottom: source.layoutChild.bottomMargin ?? 0,
        left: source.layoutChild.leftMargin ?? 0,
      });
  }
  copyProperties(source, target, ['x', 'y']);
  for (const [index, child] of originals.entries())
    copyAppearance(child, copies[index], plans);
}

function nativeSizeConstraints(shape: Shape): unknown {
  // This host accepts numeric bounds but cannot clear them with null. Keep
  // differently constrained instances in separate native component masters.
  const layout = shape.layoutChild;
  return [
    layout?.minWidth ?? null,
    layout?.maxWidth ?? null,
    layout?.minHeight ?? null,
    layout?.maxHeight ?? null,
  ];
}

function copyProperties<T extends object>(
  source: T,
  target: T,
  keys: (keyof T)[],
): void {
  for (const key of keys) {
    const value = source[key];
    if (
      value !== undefined &&
      JSON.stringify(value) !== JSON.stringify(target[key])
    )
      target[key] = value;
  }
}

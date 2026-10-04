import type { Board, Shape } from '@penpot/plugin-types';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { applySlotPadding } from './penpot-slot-padding';

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
  if (source.type !== target.type)
    throw new Error(`Native component shape type differs: ${source.name}`);
  const originals = shapeChildren(source),
    copies = shapeChildren(target);
  if (originals.length !== copies.length)
    throw new Error(`Native component structure differs: ${source.name}`);
  const keys = new Set([
    ...target.getSharedPluginDataKeys(ZUI_METADATA_NAMESPACE),
    ...source.getSharedPluginDataKeys(ZUI_METADATA_NAMESPACE),
  ]);
  for (const key of keys) {
    const value = source.getSharedPluginData(ZUI_METADATA_NAMESPACE, key) ?? '';
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
    copyNativeComponentAppearance(child, copies[index]);
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

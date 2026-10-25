import type { Shape } from '@penpot/plugin-types';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_NODE,
  ZUI_ROLE_TEXT,
} from './metadata';

const NATIVE_LAYOUT_GEOMETRY = 'native-semantic-layout-geometry';

function nativeLayoutState(shape: Shape) {
  if (typeof shape.getSharedPluginData !== 'function') return null;
  const metadata = (target: Shape, key: string) =>
    target.getSharedPluginData(ZUI_METADATA_NAMESPACE, key);
  const parent = shape.parent, role = metadata(shape, ZUI_METADATA_ROLE);
  if (parent?.type !== 'board' || typeof parent.getSharedPluginData !== 'function' ||
    metadata(parent, ZUI_METADATA_ROLE) !== ZUI_ROLE_NODE)
    return null;
  const part = shape.type === 'text' && role === ZUI_ROLE_TEXT ? 'text'
    : role === ZUI_ROLE_AUXILIARY && metadata(shape, 'native-semantic-glyph') ? 'glyph' : null;
  if (!part) return null;
  const source = { parentId: parent.id, nodeId: metadata(parent, ZUI_METADATA_NODE_ID),
    sourcePath: metadata(parent, 'sourcePath'), sourceNodeId: metadata(parent, 'sourceNodeId'),
    instancePath: metadata(parent, 'instancePath'), controlId: metadata(parent, 'controlId') };
  if (!Object.values(source).every(value => typeof value === 'string' && value.length > 0)) return null;
  const round = (value: number) => Math.round(value * 100) / 100;
  const geometry = { x: round(shape.x - parent.x), y: round(shape.y - parent.y),
    width: round(shape.width), height: round(shape.height) };
  if (!Object.values(geometry).every(Number.isFinite) || geometry.width <= 0 || geometry.height <= 0)
    return null;
  // Table cells have a stable source fragment within the existing owner tuple.
  // Other native glyph/text records retain their existing version-1 shape.
  const tableCell = part === 'text' && metadata(parent, 'component') === 'Table'
    ? { shapeId: shape.id, fragment: metadata(shape, 'text-fragment') || 'primary' }
    : null;
  return { version: 1, part, source,
    ...(tableCell ? { tableCell } : {}),
    ...(part === 'glyph' ? { family: metadata(shape, 'native-semantic-glyph'),
      asset: metadata(shape, 'native-semantic-glyph-asset') } : {}), geometry };
}

/** Called only after the existing native painter applies its source geometry. */
export function recordNativeSemanticLayoutGeometry(shape: Shape): void {
  const state = nativeLayoutState(shape);
  if (!state) return;
  const encoded = JSON.stringify(state);
  if (shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, NATIVE_LAYOUT_GEOMETRY) !== encoded)
    shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, NATIVE_LAYOUT_GEOMETRY, encoded);
}

/** A direct edit or lost owner tuple no longer matches the applied layout. */
export function hasCurrentNativeSemanticLayoutGeometry(shape: Shape | undefined): boolean {
  if (!shape) return false;
  const state = nativeLayoutState(shape);
  return state !== null &&
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, NATIVE_LAYOUT_GEOMETRY) === JSON.stringify(state);
}

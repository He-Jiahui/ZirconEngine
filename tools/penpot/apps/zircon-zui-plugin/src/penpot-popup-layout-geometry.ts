import type { Shape } from '@penpot/plugin-types';
import {
  ZUI_METADATA_EXPLICIT_OVERLAY,
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_ASSET,
} from './metadata';

const POPUP_LAYOUT_GEOMETRY = 'popup-layout-geometry';
export function popupLayoutGeometry(shape: Shape) {
  const round = (value: number) => Math.round(value * 100) / 100;
  return {
    parent: shape.parent?.id,
    x: round(shape.x - (shape.parent?.x ?? 0)),
    y: round(shape.y - (shape.parent?.y ?? 0)),
    width: round(shape.width),
    height: round(shape.height),
  };
}
/** Source-owned popup layout may move; direct edits still differ from the record. */
export function hasCurrentPopupLayoutGeometry(shape: Shape): boolean {
  if (
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !==
    ZUI_ROLE_AUXILIARY
  )
    return false;
  const source =
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'popup-overlay-for') ||
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'popup-content-for');
  if (!source) return false;
  let overlay: Shape | null = shape;
  while (
    overlay &&
    overlay.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'popup-overlay-for') !==
      source
  )
    overlay = overlay.parent;
  if (
    !overlay ||
    overlay.type !== 'board' ||
    overlay.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !==
      ZUI_ROLE_AUXILIARY ||
    overlay.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_EXPLICIT_OVERLAY,
    ) !== 'true' ||
    overlay.parent?.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_ROLE,
    ) !== ZUI_ROLE_ASSET
  )
    return false;
  const current = popupLayoutGeometry(shape);
  if (
    ![current.x, current.y, current.width, current.height].every(
      Number.isFinite,
    )
  )
    return false;
  const encoded = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    POPUP_LAYOUT_GEOMETRY,
  );
  if (!encoded) return false;
  try {
    return JSON.stringify(JSON.parse(encoded)) === JSON.stringify(current);
  } catch {
    return false;
  }
}

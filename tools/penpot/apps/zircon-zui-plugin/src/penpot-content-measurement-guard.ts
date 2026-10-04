import type { Shape } from '@penpot/plugin-types';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_ASSET,
} from './metadata';
/** These invisible probes are regenerated from current source typography. */
export function isCurrentContentMeasurement(shape: Shape): boolean {
  if (
    shape.type !== 'text' ||
    shape.opacity !== 0 ||
    shape.growType !== 'auto-width' ||
    (shape.layoutChild != null && shape.layoutChild.absolute !== true) ||
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !==
      ZUI_ROLE_AUXILIARY ||
    shape.parent?.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_ROLE,
    ) !== ZUI_ROLE_ASSET
  )
    return false;
  const id = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'linear-content-measurement',
  );
  const encoded = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'linear-content-measurement-signature',
  );
  if (!id || !encoded) return false;
  try {
    const signature = JSON.parse(encoded) as {
      text?: unknown;
      style?: {
        family?: unknown;
        size?: unknown;
        weight?: unknown;
        lineHeight?: unknown;
      };
      runs?: unknown[];
    };
    const style = signature.style;
    return (
      signature.text === shape.characters &&
      style?.family === shape.fontFamily &&
      Number(style?.size) === Number(shape.fontSize) &&
      Number(style?.lineHeight) === Number(shape.lineHeight) &&
      (shape.fontWeight === String(style?.weight) ||
        (shape.fontWeight === 'mixed' &&
          Array.isArray(signature.runs) &&
          signature.runs.length > 0))
    );
  } catch {
    return false;
  }
}

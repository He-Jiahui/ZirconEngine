import type { Shape } from '@penpot/plugin-types';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_ASSET,
} from './metadata';

// Penpot rounds auto-width text to whole pixels. Measuring the same font at
// this scale bounds that rounding to 1/16 logical pixel without resizing any
// visible text. A missing scale belongs only to a validated legacy 1x probe.
export const LINEAR_CONTENT_MEASUREMENT_SCALE = 16;

/** Return the scale only for a source-signed, invisible intrinsic probe. */
export function currentContentMeasurementScale(shape: Shape): number | undefined {
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
    return undefined;
  const id = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'linear-content-measurement',
  );
  const encoded = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'linear-content-measurement-signature',
  );
  if (!id || !encoded) return undefined;
  try {
    const signature = JSON.parse(encoded) as {
      text?: unknown;
      measurementScale?: unknown;
      style?: {
        family?: unknown;
        size?: unknown;
        weight?: unknown;
        lineHeight?: unknown;
      };
      runs?: unknown[];
    };
    if (!signature || typeof signature !== 'object' || Array.isArray(signature))
      return undefined;
    const scaled = Object.hasOwn(signature, 'measurementScale');
    if (scaled && signature.measurementScale !== LINEAR_CONTENT_MEASUREMENT_SCALE)
      return undefined;
    const scale = scaled ? LINEAR_CONTENT_MEASUREMENT_SCALE : 1;
    const style = signature.style;
    if (
      signature.text === shape.characters &&
      typeof style?.family === 'string' &&
      style.family.trim().length > 0 &&
      style?.family === shape.fontFamily &&
      typeof style.size === 'number' &&
      Number.isFinite(style.size * scale) &&
      style.size > 0 &&
      style.size * scale === Number(shape.fontSize) &&
      typeof style.lineHeight === 'number' &&
      Number.isFinite(style.lineHeight) &&
      style.lineHeight > 0 &&
      // Penpot lineHeight is a unitless multiplier, not a pixel length.
      style.lineHeight === Number(shape.lineHeight) &&
      // applyPenpotTextStyle currently authors zero pixel letter spacing.
      // A nonzero/mixed edit must not become an intrinsic source measurement.
      typeof shape.letterSpacing === 'string' &&
      shape.letterSpacing.trim() !== '' &&
      Number(shape.letterSpacing) === 0 &&
      typeof style.weight === 'string' &&
      style.weight.length > 0 &&
      (shape.fontWeight === String(style?.weight) ||
        (shape.fontWeight === 'mixed' &&
          Array.isArray(signature.runs) &&
          signature.runs.length > 0))
    )
      return scale;
    return undefined;
  } catch {
    return undefined;
  }
}

/** These invisible probes are regenerated from current source typography. */
export function isCurrentContentMeasurement(shape: Shape): boolean {
  return currentContentMeasurementScale(shape) !== undefined;
}

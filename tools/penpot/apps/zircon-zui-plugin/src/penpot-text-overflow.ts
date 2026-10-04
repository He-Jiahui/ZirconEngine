import type { Board, Shape, Text } from '@penpot/plugin-types';
import { applyPenpotTextStyle } from './penpot-text-style';
import { waitForStableLayout } from './penpot-layout-settlement';
import { EMPTY_TEXT_SENTINEL } from './penpot-capture-validation';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_TEXT,
} from './metadata';

const OVERFLOW_SOURCE = 'single-line-source';
const OVERFLOW_DISPLAY = 'single-line-display';
export const SINGLE_LINE_OVERFLOW = 'single-line-overflow';
const MAX_NATIVE_TEXT_MEASUREMENTS = 8192;

// The official frontend performs a full layout pass for every temporary
// auto-width probe. Review cases at a different DPI often have identical
// logical geometry, so retain measurements for the current plugin session.
// The cache stores only font-aware native measurements; it never changes the
// authored text saved for export.
const nativeTextMeasurements = new Map<string, number>();

export function cachedTextMeasure(
  cache: Map<string, number>,
  styleKey: string,
  measure: (value: string) => Promise<number>,
): (value: string) => Promise<number> {
  return async (value: string) => {
    const key = JSON.stringify([styleKey, value]);
    const cached = cache.get(key);
    if (cached !== undefined) {
      // Touch the entry so bounded eviction keeps measurements reused by
      // consecutive review cases.
      cache.delete(key);
      cache.set(key, cached);
      return cached;
    }
    const result = await measure(value);
    if (!Number.isFinite(result) || result <= 0)
      throw new Error('Invalid native field text measurement');
    while (cache.size >= MAX_NATIVE_TEXT_MEASUREMENTS) {
      const oldest = cache.keys().next().value;
      if (oldest === undefined) break;
      cache.delete(oldest);
    }
    cache.set(key, result);
    return result;
  };
}

export async function measuredEllipsis(
  source: string,
  width: number,
  measure: (text: string) => Promise<number>,
): Promise<string> {
  // Empty native fields retain this invisible, editable Penpot placeholder.
  // Its intrinsic 1x1 box cannot satisfy a visible-glyph measurement probe.
  if (source === EMPTY_TEXT_SENTINEL) return source;
  if ((await measure(source)) <= width + 0.01) return source;
  const marker = '\u2026';
  if ((await measure(marker)) > width + 0.01)
    throw new Error('Native text field cannot fit its ellipsis marker');
  const segments = Array.from(
    new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(source),
    (part) => part.segment,
  );
  let low = 0,
    high = segments.length;
  while (low < high) {
    const middle = Math.ceil((low + high) / 2);
    if (
      (await measure(segments.slice(0, middle).join('') + marker)) <=
      width + 0.01
    )
      low = middle;
    else high = middle - 1;
  }
  return segments.slice(0, low).join('') + marker;
}

export function semanticTextCharacters(shape: Text): string {
  const display = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    OVERFLOW_DISPLAY,
  );
  const source = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    OVERFLOW_SOURCE,
  );
  if (display && shape.characters === display) return source;
  if (display && display !== source && shape.characters.includes('\u2026'))
    throw new Error(
      'Cannot edit an abbreviated field value; replace it with the complete value without the generated ellipsis.',
    );
  return shape.characters;
}

/** Fit using Penpot's selected font; preserve the full authored value for export. */
export async function fitNativeSingleLineTexts(asset: Board): Promise<void> {
  const texts: Text[] = [];
  const visit = (shape: Shape) => {
    if (
      shape.type === 'text' &&
      !shape.hidden &&
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) ===
        ZUI_ROLE_TEXT &&
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, SINGLE_LINE_OVERFLOW)
    )
      texts.push(shape);
    if ('children' in shape) for (const child of shape.children) visit(child);
  };
  visit(asset);
  for (const text of texts) {
    const source = semanticTextCharacters(text);
    if (!source.trim()) continue;
    if (/[\r\n\u2028\u2029]/u.test(source))
      throw new Error(
        'Multiline input requires an explicit native field projection',
      );
    let probe: Text | null = null;
    try {
      const measure = cachedTextMeasure(
        nativeTextMeasurements,
        textMeasurementStyleKey(text),
        async (value) => {
          if (!probe) {
            probe = penpot.createText(value);
            if (!probe)
              throw new Error('Cannot create native field font measurement');
            applyPenpotTextStyle(probe, {
              family: text.fontFamily,
              id: text.fontId,
              weight: text.fontWeight,
              size: Number(text.fontSize),
              lineHeight: Number(text.lineHeight),
            });
            probe.growType = 'auto-width';
            probe.opacity = 0;
            asset.appendChild(probe);
            if (probe.layoutChild) probe.layoutChild.absolute = true;
          }
          probe.characters = value;
          await waitForStableLayout(
            () => {},
            () => [probe!.width, probe!.height],
            {
              intervalMs: 40,
              quietMs: 160,
              timeoutMs: 10_000,
              // Penpot creates an auto-width text probe at a tiny placeholder
              // size and fills in font metrics on the next layout pass. Do not
              // accept that placeholder as a settled measurement; otherwise
              // large composite imports race the probe and fail with a false
              // "1 -> width/height" instability error.
              isReady: ([width, height]) => width > 1 && height > 1,
            },
          );
          return probe.width;
        },
      );
      const display = await measuredEllipsis(source, text.width, measure);
      text.setSharedPluginData(ZUI_METADATA_NAMESPACE, OVERFLOW_SOURCE, source);
      text.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        OVERFLOW_DISPLAY,
        display,
      );
      text.characters = display;
    } finally {
      // TypeScript cannot observe the assignment performed inside the async
      // measurement closure and narrows `probe` to `never` here. Keep the
      // runtime cleanup explicit while preserving the nullable API contract.
      (probe as Text | null)?.remove();
    }
  }
}

function textMeasurementStyleKey(text: Text): string {
  return JSON.stringify({
    id: text.fontId,
    family: text.fontFamily,
    variant: text.fontVariantId,
    weight: text.fontWeight,
    size: text.fontSize,
    lineHeight: text.lineHeight,
    letterSpacing: text.letterSpacing,
  });
}

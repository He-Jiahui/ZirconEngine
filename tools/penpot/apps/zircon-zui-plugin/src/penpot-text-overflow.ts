import type { Board, Shape, Text } from '@penpot/plugin-types';
import { applyPenpotTextStyle } from './penpot-text-style';
import { fontResourcesForShape } from './bridge/penpot-font-selection';
import { waitForStableLayout, type LayoutSettlementDiagnostic } from './penpot-layout-settlement';
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
  onCache?: (hit: boolean) => void,
): (value: string) => Promise<number> {
  return async (value: string) => {
    const key = JSON.stringify([styleKey, value]);
    const cached = cache.get(key);
    try {
      onCache?.(cached !== undefined);
    } catch {
      // Optional diagnostics cannot change native measurement behavior.
    }
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

/** A source overflow change must restore the full value before dropping its display mapping. */
export function setSingleLineTextOverflow(
  text: Text,
  overflow: 'ellipsis' | undefined,
): void {
  const previous = text.getSharedPluginData(ZUI_METADATA_NAMESPACE, SINGLE_LINE_OVERFLOW);
  if (previous && previous !== 'ellipsis')
    throw new Error('Unknown native single-line overflow mapping; re-import its source.');
  if (previous === 'ellipsis' && overflow === undefined) {
    const source = semanticTextCharacters(text);
    if (text.characters !== source) text.characters = source;
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, OVERFLOW_SOURCE, '');
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, OVERFLOW_DISPLAY, '');
  }
  const next = overflow ?? '';
  if (previous !== next)
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, SINGLE_LINE_OVERFLOW, next);
}

/** Fit using Penpot's selected font; preserve the full authored value for export. */
export interface NativeTextFitDiagnostic {
  event: string;
  elapsedMs: number;
  operationMs?: number;
  fieldOrdinal?: number;
  fieldCount?: number;
  shapeVisitCount?: number;
  measurementOrdinal?: number;
  cacheHit?: boolean;
  styleKeyDigest?: string;
  sampleCount?: number;
  sampleMs?: number;
  snapshotMs?: number;
  snapshotOrdinal?: number;
  longestSampleMs?: number;
  lastReady?: boolean;
  widthReadCount?: number;
  widthReadTotalMs?: number;
  widthReadMaxMs?: number;
  heightReadCount?: number;
  heightReadTotalMs?: number;
  heightReadMaxMs?: number;
}

/** Fit using Penpot's selected font; preserve the full authored value for export. */
export async function fitNativeSingleLineTexts(
  asset: Board,
  onDiagnostic?: (diagnostic: NativeTextFitDiagnostic) => void,
): Promise<void> {
  const fitStarted = onDiagnostic ? Date.now() : 0;
  const report = (
    event: string,
    details: Omit<NativeTextFitDiagnostic, 'event' | 'elapsedMs'> = {},
  ): void => {
    if (!onDiagnostic) return;
    try {
      onDiagnostic({ event, elapsedMs: Date.now() - fitStarted, ...details });
    } catch {
      // Optional diagnostics cannot change the fit result.
    }
  };
  const timed = <T>(
    event: string,
    operation: () => T,
    fieldOrdinal?: number,
    measurementOrdinal?: number,
    styleKeyDigest?: string,
  ): T => {
    if (!onDiagnostic) return operation();
    const started = Date.now();
    const details = {
      fieldOrdinal,
      fieldCount: texts.length,
      measurementOrdinal,
      styleKeyDigest,
    };
    report(event + '-started', details);
    try {
      const result = operation();
      report(event + '-completed', {
        ...details,
        operationMs: Date.now() - started,
      });
      return result;
    } catch (error) {
      report(event + '-failed', {
        ...details,
        operationMs: Date.now() - started,
      });
      throw error;
    }
  };

  const texts: Text[] = [];
  let shapeVisitCount = 0;
  report('shape-scan-started');
  const visit = (shape: Shape) => {
    if (onDiagnostic) {
      shapeVisitCount += 1;
      if (shapeVisitCount === 1 || shapeVisitCount % 16 === 0)
        report('shape-scan-progress', { shapeVisitCount });
    }
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
  report('shape-scan-completed', {
    shapeVisitCount,
    fieldCount: texts.length,
  });

  for (let index = 0; index < texts.length; index += 1) {
    const text = texts[index];
    const fieldOrdinal = index + 1;
    let probe: Text | null = null;
    let styleKeyDigest: string | undefined;
    let measurementOrdinal = 0;
    let widthReadCount = 0;
    let widthReadTotalMs = 0;
    let widthReadMaxMs = 0;
    let heightReadCount = 0;
    let heightReadTotalMs = 0;
    let heightReadMaxMs = 0;
    report('field-started', { fieldOrdinal, fieldCount: texts.length });
    try {
      const source = timed(
        'semantic-source-read',
        () => semanticTextCharacters(text),
        fieldOrdinal,
      );
      if (!source.trim()) {
        report('field-skipped-empty', {
          fieldOrdinal,
          fieldCount: texts.length,
        });
        continue;
      }
      if (/[\r\n\u2028\u2029]/u.test(source))
        throw new Error(
          'Multiline input requires an explicit native field projection',
        );
      const styleKey = timed(
        'style-key-read',
        () => textMeasurementStyleKey(text),
        fieldOrdinal,
      );
      styleKeyDigest = nativeTextStyleDigest(styleKey);
      const cachedMeasure = cachedTextMeasure(
        nativeTextMeasurements,
        styleKey,
        async (value) => {
          if (!probe) {
            probe = timed(
              'probe-create',
              () => penpot.createText(value),
              fieldOrdinal,
              measurementOrdinal,
              styleKeyDigest,
            );
            if (!probe)
              throw new Error('Cannot create native field font measurement');
            const fontResources = timed(
              'probe-font-resource-lookup',
              () => fontResourcesForShape(text),
              fieldOrdinal,
              measurementOrdinal,
              styleKeyDigest,
            );
            timed(
              'probe-font-style-apply',
              () =>
                applyPenpotTextStyle(probe!, {
                  family: text.fontFamily,
                  id: text.fontId,
                  fontResources,
                  weight: text.fontWeight,
                  size: Number(text.fontSize),
                  lineHeight: Number(text.lineHeight),
                }),
              fieldOrdinal,
              measurementOrdinal,
              styleKeyDigest,
            );
            timed(
              'probe-grow-type-write',
              () => {
                probe!.growType = 'auto-width';
              },
              fieldOrdinal,
              measurementOrdinal,
              styleKeyDigest,
            );
            timed(
              'probe-opacity-write',
              () => {
                probe!.opacity = 0;
              },
              fieldOrdinal,
              measurementOrdinal,
              styleKeyDigest,
            );
            timed(
              'probe-append',
              () => asset.appendChild(probe!),
              fieldOrdinal,
              measurementOrdinal,
              styleKeyDigest,
            );
            if (
              timed(
                'probe-layout-child-read',
                () => probe!.layoutChild,
                fieldOrdinal,
                measurementOrdinal,
                styleKeyDigest,
              )
            )
              timed(
                'probe-layout-absolute-write',
                () => {
                  probe!.layoutChild!.absolute = true;
                },
                fieldOrdinal,
                measurementOrdinal,
                styleKeyDigest,
              );
          }
          timed(
            'probe-characters-write',
            () => {
              probe!.characters = value;
            },
            fieldOrdinal,
            measurementOrdinal,
            styleKeyDigest,
          );
          const settleStarted = onDiagnostic ? Date.now() : 0;
          report('probe-settle-started', {
            fieldOrdinal,
            fieldCount: texts.length,
            measurementOrdinal,
            styleKeyDigest,
          });
          try {
            await waitForStableLayout(
              () => {},
              () => {
                if (!onDiagnostic)
                  return [probe!.width, probe!.height] as [number, number];
                const snapshotOrdinal = Math.max(
                  widthReadCount,
                  heightReadCount,
                ) + 1;
                const widthStarted = Date.now();
                const widthOrdinal = ++widthReadCount;
                if (widthOrdinal <= 8 || widthOrdinal % 50 === 0)
                  report('probe-width-read-started', {
                  fieldOrdinal,
                  fieldCount: texts.length,
                  measurementOrdinal,
                  styleKeyDigest,
                  snapshotOrdinal,
                  widthReadCount,
                });
                const width = probe!.width;
                const widthMs = Date.now() - widthStarted;
                widthReadTotalMs += widthMs;
                widthReadMaxMs = Math.max(widthReadMaxMs, widthMs);
                if (widthOrdinal <= 8 || widthOrdinal % 50 === 0)
                  report('probe-width-read-completed', {
                    fieldOrdinal,
                    fieldCount: texts.length,
                    measurementOrdinal,
                    styleKeyDigest,
                    snapshotOrdinal,
                    operationMs: widthMs,
                    widthReadCount,
                    widthReadTotalMs,
                    widthReadMaxMs,
                  });

                const heightStarted = Date.now();
                const heightOrdinal = ++heightReadCount;
                if (heightOrdinal <= 8 || heightOrdinal % 50 === 0)
                  report('probe-height-read-started', {
                  fieldOrdinal,
                  fieldCount: texts.length,
                  measurementOrdinal,
                  styleKeyDigest,
                  snapshotOrdinal,
                  heightReadCount,
                });
                const height = probe!.height;
                const heightMs = Date.now() - heightStarted;
                heightReadTotalMs += heightMs;
                heightReadMaxMs = Math.max(heightReadMaxMs, heightMs);
                if (heightOrdinal <= 8 || heightOrdinal % 50 === 0)
                  report('probe-height-read-completed', {
                    fieldOrdinal,
                    fieldCount: texts.length,
                    measurementOrdinal,
                    styleKeyDigest,
                    snapshotOrdinal,
                    operationMs: heightMs,
                    heightReadCount,
                    heightReadTotalMs,
                    heightReadMaxMs,
                  });
                if (snapshotOrdinal <= 8 || snapshotOrdinal % 50 === 0)
                  report('probe-snapshot-completed', {
                    fieldOrdinal,
                    fieldCount: texts.length,
                    measurementOrdinal,
                    styleKeyDigest,
                    snapshotOrdinal,
                    operationMs: Date.now() - widthStarted,
                    widthReadCount,
                    widthReadTotalMs,
                    widthReadMaxMs,
                    heightReadCount,
                    heightReadTotalMs,
                    heightReadMaxMs,
                    lastReady: width > 1 && height > 1,
                  });
                return [width, height];
              },
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
                onDiagnostic: (diagnostic: LayoutSettlementDiagnostic) =>
                  report('probe-settle-' + diagnostic.event, {
                    fieldOrdinal,
                    fieldCount: texts.length,
                    measurementOrdinal,
                    styleKeyDigest,
                    sampleCount: diagnostic.sampleCount,
                    sampleMs: diagnostic.sampleMs,
                    snapshotMs: diagnostic.snapshotMs,
                    longestSampleMs: diagnostic.longestSampleMs,
                    lastReady: diagnostic.lastReady,
                    widthReadCount,
                    widthReadTotalMs,
                    widthReadMaxMs,
                    heightReadCount,
                    heightReadTotalMs,
                    heightReadMaxMs,
                  }),
              },
            );
            report('probe-settle-completed', {
              fieldOrdinal,
              fieldCount: texts.length,
              measurementOrdinal,
              styleKeyDigest,
              operationMs: onDiagnostic
                ? Date.now() - settleStarted
                : undefined,
              widthReadCount,
              widthReadTotalMs,
              widthReadMaxMs,
              heightReadCount,
              heightReadTotalMs,
              heightReadMaxMs,
            });
          } catch (error) {
            report('probe-settle-failed', {
              fieldOrdinal,
              fieldCount: texts.length,
              measurementOrdinal,
              styleKeyDigest,
              operationMs: onDiagnostic
                ? Date.now() - settleStarted
                : undefined,
              widthReadCount,
              widthReadTotalMs,
              widthReadMaxMs,
              heightReadCount,
              heightReadTotalMs,
              heightReadMaxMs,
            });
            throw error;
          }
          return timed(
            'probe-final-width-read',
            () => probe!.width,
            fieldOrdinal,
            measurementOrdinal,
            styleKeyDigest,
          );
        },
        (cacheHit) => {
          measurementOrdinal += 1;
          report(cacheHit ? 'measurement-cache-hit' : 'measurement-cache-miss', {
            fieldOrdinal,
            fieldCount: texts.length,
            measurementOrdinal,
            cacheHit,
            styleKeyDigest,
          });
        },
      );
      const measure = async (value: string): Promise<number> => {
        const requestedOrdinal = measurementOrdinal + 1;
        report('measurement-call-started', {
          fieldOrdinal,
          fieldCount: texts.length,
          measurementOrdinal: requestedOrdinal,
          styleKeyDigest,
        });
        try {
          const width = await cachedMeasure(value);
          report('measurement-call-completed', {
            fieldOrdinal,
            fieldCount: texts.length,
            measurementOrdinal,
            styleKeyDigest,
          });
          return width;
        } catch (error) {
          report('measurement-call-failed', {
            fieldOrdinal,
            fieldCount: texts.length,
            measurementOrdinal,
            styleKeyDigest,
          });
          throw error;
        }
      };
      const targetWidth = timed(
        'target-width-read',
        () => text.width,
        fieldOrdinal,
        undefined,
        styleKeyDigest,
      );
      const display = await measuredEllipsis(source, targetWidth, measure);
      timed(
        'source-metadata-write',
        () =>
          text.setSharedPluginData(
            ZUI_METADATA_NAMESPACE,
            OVERFLOW_SOURCE,
            source,
          ),
        fieldOrdinal,
        undefined,
        styleKeyDigest,
      );
      timed(
        'display-metadata-write',
        () =>
          text.setSharedPluginData(
            ZUI_METADATA_NAMESPACE,
            OVERFLOW_DISPLAY,
            display,
          ),
        fieldOrdinal,
        undefined,
        styleKeyDigest,
      );
      timed(
        'display-characters-write',
        () => {
          text.characters = display;
        },
        fieldOrdinal,
        undefined,
        styleKeyDigest,
      );
      report('field-completed', {
        fieldOrdinal,
        fieldCount: texts.length,
        measurementOrdinal,
        styleKeyDigest,
      });
    } catch (error) {
      report('field-failed', {
        fieldOrdinal,
        fieldCount: texts.length,
        measurementOrdinal,
        styleKeyDigest,
      });
      throw error;
    } finally {
      if (probe)
        timed(
          'probe-remove',
          () => probe!.remove(),
          fieldOrdinal,
          measurementOrdinal,
          styleKeyDigest,
        );
    }
  }
}

function nativeTextStyleDigest(styleKey: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < styleKey.length; index += 1)
    hash = Math.imul(hash ^ styleKey.charCodeAt(index), 0x01000193);
  return (hash >>> 0).toString(16).padStart(8, '0');
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

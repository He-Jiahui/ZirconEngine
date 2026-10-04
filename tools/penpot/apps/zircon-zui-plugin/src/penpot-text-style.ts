import type { Font, FontVariant, Text } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import type {
  ProjectionText,
  ProjectionTextRun,
} from './bridge/penpot-projection-model';
import { resolveZuiTextStyle } from './bridge/zui-text-style';

export interface PenpotTextStyle {
  family: string;
  id: string;
  size: number;
  weight: string;
  lineHeight: number;
}

export function resolvePenpotTextStyle(
  document: ZuiDocument,
  node: ZuiNode,
  projected?: ProjectionText | null,
): PenpotTextStyle {
  const resolved = projected
    ? {
        family: projected.fontFamily,
        size: projected.fontSize ?? 14,
        weight:
          projected.fontWeight === 'regular'
            ? '400'
            : (projected.fontWeight ?? '400'),
        lineHeight: projected.lineHeight,
      }
    : resolveZuiTextStyle(document, node);
  const ids: Record<string, string> = {
    'Fira Sans': 'firasans',
    'Fira Mono': 'firamono',
    'Noto Sans SC': 'notosanssc',
    'Source Sans Pro': 'sourcesanspro',
    'Work Sans': 'worksans',
  };
  return { ...resolved, id: ids[resolved.family] ?? resolved.family };
}

export function applyPenpotTextStyle(text: Text, style: PenpotTextStyle): void {
  const font = exactFont(style.family);
  const variant = normalVariant(font, style.weight);
  font.applyToText(text, variant);
  text.fontSize = String(style.size);
  text.lineHeight = String(style.lineHeight);
  text.letterSpacing = '0';
}

/** Apply inline emphasis to ranges of one existing Penpot text shape. */
export function applyPenpotTextRuns(
  text: Text,
  style: PenpotTextStyle,
  runs: readonly ProjectionTextRun[] | undefined,
): void {
  if (!runs?.length) return;
  const font = exactFont(style.family);
  for (const run of runs) {
    if (
      !Number.isInteger(run.start) ||
      !Number.isInteger(run.end) ||
      run.start < 0 ||
      run.end <= run.start ||
      run.end > text.characters.length
    ) {
      throw new Error(
        `Invalid inline text range ${run.start}-${run.end} for ${text.name || 'Penpot text'}`,
      );
    }
    const variant = closestNormalVariant(font, run.fontWeight);
    font.applyToRange(text.getRange(run.start, run.end), variant);
  }
}

function exactFont(family: string): Font {
  // Penpot's findByName/findById use substring matching, including condensed families.
  const fonts = penpot.fonts.all.filter((font) => font.fontFamily === family);
  if (fonts.length !== 1) {
    throw new Error(
      `Expected one exact Penpot font family ${family}, found ${fonts.length}`,
    );
  }
  return fonts[0];
}

function normalVariant(font: Font, weight: string): FontVariant {
  const variant = font.variants.find(
    (candidate) =>
      candidate.fontWeight === normalizeWeight(weight) &&
      candidate.fontStyle === 'normal',
  );
  if (!variant)
    throw new Error(
      `Penpot font ${font.fontFamily} has no normal weight ${weight}`,
    );
  return variant;
}

function closestNormalVariant(font: Font, weight: string): FontVariant {
  const requested = normalizeWeight(weight);
  const exact = font.variants.find(
    (candidate) =>
      candidate.fontWeight === requested && candidate.fontStyle === 'normal',
  );
  if (exact) return exact;

  const numeric = Number(requested);
  const candidates = font.variants.filter(
    (candidate) => candidate.fontStyle === 'normal',
  );
  if (Number.isFinite(numeric)) {
    candidates.sort((left, right) => {
      const distance =
        Math.abs(Number(left.fontWeight) - numeric) -
        Math.abs(Number(right.fontWeight) - numeric);
      return distance || Number(right.fontWeight) - Number(left.fontWeight);
    });
  }
  const fallback = candidates[0];
  if (!fallback)
    throw new Error(
      `Penpot font ${font.fontFamily} has no normal variant for inline weight ${weight}`,
    );
  return fallback;
}

function normalizeWeight(weight: string): string {
  return weight === 'regular' ? '400' : weight;
}

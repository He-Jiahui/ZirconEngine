import type { ProjectionTextRun } from './bridge/penpot-projection-model';

/**
 * The only inline format currently projected by the ZUI bridge.
 *
 * Keeping the format name explicit is intentional: authored text remains a
 * normal, contiguous string, while the renderer can attach semantic ranges
 * without inventing additional text blocks.
 */
export const MARKDOWN_INLINE_V1 = 'markdown_inline_v1';

export interface ParsedProjectionText {
  characters: string;
  runs: ProjectionTextRun[];
}

/**
 * Parse the small inline subset used by the retained UI fixtures.
 *
 * `**strong**` markers are removed from the displayed string and represented
 * as a range in that same string. Unsupported formats and unmatched markers
 * are preserved verbatim so source text is never silently rewritten.
 */
export function parseProjectionText(
  source: string,
  format: unknown,
): ParsedProjectionText {
  if (format !== MARKDOWN_INLINE_V1 || !source.includes('**'))
    return { characters: source, runs: [] };

  const characters: string[] = [];
  const runs: ProjectionTextRun[] = [];
  let cursor = 0;
  let outputLength = 0;

  const append = (value: string): void => {
    if (!value) return;
    characters.push(value);
    outputLength += value.length;
  };

  while (cursor < source.length) {
    const opening = unescapedMarker(source, cursor);
    if (opening < 0) {
      append(source.slice(cursor));
      break;
    }

    append(source.slice(cursor, opening));
    const closing = unescapedMarker(source, opening + 2);
    if (closing < 0 || closing === opening + 2) {
      // A malformed or empty strong span is ordinary authored text.
      append(source.slice(opening));
      break;
    }

    const start = outputLength;
    append(source.slice(opening + 2, closing));
    const end = outputLength;
    if (end > start) runs.push({ start, end, fontWeight: '700' });
    cursor = closing + 2;
  }

  return { characters: characters.join(''), runs };
}

function unescapedMarker(source: string, from: number): number {
  let candidate = source.indexOf('**', from);
  while (candidate >= 0) {
    let backslashes = 0;
    for (
      let index = candidate - 1;
      index >= 0 && source[index] === '\\';
      index--
    )
      backslashes++;
    if (backslashes % 2 === 0) return candidate;
    candidate = source.indexOf('**', candidate + 2);
  }
  return -1;
}

import type { ZuiDocument, ZuiNode, ZuiTable } from './zui-document';
import { ZuiDocumentError } from './zui-document';
import { resolveDesignNumber } from './zui-prefab-system';

export interface ZuiTextStyle {
  family: string;
  size: number;
  weight: string;
  lineHeight: number;
}

/** Resolve an effective node after the source stylesheet and state cascade. */
export function resolveZuiTextStyle(
  document: ZuiDocument,
  node: ZuiNode,
  defaults: { size?: number; weight?: string } = {},
): ZuiTextStyle {
  const props = node.props ?? {};
  const font = asTable(props['font']);
  const code = props['component_variant'] === 'code';
  const family =
    stringValue(document, font['family'] ?? props['font_family']) ??
    stringValue(
      document,
      document.tokens?.[
        code ? 'editor.typography.code.family' : 'editor.typography.ui.family'
      ],
    ) ??
    (code ? 'Fira Mono' : 'Fira Sans');
  const size =
    numberValue(document, font['size'] ?? props['font_size']) ??
    defaults.size ??
    numberValue(document, document.tokens?.['editor.typography.body.size']) ??
    14;
  const rawWeight =
    font['weight'] ?? props['font_weight'] ?? props['text_font_weight'];
  const weight =
    rawWeight === 'regular'
      ? '400'
      : String(numberValue(document, rawWeight) ?? defaults.weight ?? '400');
  const absolute = numberValue(
    document,
    font['line_height'] ?? props['line_height'],
  );
  const ratio = numberValue(
    document,
    font['line_height_ratio'] ?? props['line_height_ratio'],
  );
  const lineHeight =
    absolute !== undefined
      ? absolute / size
      : (ratio ??
        numberValue(
          document,
          document.tokens?.['editor.typography.line_height'],
        ) ??
        1.4);
  if (size <= 0 || lineHeight <= 0 || !family.trim())
    throw new ZuiDocumentError('Unsupported nonpositive or empty text style.');
  return { family, size, weight, lineHeight };
}

function asTable(value: unknown): ZuiTable {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : {};
}

function resolvedToken(document: ZuiDocument, value: unknown): unknown {
  const visited = new Set<string>();
  while (typeof value === 'string' && value.startsWith('$')) {
    if (visited.has(value))
      throw new ZuiDocumentError(`Cyclic font token: ${value}`);
    visited.add(value);
    if (!Object.hasOwn(document.tokens ?? {}, value.slice(1)))
      throw new ZuiDocumentError(`Missing font token: ${value}`);
    value = document.tokens![value.slice(1)];
  }
  return value;
}

function stringValue(
  document: ZuiDocument,
  value: unknown,
): string | undefined {
  if (value === undefined) return undefined;
  const resolved = resolvedToken(document, value);
  if (typeof resolved !== 'string')
    throw new ZuiDocumentError('Unsupported nonstring font family.');
  return resolved;
}

function numberValue(
  document: ZuiDocument,
  value: unknown,
): number | undefined {
  if (value === undefined) return undefined;
  const resolved = resolveDesignNumber(
    document,
    resolvedToken(document, value),
  );
  if (resolved === null)
    throw new ZuiDocumentError(
      `Unsupported numeric text style: ${String(value)}`,
    );
  return resolved;
}

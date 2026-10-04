import type { Board, Shape, Text } from '@penpot/plugin-types';
import { semanticTextCharacters } from './penpot-text-overflow';
import {
  assertControlParts,
  assertControlTextGeometry,
} from './penpot-control-guard';
import type {
  ProjectionEditableState,
  ProjectionText,
} from './bridge/penpot-projection-model';
import {
  capturedMappedTextStyle,
  capturedFontTypography,
  capturedSemanticText,
  singleSolidFill,
} from './penpot-capture-validation';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_EMPTY_TEXT,
  ZUI_METADATA_TEXT_PROPERTY,
  ZUI_METADATA_TEXT_BASE_WEIGHT,
  ZUI_METADATA_TEXT_STYLE_GUARD,
  ZUI_ROLE_TEXT,
} from './metadata';

export const TEXT_FRAGMENT_KEY = 'text-fragment';

export function unsupportedTextStyle(text: Text): unknown {
  return {
    fontStyle: text.fontStyle,
    letterSpacing: text.letterSpacing,
    textTransform: text.textTransform,
    direction: text.direction,
    textDecoration: text.textDecoration,
    verticalAlign: text.verticalAlign,
    strokes: text.strokes,
  };
}

export function captureNodeTexts(
  board: Board,
  baseline?: ProjectionEditableState,
): Pick<ProjectionEditableState, 'text' | 'textFragments'> {
  const nodeId = metadata(board, ZUI_METADATA_NODE_ID) || board.name;
  assertControlParts(board);
  const candidates = board.children.filter(
    (shape) => metadata(shape, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT,
  );
  const captured = new Map<string, ProjectionText>();
  for (const candidate of candidates) {
    assertControlTextGeometry(board, candidate);
    const key = metadata(candidate, TEXT_FRAGMENT_KEY) || 'primary';
    if (captured.has(key))
      throw new Error(
        `Semantic node ${nodeId} contains more than one direct editable ZUI text shape for ${key}.`,
      );
    if (candidate.type !== 'text')
      throw new Error(
        `Editable ZUI text for node ${nodeId} is not a text shape.`,
      );
    if (metadata(candidate, ZUI_METADATA_NODE_ID) !== nodeId)
      throw new Error(
        `Editable ZUI text for node ${nodeId} belongs to a different semantic node.`,
      );
    const textFill = singleSolidFill(
      candidate.fills,
      `Editable text for node ${nodeId}`,
    );
    const expected = metadata(candidate, ZUI_METADATA_TEXT_STYLE_GUARD);
    const currentUnsupportedStyle = JSON.stringify(
      unsupportedTextStyle(candidate),
    );
    if (!expected || expected !== currentUnsupportedStyle)
      throw new Error(
        `Editable text for node ${nodeId} changed a text style outside the .zui bridge profile; restore it or re-import the asset. Expected ${expected || '<missing>'}, found ${currentUnsupportedStyle}.`,
      );
    const property = metadata(candidate, ZUI_METADATA_TEXT_PROPERTY);
    if (
      ![
        '',
        'text',
        'value_text',
        'query',
        'value',
        'placeholder',
        'title',
        'message',
        'label',
        'label_text',
        'group_label',
        'options',
      ].includes(property)
    )
      throw new Error(
        `Editable text for node ${nodeId} has unknown property metadata ${property}.`,
      );
    const capturedWeight =
      candidate.fontWeight === 'mixed'
        ? metadata(candidate, ZUI_METADATA_TEXT_BASE_WEIGHT)
        : candidate.fontWeight;
    captured.set(key, {
      characters: capturedSemanticText(
        semanticTextCharacters(candidate),
        metadata(candidate, ZUI_METADATA_EMPTY_TEXT) === 'true',
      ),
      property: property ? (property as ProjectionText['property']) : null,
      color: textFill?.fillColor ?? null,
      colorOpacity: textFill?.fillOpacity ?? 1,
      ...capturedMappedTextStyle(
        candidate.fontSize,
        capturedWeight,
        candidate.align,
        `Editable text for node ${nodeId}`,
      ),
      ...capturedFontTypography(
        candidate.fontFamily,
        candidate.lineHeight,
        `Editable text for node ${nodeId}`,
      ),
    });
  }
  if (baseline) {
    const keys = new Set([
      ...(baseline.text ? ['primary'] : []),
      ...Object.keys(baseline.textFragments ?? {}),
    ]);
    if (
      candidates.length !== keys.size ||
      [...captured.keys()].some((key) => !keys.has(key))
    )
      throw new Error(
        `Semantic node ${nodeId} gained or lost a direct editable ZUI text shape or fragment.`,
      );
  }
  const text = captured.get('primary') ?? null;
  captured.delete('primary');
  return {
    text,
    ...(captured.size ? { textFragments: Object.fromEntries(captured) } : {}),
  };
}

function metadata(shape: Shape, key: string): string {
  return shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, key);
}

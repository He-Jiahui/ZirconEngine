import type { Board, Text } from '@penpot/plugin-types';
import type {
  ProjectedZuiNode,
  ProjectionText,
} from './bridge/penpot-projection-model';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { PENPOT_PALETTE, type ZuiPrefabRole } from './bridge/zui-prefab-system';
import { inputControlGeometry } from './bridge/zui-input-projection';
import { EMPTY_TEXT_SENTINEL } from './penpot-capture-validation';
import {
  applyPenpotTextRuns,
  applyPenpotTextStyle,
  resolvePenpotTextStyle,
} from './penpot-text-style';
import { TEXT_FRAGMENT_KEY, unsupportedTextStyle } from './penpot-text-capture';
import { layoutSliderText } from './penpot-slider-controls';
import { layoutSegmentedText } from './penpot-segmented-controls';
import { layoutPropertyRowText } from './penpot-property-row-controls';
import { layoutFieldText } from './penpot-field-controls';
import { SINGLE_LINE_OVERFLOW } from './penpot-text-overflow';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_TEXT_PROPERTY,
  ZUI_METADATA_EMPTY_TEXT,
  ZUI_METADATA_TEXT_BASE_WEIGHT,
  ZUI_METADATA_TEXT_STYLE_GUARD,
  ZUI_ROLE_TEXT,
} from './metadata';

export function createSemanticTexts(
  board: Board,
  projection: ProjectedZuiNode,
  document: ZuiDocument,
  node: ZuiNode,
): void {
  const entries: Array<[string, ProjectionText]> = [
    ...(projection.text
      ? [['primary', projection.text] as [string, ProjectionText]]
      : []),
    ...Object.entries(projection.textFragments ?? {}),
  ];
  for (const [key, projected] of entries) {
    const empty = projected.characters === '';
    const text = penpot.createText(
      empty ? EMPTY_TEXT_SENTINEL : projected.characters,
    );
    if (!text)
      throw new Error(
        `Penpot could not create the editable text for node ${projection.nodeId}`,
      );
    const inset = textInsets(projection.prefabRole, board.width);
    if (
      board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'has-leading-icon') ===
      'true'
    )
      inset.left = Math.max(inset.left, 36);
    if (
      board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'has-trailing-icon') ===
      'true'
    )
      inset.right = Math.max(inset.right, 36);
    const style = resolvePenpotTextStyle(document, node, projected);
    const height = Math.max(
      1,
      Math.min(
        board.height,
        style.size *
          style.lineHeight *
          Math.max(1, projected.characters.split('\n').length),
      ),
    );
    text.name =
      key === 'primary'
        ? `ZUI text · ${projection.nodeId}`
        : `ZUI text ${key} · ${projection.nodeId}`;
    text.growType = 'fixed';
    applyPenpotTextStyle(text, style);
    setMetadata(text, ZUI_METADATA_TEXT_BASE_WEIGHT, style.weight);
    applyPenpotTextRuns(text, style, projected.richTextRuns);
    text.align = projected.align ?? 'left';
    text.verticalAlign = 'center';
    text.letterSpacing = '0';
    text.fills = [
      {
        fillColor: projected.color ?? PENPOT_PALETTE.text,
        fillOpacity: projected.colorOpacity,
      },
    ];
    text.resize(Math.max(1, board.width - inset.left - inset.right), height);
    board.appendChild(text);
    text.x = board.x + inset.left;
    text.y = board.y + Math.max(0, (board.height - height) / 2);
    if (projection.inputControl) {
      const geometry = inputControlGeometry(
        projection.inputControl,
        board.width,
        board.height,
      ).text;
      text.resize(geometry.width, geometry.height);
      text.x = board.x + geometry.x;
      text.y = board.y + geometry.y;
      text.verticalAlign = 'top';
    }
    // Editor fields and other native control projections use an explicit
    // text lane whose top edge is authored by the control geometry.  Record
    // that bridge-owned vertical alignment in the style guard before the
    // first export; otherwise the refresh pass looks like an out-of-band
    // Penpot edit.
    if (
      projection.field ||
      projection.propertyRow ||
      projection.slider ||
      projection.segmented
    )
      text.verticalAlign = 'top';
    setMetadata(text, ZUI_METADATA_ROLE, ZUI_ROLE_TEXT);
    setMetadata(text, ZUI_METADATA_NODE_ID, projection.nodeId);
    setMetadata(text, ZUI_METADATA_TEXT_PROPERTY, projected.property ?? '');
    setMetadata(text, ZUI_METADATA_EMPTY_TEXT, empty ? 'true' : 'false');
    if (key !== 'primary') setMetadata(text, TEXT_FRAGMENT_KEY, key);
    if (text.layoutChild) text.layoutChild.absolute = true;
    if (projection.slider) layoutSliderText(board, text, projection.slider);
    if (projection.field) {
      setMetadata(text, SINGLE_LINE_OVERFLOW, 'ellipsis');
      layoutFieldText(board, text, projection.field);
    }
    if (projection.segmented)
      layoutSegmentedText(board, text, projection.segmented);
    if (projection.table) text.verticalAlign = 'top';
    if (projection.propertyRow)
      layoutPropertyRowText(board, text, projection.propertyRow);
    // Table rows depend on their parent flex width. Their first stable width
    // exists only after the complete asset tree has been created and laid out;
    // refreshAssetTextLayout applies the table projection in that phase.
    setMetadata(
      text,
      ZUI_METADATA_TEXT_STYLE_GUARD,
      JSON.stringify(unsupportedTextStyle(text)),
    );
  }
}

export function textInsets(
  role: ZuiPrefabRole,
  width: number,
): { left: number; right: number; top: number } {
  if (role === 'toggle')
    return width < 80
      ? { left: 20, right: 6, top: 4 }
      : { left: 28, right: 10, top: 4 };
  if (['button', 'tab', 'chip', 'badge'].includes(role)) {
    const inset = width < 64 ? 6 : 10;
    return { left: inset, right: inset, top: 4 };
  }
  if (role === 'field' || role === 'row')
    return { left: 10, right: 10, top: 4 };
  if (role === 'icon-button') return { left: 4, right: 4, top: 4 };
  return { left: 0, right: 0, top: 0 };
}

function setMetadata(text: Text, key: string, value: string): void {
  text.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
}

import type { Board, Shape } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { PENPOT_PALETTE } from './bridge/zui-prefab-system';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
} from './metadata';
import { applyPenpotTextStyle } from './penpot-text-style';

export function tokenOverviewSize(document: ZuiDocument): {
  width: number;
  height: number;
} {
  const tokenCount = Object.entries(document.tokens ?? {}).filter(
    ([key, value]) =>
      !key.startsWith('zr.penpot.') &&
      typeof value === 'string' &&
      value.trim() !== '',
  ).length;
  return {
    width: 568,
    height: Math.max(176, 88 + Math.min(24, tokenCount) * 32),
  };
}

export function createTokenOverview(
  assetBoard: Board,
  document: ZuiDocument,
): void {
  const tokenRows = Object.entries(document.tokens ?? {})
    .filter(
      (entry): entry is [string, string] =>
        !entry[0].startsWith('zr.penpot.') &&
        typeof entry[1] === 'string' &&
        entry[1].trim() !== '',
    )
    .slice(0, 24);
  const overviewHeight = tokenOverviewSize(document).height - 48;
  const overview = penpot.createBoard();
  overview.name = 'Penpot · tokens and styles';
  overview.resize(520, overviewHeight);
  overview.x = assetBoard.x + 24;
  overview.y = assetBoard.y + 24;
  overview.fills = [{ fillColor: PENPOT_PALETTE.surface, fillOpacity: 1 }];
  overview.strokes = [
    {
      strokeColor: PENPOT_PALETTE.border,
      strokeWidth: 1,
      strokeStyle: 'solid',
      strokeAlignment: 'inner',
    },
  ];
  overview.borderRadius = 8;
  setMetadata(overview, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
  assetBoard.appendChild(overview);

  createOverviewText(
    overview,
    document.asset.display_name?.trim() || document.asset.id,
    'Penpot · style title',
    24,
    20,
    18,
    '600',
    PENPOT_PALETTE.text,
  );
  createOverviewText(
    overview,
    tokenRows.length > 0
      ? `${tokenRows.length} token values · editable swatches`
      : 'Stylesheet rules are preserved as metadata',
    'Penpot · style summary',
    24,
    50,
    11,
    '400',
    PENPOT_PALETTE.textMuted,
  );

  tokenRows.forEach(([token, value], index) => {
    const row = penpot.createBoard();
    row.name = `Token · ${token}`;
    row.resize(472, 24);
    row.x = overview.x + 24;
    row.y = overview.y + 80 + index * 32;
    row.fills = [{ fillColor: PENPOT_PALETTE.surfaceInset, fillOpacity: 1 }];
    row.borderRadius = 4;
    setMetadata(row, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
    overview.appendChild(row);

    const swatch = penpot.createBoard();
    swatch.name = `Swatch · ${token}`;
    swatch.resize(24, 24);
    swatch.x = row.x;
    swatch.y = row.y;
    const color = literalColor(value);
    swatch.fills = color
      ? [{ fillColor: color.color, fillOpacity: color.opacity }]
      : [{ fillColor: PENPOT_PALETTE.borderStrong, fillOpacity: 1 }];
    swatch.strokes = [
      {
        strokeColor: PENPOT_PALETTE.textSubtle,
        strokeWidth: 1,
        strokeStyle: 'solid',
        strokeAlignment: 'inner',
      },
    ];
    swatch.borderRadius = 4;
    setMetadata(swatch, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
    row.appendChild(swatch);

    createOverviewText(
      row,
      `${token}  ${String(value)}`,
      `Token label · ${token}`,
      36,
      4,
      11,
      '400',
      PENPOT_PALETTE.text,
      432,
    );
  });

}

function createOverviewText(
  parent: Board,
  characters: string,
  name: string,
  x: number,
  y: number,
  fontSize: number,
  fontWeight: string,
  color: string,
  width = 472,
): void {
  const text = penpot.createText(characters);
  if (!text) return;
  text.name = name;
  text.growType = 'fixed';
  applyPenpotTextStyle(text, {
    family: 'Fira Sans',
    id: 'firasans',
    size: fontSize,
    weight: fontWeight,
    lineHeight: 1.4,
  });
  text.fills = [{ fillColor: color, fillOpacity: 1 }];
  text.resize(width, Math.max(16, fontSize * 1.45));
  parent.appendChild(text);
  text.x = parent.x + x;
  text.y = parent.y + y;
  if (text.layoutChild) text.layoutChild.absolute = true;
  setMetadata(text, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
}

function literalColor(
  value: string,
): { color: string; opacity: number } | null {
  const trimmed = value.trim();
  if (trimmed.toLowerCase() === 'transparent') {
    return { color: '#000000', opacity: 0 };
  }
  const match = trimmed.match(/^#([0-9a-fA-F]{3,4}|[0-9a-fA-F]{6,8})$/);
  if (!match) return null;
  const digits =
    match[1].length <= 4
      ? [...match[1]].map((digit) => `${digit}${digit}`).join('')
      : match[1];
  return {
    color: `#${digits.slice(0, 6)}`,
    opacity:
      digits.length === 8 ? Number.parseInt(digits.slice(6), 16) / 255 : 1,
  };
}

function setMetadata(shape: Shape, key: string, value: string): void {
  shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
}

import type { Fill, LibraryColor, Stroke } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { styledPreviewNode } from './bridge/zui-style-projection';
import { authoredPaintPreviewNode } from './bridge/zui-paint-properties';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { singleSolidFill, singleSolidStroke } from './penpot-capture-validation';

type ColorLibrary = { readonly id: string; readonly colors: readonly LibraryColor[] };
type SourceColor = { color: string; opacity: number };
type ColorSource = {
  schema: 'zircon.penpot-token-source.v1'; version: 1;
  sourcePath: string; sourceSha256: string; token: string;
  sourceTable: string; sourceKey: string; sourceIndex?: number;
  originalValue: string; penpotStyleId: string;
};

const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);

export type SourceColorLookup = (nodeId: string,
  property: 'background_color' | 'border_color' | 'foreground_color',
  textProperty?: string | null) => unknown;

/** Per-operation index: documents are editable, so no cross-operation cache. */
export function sourceColorProperties(document: ZuiDocument): SourceColorLookup {
  const parents = new Map<string, string>();
  for (const [id, node] of Object.entries(document.nodes ?? {})) for (const child of node.children ?? []) {
    if (parents.has(child.node)) throw new Error(`Ambiguous color source parent: ${child.node}`);
    parents.set(child.node, id);
  }
  const resolved = new Map<string, ZuiNode>();
  return (nodeId, property, textProperty) => {
    const node = document.nodes?.[nodeId];
    if (!node) return undefined;
    let styled = resolved.get(nodeId);
    if (!styled) {
      const ancestors: ZuiNode[] = [], seen = new Set([nodeId]);
      let parentId = parents.get(nodeId);
      while (parentId) {
        if (seen.has(parentId)) throw new Error(`Cyclic color source ancestry: ${parentId}`);
        seen.add(parentId); ancestors.push(document.nodes![parentId]); parentId = parents.get(parentId);
      }
      styled = styledPreviewNode(document, node, ancestors);
      resolved.set(nodeId, styled);
    }
    // Interactive painters own their state palettes. Do not bind the normal
    // token just because two distinct state colors happen to be equal.
    const preview = styled;
    if (['disabled', 'hovered', 'pressed', 'selected', 'focused', 'popup_open', 'loading', 'dragging']
      .some(key => preview.props?.[key] === true)) return undefined;
    const props = authoredPaintPreviewNode(document, preview, nodeId).props;
    return property === 'foreground_color' && textProperty === 'placeholder'
      ? props?.['placeholder_color'] : props?.[property];
  };
}

/** Resolve the authored cascade, never infer a token from a matching hex value. */
export function sourceColorProperty(
  document: ZuiDocument, nodeId: string,
  property: 'background_color' | 'border_color' | 'foreground_color',
  textProperty?: string | null,
): unknown {
  return sourceColorProperties(document)(nodeId, property, textProperty);
}

function sourceColor(document: ZuiDocument, value: unknown): SourceColor | undefined {
  const seen = new Set<string>();
  while (typeof value === 'string' && value.startsWith('$')) {
    const token = value.slice(1);
    if (seen.has(token) || !Object.hasOwn(document.tokens ?? {}, token))
      throw new Error(`Missing or cyclic shared color token: ${token}`);
    seen.add(token); value = document.tokens![token];
  }
  if (typeof value !== 'string' || !/^#[\da-f]{6}(?:[\da-f]{2})?$/i.test(value))
    return undefined;
  return { color: value.slice(0, 7),
    opacity: value.length === 9 ? parseInt(value.slice(7), 16) / 255 : 1 };
}

function libraries(): ColorLibrary[] {
  if (typeof penpot === 'undefined' || !penpot.library) return [];
  return [penpot.library.local, ...(penpot.library.connected ?? [])];
}

/** A published name or color alone does not establish canonical ownership. */
export function resolveSourceColor(
  document: ZuiDocument, authoredValue: unknown,
  available: readonly ColorLibrary[] = libraries(),
): LibraryColor | undefined {
  if (typeof authoredValue !== 'string' || !authoredValue.startsWith('$')) return;
  const token = authoredValue.slice(1);
  const matches = new Map<string, LibraryColor>();
  for (const library of available) for (const color of library.colors ?? []) {
    const encoded = color.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'token-source');
    if (!encoded) continue;
    let value: unknown;
    try { value = JSON.parse(encoded); }
    catch { throw new Error(`Invalid shared color source: ${color.id}`); }
    if (!record(value) || value['token'] !== token) continue;
    const source = value as unknown as ColorSource;
    const fingerprints = document['penpot_source_fingerprints'];
    const owners = document['penpot_token_source_owners'];
    const owner = record(owners) ? owners[token] : undefined;
    const pins = Array.isArray(fingerprints) ? fingerprints.filter(pin =>
      record(pin) && pin['sourcePath'] === source.sourcePath) : [];
    const palette = document['penpot_host_palette'];
    const entry = record(palette) ? palette[source.sourceKey] : undefined;
    const originalValue = source.sourceIndex === undefined ? entry :
      Array.isArray(entry) && Number.isInteger(source.sourceIndex) && source.sourceIndex >= 0
        ? entry[source.sourceIndex] : undefined;
    if (source.schema !== 'zircon.penpot-token-source.v1' || source.version !== 1 ||
        source.penpotStyleId !== color.id || color.libraryId !== library.id ||
        source.sourcePath !== document['penpot_host_theme_source'] ||
        !record(owner) || owner['sourcePath'] !== source.sourcePath || owner['sha256'] !== source.sourceSha256 ||
        source.sourceTable !== 'palette' || originalValue !== source.originalValue ||
        !/^[a-f\d]{64}$/i.test(source.sourceSha256) || pins.length !== 1 ||
        !record(pins[0]) || pins[0]['sha256'] !== source.sourceSha256)
      throw new Error(`Shared color source closure differs: ${token}`);
    const expected = sourceColor(document, authoredValue);
    const original = sourceColor(document, source.originalValue);
    const actual = color.asFill();
    if (!expected || !original || expected.color.toLowerCase() !== original.color.toLowerCase() ||
        expected.opacity !== original.opacity ||
        actual.fillColor?.toLowerCase() !== expected.color.toLowerCase() ||
        (actual.fillOpacity ?? 1) !== expected.opacity ||
        actual.fillColorRefId !== color.id || actual.fillColorRefFile !== library.id ||
        actual.fillColorGradient || actual.fillImage)
      throw new Error(`Shared color changed outside its source owner: ${token}`);
    matches.set(`${library.id}/${color.id}`, color);
  }
  if (matches.size > 1) throw new Error(`Ambiguous shared color token: ${token}`);
  return matches.values().next().value;
}

function sameColor(expected: SourceColor, color: string | undefined, opacity: number | undefined): boolean {
  return expected.color.toLowerCase() === color?.toLowerCase() && expected.opacity === (opacity ?? 1);
}

/** Bind only an explicit, unchanged source paint; state-owned paints stay separate. */
export function bindSourceFill(fill: Fill, document: ZuiDocument, authoredValue: unknown): Fill {
  singleSolidFill([fill], 'Authored source fill');
  const expected = sourceColor(document, authoredValue);
  if (!expected || !sameColor(expected, fill.fillColor, fill.fillOpacity)) return fill;
  const color = resolveSourceColor(document, authoredValue);
  return color ? { ...fill, ...color.asFill() } : fill;
}

export function bindSourceStroke(stroke: Stroke, document: ZuiDocument, authoredValue: unknown): Stroke {
  singleSolidStroke([stroke], 'Authored source stroke');
  const expected = sourceColor(document, authoredValue);
  if (!expected || !sameColor(expected, stroke.strokeColor, stroke.strokeOpacity)) return stroke;
  const color = resolveSourceColor(document, authoredValue);
  if (!color) return stroke;
  const referenced = color.asStroke();
  if (referenced.strokeColorRefId !== color.id || referenced.strokeColorRefFile !== color.libraryId ||
      !sameColor(expected, referenced.strokeColor, referenced.strokeOpacity) ||
      referenced.strokeColorGradient || referenced.strokeImage)
    throw new Error('Shared color SDK stroke differs from its source.');
  // Width, alignment and caps belong to the control, not to the color asset.
  return { ...stroke, strokeColorRefId: referenced.strokeColorRefId,
    strokeColorRefFile: referenced.strokeColorRefFile };
}

/** Accept a reference only when the original token still owns its exact value.
 * The reconciler then preserves that original token; no referenced paint is
 * converted into a new local literal or silently exported to another owner.
 */
export function captureSourceFill(
  fills: readonly Fill[] | 'mixed', owner: string,
  document?: ZuiDocument, authoredValue?: unknown,
): Fill | undefined {
  if (fills === 'mixed' || fills.length !== 1 ||
      (!fills[0].fillColorRefId && !fills[0].fillColorRefFile))
    return singleSolidFill(fills, owner);
  const fill = fills[0];
  const color = document && resolveSourceColor(document, authoredValue);
  const expected = document && sourceColor(document, authoredValue);
  if (!color || !expected || fill.fillColorRefId !== color.id ||
      fill.fillColorRefFile !== color.libraryId || !sameColor(expected, fill.fillColor, fill.fillOpacity))
    throw new Error(`${owner} has an unmapped shared color reference.`);
  singleSolidFill([{ ...fill, fillColorRefId: undefined, fillColorRefFile: undefined }], owner);
  return fill.fillColor === expected.color ? fill : { ...fill, fillColor: expected.color };
}

export function captureSourceStroke(
  strokes: readonly Stroke[], owner: string,
  document?: ZuiDocument, authoredValue?: unknown,
): Stroke | undefined {
  if (strokes.length !== 1 || (!strokes[0].strokeColorRefId && !strokes[0].strokeColorRefFile))
    return singleSolidStroke(strokes, owner);
  const stroke = strokes[0];
  const color = document && resolveSourceColor(document, authoredValue);
  const expected = document && sourceColor(document, authoredValue);
  if (!color || !expected || stroke.strokeColorRefId !== color.id ||
      stroke.strokeColorRefFile !== color.libraryId || !sameColor(expected, stroke.strokeColor, stroke.strokeOpacity))
    throw new Error(`${owner} has an unmapped shared color reference.`);
  singleSolidStroke([{ ...stroke, strokeColorRefId: undefined, strokeColorRefFile: undefined }], owner);
  return stroke.strokeColor === expected.color ? stroke : { ...stroke, strokeColor: expected.color };
}

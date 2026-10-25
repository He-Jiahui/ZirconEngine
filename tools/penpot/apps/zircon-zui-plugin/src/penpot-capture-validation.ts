import type { Fill, LibraryColor, Stroke } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import type { PenpotAssetSnapshot, ProjectionEditableState, ProjectionText } from './bridge/penpot-projection-model';
import { validateSourcePath } from './bridge/zui-export-apply-contract';
import { sourceColorProperties, type SourceColorLookup } from './penpot-source-colors';
import { ZUI_METADATA_NAMESPACE } from './metadata';

export interface CapturedGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LayoutPoint {
  x: number;
  y: number;
}

export interface LayoutSize {
  width: number;
  height: number;
}

export interface CapturedLayoutSizing {
  horizontal: string | null;
  vertical: string | null;
}

export type CapturedContainerKind = 'free' | 'flex' | 'grid';

export function capturedHeightMin(
  nodeId: string,
  authoredHeightMin: number | null,
  autoLayoutParent: boolean,
  layoutChild: { readonly minHeight?: unknown } | null | undefined,
  baseline?: { heightMin?: number },
): number | undefined {
  const authored = authoredHeightMin !== null &&
    Number.isFinite(authoredHeightMin) && authoredHeightMin >= 0;
  const baselineMapped = baseline !== undefined &&
    Object.hasOwn(baseline, 'heightMin');
  if (!authored) {
    if (baselineMapped)
      throw new Error(`Semantic node ${nodeId} lost its authored height.min mapping.`);
    return undefined;
  }
  if (!autoLayoutParent) {
    if (baselineMapped || layoutChild != null)
      throw new Error(`Semantic node ${nodeId} lost its Penpot minHeight capability.`);
    return undefined;
  }
  if (baseline !== undefined && !baselineMapped)
    throw new Error(`Semantic node ${nodeId} introduced a Penpot minHeight mapping; re-import the .zui asset.`);
  const heightMin = layoutChild?.minHeight;
  if (typeof heightMin !== 'number' || !Number.isFinite(heightMin) || heightMin < 0)
    throw new Error(`Semantic node ${nodeId} has no finite nonnegative Penpot minHeight.`);
  return heightMin;
}

export function resolvedAnchoredPosition(
  parent: LayoutSize,
  child: LayoutSize,
  anchor: LayoutPoint,
  pivot: LayoutPoint,
  position: LayoutPoint,
): LayoutPoint {
  return {
    x: parent.width * anchor.x - child.width * pivot.x + position.x,
    y: parent.height * anchor.y - child.height * pivot.y + position.y,
  };
}

export function authoredAnchoredPosition(
  parent: LayoutSize,
  child: LayoutSize,
  anchor: LayoutPoint,
  pivot: LayoutPoint,
  resolved: LayoutPoint,
): LayoutPoint {
  return {
    x: resolved.x - parent.width * anchor.x + child.width * pivot.x,
    y: resolved.y - parent.height * anchor.y + child.height * pivot.y,
  };
}

export function gridCellForChildIndex(
  index: number,
  columns = 1,
): {
  row: number;
  column: number;
} {
  if (!Number.isInteger(index) || index < 0) {
    throw new Error('Grid child index must be a non-negative integer.');
  }
  if (!Number.isInteger(columns) || columns <= 0) {
    throw new Error('Grid column count must be a positive integer.');
  }
  // Penpot grid cell coordinates are one-based, unlike track mutation indexes.
  return {
    row: Math.floor(index / columns) + 1,
    column: (index % columns) + 1,
  };
}

export function stableSemanticChildOrder(
  childNodeIds: readonly string[],
  siblingIndexByNode: ReadonlyMap<string, number>,
): string[] {
  const traversalIndex = new Map(
    childNodeIds.map((nodeId, index) => [nodeId, index]),
  );
  return [...childNodeIds].sort((left, right) => {
    const leftIndex = siblingIndexByNode.get(left);
    const rightIndex = siblingIndexByNode.get(right);
    if (leftIndex !== undefined && rightIndex !== undefined) {
      return leftIndex - rightIndex;
    }
    if (leftIndex !== undefined) return -1;
    if (rightIndex !== undefined) return 1;
    return (traversalIndex.get(left) ?? 0) - (traversalIndex.get(right) ?? 0);
  });
}

export function resolvedParentContainerKinds(
  sourceParentId: string | null,
  currentParentId: string | null,
  baselineKinds: ReadonlyMap<string, CapturedContainerKind>,
  currentKinds: ReadonlyMap<string, CapturedContainerKind>,
): {
  baseline: CapturedContainerKind | null;
  current: CapturedContainerKind | null;
} {
  return {
    baseline:
      sourceParentId === null
        ? 'free'
        : (baselineKinds.get(sourceParentId) ?? null),
    current:
      currentParentId === null
        ? 'free'
        : (currentKinds.get(currentParentId) ?? null),
  };
}

export const EMPTY_TEXT_SENTINEL = '\u200b';

export function capturedSemanticText(
  characters: string,
  originatedEmpty: boolean,
): string {
  if (!originatedEmpty) return characters;
  return characters.startsWith(EMPTY_TEXT_SENTINEL)
    ? characters.slice(EMPTY_TEXT_SENTINEL.length)
    : characters;
}

export function capturedMappedTextStyle(
  fontSize: string | 'mixed',
  fontWeight: string | 'mixed',
  align: 'left' | 'center' | 'right' | 'justify' | 'mixed' | null,
  owner: string,
): {
  fontSize: number;
  fontWeight: string;
  align: 'left' | 'center' | 'right' | 'justify';
} {
  const numericFontSize = Number(fontSize);
  if (
    fontSize === 'mixed' ||
    !Number.isFinite(numericFontSize) ||
    numericFontSize <= 0
  ) {
    throw new Error(`${owner} has a mixed or invalid font size.`);
  }
  if (fontWeight === 'mixed' || fontWeight.trim() === '') {
    throw new Error(`${owner} has a mixed or invalid font weight.`);
  }
  if (align === 'mixed' || align === null) {
    throw new Error(`${owner} has mixed or missing text alignment.`);
  }
  return { fontSize: numericFontSize, fontWeight, align };
}

export function capturedFontTypography(
  family: string,
  lineHeight: string,
  owner: string,
): { fontFamily: string; lineHeight: number } {
  if (family === 'mixed' || !family.trim())
    throw new Error(`${owner} has a mixed or missing font family.`);
  const ratio = Number(lineHeight);
  if (!Number.isFinite(ratio) || ratio <= 0)
    throw new Error(`${owner} has a mixed or invalid line height.`);
  return { fontFamily: family, lineHeight: ratio };
}

export function symmetricLayoutGap(
  rowGap: number,
  columnGap: number,
  owner: string,
): number {
  if (
    !Number.isFinite(rowGap) ||
    !Number.isFinite(columnGap) ||
    Math.abs(rowGap - columnGap) >= 0.01
  ) {
    throw new Error(
      `${owner} has asymmetric row and column gaps; .zui exposes one container gap.`,
    );
  }
  return rowGap;
}

export function authoredGeometry(
  current: CapturedGeometry,
  baseline: CapturedGeometry | undefined,
  autoLayoutChild: boolean,
  horizontalSizing: string | undefined,
  verticalSizing: string | undefined,
): CapturedGeometry {
  if (!baseline || !autoLayoutChild) return current;
  return {
    x: baseline.x,
    y: baseline.y,
    width: horizontalSizing === 'fill' ? baseline.width : current.width,
    height: verticalSizing === 'fill' ? baseline.height : current.height,
  };
}

export function layoutSizingIsCompatible(
  baseline: CapturedLayoutSizing,
  current: CapturedLayoutSizing,
  baselineParentKind: 'free' | 'flex' | 'grid' | null,
  currentParentKind: 'free' | 'flex' | 'grid' | null,
): boolean {
  if (
    baseline.horizontal === current.horizontal &&
    baseline.vertical === current.vertical
  ) {
    return true;
  }
  if (baselineParentKind === null || currentParentKind === null) return false;
  const baselineAuto = baselineParentKind !== 'free';
  const currentAuto = currentParentKind !== 'free';
  if (baselineAuto === currentAuto) return false;
  if (!baselineAuto && currentAuto) {
    return (
      baseline.horizontal === null &&
      baseline.vertical === null &&
      current.horizontal === 'fix' &&
      current.vertical === 'fix'
    );
  }
  return current.horizontal === null && current.vertical === null;
}


/** A component asset still owns its canonical definition. A native copy's
 * root copy's paint must target a consumer invocation, never that definition.
 * Nested copy nodes and view owners retain their existing source guards. */
export function assertCanonicalCopyPaintUnchanged(
  document: ZuiDocument,
  nodeId: string,
  nativeCopy: boolean,
  baseline: ProjectionEditableState | undefined,
  current: ProjectionEditableState,
): void {
  if (document.asset.kind !== 'component' || !nativeCopy || !baseline ||
      !Object.values(document.components ?? {}).some(component => component.root === nodeId)) return;
  const sameColor = (left: string | null, right: string | null): boolean =>
    left?.toLowerCase() === right?.toLowerCase();
  const sameTextPaint = (
    left: ProjectionText | null | undefined,
    right: ProjectionText | null | undefined,
  ): boolean => left == null || right == null ? left === right :
    sameColor(left.color, right.color) && left.colorOpacity === right.colorOpacity;
  const paint = baseline.paint, next = current.paint;
  const fragments = new Set([
    ...Object.keys(baseline.textFragments ?? {}),
    ...Object.keys(current.textFragments ?? {}),
  ]);
  if (
    sameColor(paint.fillColor, next.fillColor) &&
    paint.fillOpacity === next.fillOpacity &&
    sameColor(paint.strokeColor, next.strokeColor) &&
    paint.strokeOpacity === next.strokeOpacity &&
    paint.strokeWidth === next.strokeWidth &&
    paint.borderRadius === next.borderRadius &&
    paint.opacity === next.opacity &&
    sameTextPaint(baseline.text, current.text) &&
    [...fragments].every(key => sameTextPaint(
      baseline.textFragments?.[key], current.textFragments?.[key]))
  ) return;
  throw new Error(
    `Native copy paint override on ${nodeId} requires a consumer source owner; it cannot be exported into the canonical component definition.`,
  );
}

export function singleSolidFill(
  fills: readonly Fill[] | 'mixed',
  owner: string,
  reference?: SourcePaintBinding,
): Fill | undefined {
  if (fills === 'mixed') {
    throw unsupportedPaint(owner, 'mixed fills');
  }
  if (fills.length > 1) {
    throw unsupportedPaint(owner, 'multiple fills');
  }
  const fill = fills[0];
  if (!fill) return undefined;
  if (
    !fill.fillColor ||
    fill.fillColorGradient != null ||
    fill.fillImage != null
  ) {
    throw unsupportedPaint(owner, 'a non-solid or referenced fill');
  }
  if (fill.fillColorRefFile != null || fill.fillColorRefId != null) {
    const color = validatePaintReference(reference, 'fill', fill.fillColorRefFile,
      fill.fillColorRefId, fill.fillColor, fill.fillOpacity);
    // Keep the native reference pair. Canonical casing avoids a false literal
    // edit in the reconciler while the original token remains its source owner.
    return { ...fill, fillColor: color };
  }
  return fill;
}

export function singleSolidStroke(
  strokes: readonly Stroke[],
  owner: string,
  reference?: SourcePaintBinding,
): Stroke | undefined {
  if (strokes.length > 1) {
    throw unsupportedPaint(owner, 'multiple strokes');
  }
  const stroke = strokes[0];
  if (!stroke) return undefined;
  if (
    !stroke.strokeColor ||
    stroke.strokeColorGradient != null ||
    stroke.strokeImage != null ||
    (stroke.strokeStyle !== undefined && stroke.strokeStyle !== 'solid') ||
    (stroke.strokeAlignment !== undefined &&
      stroke.strokeAlignment !== 'inner') ||
    stroke.strokeCapStart != null ||
    stroke.strokeCapEnd != null
  ) {
    throw unsupportedPaint(owner, 'an unsupported stroke');
  }
  if (stroke.strokeColorRefFile != null || stroke.strokeColorRefId != null) {
    const color = validatePaintReference(reference, 'stroke', stroke.strokeColorRefFile,
      stroke.strokeColorRefId, stroke.strokeColor, stroke.strokeOpacity);
    return { ...stroke, strokeColor: color };
  }
  return stroke;
}

function unsupportedPaint(owner: string, detail: string): Error {
  return new Error(
    `${owner} uses ${detail}; the .zui bridge only exports one solid fill and one inner solid stroke.`,
  );
}

export interface SourcePaintIdentity {
  nodeId: string;
  sourcePath: string;
  sourceNodeId: string;
  controlId: string | null;
  instancePath: string;
}

type VisualColorProperty = 'background_color' | 'border_color' | 'foreground_color';
type ColorLibrary = { readonly id: string; readonly colors: readonly LibraryColor[] };
type PaintKind = 'fill' | 'stroke';
/** A binding is authoritative only while present in the private operation map. */
export interface SourcePaintBinding { readonly kind: 'source-paint-binding' }
export interface SourcePaintCaptureContext {
  binding(identity: SourcePaintIdentity, property: VisualColorProperty,
    textProperty?: string | null, fragment?: string): SourcePaintBinding;
  attach(snapshot: PenpotAssetSnapshot): void;
}
type PaintOperation = {
  document: ZuiDocument;
  closure: string;
  sourceSeal: string;
  libraries: () => readonly ColorLibrary[];
  colorProperty: SourceColorLookup;
  captures: Map<string, ReferencedPaint>;
};
type PaintBinding = {
  operation: PaintOperation;
  identity: SourcePaintIdentity;
  property: VisualColorProperty;
  textProperty?: string | null;
  fragment?: string;
};
type ReferencedPaint = {
  binding: PaintBinding;
  kind: PaintKind;
  file: string;
  id: string;
  expression: string;
  color: string;
  opacity: number;
  libraryRecord: string;
};
const paintBindings = new WeakMap<SourcePaintBinding, PaintBinding>();
const capturedPaints = new WeakMap<PenpotAssetSnapshot, {
  operation: PaintOperation; snapshotSeal: string;
}>();
const paintRecord = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value) && !(value instanceof Date);
const uuid = (value: unknown): value is string =>
  typeof value === 'string' && /^[a-f\d]{8}(?:-[a-f\d]{4}){3}-[a-f\d]{12}$/i.test(value);
const paintSeal = (value: unknown): string => JSON.stringify(value,
  (_key, entry: unknown) => typeof entry === 'bigint' ? { bigint: String(entry) } : entry);

function paintClosure(document: ZuiDocument): string {
  return paintSeal([document.asset.id, document.tokens, document['penpot_source_fingerprints'],
    document['penpot_token_source_owners'], document['penpot_host_palette'],
    document['penpot_host_theme_source']]);
}

function paintPins(document: ZuiDocument): Map<string, string> {
  const raw = document['penpot_source_fingerprints'];
  if (!Array.isArray(raw)) throw new Error('Referenced paint has no source dependency closure.');
  const pins = new Map<string, string>();
  for (const pin of raw) {
    if (!paintRecord(pin) || typeof pin['sourcePath'] !== 'string' ||
        typeof pin['sha256'] !== 'string' || !/^[a-f\d]{64}$/i.test(pin['sha256']) ||
        pins.has(pin['sourcePath']))
      throw new Error('Referenced paint has invalid or duplicate source dependency pins.');
    validateSourcePath(pin['sourcePath']);
    pins.set(pin['sourcePath'], pin['sha256']);
  }
  return pins;
}

function assertPaintIdentity(document: ZuiDocument, identity: SourcePaintIdentity): void {
  const node = document.nodes?.[identity.nodeId];
  const pins = paintPins(document);
  if (!node || !identity.sourcePath || !identity.sourceNodeId ||
      node['penpot_review_source_path'] !== identity.sourcePath ||
      node['penpot_review_source_node_id'] !== identity.sourceNodeId ||
      node['penpot_review_instance_path'] !== identity.instancePath ||
      (typeof node.control_id === 'string' ? node.control_id : null) !== identity.controlId ||
      !pins.has(identity.sourcePath))
    throw new Error(`Referenced paint lost its source owner/node/control identity: ${identity.nodeId}`);
  let calls: unknown;
  try { calls = JSON.parse(identity.instancePath); }
  catch { throw new Error(`Referenced paint has invalid instancePath: ${identity.nodeId}`); }
  if (!Array.isArray(calls) || JSON.stringify(calls) !== identity.instancePath)
    throw new Error(`Referenced paint has noncanonical instancePath: ${identity.nodeId}`);
  for (const call of calls) {
    if (!paintRecord(call) || Object.keys(call).join(',') !== 'sourcePath,sourceNodeId' ||
        typeof call['sourcePath'] !== 'string' || typeof call['sourceNodeId'] !== 'string' ||
        !call['sourceNodeId'].trim() || !pins.has(call['sourcePath']))
      throw new Error(`Referenced paint has an unpinned source invocation: ${identity.nodeId}`);
    validateSourcePath(call['sourcePath']);
  }
}

function tokenColor(document: ZuiDocument, expression: unknown): { color: string; opacity: number } {
  const seen = new Set<string>();
  while (typeof expression === 'string' && expression.startsWith('$')) {
    const token = expression.slice(1);
    if (seen.has(token) || !Object.hasOwn(document.tokens ?? {}, token))
      throw new Error(`Referenced paint has a missing or cyclic source token: ${token}`);
    seen.add(token); expression = document.tokens![token];
  }
  if (typeof expression !== 'string' || !/^#[a-f\d]{6}(?:[a-f\d]{2})?$/i.test(expression))
    throw new Error('Referenced paint source is not a solid RGBA token.');
  return { color: expression.slice(0, 7),
    opacity: expression.length === 9 ? parseInt(expression.slice(7), 16) / 255 : 1 };
}

/** Create once per capture. Read SDK records again at export, never trust a
 * copied name, a matching literal, a serialized marker or a previous revision. */
export function createSourcePaintCaptureContext(
  document: ZuiDocument,
  libraries: () => readonly ColorLibrary[] = () =>
    typeof penpot === 'undefined' || !penpot.library ? [] :
      [penpot.library.local, ...(penpot.library.connected ?? [])],
): SourcePaintCaptureContext {
  const operation: PaintOperation = { document, closure: paintClosure(document), libraries,
    sourceSeal: paintSeal(document),
    colorProperty: sourceColorProperties(document),
    captures: new Map() };
  return {
    binding(identity, property, textProperty, fragment) {
      const result: SourcePaintBinding = Object.freeze({ kind: 'source-paint-binding' });
      paintBindings.set(result, { operation, identity: { ...identity }, property, textProperty, fragment });
      return result;
    },
    attach(snapshot) {
      if (snapshot.assetId !== document.asset.id || capturedPaints.has(snapshot))
        throw new Error('Referenced paint capture belongs to a different or already sealed snapshot.');
      capturedPaints.set(snapshot, { operation, snapshotSeal: paintSeal(snapshot) });
    },
  };
}

function resolvePaintReference(
  binding: PaintBinding, kind: PaintKind, file: unknown, id: unknown,
  color: string | undefined, opacity: number | undefined,
): ReferencedPaint {
  const { operation, identity, property, textProperty } = binding;
  const document = operation.document;
  if (operation.closure !== paintClosure(document) || operation.sourceSeal !== paintSeal(document))
    throw new Error('Referenced paint source closure changed during capture.');
  if (!uuid(file) || !uuid(id)) throw new Error('Referenced paint requires a complete UUID file/style pair.');
  if (kind === 'stroke' ? property !== 'border_color' : property === 'border_color')
    throw new Error('Referenced paint binding has the wrong visual property.');
  if ((property === 'foreground_color') !== Boolean(binding.fragment) ||
      (property !== 'foreground_color' && textProperty != null))
    throw new Error('Referenced paint binding has no matching semantic/text fragment.');
  assertPaintIdentity(document, identity);
  const expression = operation.colorProperty(identity.nodeId, property, textProperty);
  if (typeof expression !== 'string' || !expression.startsWith('$'))
    throw new Error('Referenced paint has no original visual property token.');
  const expected = tokenColor(document, expression);
  const matches = operation.libraries().filter(library => library.id === file);
  if (matches.length !== 1) throw new Error('Referenced paint library is not uniquely available in the current SDK context.');
  const colors = matches[0].colors.filter(entry => entry.id === id);
  if (colors.length !== 1 || colors[0].libraryId !== file)
    throw new Error('Referenced paint style is not owned by its current library.');
  const actual = colors[0];
  const encoded = actual.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'token-source');
  let source: unknown;
  try { source = JSON.parse(encoded); }
  catch { throw new Error('Referenced paint library style has no valid token-source record.'); }
  if (!paintRecord(source)) throw new Error('Referenced paint library style has no token-source record.');
  const palette = document['penpot_host_palette'];
  const key = source['sourceKey'], index = source['sourceIndex'];
  const entry = paintRecord(palette) && typeof key === 'string' ? palette[key] : undefined;
  const indexed = index != null;
  const original = indexed && Array.isArray(entry) && typeof index === 'number' &&
      Number.isInteger(index) && index >= 0 ? entry[index] : !indexed && !Array.isArray(entry) ? entry : undefined;
  // Use the loader's actual palette naming rule, including semantic prefixes.
  const token = key === 'surface' && indexed ? `editor.surface.${index}` :
    typeof key === 'string' && !indexed ?
      `editor.${/^(success|info|warning|error)(_|$)/.test(key) ? 'semantic.' : ''}${key.replaceAll('_', '.')}` : undefined;
  const owners = document['penpot_token_source_owners'];
  const owner = paintRecord(owners) ? owners[expression.slice(1)] : undefined;
  const pins = paintPins(document);
  if (source['schema'] !== 'zircon.penpot-token-source.v1' || source['version'] !== 1 ||
      source['penpotStyleId'] !== id || source['token'] !== expression.slice(1) ||
      token !== expression.slice(1) || source['sourceTable'] !== 'palette' ||
      source['sourcePath'] !== document['penpot_host_theme_source'] ||
      typeof source['sourcePath'] !== 'string' || !paintRecord(owner) ||
      owner['sourcePath'] !== source['sourcePath'] || owner['sha256'] !== source['sourceSha256'] ||
      pins.get(source['sourcePath']) !== source['sourceSha256'] ||
      original === undefined || original !== source['originalValue'])
    throw new Error('Referenced paint token/source dependency or original palette identity differs.');
  const originalColor = tokenColor(document, original);
  const fill = actual.asFill();
  const validColor = (rgb: unknown, alpha: unknown): boolean =>
    typeof rgb === 'string' && rgb.toLowerCase() === expected.color.toLowerCase() &&
    typeof (alpha === undefined ? 1 : alpha) === 'number' &&
    Number.isFinite(alpha === undefined ? 1 : alpha) &&
    (alpha === undefined ? 1 : alpha) === expected.opacity;
  if (!validColor(originalColor.color, originalColor.opacity) ||
      !validColor(actual.color, actual.opacity) || actual.gradient != null || actual.image != null ||
      !validColor(fill.fillColor, fill.fillOpacity) || fill.fillColorRefFile !== file ||
      fill.fillColorRefId !== id || fill.fillColorGradient != null || fill.fillImage != null ||
      !validColor(color, opacity))
    throw new Error('Referenced paint value/opacity or current SDK library record differs from its source token.');
  if (kind === 'stroke') {
    const stroke = actual.asStroke();
    if (!validColor(stroke.strokeColor, stroke.strokeOpacity) || stroke.strokeColorRefFile !== file ||
        stroke.strokeColorRefId !== id || stroke.strokeColorGradient != null || stroke.strokeImage != null)
      throw new Error('Referenced paint SDK stroke differs from its source token.');
  }
  return { binding, kind, file, id, expression, ...expected,
    libraryRecord: paintSeal([file, id, encoded, actual.color, actual.opacity ?? 1, fill]) };
}

function validatePaintReference(
  reference: SourcePaintBinding | undefined, kind: PaintKind,
  file: unknown, id: unknown, color: string | undefined, opacity: number | undefined,
): string {
  const binding = reference && paintBindings.get(reference);
  if (!binding) throw new Error('Referenced paint has no validated source capture context.');
  const resolved = resolvePaintReference(binding, kind, file, id, color, opacity);
  const key = paintSeal([binding.identity.nodeId, binding.property, binding.fragment, kind]);
  const previous = binding.operation.captures.get(key);
  if (previous && paintSeal([previous.file, previous.id, previous.expression, previous.libraryRecord]) !==
      paintSeal([resolved.file, resolved.id, resolved.expression, resolved.libraryRecord]))
    throw new Error('Referenced paint changed within a single capture operation.');
  binding.operation.captures.set(key, resolved);
  return resolved.color;
}

function effectivePaintDocument(document: ZuiDocument): ZuiDocument {
  const host = document['penpot_review_host'];
  return paintRecord(host) && paintRecord(host['nodes'])
    ? { ...(host as ZuiDocument), penpot_source_fingerprints: document['penpot_source_fingerprints'] }
    : document;
}

/** Called before the original-source no-edit fast path as well as mapped edits.
 * References never become export literals or replace the original token. */
export function assertCapturedSourcePaintReferences(
  snapshot: PenpotAssetSnapshot | undefined, before: ZuiDocument, after: ZuiDocument,
): void {
  const captured = snapshot && capturedPaints.get(snapshot);
  if (!captured || captured.operation.captures.size === 0) return;
  const { operation } = captured;
  const original = effectivePaintDocument(before), exported = effectivePaintDocument(after);
  if (captured.snapshotSeal !== paintSeal(snapshot) ||
      paintClosure(original) !== operation.closure || paintClosure(exported) !== operation.closure)
    throw new Error('Referenced paint capture/source closure changed before original-owner export.');
  const documents = [original, exported].map(document => ({ document, colorProperty: sourceColorProperties(document) }));
  for (const saved of operation.captures.values()) {
    const { binding } = saved;
    for (const { document, colorProperty } of documents) {
      assertPaintIdentity(document, binding.identity);
      if (colorProperty(binding.identity.nodeId, binding.property, binding.textProperty) !== saved.expression)
        throw new Error('Referenced paint export must preserve its original visual token expression.');
    }
    const current = resolvePaintReference(binding, saved.kind, saved.file, saved.id, saved.color, saved.opacity);
    if (current.libraryRecord !== saved.libraryRecord)
      throw new Error('Referenced paint library style changed after capture.');
    const shapes = snapshot!.shapes.filter(shape => shape.nodeId === binding.identity.nodeId);
    const shape = shapes[0];
    if (shapes.length !== 1 || shape.sourcePath !== binding.identity.sourcePath ||
        shape.sourceNodeId !== binding.identity.sourceNodeId || shape.controlId !== binding.identity.controlId ||
        shape.instancePath !== binding.identity.instancePath)
      throw new Error('Referenced paint snapshot does not retain its original owner identity.');
    const text = binding.fragment === 'primary' ? shape.current.text :
      binding.fragment ? shape.current.textFragments?.[binding.fragment] : undefined;
    const color = binding.fragment ? text?.color : saved.kind === 'stroke'
      ? shape.current.paint.strokeColor : shape.current.paint.fillColor;
    const opacity = binding.fragment ? text?.colorOpacity : saved.kind === 'stroke'
      ? shape.current.paint.strokeOpacity : shape.current.paint.fillOpacity;
    if (color !== saved.color || opacity !== saved.opacity ||
        (binding.fragment && text?.property !== (binding.textProperty || null)))
      throw new Error('Referenced paint snapshot lost its validated color/text property.');
  }
}

/** Native Wrap derives parent alignment; unsupported authoring must be explicit. */
export function validateCapturedWrapAlignment(
  nodeId: string,
  wrap: boolean,
  alignItems: string | undefined,
  alignContent: string | undefined,
): void {
  if (wrap && (alignItems !== 'start' || alignContent !== 'start'))
    throw new Error(
      `Semantic node ${nodeId} changed native Wrap parent alignment outside the ZUI bridge profile; use supported child slot alignment or restore the imported parent alignment.`,
    );
}

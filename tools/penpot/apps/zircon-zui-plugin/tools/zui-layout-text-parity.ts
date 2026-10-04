import {
  canonicalSha256,
  caseSha256,
  isWorkbenchStateReviewCase,
  type LayoutReviewCase,
} from './zui-layout-review-contract';
import {
  semanticTextIdentity,
  mapMeasuredTexts,
} from './zui-layout-text-mapping';
import {
  isSemanticNodePaintVisible,
  type RendererRuntimeAssets,
} from './zui-layout-semantic-parity';

interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface FontFaceUsage {
  familyName: string;
  glyphCount: number;
  postScriptName?: string;
  resourcePath?: string;
  sha256?: string;
  sourceUrl?: string;
}

interface PenpotText {
  shapeId: string;
  ancestorShapeIds?: string[];
  text: string;
  fontSize: string;
  fontFamily?: string;
  fontWeight?: string;
  lineHeight?: string;
  letterSpacing?: string;
  lines: Rect[];
  lineTexts?: string[];
  fonts: FontFaceUsage[];
}

interface NativeText {
  sourcePath?: string;
  sourceNodeId?: string;
  instancePath?: string;
  nodeId: string;
  controlId?: string | null;
  text: string;
  fonts?: FontFaceUsage[];
  layout: {
    font_size: number;
    font_family?: string;
    font_weight?: string | number;
    line_height?: string | number;
    letter_spacing?: string | number;
    lines: Array<{ text: string; frame: Rect }>;
  } | null;
}

interface TextArtifact {
  case?: LayoutReviewCase;
  caseSha256?: string;
  coordinateSpace?: string;
  fontAudit?: { loaded?: boolean };
  nodes?: NativeText[];
  texts?: PenpotText[];
}

function cssLength(value: string | number | undefined): number | null {
  if (typeof value === 'number') return Number.isFinite(value) ? value : null;
  if (typeof value !== 'string') return null;
  const match = value.trim().match(/^(-?(?:\d+\.?\d*|\.\d+))(?:px)?$/i);
  if (!match) return null;
  const parsed = Number(match[1]);
  return Number.isFinite(parsed) ? parsed : null;
}

function cssLineHeight(
  value: string | number | undefined,
): number | 'normal' | null {
  if (value === 'normal') return 'normal';
  return cssLength(value);
}

function cssFontWeight(value: string | number | undefined): number | null {
  if (typeof value === 'number')
    return Number.isFinite(value) ? value : null;
  if (value === 'normal') return 400;
  if (value === 'bold') return 700;
  return cssLength(value);
}

function primaryFamily(value: string | undefined): string | null {
  if (!value?.trim()) return null;
  const family = value.split(',')[0]?.trim();
  return family?.replace(/^(['"])(.*)\1$/, '$2') || null;
}

function faceIdentity(fonts: FontFaceUsage[]): unknown[] {
  return fonts
    .map(({ familyName, glyphCount, postScriptName, sha256 }) => ({
      familyName,
      glyphCount,
      postScriptName,
      sha256,
    }))
    .sort((a, b) =>
      canonicalSha256(a).localeCompare(canonicalSha256(b)),
    );
}

function actualFaceErrors(
  fonts: FontFaceUsage[] | undefined,
  renderer: 'penpot' | 'engine',
  runtimeAssets: Array<[string, string]> | undefined,
  nodeId: string,
  strict: boolean,
): string[] {
  if (
    !Array.isArray(fonts) ||
    !fonts.length ||
    fonts.some(
      (font) =>
        !font?.familyName ||
        !Number.isFinite(font.glyphCount) ||
        font.glyphCount <= 0,
    )
  )
    return [`Missing actual font usage: ${nodeId}`];
  if (!strict) return [];

  const fingerprints = new Set(
    (runtimeAssets ?? []).map(([path, hash]) => `${path}\0${hash}`),
  );
  const errors: string[] = [];
  for (const font of fonts) {
    if (
      !font.postScriptName ||
      !font.resourcePath ||
      !font.sha256 ||
      (renderer === 'penpot' && !font.sourceUrl) ||
      !/^[0-9a-f]{64}$/.test(font.sha256)
    ) {
      errors.push(`Incomplete actual font face proof: ${nodeId}`);
      continue;
    }
    if (!fingerprints.has(`${font.resourcePath}\0${font.sha256}`))
      errors.push(`Actual font face lacks file fingerprint: ${nodeId}`);
  }
  return errors;
}

export function compareTextEvidence(
  penpotValue: unknown,
  nativeValue: unknown,
  geometryValue: unknown,
  reviewCase: LayoutReviewCase,
  runtimeAssets: RendererRuntimeAssets = {},
  nativeGeometryValue?: unknown,
): string[] {
  const errors: string[] = [];
  const strictWorkbench = isWorkbenchStateReviewCase(reviewCase);
  const penpot = penpotValue as TextArtifact;
  const native = nativeValue as TextArtifact;
  type SemanticTextNode = {
    sourcePath?: string;
    sourceNodeId?: string;
    instancePath?: string;
    nodeId: string;
    controlId?: string | null;
    shapeId?: string;
    visible: boolean;
    text: string | null;
    bounds?: { x: number; y: number; width: number; height: number };
    clip?: boolean;
    clipBounds?: { x: number; y: number; width: number; height: number } | null;
  };
  type TextGeometry = {
    case?: LayoutReviewCase;
    caseSha256?: string;
    coordinateSpace?: string;
    semanticAudit?: { complete?: boolean };
    windowMetrics?: {
      logicalSize?: { width?: number; height?: number };
    };
    layout?: {
      semanticNodes?: SemanticTextNode[];
    };
  };
  const geometry = geometryValue as TextGeometry;
  const nativeGeometry = (nativeGeometryValue ?? geometryValue) as TextGeometry;
  if (
    !Array.isArray(penpot?.texts) ||
    penpot.fontAudit?.loaded !== true ||
    !Array.isArray(native?.nodes) ||
    native.coordinateSpace !== 'logical' ||
    !native.case ||
    caseSha256(native.case) !== caseSha256(reviewCase)
  )
    return ['Missing or stale measured text evidence'];
  if (strictWorkbench) {
    if (
      !penpot.case ||
      caseSha256(penpot.case) !== caseSha256(reviewCase) ||
      penpot.caseSha256 !== caseSha256(reviewCase)
    )
      errors.push('penpot: stale text case');
    if (penpot.coordinateSpace !== 'logical')
      errors.push('penpot: text is not in logical coordinates');
    if (
      native.caseSha256 !== caseSha256(reviewCase) ||
      native.fontAudit?.loaded !== true
    )
      errors.push('engine: stale or incomplete text case/font audit');
    if (
      !geometry.case ||
      caseSha256(geometry.case) !== caseSha256(reviewCase) ||
      geometry.caseSha256 !== caseSha256(reviewCase) ||
      geometry.coordinateSpace !== 'logical' ||
      geometry.semanticAudit?.complete !== true ||
      !nativeGeometry.case ||
      caseSha256(nativeGeometry.case) !== caseSha256(reviewCase) ||
      nativeGeometry.caseSha256 !== caseSha256(reviewCase) ||
      nativeGeometry.coordinateSpace !== 'logical' ||
      nativeGeometry.semanticAudit?.complete !== true
    )
      errors.push('Missing or stale logical semantic text mapping');
  }
  const nodes = geometry?.layout?.semanticNodes;
  if (!Array.isArray(nodes)) return [...errors, 'Missing semantic text mapping'];
  const nativeSemanticNodes = nativeGeometry?.layout?.semanticNodes;
  if (strictWorkbench && !Array.isArray(nativeSemanticNodes))
    return [...errors, 'Missing native semantic text mapping'];
  const mapped = mapMeasuredTexts(nodes, penpot.texts);
  const consumed = new Set<PenpotText>();
  const ids = new Set<string>();
  const semanticNodesByIdentity = new Map(
    nodes.map((node) => [
      strictWorkbench ? semanticTextIdentity(node) : node.nodeId,
      node,
    ]),
  );
  const nativeSemanticNodesByIdentity = new Map(
    (nativeSemanticNodes ?? []).map((node) => [
      strictWorkbench ? semanticTextIdentity(node) : node.nodeId,
      node,
    ]),
  );
  for (const node of nodes.filter((node) => node.text?.trim())) {
    const identity = strictWorkbench
      ? semanticTextIdentity(node)
      : node.nodeId;
    const nativeSemanticNode = nativeSemanticNodesByIdentity.get(identity);
    if (strictWorkbench && !nativeSemanticNode) {
      errors.push(`Missing native semantic text mapping: ${node.nodeId}`);
      continue;
    }
    const penpotVisible =
      !strictWorkbench || isSemanticNodePaintVisible(node, geometry);
    const engineVisible =
      !strictWorkbench ||
      isSemanticNodePaintVisible(nativeSemanticNode, nativeGeometry);
    if (!penpotVisible && !engineVisible) continue;
    ids.add(identity);
    const browser = mapped.get(identity) ?? [];
    const engine = native.nodes.filter((text) =>
      strictWorkbench
        ? text.sourcePath === node.sourcePath &&
          text.sourceNodeId === node.sourceNodeId &&
          text.instancePath === node.instancePath
        : text.nodeId === node.nodeId,
    );
    if (
      (penpotVisible && browser.length !== 1) ||
      (engineVisible && engine.length !== 1)
    ) {
      errors.push(`Unmatched or fragmented measured text: ${node.nodeId}`);
      continue;
    }
    const a = browser[0],
      b = engine[0];
    if (a) consumed.add(a);
    if (penpotVisible && a?.text !== node.text)
      errors.push(`Rendered text differs: ${node.nodeId}`);
    if (engineVisible && b?.text !== node.text)
      errors.push(`Rendered text differs: ${node.nodeId}`);
    if (strictWorkbench && penpotVisible !== engineVisible) {
      errors.push(`Text paint visibility differs: ${node.nodeId}`);
      if (penpotVisible && a) {
        if (
          !Array.isArray(a.lines) ||
          !a.lines.length ||
          !Array.isArray(a.lineTexts) ||
          a.lineTexts.length !== a.lines.length ||
          a.fontFamily === undefined ||
          a.fontWeight === undefined ||
          a.lineHeight === undefined ||
          a.letterSpacing === undefined ||
          !Number.isFinite(Number.parseFloat(a.fontSize))
        )
          errors.push(`Incomplete strict Penpot text evidence: ${node.nodeId}`);
        errors.push(
          ...actualFaceErrors(
            a.fonts,
            'penpot',
            runtimeAssets.penpot,
            node.nodeId,
            true,
          ),
        );
      }
      if (engineVisible && b) {
        if (
          b.controlId !== node.controlId ||
          !node.sourcePath ||
          !b.layout?.font_family ||
          b.layout.font_weight === undefined ||
          b.layout.line_height === undefined ||
          b.layout.letter_spacing === undefined ||
          !Number.isFinite(b.layout.font_size) ||
          !Array.isArray(b.layout.lines) ||
          !b.layout.lines.length ||
          b.layout.lines.some((line) => typeof line.text !== 'string')
        )
          errors.push(`Incomplete strict engine text evidence: ${node.nodeId}`);
        errors.push(
          ...actualFaceErrors(
            b.fonts,
            'engine',
            runtimeAssets.engine,
            node.nodeId,
            true,
          ),
        );
      }
      continue;
    }
    if (!a || !b) continue;
    if (a.text !== node.text || b.text !== node.text)
      errors.push(`Rendered text differs: ${node.nodeId}`);
    if (
      strictWorkbench &&
      (b.controlId !== node.controlId ||
        !node.sourcePath ||
        !Array.isArray(a.lineTexts) ||
        !b.layout?.font_family ||
        a.fontFamily === undefined ||
        a.fontWeight === undefined ||
        a.lineHeight === undefined ||
        a.letterSpacing === undefined ||
        b.layout.font_weight === undefined ||
        b.layout.line_height === undefined ||
        b.layout.letter_spacing === undefined)
    )
      errors.push(`Incomplete strict text style evidence: ${node.nodeId}`);
    errors.push(
      ...actualFaceErrors(
        a.fonts,
        'penpot',
        runtimeAssets.penpot,
        node.nodeId,
        strictWorkbench,
      ),
      ...(strictWorkbench
        ? actualFaceErrors(
            b.fonts,
            'engine',
            runtimeAssets.engine,
            node.nodeId,
            true,
          )
        : []),
    );
    if (strictWorkbench && a.fonts && b.fonts) {
      if (
        canonicalSha256(faceIdentity(a.fonts)) !==
        canonicalSha256(faceIdentity(b.fonts))
      )
        errors.push(`Actual font face differs: ${node.nodeId}`);
      const browserStyle = {
        family: primaryFamily(a.fontFamily),
        size: cssLength(a.fontSize),
        weight: cssFontWeight(a.fontWeight),
        lineHeight: cssLineHeight(a.lineHeight),
        letterSpacing:
          a.letterSpacing === 'normal' ? 0 : cssLength(a.letterSpacing),
      };
      const engineStyle = {
        family: primaryFamily(b.layout?.font_family),
        size: b.layout?.font_size,
        weight: cssFontWeight(b.layout?.font_weight),
        lineHeight: cssLineHeight(b.layout?.line_height),
        letterSpacing:
          b.layout?.letter_spacing === 'normal'
            ? 0
            : cssLength(b.layout?.letter_spacing),
      };
      if (
        !Number.isFinite(browserStyle.size) ||
        !Number.isFinite(engineStyle.size) ||
        canonicalSha256(browserStyle) !== canonicalSha256(engineStyle)
      )
        errors.push(`Rendered text style differs: ${node.nodeId}`);
    } else if (Math.abs(Number.parseFloat(a.fontSize) - (b.layout?.font_size ?? NaN)) > 0.01) {
      errors.push(`Missing or different measured font size: ${node.nodeId}`);
    }
    if (
      !b.layout ||
      !Number.isFinite(b.layout.font_size) ||
      !Number.isFinite(Number.parseFloat(a.fontSize))
    ) {
      errors.push(`Missing or different measured font size: ${node.nodeId}`);
      continue;
    }
    if (
      !Array.isArray(a.lines) ||
      !a.lines.length ||
      !Array.isArray(b.layout.lines) ||
      a.lines.length !== b.layout.lines.length
    ) {
      errors.push(`Rendered text line count differs: ${node.nodeId}`);
      continue;
    }
    if (
      strictWorkbench &&
      (a.lineTexts?.length !== a.lines.length ||
        a.lines.some(
          (_line, index) =>
            typeof a.lineTexts?.[index] !== 'string' ||
            a.lineTexts[index] !== b.layout?.lines[index]?.text,
        ))
    )
      errors.push(`Rendered text line breaks differ: ${node.nodeId}`);
    for (const [index, line] of a.lines.entries()) {
      const other = b.layout.lines[index].frame;
      for (const axis of ['x', 'y', 'width', 'height'] as const) {
        if (
          !Number.isFinite(line?.[axis]) ||
          !Number.isFinite(other?.[axis]) ||
          Math.abs(line[axis] - other[axis]) > 1
        )
          errors.push(`Text geometry differs: ${node.nodeId}.${index}.${axis}`);
      }
    }
  }
  if (
    penpot.texts.some((text) => {
      if (!text.text?.trim() || consumed.has(text)) return false;
      const owner = [...mapped.entries()].find(([, fragments]) =>
        fragments.includes(text),
      )?.[0];
      const semanticOwner = owner
        ? semanticNodesByIdentity.get(owner)
        : undefined;
      return (
        !semanticOwner ||
        !strictWorkbench ||
        isSemanticNodePaintVisible(semanticOwner, geometry)
      );
    }) ||
    native.nodes.some((node) => {
      const identity = strictWorkbench
        ? semanticTextIdentity(node)
        : node.nodeId;
      if (!node.text?.trim() || ids.has(identity)) return false;
      const semanticOwner = nativeSemanticNodesByIdentity.get(identity);
      return (
        !semanticOwner ||
        !strictWorkbench ||
        isSemanticNodePaintVisible(semanticOwner, nativeGeometry)
      );
    })
  )
    errors.push('Unmatched rendered text nodes');
  return errors;
}

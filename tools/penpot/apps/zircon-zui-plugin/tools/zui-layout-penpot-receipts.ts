import type { Page } from 'playwright';
import {
  isSemanticNodePaintVisible,
  type ExpectedResourceUse,
} from './zui-layout-semantic-parity';

export interface PenpotSemanticStyleOwner {
  shapeId?: string;
  sourcePath?: string | null;
  sourceNodeId?: string | null;
  instancePath?: string | null;
  visible?: boolean;
  bounds?: { x: number; y: number; width: number; height: number };
  clip?: boolean;
  clipBounds?: { x: number; y: number; width: number; height: number } | null;
}

export interface PenpotShadowLayer {
  offsetX: number;
  offsetY: number;
  blurRadius: number;
  spreadRadius: number;
  radius: number;
  color: string;
  opacity: number;
  inset: boolean;
}

export interface PenpotStyleInventory {
  version: 1;
  complete: boolean;
  properties: {
    foregroundColor: string | null;
    backgroundColor: string | null;
    borderColor: string | null;
    borderWidth: number | null;
    borderRadius: number | null;
    opacity: number | null;
    boxShadow: { complete: boolean; layers: PenpotShadowLayer[] };
  };
  reason?: string;
}

export interface PenpotMediaReceipt {
  kind: 'raster' | 'vector';
  path: string;
  sha256: string;
  loaded: boolean;
  sourcePath?: string;
  sourceNodeId?: string;
  instancePath?: string;
}

export interface PenpotRuntimeReceiptSet {
  styles: Map<string, PenpotStyleInventory>;
  media: PenpotMediaReceipt[];
  runtimeAssetFingerprints: Array<[string, string]>;
  pendingReasons: string[];
}

interface RawStyleReceipt {
  shapeId: string;
  properties?: PenpotStyleInventory['properties'];
  reason?: string;
}

interface RawMediaReceipt {
  shapeId?: string;
  sha256?: string;
  kind?: 'raster' | 'vector';
  loaded: boolean;
  reason?: string;
}

interface AuthoredFileFingerprint {
  sourcePath: string;
  sha256: string;
}

function authoredOwnerKey(node: PenpotSemanticStyleOwner): string | undefined {
  if (
    typeof node.sourcePath !== 'string' ||
    !node.sourcePath ||
    typeof node.sourceNodeId !== 'string' ||
    !node.sourceNodeId ||
    typeof node.instancePath !== 'string' ||
    !node.instancePath
  )
    return undefined;
  return JSON.stringify([node.sourcePath, node.sourceNodeId, node.instancePath]);
}

export function penpotStyleReceiptIsRequired(
  shapeId: string,
  nodes: readonly PenpotSemanticStyleOwner[],
  viewport: { width: number; height: number },
): boolean {
  const owners = nodes.filter((node) => node.shapeId === shapeId);
  if (owners.length !== 1) return true;
  return isSemanticNodePaintVisible(owners[0], {
    windowMetrics: { logicalSize: viewport },
  });
}

/** Read final mounted Penpot paint and decoded image bytes for each semantic shape. */
export async function capturePenpotRuntimeReceipts(
  page: Page,
  boardId: string,
  nodes: PenpotSemanticStyleOwner[],
  dependencies: AuthoredFileFingerprint[],
  viewport: { width: number; height: number },
  expectedResourceUses?: ExpectedResourceUse[],
): Promise<PenpotRuntimeReceiptSet> {
  const shapeIds = nodes.map((node) => node.shapeId ?? '');
  const captured = await page.evaluate(
    async ({ boardId, shapeIds }) => {
      const root = document.querySelector(`#shape-${CSS.escape(boardId)}`);
      if (!root) throw new Error('Missing mounted Penpot workbench board');
      const nodeForShape = new Map<string, number>();
      shapeIds.forEach((shapeId, index) => {
        if (shapeId) nodeForShape.set(shapeId.replace(/^shape-/, ''), index);
      });
      const elementForShape = new Map<string, Element>();
      const styleReceipts: RawStyleReceipt[] = [];
      for (const shapeId of shapeIds) {
        if (!shapeId || elementForShape.has(shapeId)) continue;
        const id = shapeId.startsWith('shape-') ? shapeId : `shape-${shapeId}`;
        const candidate = document.getElementById(id) ?? document.getElementById(shapeId);
        if (!candidate || !root.contains(candidate)) {
          styleReceipts.push({ shapeId, reason: 'semantic shape has no mounted Penpot element' });
          continue;
        }
        elementForShape.set(shapeId, candidate);
        const elements = [candidate, ...candidate.querySelectorAll('*')].filter(
          (element) => {
            const style = getComputedStyle(element);
            return (
              style.display !== 'none' &&
              style.visibility !== 'hidden' &&
              style.visibility !== 'collapse'
            );
          },
        );
        const normalizeColor = (value: string): string | undefined => {
          const match = value.match(
            /^rgba?\(\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)\s*,\s*(\d+(?:\.\d+)?)(?:\s*,\s*(\d+(?:\.\d+)?))?\s*\)$/i,
          );
          if (!match) return undefined;
          const channels = match.slice(1, 4).map(Number);
          const alpha = match[4] === undefined ? 1 : Number(match[4]);
          if (
            channels.some((value) => !Number.isFinite(value) || value < 0 || value > 255) ||
            !Number.isFinite(alpha) || alpha < 0 || alpha > 1
          )
            return undefined;
          return `rgba(${channels.map(Math.round).join(',')},${Number(alpha.toFixed(6))})`;
        };
        const effectivePaintColor = (
          value: string,
          paintOpacity: string | number | undefined,
        ): string | undefined => {
          const normalized = normalizeColor(value);
          const opacity = paintOpacity === undefined ? 1 : Number(paintOpacity);
          if (!normalized || !Number.isFinite(opacity) || opacity < 0 || opacity > 1)
            return undefined;
          const match = normalized.match(
            /^rgba\((\d+),(\d+),(\d+),([\d.]+)\)$/,
          );
          if (!match) return undefined;
          const alpha = Number(match[4]) * opacity;
          return `rgba(${match[1]},${match[2]},${match[3]},${Number(alpha.toFixed(6))})`;
        };
        const paintOpacity = (
          style: CSSStyleDeclaration,
          cssName: 'fill-opacity' | 'stroke-opacity',
        ): number | undefined => {
          const raw = style.getPropertyValue(cssName);
          if (raw === '') return 1;
          const value = Number(raw);
          return Number.isFinite(value) && value >= 0 && value <= 1
            ? value
            : undefined;
        };
        const splitShadowList = (value: string): string[] | undefined => {
          const parts: string[] = [];
          let depth = 0;
          let start = 0;
          for (let index = 0; index < value.length; index += 1) {
            const character = value[index];
            if (character === '(') depth += 1;
            else if (character === ')') depth -= 1;
            else if (character === ',' && depth === 0) {
              parts.push(value.slice(start, index).trim());
              start = index + 1;
            }
            if (depth < 0) return undefined;
          }
          if (depth !== 0) return undefined;
          const final = value.slice(start).trim();
          if (final) parts.push(final);
          return parts;
        };
        const parseShadow = (
          value: string,
          radius: number,
          effectiveOpacity: number,
        ): PenpotShadowLayer[] | undefined => {
          if (value === 'none') return [];
          const parts = splitShadowList(value);
          if (!parts?.length) return undefined;
          const layers: PenpotShadowLayer[] = [];
          for (const part of parts) {
            const colorValue = part.match(/rgba?\([^)]*\)/i)?.[0];
            const normalizedColor = colorValue
              ? normalizeColor(colorValue)
              : undefined;
            const lengths = part.match(/-?(?:\d+\.?\d*|\.\d+)px/g) ?? [];
            const parsed = lengths.map((length) => Number.parseFloat(length));
            if (
              !normalizedColor ||
              parsed.length < 2 ||
              parsed.length > 4 ||
              parsed.some((number) => !Number.isFinite(number))
            )
              return undefined;
            const alpha = Number(
              normalizedColor.match(/,((?:\d+\.?\d*|\.\d+))\)$/)?.[1] ?? 1,
            );
            const opaqueColor = normalizedColor.replace(/,[^,]+\)$/, ',1)');
            layers.push({
              offsetX: parsed[0]!,
              offsetY: parsed[1]!,
              blurRadius: parsed[2] ?? 0,
              spreadRadius: parsed[3] ?? 0,
              radius,
              color: opaqueColor,
              opacity: Number((alpha * effectiveOpacity).toFixed(6)),
              inset: /\binset\b/i.test(part),
            });
          }
          return layers;
        };
        const elementRadius = (
          element: Element,
          style: CSSStyleDeclaration,
        ): { radius: number; complete: boolean } => {
          const names = [
            'borderTopLeftRadius',
            'borderTopRightRadius',
            'borderBottomRightRadius',
            'borderBottomLeftRadius',
          ] as const;
          const values = names.map((name) => {
            const match = style[name].match(/^(-?(?:\d+\.?\d*|\.\d+))px$/);
            return match ? Number(match[1]) : undefined;
          });
          let radius = values[0] ?? 0;
          let complete = values.every((value) => value !== undefined && value === radius);
          if (element.tagName.toLowerCase() === 'rect') {
            const rx = Number.parseFloat(element.getAttribute('rx') ?? '0');
            const ry = Number.parseFloat(element.getAttribute('ry') ?? String(rx));
            if (!Number.isFinite(rx) || !Number.isFinite(ry) || Math.abs(rx - ry) > 0.001)
              complete = false;
            else if (rx > 0) radius = rx;
          }
          return { radius, complete };
        };
        const unique = (values: Array<string | undefined>): string[] =>
          [...new Set(values.filter((value): value is string => value !== undefined))];
        const textColors: string[] = [];
        const fills: string[] = [];
        const strokes: string[] = [];
        const strokeWidths: number[] = [];
        const radii: number[] = [];
        const shadows: PenpotShadowLayer[] = [];
        let shadowReceiptComplete = true;
        let styleReceiptComplete = true;
        const unsupportedPaint: string[] = [];
        const effectiveOpacityForElement = (element: Element): number => {
          let opacity = 1;
          for (
            let current: Element | null = element;
            current && root.contains(current);
            current = current.parentElement
          ) {
            const factor = Number(getComputedStyle(current).opacity);
            if (!Number.isFinite(factor) || factor < 0 || factor > 1)
              return Number.NaN;
            opacity *= factor;
            if (current === root) break;
          }
          return opacity;
        };
        for (const element of elements) {
          const style = getComputedStyle(element);
          const tag = element.tagName.toLowerCase();
          const shapeRadius = elementRadius(element, style);
          if (!shapeRadius.complete) styleReceiptComplete = false;
          const effectiveOpacity = effectiveOpacityForElement(element);
          const shadowLayers = parseShadow(
            style.boxShadow,
            shapeRadius.radius,
            effectiveOpacity,
          );
          if (!shadowLayers || (shadowLayers.length > 0 && !shapeRadius.complete))
            shadowReceiptComplete = false;
          else shadows.push(...shadowLayers);
          if (style.filter !== 'none') shadowReceiptComplete = false;
          const isText =
            tag === 'text' ||
            tag === 'tspan' ||
            element.closest('foreignObject') !== null;
          if (isText) {
            const color =
              tag === 'text' || tag === 'tspan'
                ? effectivePaintColor(
                    style.fill,
                    paintOpacity(style, 'fill-opacity'),
                  )
                : normalizeColor(style.color);
            if (color && !color.endsWith(',0)')) textColors.push(color);
            else if (
              tag === 'text' ||
              tag === 'tspan'
                ? style.fill !== 'none' && !style.fill.endsWith(',0)')
                : style.color !== 'transparent'
            )
              unsupportedPaint.push('foreground');
          }
          if (!isText && /^(path|rect|circle|ellipse|polygon|polyline|image|use)$/.test(tag)) {
            const fill = effectivePaintColor(
              style.fill,
              paintOpacity(style, 'fill-opacity'),
            );
            if (fill && !fill.endsWith(',0)')) fills.push(fill);
            else if (style.fill !== 'none' && !style.fill.endsWith(',0)'))
              unsupportedPaint.push('background');
          }
          const stroke = effectivePaintColor(
            style.stroke,
            paintOpacity(style, 'stroke-opacity'),
          );
          if (stroke && !stroke.endsWith(',0)')) {
            strokes.push(stroke);
            const width = Number.parseFloat(style.strokeWidth);
            if (Number.isFinite(width)) strokeWidths.push(width);
          } else if (style.stroke !== 'none' && !style.stroke.endsWith(',0)')) {
            unsupportedPaint.push('border');
          }
          const cssRadius = style.borderTopLeftRadius;
          const radiusMatch = cssRadius.match(/^(-?(?:\d+\.?\d*|\.\d+))px$/);
          if (radiusMatch) radii.push(Number(radiusMatch[1]));
          if (tag === 'rect') {
            const rx = Number.parseFloat(element.getAttribute('rx') ?? '0');
            const ry = Number.parseFloat(element.getAttribute('ry') ?? String(rx));
            if (Number.isFinite(rx) && Number.isFinite(ry)) {
              if (Math.abs(rx - ry) > 0.001) radii.push(Number.NaN);
              else if (rx > 0) radii.push(rx);
            }
          }
        }
        const foreground = unique(textColors);
        const background = unique(fills);
        const border = unique(strokes);
        const widthSet = unique(strokeWidths.map((width) => String(width)));
        const radiusSet = unique(radii.map((radius) => String(radius)));
        const effectiveOpacities = elements.map(effectiveOpacityForElement);
        const opacitySet = unique(effectiveOpacities.map((value) => String(value)));
        const opacity = opacitySet.length ? Number(opacitySet[0]) : Number.NaN;
        const properties = {
          foregroundColor: foreground[0] ?? null,
          backgroundColor: background[0] ?? null,
          borderColor: border[0] ?? null,
          borderWidth: border.length ? Number(widthSet[0]) : 0,
          borderRadius: radiusSet.length ? Number(radiusSet[0]) : 0,
          opacity,
          boxShadow: {
            complete: shadowReceiptComplete,
            layers: shadows,
          },
        };
        const reason =
          foreground.length > 1 ||
          background.length > 1 ||
          border.length > 1 ||
          widthSet.length > 1 ||
          radiusSet.length > 1 ||
          opacitySet.length !== 1 ||
          !styleReceiptComplete ||
          !shadowReceiptComplete ||
          unsupportedPaint.length > 0 ||
          radii.some((radius) => !Number.isFinite(radius)) ||
          !Number.isFinite(opacity) ||
          opacity < 0 ||
          opacity > 1
            ? 'mounted Penpot shape has conflicting effective paint values'
            : undefined;
        styleReceipts.push({ shapeId, properties, reason });
      }

      const mediaReceipts: RawMediaReceipt[] = [];
      for (const image of root.querySelectorAll('image')) {
        const href =
          image.getAttribute('href') ?? image.getAttribute('xlink:href');
        if (!href) {
          mediaReceipts.push({ loaded: false, reason: 'mounted image has no source URL' });
          continue;
        }
        let shapeId: string | undefined;
        for (let parent: Element | null = image; parent && root.contains(parent); parent = parent.parentElement) {
          const id = parent.id.startsWith('shape-') ? parent.id.slice(6) : undefined;
          if (id && nodeForShape.has(id)) {
            shapeId = shapeIds[nodeForShape.get(id)!];
            break;
          }
        }
        try {
          const response = await fetch(href);
          if (!response.ok) throw new Error(`image fetch returned ${response.status}`);
          const bytes = await response.arrayBuffer();
          const digest = await crypto.subtle.digest('SHA-256', bytes);
          const sha256 = [...new Uint8Array(digest)]
            .map((value) => value.toString(16).padStart(2, '0'))
            .join('');
          const contentType = response.headers.get('content-type') ?? '';
          const prefix = new TextDecoder().decode(bytes.slice(0, 256)).trimStart();
          const kind = /svg/i.test(contentType) || /^<svg\b/i.test(prefix) ? 'vector' : 'raster';
          mediaReceipts.push({ shapeId, sha256, kind, loaded: true });
        } catch (error) {
          mediaReceipts.push({
            shapeId,
            loaded: false,
            reason: error instanceof Error ? error.message : String(error),
          });
        }
      }
      return { styleReceipts, mediaReceipts };
    },
    { boardId, shapeIds },
  );

  const pendingReasons: string[] = [];
  const styles = new Map<string, PenpotStyleInventory>();
  for (const receipt of captured.styleReceipts as RawStyleReceipt[]) {
    if (!receipt.properties || receipt.reason) {
      if (penpotStyleReceiptIsRequired(receipt.shapeId, nodes, viewport))
        pendingReasons.push(
          `style ${receipt.shapeId || '(unidentified shape)'}: ${receipt.reason ?? 'missing effective values'}`,
        );
      styles.set(receipt.shapeId, {
        version: 1,
        complete: false,
        properties: receipt.properties ?? {
          foregroundColor: null,
          backgroundColor: null,
          borderColor: null,
          borderWidth: null,
          borderRadius: null,
          opacity: null,
          boxShadow: { complete: false, layers: [] },
        },
        reason: receipt.reason,
      });
    } else styles.set(receipt.shapeId, { version: 1, complete: true, properties: receipt.properties });
  }

  const dependencyByHash = new Map<string, AuthoredFileFingerprint[]>();
  for (const dependency of dependencies) {
    const paths = dependencyByHash.get(dependency.sha256) ?? [];
    paths.push(dependency);
    dependencyByHash.set(dependency.sha256, paths);
  }
  const assets: PenpotMediaReceipt[] = [];
  const runtimeAssetFingerprints = new Map<string, string>();
  const matchedExpectedResources = new Set<string>();
  for (const receipt of captured.mediaReceipts as RawMediaReceipt[]) {
    const owner = nodes.find((node) => node.shapeId === receipt.shapeId);
    const ownerKey = owner ? authoredOwnerKey(owner) : undefined;
    const candidates = receipt.sha256 ? dependencyByHash.get(receipt.sha256) ?? [] : [];
    const expectedCandidates = expectedResourceUses?.filter(
      (use) =>
        use.sourcePath === owner?.sourcePath &&
        use.sourceNodeId === owner?.sourceNodeId &&
        use.instancePath === owner?.instancePath &&
        use.sha256 === receipt.sha256 &&
        use.kind === receipt.kind,
    ) ?? [];
    const expected: ExpectedResourceUse | undefined = expectedResourceUses
      ? expectedCandidates.length === 1
        ? expectedCandidates[0]
        : undefined
      : candidates.length === 1 &&
          ownerKey &&
          owner?.sourcePath &&
          owner.sourceNodeId &&
          owner.instancePath &&
          receipt.kind
        ? {
            kind: receipt.kind,
            path: candidates[0]!.sourcePath,
            sha256: candidates[0]!.sha256,
            sourcePath: owner.sourcePath,
            sourceNodeId: owner.sourceNodeId,
            instancePath: owner.instancePath,
          }
        : undefined;
    if (!receipt.loaded || !receipt.sha256 || !receipt.kind || !ownerKey || !expected) {
      pendingReasons.push(
        `media ${receipt.shapeId ?? '(unowned image)'}: ${receipt.reason ?? 'loaded bytes do not identify one source-derived file and semantic owner'}`,
      );
      assets.push({
        kind: receipt.kind ?? 'raster',
        path: '',
        sha256: receipt.sha256 ?? '',
        loaded: false,
      });
      continue;
    }
    if (expectedResourceUses)
      matchedExpectedResources.add(
        JSON.stringify([
          expected.kind,
          expected.path,
          expected.sha256,
          expected.sourcePath,
          expected.sourceNodeId,
          expected.instancePath,
        ]),
      );
    runtimeAssetFingerprints.set(expected.path, expected.sha256);
    assets.push({
      kind: receipt.kind,
      path: expected.path,
      sha256: receipt.sha256,
      loaded: true,
      sourcePath: owner!.sourcePath!,
      sourceNodeId: owner!.sourceNodeId!,
      instancePath: owner!.instancePath!,
    });
  }
  for (const expected of expectedResourceUses ?? []) {
    const key = JSON.stringify([
      expected.kind,
      expected.path,
      expected.sha256,
      expected.sourcePath,
      expected.sourceNodeId,
      expected.instancePath,
    ]);
    if (!matchedExpectedResources.has(key))
      pendingReasons.push(
        `media ${expected.sourcePath}#${expected.sourceNodeId}@${expected.instancePath}: expected source media was not mounted and loaded`,
      );
  }
  return {
    styles,
    media: assets,
    runtimeAssetFingerprints: [...runtimeAssetFingerprints.entries()].sort(
      ([a], [b]) => a.localeCompare(b),
    ),
    pendingReasons,
  };
}

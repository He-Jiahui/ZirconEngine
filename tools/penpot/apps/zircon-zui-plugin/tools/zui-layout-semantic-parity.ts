import {
  canonicalSha256,
  caseSha256,
  isWorkbenchStateReviewCase,
  type LayoutReviewCase,
  type WorkbenchStateSelector,
} from './zui-layout-review-contract';
import {
  runtimeLoadedSourceErrors,
  runtimeTokenEvidenceErrors,
  workbenchLocaleEvidenceErrors,
  type RuntimeSourceFingerprint,
} from './zui-layout-runtime-provenance';

interface SemanticNode {
  nodeId: string;
  sourceNodeId?: string;
  sourcePath?: string;
  instancePath?: string;
  controlId?: string | null;
  parentNodeId: string | null;
  parentSourcePath?: string | null;
  parentSourceNodeId?: string | null;
  parentInstancePath?: string | null;
  component: string;
  visible: boolean;
  detached: boolean;
  clip?: boolean;
  clipBounds?: { x: number; y: number; width: number; height: number } | null;
  text: string | null;
  bounds: { x: number; y: number; width: number; height: number };
  effectiveStyle?: {
    complete?: boolean;
    properties?: Record<string, unknown>;
  };
  styleInventory?: {
    version?: number;
    complete?: boolean;
    properties?: Record<string, unknown>;
  };
}

interface ShadowLayer {
  offsetX: number;
  offsetY: number;
  blurRadius: number;
  spreadRadius: number;
  radius: number;
  color: string;
  opacity: number;
  inset: boolean;
}

interface GeometryDocument {
  case?: LayoutReviewCase;
  caseSha256?: string;
  coordinateSpace?: string;
  windowMetrics?: {
    logicalSize?: { width?: number; height?: number };
  };
  sourceIdentityProvenance?: {
    sourceMapFingerprint?: [string, string] | null;
    sources?: Array<[string, string]>;
    runtimeLoadedSources?: unknown;
  };
  activeDesignTokens?: unknown;
  consumedTokens?: unknown;
  semanticAudit?: { complete?: boolean; expectedNodeCount?: number };
  assetAudit?: {
    complete?: boolean;
    resources?: Array<{
      kind: string;
      path: string;
      sha256: string;
      loaded: boolean;
      sourcePath?: string;
      sourceNodeId?: string;
      instancePath?: string;
    }>;
  };
  hostOverlayAudit?: {
    complete?: boolean;
    windows?: Array<{ windowId: string }>;
  };
  layout?: { semanticNodes?: SemanticNode[] };
}

export interface RendererRuntimeAssets {
  penpot?: Array<[string, string]>;
  engine?: Array<[string, string]>;
  authoredSources?: Array<[string, string]>;
  engineCapturePrograms?: Array<[string, string]>;
  expectedSemanticNodes?: string[];
  expectedResourceUses?: ExpectedResourceUse[];
  expectedHostOverlayIds?: string[];
  expectedSourceClosure?: RuntimeSourceFingerprint[];
  requireRuntimeProvenance?: boolean;
}

export interface ExpectedResourceUse {
  kind: 'raster' | 'vector';
  path: string;
  sha256: string;
  sourcePath: string;
  sourceNodeId: string;
  instancePath: string;
}

/**
 * Measured paint receipts are required only where a semantic node can
 * contribute pixels in this case's logical viewport. Missing viewport or
 * clip data returns true so strict consumers fail closed until the envelope
 * is repaired.
 */
export function isSemanticNodePaintVisible(
  value: unknown,
  evidence: unknown,
): boolean {
  const node = value as SemanticNode | undefined;
  const document = evidence as GeometryDocument | undefined;
  if (node?.visible !== true) return false;
  const viewportWidth = document?.windowMetrics?.logicalSize?.width;
  const viewportHeight = document?.windowMetrics?.logicalSize?.height;
  if (
    typeof viewportWidth !== 'number' ||
    !Number.isFinite(viewportWidth) ||
    typeof viewportHeight !== 'number' ||
    !Number.isFinite(viewportHeight)
  )
    return true;
  const bounds = node.bounds;
  if (
    !bounds ||
    !['x', 'y', 'width', 'height'].every((key) =>
      Number.isFinite(bounds[key as keyof SemanticNode['bounds']]),
    )
  )
    return true;
  if (
    bounds.width <= 0 ||
    bounds.height <= 0 ||
    viewportWidth <= 0 ||
    viewportHeight <= 0
  )
    return false;
  const intersects = (left: number, top: number, right: number, bottom: number) =>
    bounds.x + bounds.width > left &&
    bounds.y + bounds.height > top &&
    bounds.x < right &&
    bounds.y < bottom;
  if (!intersects(0, 0, viewportWidth, viewportHeight)) return false;
  if (node.clip !== true) return true;
  const clip = node.clipBounds;
  if (
    !clip ||
    !['x', 'y', 'width', 'height'].every((key) =>
      Number.isFinite(clip[key as keyof NonNullable<SemanticNode['clipBounds']>]),
    )
  )
    return true;
  if (clip.width <= 0 || clip.height <= 0) return false;
  return intersects(
    clip.x,
    clip.y,
    clip.x + clip.width,
    clip.y + clip.height,
  );
}

const REQUIRED_EFFECTIVE_STYLE_FIELDS = [
  'foregroundColor',
  'backgroundColor',
  'borderColor',
  'borderWidth',
  'borderRadius',
  'opacity',
  'boxShadow',
] as const;

export const REQUIRED_STYLE_INVENTORY_FIELDS =
  REQUIRED_EFFECTIVE_STYLE_FIELDS;

function isJsonValue(value: unknown): boolean {
  if (
    value === null ||
    typeof value === 'string' ||
    typeof value === 'boolean' ||
    (typeof value === 'number' && Number.isFinite(value))
  )
    return true;
  if (Array.isArray(value)) return value.every(isJsonValue);
  if (typeof value !== 'object') return false;
  return Object.values(value as Record<string, unknown>).every(isJsonValue);
}

function hasCanonicalEffectiveStyle(
  properties: Record<string, unknown> | undefined,
): boolean {
  if (!properties) return false;
  const color = (value: unknown): boolean => {
    if (typeof value !== 'string') return false;
    const match = value.match(
      /^rgba\((\d{1,3}),(\d{1,3}),(\d{1,3}),(0|1|0?\.\d+)\)$/,
    );
    return Boolean(
      match &&
        match.slice(1, 4).every((channel) => Number(channel) <= 255) &&
        Number(match[4]) >= 0 &&
        Number(match[4]) <= 1,
    );
  };
  return (
    (properties['foregroundColor'] === null || color(properties['foregroundColor'])) &&
    (properties['backgroundColor'] === null || color(properties['backgroundColor'])) &&
    (properties['borderColor'] === null || color(properties['borderColor'])) &&
    typeof properties['borderWidth'] === 'number' &&
    Number.isFinite(properties['borderWidth']) &&
    properties['borderWidth'] >= 0 &&
    typeof properties['borderRadius'] === 'number' &&
    Number.isFinite(properties['borderRadius']) &&
    properties['borderRadius'] >= 0 &&
    typeof properties['opacity'] === 'number' &&
    Number.isFinite(properties['opacity']) &&
    properties['opacity'] >= 0 &&
    properties['opacity'] <= 1 &&
    isCompleteShadowReceipt(properties['boxShadow'])
  );
}

function isCompleteShadowReceipt(value: unknown): value is {
  complete: true;
  layers: ShadowLayer[];
} {
  if (
    !value ||
    typeof value !== 'object' ||
    Array.isArray(value) ||
    Object.keys(value).sort().join(',') !== 'complete,layers' ||
    (value as Record<string, unknown>)['complete'] !== true ||
    !Array.isArray((value as Record<string, unknown>)['layers'])
  )
    return false;
  return ((value as { layers: unknown[] }).layers).every((layer) => {
    if (!layer || typeof layer !== 'object' || Array.isArray(layer)) return false;
    const item = layer as Record<string, unknown>;
    const color = item['color'];
    const match =
      typeof color === 'string' &&
      color.match(/^rgba\((\d{1,3}),(\d{1,3}),(\d{1,3}),1\)$/);
    return (
      Object.keys(item).sort().join(',') ===
        'blurRadius,color,inset,offsetX,offsetY,opacity,radius,spreadRadius' &&
      ['offsetX', 'offsetY', 'blurRadius', 'spreadRadius', 'radius', 'opacity'].every(
        (key) => typeof item[key] === 'number' && Number.isFinite(item[key]),
      ) &&
      Boolean(
        match && match.slice(1, 4).every((channel) => Number(channel) <= 255),
      ) &&
      typeof item['inset'] === 'boolean' &&
      Number(item['blurRadius']) >= 0 &&
      Number(item['radius']) >= 0 &&
      Number(item['opacity']) >= 0 &&
      Number(item['opacity']) <= 1
    );
  });
}

export function sourceSemanticIdentity(
  node: Pick<SemanticNode, 'sourcePath' | 'sourceNodeId' | 'instancePath'>,
): string {
  return JSON.stringify([node.sourcePath, node.sourceNodeId, node.instancePath]);
}

function displayIdentity(
  node: Pick<SemanticNode, 'sourcePath' | 'sourceNodeId' | 'instancePath'>,
): string {
  return `${node.sourcePath}#${node.sourceNodeId}@${node.instancePath}`;
}

function isAuthoredInstancePath(value: unknown): value is string {
  if (typeof value !== 'string' || !value) return false;
  try {
    const steps = JSON.parse(value) as unknown;
    return (
      Array.isArray(steps) &&
      steps.every(
        (step) =>
          step &&
          typeof step === 'object' &&
          !Array.isArray(step) &&
          Object.keys(step).join(',') === 'sourcePath,sourceNodeId' &&
          typeof step.sourcePath === 'string' &&
          !!step.sourcePath.trim() &&
          typeof step.sourceNodeId === 'string' &&
          !!step.sourceNodeId.trim(),
      ) &&
      JSON.stringify(steps) === value
    );
  } catch {
    return false;
  }
}

function assetAuditErrors(
  document: GeometryDocument,
  renderer: 'penpot' | 'engine',
  runtimeAssets: Array<[string, string]> | undefined,
  expectedResourceUses: ExpectedResourceUse[] | undefined,
): string[] {
  const errors: string[] = [];
  const audit = document.assetAudit;
  if (!Array.isArray(expectedResourceUses))
    return [`${renderer}: source-derived media inventory is unavailable`];
  if (
    audit?.complete !== true ||
    !Array.isArray(audit.resources)
  )
    return [`${renderer}: incomplete runtime asset audit`];
  const fingerprints = new Set(
    (runtimeAssets ?? []).map(([path, hash]) => `${path}\0${hash}`),
  );
  for (const resource of audit.resources) {
    if (
      !resource ||
      (resource.kind !== 'raster' && resource.kind !== 'vector') ||
      typeof resource.path !== 'string' ||
      !resource.path ||
      resource.path.startsWith('/') ||
      resource.path.includes('\\') ||
      resource.path.split('/').some((part) => part === '.' || part === '..') ||
      typeof resource.sha256 !== 'string' ||
      !/^[0-9a-f]{64}$/.test(resource.sha256) ||
      resource.loaded !== true ||
      typeof resource.sourcePath !== 'string' ||
      !resource.sourcePath.trim() ||
      typeof resource.sourceNodeId !== 'string' ||
      !resource.sourceNodeId.trim() ||
      !isAuthoredInstancePath(resource.instancePath)
    ) {
      errors.push(`${renderer}: invalid or unavailable runtime asset`);
      continue;
    }
    if (!fingerprints.has(`${resource.path}\0${resource.sha256}`))
      errors.push(
        `${renderer}: runtime asset lacks file fingerprint: ${resource.path}`,
      );
  }
  const identity = (resource: ExpectedResourceUse) =>
    JSON.stringify([
      resource.kind,
      resource.path,
      resource.sha256,
      resource.sourcePath,
      resource.sourceNodeId,
      resource.instancePath,
    ]);
  const expected = new Set(expectedResourceUses.map(identity));
  const actual = new Set(
    (audit.resources ?? []).map((resource) =>
      JSON.stringify([
        resource.kind,
        resource.path,
        resource.sha256,
        resource.sourcePath,
        resource.sourceNodeId,
        resource.instancePath,
      ]),
    ),
  );
  for (const use of expectedResourceUses)
    if (!actual.has(identity(use)))
      errors.push(
        `${renderer}: missing source-derived media use ${use.sourcePath}#${use.sourceNodeId}@${use.instancePath}`,
      );
  for (const resource of audit.resources ?? [])
    if (
      !expected.has(
        JSON.stringify([
          resource.kind,
          resource.path,
          resource.sha256,
          resource.sourcePath,
          resource.sourceNodeId,
          resource.instancePath,
        ]),
      )
    )
      errors.push(`${renderer}: unexpected runtime media use ${resource.path}`);
  return errors;
}

function hostOverlayAuditErrors(
  document: GeometryDocument,
  renderer: 'penpot' | 'engine',
  expectedWindowIds: string[] | undefined,
): string[] {
  const errors: string[] = [];
  const audit = document.hostOverlayAudit;
  if (
    !Array.isArray(expectedWindowIds) ||
    !audit ||
    Object.keys(audit).sort().join(',') !== 'complete,windows' ||
    audit?.complete !== true ||
    !Array.isArray(audit.windows)
  )
    return [`${renderer}: incomplete host overlay audit`];
  const actualWindowIds = audit.windows.map((window) => {
    if (
      !window ||
      typeof window !== 'object' ||
      Object.keys(window).sort().join(',') !== 'windowId'
    )
      return undefined;
    return window.windowId;
  });
  if (
    actualWindowIds.some((id) => typeof id !== 'string' || !id.trim()) ||
    new Set(actualWindowIds).size !== actualWindowIds.length ||
    canonicalSha256(actualWindowIds) !== canonicalSha256(expectedWindowIds)
  )
    errors.push(`${renderer}: host overlay inventory differs from workbench layout`);
  if (expectedWindowIds.length)
    errors.push(
      `${renderer}: visible floating windows lack complete factual overlay receipts`,
    );
  return errors;
}

function assetIdentity(resources: NonNullable<GeometryDocument['assetAudit']>['resources']) {
  const identities = (resources ?? [])
    .map((resource) => {
      if (!resource || typeof resource !== 'object') return null;
      const { kind, path, sha256, sourcePath, sourceNodeId, instancePath } = resource;
      return {
        kind,
        path,
        sha256,
        sourcePath: sourcePath ?? null,
        sourceNodeId: sourceNodeId ?? null,
        instancePath: instancePath ?? null,
      };
    })
    .map((resource) => [canonicalSha256(resource), resource] as const)
    .sort(([left], [right]) => left.localeCompare(right));
  return [...new Map(identities).values()];
}

function sourceIdentityProvenanceErrors(
  document: GeometryDocument,
  runtimeAssets: RendererRuntimeAssets,
): string[] {
  const errors: string[] = [];
  const provenance = document.sourceIdentityProvenance;
  if (
    !provenance ||
    !Array.isArray(provenance.sources) ||
    !Array.isArray(provenance.sourceMapFingerprint)
  )
    return ['engine: missing source identity provenance'];

  const sourceFingerprints = new Map(runtimeAssets.authoredSources ?? []);
  const seen = new Set<string>();
  for (const source of provenance.sources) {
    if (
      !Array.isArray(source) ||
      source.length !== 2 ||
      typeof source[0] !== 'string' ||
      !source[0] ||
      typeof source[1] !== 'string' ||
      !/^[0-9a-f]{64}$/.test(source[1]) ||
      seen.has(source[0])
    ) {
      errors.push('engine: invalid or duplicate source identity fingerprint');
      continue;
    }
    seen.add(source[0]);
    if (sourceFingerprints.get(source[0]) !== source[1])
      errors.push(`engine: source identity is not current: ${source[0]}`);
  }

  const semanticSources = new Set(
    (document.layout?.semanticNodes ?? [])
      .map((node) => node.sourcePath)
      .filter((path): path is string => typeof path === 'string' && !!path),
  );
  for (const sourcePath of semanticSources)
    if (!seen.has(sourcePath))
      errors.push(`engine: semantic source lacks identity fingerprint: ${sourcePath}`);

  const [mapPath, mapHash] = provenance.sourceMapFingerprint;
  if (
    provenance.sourceMapFingerprint.length !== 2 ||
    typeof mapPath !== 'string' ||
    !mapPath ||
    typeof mapHash !== 'string' ||
    !/^[0-9a-f]{64}$/.test(mapHash) ||
    !(runtimeAssets.engineCapturePrograms ?? []).some(
      ([path, hash]) => path === mapPath && hash === mapHash,
    )
  )
    errors.push('engine: source identity map lacks current capture fingerprint');
  return errors;
}

// Compare logical node bounds, never framebuffer size or PNG similarity.
export function compareSemanticGeometry(
  penpot: unknown,
  engine: unknown,
  reviewCase: LayoutReviewCase,
  runtimeAssets: RendererRuntimeAssets = {},
) {
  const errors: string[] = [];
  const strictWorkbench = isWorkbenchStateReviewCase(reviewCase);
  const read = (
    value: unknown,
    renderer: 'penpot' | 'engine',
  ): Map<string, SemanticNode> => {
    const document = value as GeometryDocument;
    const nodes = document?.layout?.semanticNodes;
    if (!document?.case || caseSha256(document.case) !== caseSha256(reviewCase))
      errors.push(`${renderer}: stale geometry case`);
    if (strictWorkbench) {
      if (document?.caseSha256 !== caseSha256(reviewCase))
        errors.push(`${renderer}: stale geometry case hash`);
      if (document?.coordinateSpace !== 'logical')
        errors.push(`${renderer}: geometry is not in logical coordinates`);
      const logicalSize = document?.windowMetrics?.logicalSize;
      if (
        !logicalSize ||
        !Number.isFinite(logicalSize.width) ||
        !Number.isFinite(logicalSize.height) ||
        logicalSize.width !== reviewCase.viewport.width ||
        logicalSize.height !== reviewCase.viewport.height
      )
        errors.push(`${renderer}: logical viewport bounds differ from review case`);
      if (
        document?.semanticAudit?.complete !== true ||
        document.semanticAudit.expectedNodeCount !==
          runtimeAssets.expectedSemanticNodes?.length
      )
        errors.push(`${renderer}: incomplete semantic coverage`);
      errors.push(
        ...assetAuditErrors(
          document,
          renderer,
          runtimeAssets[renderer],
          runtimeAssets.expectedResourceUses,
        ),
        ...hostOverlayAuditErrors(
          document,
          renderer,
          runtimeAssets.expectedHostOverlayIds,
        ),
      );
    }
    if (!Array.isArray(nodes) || !nodes.length) {
      errors.push(`${renderer}: missing semantic geometry`);
      return new Map();
    }
    const result = new Map<string, SemanticNode>();
    const renderNodeIds = new Set<string>();
    for (const node of nodes) {
      if (
        !node ||
        typeof node.nodeId !== 'string' ||
        !node.nodeId ||
        (strictWorkbench && renderNodeIds.has(node.nodeId)) ||
        (strictWorkbench
          ? typeof node.sourcePath !== 'string' ||
            !node.sourcePath ||
            typeof node.sourceNodeId !== 'string' ||
            !node.sourceNodeId.trim() ||
            !isAuthoredInstancePath(node.instancePath) ||
            result.has(sourceSemanticIdentity(node))
          : result.has(node.nodeId)) ||
        typeof node.visible !== 'boolean' ||
        typeof node.detached !== 'boolean' ||
        (strictWorkbench && typeof node.clip !== 'boolean') ||
        (strictWorkbench &&
          node.clipBounds !== null &&
          (!node.clipBounds ||
            ['x', 'y', 'width', 'height'].some(
              (key) =>
                !Number.isFinite(
                  node.clipBounds![key as keyof NonNullable<SemanticNode['clipBounds']>],
                ),
            ) ||
            node.clipBounds.width < 0 ||
            node.clipBounds.height < 0)) ||
        (strictWorkbench && node.clip !== Boolean(node.clipBounds)) ||
        !(
          node.parentNodeId === null || typeof node.parentNodeId === 'string'
        ) ||
        (strictWorkbench &&
          node.parentSourcePath !== undefined &&
          node.parentSourcePath !== null &&
          (typeof node.parentSourcePath !== 'string' ||
             !node.parentSourcePath.trim())) ||
        (strictWorkbench &&
          node.parentInstancePath !== undefined &&
          node.parentInstancePath !== null &&
          !isAuthoredInstancePath(node.parentInstancePath)) ||
        typeof node.component !== 'string' ||
        (strictWorkbench && !node.component.trim()) ||
        !(node.text === null || typeof node.text === 'string') ||
        !node.bounds ||
        ['x', 'y', 'width', 'height'].some(
          (key) =>
            !Number.isFinite(node.bounds[key as keyof SemanticNode['bounds']]),
        ) ||
        node.bounds.width < 0 ||
        node.bounds.height < 0
      ) {
        errors.push(
          `${renderer}: invalid or duplicate semantic node ${node?.nodeId}`,
        );
        continue;
      }
      renderNodeIds.add(node.nodeId);
      if (
        strictWorkbench &&
        !(
          node.controlId === null ||
          (typeof node.controlId === 'string' && node.controlId.trim())
        )
      ) {
        errors.push(
          `${renderer}: missing source control identity ${displayIdentity(node)}`,
        );
        continue;
      }
      if (
        strictWorkbench &&
        (node.parentSourceNodeId === null
          ? node.parentSourcePath !== null || node.parentInstancePath !== null
          : typeof node.parentSourceNodeId !== 'string' ||
            !node.parentSourceNodeId.trim() ||
            typeof node.parentSourcePath !== 'string' ||
            !node.parentSourcePath.trim() ||
            !isAuthoredInstancePath(node.parentInstancePath))
      ) {
        errors.push(
          `${renderer}: missing source parent identity ${displayIdentity(node)}`,
        );
        continue;
      }
      if (strictWorkbench && isSemanticNodePaintVisible(node, document)) {
        const style = node.styleInventory;
        if (
          style?.version !== 1 ||
          style?.complete !== true ||
          !style.properties ||
          Object.keys(style.properties).sort().join(',') !==
            [...REQUIRED_STYLE_INVENTORY_FIELDS].sort().join(',') ||
          REQUIRED_EFFECTIVE_STYLE_FIELDS.some(
            (field) => !Object.hasOwn(style.properties!, field),
          ) ||
          !Object.values(style.properties).every(isJsonValue) ||
          !hasCanonicalEffectiveStyle(style.properties)
        ) {
          errors.push(
            `${renderer}: missing complete style inventory ${displayIdentity(node)}`,
          );
          continue;
        }
      }
      result.set(
        strictWorkbench ? sourceSemanticIdentity(node) : node.nodeId,
        node,
      );
    }
    if (strictWorkbench) {
      if (!Array.isArray(runtimeAssets.expectedSemanticNodes))
        errors.push(`${renderer}: source-derived semantic inventory is unavailable`);
      else {
        const expected = new Set(runtimeAssets.expectedSemanticNodes);
        if (expected.size !== runtimeAssets.expectedSemanticNodes.length)
          errors.push(`${renderer}: source-derived semantic inventory is ambiguous`);
        for (const identity of expected)
          if (!result.has(identity))
            errors.push(`${renderer}: missing source-derived semantic node ${identity}`);
        for (const identity of result.keys())
          if (!expected.has(identity))
            errors.push(`${renderer}: unexpected semantic node ${identity}`);
        if (result.size !== expected.size)
          errors.push(`${renderer}: semantic coverage differs from source inventory`);
      }
      for (const node of result.values())
        if (node.parentNodeId !== null && !renderNodeIds.has(node.parentNodeId))
          errors.push(
            `${renderer}: missing render parent ${node.parentNodeId} for ${displayIdentity(node)}`,
          );
      for (const resource of document?.assetAudit?.resources ?? [])
        if (
          !result.has(
            JSON.stringify([
              resource.sourcePath,
              resource.sourceNodeId,
              resource.instancePath,
            ]),
          )
        )
          errors.push(
            `${renderer}: runtime asset has no semantic owner ${resource.sourcePath}#${resource.sourceNodeId}@${resource.instancePath}`,
          );
    }
    return result;
  };
  const leftDocument = penpot as GeometryDocument;
  const rightDocument = engine as GeometryDocument;
  const left = read(penpot, 'penpot');
  const right = read(engine, 'engine');
  if (strictWorkbench) {
    for (const [renderer, nodes] of [
      ['penpot', left],
      ['engine', right],
    ] as const) {
      for (const node of nodes.values()) {
        if (node.parentSourceNodeId === null) continue;
        const parent = JSON.stringify([
          node.parentSourcePath,
          node.parentSourceNodeId,
          node.parentInstancePath,
        ]);
        if (!nodes.has(parent))
          errors.push(
            `${renderer}: missing semantic parent ${node.parentSourcePath}#${node.parentSourceNodeId}@${node.parentInstancePath}`,
          );
      }
    }
  }
  let maxGeometryDeltaPx = 0;
  let textMatches = true;
  let visibilityMatches = true;
  for (const identity of new Set([...left.keys(), ...right.keys()])) {
    const a = left.get(identity),
      b = right.get(identity);
    const id =
      strictWorkbench && (a ?? b)
        ? displayIdentity(a ?? b!)
        : identity;
    if (!a || !b) {
      errors.push(`Unmatched semantic node: ${id}`);
      continue;
    }
    if (
      (!strictWorkbench && a.parentNodeId !== b.parentNodeId) ||
      a.component !== b.component ||
      (strictWorkbench &&
        (a.controlId !== b.controlId ||
          a.parentSourcePath !== b.parentSourcePath ||
          a.parentSourceNodeId !== b.parentSourceNodeId ||
          a.parentInstancePath !== b.parentInstancePath ||
          a.clip !== b.clip ||
          canonicalSha256(a.clipBounds ?? null) !==
            canonicalSha256(b.clipBounds ?? null)))
    )
      errors.push(`Semantic ownership differs: ${id}`);
    if (
      strictWorkbench &&
      (isSemanticNodePaintVisible(a, leftDocument) ||
        isSemanticNodePaintVisible(b, rightDocument)) &&
      canonicalSha256(a.styleInventory?.properties) !==
        canonicalSha256(b.styleInventory?.properties)
    )
      errors.push(`Effective style differs: ${id}`);
    if (a.text !== b.text) {
      textMatches = false;
      errors.push(`Semantic text differs: ${id}`);
    }
    if (a.visible !== b.visible || a.detached !== b.detached) {
      visibilityMatches = false;
      errors.push(`Visibility differs: ${id}`);
    }
    if (!strictWorkbench && !a.visible && !b.visible) continue;
    for (const axis of ['x', 'y', 'width', 'height'] as const) {
      const delta = Math.abs(a.bounds[axis] - b.bounds[axis]);
      maxGeometryDeltaPx = Math.max(maxGeometryDeltaPx, delta);
      if (delta > 1)
        errors.push(`Geometry differs by ${delta}px: ${id}.${axis}`);
    }
  }
  if (strictWorkbench) {
    errors.push(
      ...sourceIdentityProvenanceErrors(rightDocument, runtimeAssets),
    );
    if (runtimeAssets.requireRuntimeProvenance) {
      errors.push(
        ...runtimeTokenEvidenceErrors(leftDocument, rightDocument),
        ...runtimeLoadedSourceErrors(
          leftDocument,
          rightDocument,
          runtimeAssets.expectedSourceClosure ?? [],
        ),
        ...workbenchLocaleEvidenceErrors(
          leftDocument,
          rightDocument,
          reviewCase.locale,
        ),
      );
    }
    const selector = reviewCase.data['workbenchState'] as WorkbenchStateSelector;
    for (const [renderer, nodes] of [
      ['penpot', leftDocument?.layout?.semanticNodes ?? []],
      ['engine', rightDocument?.layout?.semanticNodes ?? []],
    ] as const) {
      const matches = nodes.filter(
        (node) =>
          node.sourcePath === selector.sourcePath &&
          node.controlId === selector.controlId &&
          (selector.sourceNodeId === undefined ||
            node.sourceNodeId === selector.sourceNodeId) &&
          (selector.instancePath === undefined ||
            node.instancePath === selector.instancePath),
      );
      if (matches.length !== 1)
        errors.push(
          `${renderer}: workbenchState selector resolves to ${matches.length} semantic nodes`,
        );
    }
    for (const override of selector.textOverrides ?? []) {
      for (const [renderer, document] of [
        ['penpot', leftDocument],
        ['engine', rightDocument],
      ] as const) {
        const matches = (document?.layout?.semanticNodes ?? []).filter(
          (node) =>
            node.sourcePath === override.sourcePath &&
            node.sourceNodeId === override.sourceNodeId &&
            node.controlId === override.controlId &&
            (override.instancePath === undefined ||
              node.instancePath === override.instancePath),
        );
        if (matches.length !== 1 || matches[0]?.text !== override.value)
          errors.push(
            `${renderer}: text override does not resolve exactly: ${override.sourcePath}#${override.sourceNodeId}`,
          );
      }
    }
    const scrollTarget = selector.scrollTarget;
    if (scrollTarget) {
      for (const [renderer, nodes] of [
        ['penpot', leftDocument?.layout?.semanticNodes ?? []],
        ['engine', rightDocument?.layout?.semanticNodes ?? []],
      ] as const) {
        const matches = nodes.filter(
          (node) =>
            node.sourcePath === scrollTarget.sourcePath &&
            node.controlId === scrollTarget.controlId &&
            (scrollTarget.sourceNodeId === undefined ||
              node.sourceNodeId === scrollTarget.sourceNodeId) &&
            (scrollTarget.instancePath === undefined ||
              node.instancePath === scrollTarget.instancePath),
        );
        if (matches.length !== 1)
          errors.push(
            `${renderer}: workbenchState scrollTarget resolves to ${matches.length} semantic nodes`,
          );
      }
    }
    const leftAssets = assetIdentity(leftDocument?.assetAudit?.resources);
    const rightAssets = assetIdentity(rightDocument?.assetAudit?.resources);
    if (canonicalSha256(leftAssets) !== canonicalSha256(rightAssets))
      errors.push('Runtime asset usage differs');
  }
  return { errors, maxGeometryDeltaPx, textMatches, visibilityMatches };
}

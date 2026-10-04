import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { createHash } from 'node:crypto';
import { createReadStream, existsSync, realpathSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { resolve, relative, isAbsolute, dirname, basename } from 'node:path';
import { parse } from 'smol-toml';
import { chromium } from 'playwright';
import { parseZuiDocument } from '../src/bridge/zui-document';
import { reviewDocumentForCase } from '../src/bridge/zui-review-case';
import { reviewHost } from '../src/bridge/zui-review-host';
import type { CatalogEntry } from './zui-layout-catalog';
import {
  canonicalSha256,
  caseSha256,
  casesSha256,
  isWorkbenchStateReviewCase,
  requiredNativeRenderer,
  validateLayoutReviewCase,
  type DependencyFingerprint,
  type LayoutReviewCase,
  type LayoutRenderEvidence,
  type LayoutCaseReview,
} from './zui-layout-review-contract';
import {
  compareSemanticGeometry,
  isSemanticNodePaintVisible,
  type RendererRuntimeAssets,
} from './zui-layout-semantic-parity';
import { compareTextEvidence } from './zui-layout-text-parity';
import { sourceCellCoverageErrors } from './zui-layout-source-cell-coverage';
import { sourceScenarioCoverageErrors } from './zui-layout-source-scenario-coverage';
import { browserRuntimeErrors } from './zui-layout-browser-runtime';
import { deriveSourceRenderInventory } from './zui-layout-source-render-inventory';
import { verifyWorkbenchManagedInputCases } from './zui-layout-workbench-managed-inputs';
import { deriveExpectedRuntimeSourceFiles } from './zui-layout-runtime-provenance';

const REQUIRED_PENPOT_CAPTURE_PROGRAM_PATHS = [
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-visual-validation.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-visual-source-scope.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-capture-provenance.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-penpot-cases.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-cases.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-managed-inputs.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-artifact-path.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-presentation.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-workbench-projection.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/penpot-workbench-canvas-edit.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/penpot-preview-layout-audit.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-evidence.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-penpot-receipts.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-source-render-inventory.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-browser-runtime.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-semantic-parity.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-text-parity.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-text-evidence.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-font-resources.ts',
  'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-runtime-provenance.ts',
] as const;

export function missingPenpotCaptureProgramPaths(
  fingerprints: ReadonlyArray<readonly [string, string]> | undefined,
): string[] {
  const paths = new Set((fingerprints ?? []).map(([path]) => path));
  return REQUIRED_PENPOT_CAPTURE_PROGRAM_PATHS.filter(
    (path) => !paths.has(path),
  );
}

function expectedWorkbenchOverlayIds(
  reviewCase: LayoutReviewCase,
): string[] | undefined {
  if (!isWorkbenchStateReviewCase(reviewCase)) return undefined;
  const presentation = reviewCase.data['workbenchPresentation'] as
    | { layout?: { floating_windows?: unknown } }
    | undefined;
  const windows = presentation?.layout?.floating_windows;
  if (!Array.isArray(windows)) return undefined;
  const ids = windows.map((window) => {
    if (
      !window ||
      typeof window !== 'object' ||
      Array.isArray(window) ||
      typeof (window as Record<string, unknown>)['window_id'] !== 'string'
    )
      return undefined;
    return (window as Record<string, string>)['window_id'];
  });
  const validIds = ids.filter(
    (id): id is string => typeof id === 'string' && !!id.trim(),
  );
  return validIds.length === ids.length ? validIds : undefined;
}

export const bytesSha256 = (bytes: Uint8Array | string): string =>
  createHash('sha256').update(bytes).digest('hex');

export async function fileSha256(path: string): Promise<string> {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

export function containedPath(root: string, path: string): string {
  if (!path.trim()) throw new Error('Missing evidence path');
  const absolute = resolve(root, path);
  const suffix = relative(root, absolute);
  if (isAbsolute(suffix) || suffix === '..' || /^\.\.[/\\]/.test(suffix))
    throw new Error(`Path escapes root: ${path}`);
  return resolveLayoutCatalogPath(absolute);
}

// Include expanded prefab owners and their media as well as explicit imports.
export async function dependencyFingerprints(
  repoRoot: string,
  sourcePath: string,
  prepared: Record<string, unknown>,
): Promise<DependencyFingerprint[]> {
  const paths = new Set<string>();
  await collectFontDependencies(repoRoot, sourcePath, prepared, paths);
  const declared = prepared['penpot_dependency_sources'];
  if (Array.isArray(declared))
    for (const path of declared)
      if (typeof path === 'string') paths.add(path.split('#')[0]);
  const visitOwners = (value: unknown): void => {
    if (!value || typeof value !== 'object') return;
    for (const [key, child] of Object.entries(value)) {
      if (key === 'penpot_prefab_source' && typeof child === 'string')
        paths.add(child.split('#')[0]);
      else visitOwners(child);
    }
  };
  visitOwners(prepared);
  const owners = new Set([sourcePath, ...paths]);
  for (const owner of owners) {
    if (!owner.endsWith('.zui') || !owner.includes('/assets/')) continue;
    let source: unknown;
    try {
      source = parse(await readFile(containedPath(repoRoot, owner), 'utf8'));
    } catch {
      continue;
    }
    const assetRoot = `${owner.split('/assets/')[0]}/assets`;
    const visitMedia = (value: unknown, iconReference = false): void => {
      if (
        typeof value !== 'string' ||
        !value.trim() ||
        value === 'none' ||
        (value.includes('://') && !value.startsWith('res://'))
      )
        return;
      let path = value.replace(/^res:\/\//, '').split(/[?#]/, 1)[0]!;
      if (iconReference) {
        if (!path.startsWith('icons/')) path = `icons/${path}`;
        if (!/\.(png|jpe?g|webp|svg)$/i.test(path)) path += '.svg';
      } else if (!/\.(png|jpe?g|webp|svg)$/i.test(path)) {
        return;
      }
      paths.add(
        relative(
          repoRoot,
          containedPath(resolve(repoRoot, assetRoot), path),
        ).replaceAll('\\', '/'),
      );
    };
    const nodes = (source as Record<string, unknown>)['nodes'];
    for (const node of Object.values(
      (nodes ?? {}) as Record<string, Record<string, unknown>>,
    )) {
      const props = (node['props'] ?? {}) as Record<string, unknown>;
      const component = String(node['component']).toLowerCase();
      visitMedia(
        props['icon'] ??
          (component.includes('icon') ? props['source'] : undefined),
        true,
      );
      visitMedia(
        props['background_image'] ??
          (component === 'image'
            ? (props['source'] ?? props['image'] ?? props['value'])
            : undefined),
      );
    }
  }
  paths.delete(sourcePath);
  const result: DependencyFingerprint[] = [];
  for (const path of [...paths].sort()) {
    try {
      result.push({
        sourcePath: path,
        sha256: bytesSha256(await readFile(containedPath(repoRoot, path))),
      });
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
      result.push({ sourcePath: path, sha256: 'missing' });
    }
  }
  return result;
}

/**
 * Font manifests are authored as resource URIs in node styles.  Keep the
 * manifest and every binary it names in the same dependency fingerprint set
 * as imported .zui and image assets so a font change invalidates both
 * renderer receipts.  Resolution follows the source asset root first and the
 * runtime asset root as the checked-in fallback used by native capture.
 */
async function collectFontDependencies(
  repoRoot: string,
  sourcePath: string,
  prepared: Record<string, unknown>,
  paths: Set<string>,
): Promise<void> {
  const references = new Set<string>();
  const visit = (value: unknown): void => {
    if (typeof value === 'string') {
      const match = value.match(/^res:\/\/(.+\.font\.toml)(?:#.*)?$/i);
      if (match) references.add(match[1]);
      return;
    }
    if (!value || typeof value !== 'object') return;
    for (const child of Object.values(value as Record<string, unknown>))
      visit(child);
  };
  visit(prepared);
  if (!references.size) return;

  const roots = fontAssetRoots(repoRoot, sourcePath);
  for (const reference of references) {
    if (reference.includes('..') || reference.startsWith('/')) continue;
    const manifest = roots
      .map((root) => resolve(root, reference))
      .find((candidate) => {
        try {
          return (
            containedPath(repoRoot, candidate) === candidate &&
            existsSync(candidate)
          );
        } catch {
          return false;
        }
      });
    const expected = manifest
      ? relative(repoRoot, manifest).replaceAll('\\', '/')
      : `${relative(repoRoot, resolve(roots[0], reference)).replaceAll('\\', '/')}`;
    paths.add(expected);
    if (!manifest) continue;

    let document: unknown;
    try {
      document = parse(await readFile(manifest, 'utf8'));
    } catch {
      continue;
    }
    const visitSources = (value: unknown): void => {
      if (!value || typeof value !== 'object') return;
      for (const [key, child] of Object.entries(
        value as Record<string, unknown>,
      )) {
        if (key === 'source' && typeof child === 'string') {
          const binary = resolve(dirname(manifest), child);
          try {
            const contained = containedPath(repoRoot, binary);
            paths.add(relative(repoRoot, contained).replaceAll('\\', '/'));
          } catch {
            // The manifest itself remains fingerprinted; a missing/escaping
            // binary is reported by the normal `missing` dependency marker.
            paths.add(relative(repoRoot, binary).replaceAll('\\', '/'));
          }
        }
        visitSources(child);
      }
    };
    visitSources(document);
  }
}

function fontAssetRoots(repoRoot: string, sourcePath: string): string[] {
  const roots: string[] = [];
  const marker = '/assets/';
  const normalized = sourcePath.replaceAll('\\', '/');
  const assetsIndex = normalized.indexOf(marker);
  if (assetsIndex >= 0)
    roots.push(
      resolve(repoRoot, normalized.slice(0, assetsIndex + '/assets'.length)),
    );
  roots.push(resolve(repoRoot, 'zircon_runtime/assets'));
  return [...new Set(roots)];
}

export const dependenciesSha256 = (items: DependencyFingerprint[]): string =>
  canonicalSha256(
    [...items].sort((a, b) =>
      a.sourcePath < b.sourcePath ? -1 : a.sourcePath > b.sourcePath ? 1 : 0,
    ),
  );

export function dualEvidenceErrors(entry: CatalogEntry): string[] {
  const errors: string[] = [];
  if (
    !entry.cases?.length ||
    !entry.dependencySha256 ||
    !entry.penpotInputSha256 ||
    !entry.penpotInputPath ||
    !validHash(entry.sourceSha256) ||
    !validHash(entry.outputSha256) ||
    !validHash(entry.dependencySha256) ||
    !validHash(entry.penpotInputSha256) ||
    dependenciesSha256(entry.dependencyFingerprints ?? []) !==
      entry.dependencySha256 ||
    entry.dependencyFingerprints?.some((item) => !validHash(item.sha256))
  )
    return ['Missing dual-renderer catalog contract'];
  if (entry.status !== 'prepared') errors.push('Document preparation failed');
  const ids = new Set(entry.cases.map((item) => item.id));
  if (
    ids.size !== entry.cases.length ||
    entry.cases.some((item) => !item.id || item.sourcePath !== entry.sourcePath)
  )
    errors.push('Invalid or duplicate review cases');
  for (const reviewCase of entry.cases) {
    try {
      validateLayoutReviewCase(reviewCase);
    } catch (error) {
      errors.push(`Invalid review case ${reviewCase.id}: ${String(error)}`);
    }
    const presentation = reviewCase.data['workbenchPresentation'] as
      | { sourceFingerprint?: { sourcePath?: string; sha256?: string } }
      | undefined;
    if (presentation) {
      const fingerprint = presentation.sourceFingerprint;
      const currentSources = new Map<string, string>([
        [entry.sourcePath, entry.sourceSha256],
        ...(entry.dependencyFingerprints ?? []).map(
          ({ sourcePath, sha256 }) => [sourcePath, sha256] as [string, string],
        ),
      ]);
      if (
        !fingerprint?.sourcePath ||
        !validHash(fingerprint.sha256) ||
        currentSources.get(fingerprint.sourcePath) !== fingerprint.sha256
      )
        errors.push(
          `Workbench presentation source fingerprint is not current: ${reviewCase.id}`,
        );
    }
  }
  for (const renderer of ['penpotEvidence', 'engineEvidence'] as const) {
    const evidence = entry[renderer] ?? [];
    if (evidence.some((item) => !ids.has(item.caseId)))
      errors.push(`${renderer}: unexpected case`);
    for (const reviewCase of entry.cases) {
      const matches = evidence.filter((item) => item.caseId === reviewCase.id);
      const item = matches[0];
      if (
        matches.length !== 1 ||
        !item ||
        !item.screenshotPath ||
        !validHash(item.screenshotSha256)
      )
        errors.push(
          `${renderer}: ${item?.status === 'pending' ? 'pending' : 'missing or failed'} ${reviewCase.id}`,
        );
      else if (item.status !== 'passed')
        errors.push(
          item.status === 'pending'
            ? `${renderer}: pending ${reviewCase.id} (capture exists but visual or resource evidence is incomplete)`
            : `${renderer}: failed ${reviewCase.id}`,
        );
      else {
        if (
          renderer === 'engineEvidence' &&
          item.rendererKind !== requiredNativeRenderer(reviewCase)
        )
          errors.push(
            `${renderer}: wrong native host ${reviewCase.id}; expected ${requiredNativeRenderer(reviewCase)}`,
          );
        if (!item.rendererPath || !validHash(item.rendererSha256))
          errors.push(
            `${renderer}: missing renderer fingerprint ${reviewCase.id}`,
          );
        if (
          renderer === 'penpotEvidence' &&
          item.rendererKind !== 'penpot-official-frontend'
        )
          errors.push(
            `${renderer}: wrong Penpot renderer kind ${reviewCase.id}`,
          );
        if (
          renderer === 'penpotEvidence' &&
          browserRuntimeErrors(
            item.browserRuntime,
            selectedBrowserExecutablePath(),
          ).length
        )
          errors.push(
            `${renderer}: missing or invalid actual browser runtime receipt ${reviewCase.id}`,
          );
        if (
          renderer === 'penpotEvidence' &&
          (!item.captureProgramFingerprints?.length ||
            item.captureProgramFingerprints.some(
              ([path, hash]) => !path || !validHash(hash),
            ) ||
            missingPenpotCaptureProgramPaths(
              item.captureProgramFingerprints,
            ).length > 0)
        )
          errors.push(
            `${renderer}: missing capture program fingerprint ${reviewCase.id}`,
          );
        for (const kind of ['geometry', 'text'] as const)
          if (!item[`${kind}Path`] || !validHash(item[`${kind}Sha256`]))
            errors.push(
              `${renderer}: missing ${kind} evidence ${reviewCase.id}`,
            );
        if (
          item.sourceSha256 !== entry.sourceSha256 ||
          item.dependencySha256 !== entry.dependencySha256 ||
          item.caseSha256 !== caseSha256(reviewCase) ||
          (reviewCase.reviewHost &&
            item.reviewHostSha256 !== reviewCase.reviewHost.sha256) ||
          (renderer === 'penpotEvidence' &&
            item.inputSha256 !== entry.penpotInputSha256) ||
          (renderer === 'engineEvidence' &&
            item.inputSha256 !== undefined &&
            item.inputSha256 !== entry.sourceSha256)
        )
          errors.push(`${renderer}: stale ${reviewCase.id}`);
      }
    }
  }
  // Opening a popup must change what the user can see at the same scroll
  // position. An offscreen state mutation cannot count as visual evidence.
  for (const opened of entry.cases.filter((item) => item.state === 'open')) {
    const baseline = entry.cases.find(
      (item) =>
        item.state ===
          (opened.scrollPosition === 'end' ? 'scroll-after' : 'default') &&
        item.viewport.width === opened.viewport.width &&
        item.viewport.height === opened.viewport.height &&
        item.dpi === opened.dpi &&
        item.locale === opened.locale &&
        item.host === opened.host &&
        item.scrollPosition === opened.scrollPosition &&
        item.reviewHost?.sha256 === opened.reviewHost?.sha256 &&
        canonicalSha256(item.data) === canonicalSha256(opened.data),
    );
    if (!baseline) continue;
    for (const renderer of ['penpotEvidence', 'engineEvidence'] as const) {
      const openedImage = entry[renderer]?.find(
        (item) => item.caseId === opened.id,
      );
      const baselineImage = entry[renderer]?.find(
        (item) => item.caseId === baseline.id,
      );
      if (
        openedImage?.status === 'passed' &&
        baselineImage?.status === 'passed' &&
        validHash(openedImage.screenshotSha256) &&
        openedImage.screenshotSha256 === baselineImage.screenshotSha256
      )
        errors.push(
          `${renderer}: ${opened.id} is pixel-indistinguishable from ${baseline.id}`,
        );
    }
  }
  return errors;
}

const validHash = (value: unknown): value is string =>
  typeof value === 'string' && /^[0-9a-f]{64}$/.test(value);

export const renderEvidenceSha256 = (
  evidence: LayoutRenderEvidence[] = [],
): string =>
  canonicalSha256(
    [...evidence].sort((a, b) =>
      a.caseId < b.caseId ? -1 : a.caseId > b.caseId ? 1 : 0,
    ),
  );

export function caseReviewErrors(
  entry: CatalogEntry,
  reviews: LayoutCaseReview[] = [],
): string[] {
  const errors: string[] = [];
  const cases = entry.cases ?? [];
  for (const item of cases) {
    const matches = reviews.filter((review) => review.caseId === item.id);
    const review = matches[0];
    if (matches.length !== 1 || !review || !review.observations?.trim()) {
      errors.push(`Missing individual visual conclusion: ${item.id}`);
      continue;
    }
    if (
      review.status !== 'accepted' ||
      !Number.isFinite(review.maxGeometryDeltaPx) ||
      review.maxGeometryDeltaPx < 0 ||
      review.maxGeometryDeltaPx > 1 ||
      review.textMatches !== true ||
      review.visibilityMatches !== true
    )
      errors.push(`Case requires revision: ${item.id}`);
  }
  if (
    reviews.some((review) => !cases.some((item) => item.id === review.caseId))
  )
    errors.push('Unexpected case review');
  return errors;
}

export function currentDualReview(entry: CatalogEntry): boolean {
  const review = entry.review;
  // A retained-host pending capture is current provenance, but it is never a
  // current visual review, including when an older needs_revision record is
  // still attached to the entry.
  if (
    [...(entry.penpotEvidence ?? []), ...(entry.engineEvidence ?? [])].some(
      (evidence) => evidence.status === 'pending',
    )
  )
    return false;
  if (
    !review ||
    review.contract !== 'dual-renderer-v1' ||
    review.sourceSha256 !== entry.sourceSha256 ||
    review.outputSha256 !== entry.outputSha256 ||
    review.dependencySha256 !== entry.dependencySha256 ||
    review.penpotInputSha256 !== entry.penpotInputSha256 ||
    review.casesSha256 !== casesSha256(entry.cases ?? [])
  )
    return false;
  if (
    review.status === 'accepted' &&
    (dualEvidenceErrors(entry).length ||
      caseReviewErrors(entry, review.caseReviews).length)
  )
    return false;
  if (
    review.penpotEvidenceSha256 !==
      renderEvidenceSha256(entry.penpotEvidence) ||
    review.engineEvidenceSha256 !== renderEvidenceSha256(entry.engineEvidence)
  )
    return false;
  return (['penpot', 'engine'] as const).every((renderer) =>
    (entry[`${renderer}Evidence`] ?? []).every(
      (evidence) =>
        review[`${renderer}ScreenshotHashes`]?.[evidence.caseId] ===
        evidence.screenshotSha256,
    ),
  );
}

export async function verifyCurrentFiles(
  entry: CatalogEntry,
  repoRoot: string,
  catalogRoot: string,
  accepted: boolean,
): Promise<void> {
  await verifyWorkbenchManagedInputCases(entry.cases ?? []);
  const verified = new Map<string, string>();
  const check = async (
    root: string,
    path: string,
    hash: string,
  ): Promise<void> => {
    if (!validHash(hash)) throw new Error(`Invalid file fingerprint: ${path}`);
    const absolute = containedPath(root, path);
    const actual = verified.get(absolute) ?? (await fileSha256(absolute));
    if (actual !== hash)
      throw new Error(`File changed after validation: ${path}`);
    verified.set(absolute, actual);
  };
  await check(repoRoot, entry.sourcePath, entry.sourceSha256);
  await check(catalogRoot, entry.outputPath, entry.outputSha256);
  if (entry.penpotInputPath && entry.penpotInputSha256)
    await check(catalogRoot, entry.penpotInputPath, entry.penpotInputSha256);
  for (const reviewCase of entry.cases ?? []) {
    if (isWorkbenchStateReviewCase(reviewCase)) {
      const presentation = reviewCase.data['workbenchPresentation'] as
        | { sourceFingerprint?: { sourcePath?: unknown; sha256?: unknown } }
        | undefined;
      const sourceFingerprint = presentation?.sourceFingerprint;
      if (
        typeof sourceFingerprint?.sourcePath !== 'string' ||
        typeof sourceFingerprint.sha256 !== 'string'
      )
        throw new Error(
          `Missing workbench presentation source fingerprint: ${reviewCase.id}`,
        );
      await check(
        repoRoot,
        sourceFingerprint.sourcePath,
        sourceFingerprint.sha256,
      );
      if (
        ![
          [entry.sourcePath, entry.sourceSha256] as [string, string],
          ...(entry.dependencyFingerprints ?? []).map(
            ({ sourcePath, sha256 }) => [sourcePath, sha256] as [string, string],
          ),
        ].some(
          ([path, hash]) =>
            path === sourceFingerprint.sourcePath &&
            hash === sourceFingerprint.sha256,
        )
      )
        throw new Error(
          `Workbench presentation source is not a catalog dependency: ${sourceFingerprint.sourcePath}`,
        );
    }
    if (reviewCase.reviewHost)
      await check(
        catalogRoot,
        reviewCase.reviewHost.path,
        reviewCase.reviewHost.sha256,
      );
  }
  for (const dependency of entry.dependencyFingerprints ?? []) {
    if (!accepted && dependency.sha256 === 'missing') {
      try {
        await readFile(containedPath(repoRoot, dependency.sourcePath));
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === 'ENOENT') continue;
        throw error;
      }
      throw new Error(
        `Missing dependency appeared after capture: ${dependency.sourcePath}`,
      );
    }
    await check(repoRoot, dependency.sourcePath, dependency.sha256);
  }
  if (
    dependenciesSha256(entry.dependencyFingerprints ?? []) !==
    entry.dependencySha256
  )
    throw new Error('Dependency fingerprint differs');
  const evidence = [
    ...(entry.penpotEvidence ?? []),
    ...(entry.engineEvidence ?? []),
  ];
  if (!evidence.length)
    throw new Error(
      'Review requires capture evidence or an explicit failed capture',
    );
  if (accepted) {
    const errors = dualEvidenceErrors(entry);
    if (errors.length) throw new Error(errors.join('; '));

    // Per-case screenshots are evidence artifacts; accepted delivery also
    // requires both renderer-facing primary images to be present and nonempty.
    for (const [label, path] of [
      ['engine primary preview', entry.previewPath],
      ['Penpot primary preview', entry.penpotPreviewPath],
    ] as const) {
      if (!path) throw new Error(`Missing ${label} path`);
      let bytes: Buffer;
      try {
        bytes = await readFile(containedPath(catalogRoot, path));
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code === 'ENOENT')
          throw new Error(`Missing ${label}: ${path}`, { cause: error });
        throw error;
      }
      if (!bytes.length) throw new Error(`Empty ${label}: ${path}`);
    }
  }
  for (const item of evidence) {
    const reviewCase = entry.cases?.find(
      (candidate) => candidate.id === item.caseId,
    );
    if (item.status === 'pending' && !item.pendingReason?.trim())
      throw new Error(`Pending evidence has no explanation: ${item.caseId}`);
    for (const kind of ['screenshot', 'geometry', 'text'] as const) {
      const path = item[`${kind}Path`];
      const hash = item[`${kind}Sha256`];
      if (!path && !hash && item.status === 'failed' && item.error?.trim())
        continue;
      if (!path && !hash && kind !== 'screenshot' && !accepted) continue;
      if (!path || !hash)
        throw new Error(`Incomplete ${kind} evidence: ${item.caseId}`);
      await check(catalogRoot, path, hash);
      if (
        accepted &&
        kind === 'geometry' &&
        reviewCase &&
        isWorkbenchStateReviewCase(reviewCase)
      ) {
        const geometry = JSON.parse(
          await readFile(containedPath(catalogRoot, path), 'utf8'),
        ) as {
          sourceIdentityProvenance?: {
            runtimeLoadedSources?: {
              files?: Array<{
                sourcePath?: unknown;
                sha256?: unknown;
                physicalPath?: unknown;
              }>;
            };
          };
        };
        const loadedFiles =
          geometry.sourceIdentityProvenance?.runtimeLoadedSources?.files;
        if (!Array.isArray(loadedFiles) || !loadedFiles.length)
          throw new Error(`Missing loaded source files: ${item.caseId}`);
        for (const loadedFile of loadedFiles) {
          if (
            typeof loadedFile.sourcePath !== 'string' ||
            typeof loadedFile.sha256 !== 'string' ||
            typeof loadedFile.physicalPath !== 'string'
          )
            throw new Error(`Malformed loaded source file: ${item.caseId}`);
          await check(repoRoot, loadedFile.sourcePath, loadedFile.sha256);
          let currentPhysicalPath: string;
          let recordedPhysicalPath: string;
          try {
            currentPhysicalPath = realpathSync(
              containedPath(repoRoot, loadedFile.sourcePath),
            );
            recordedPhysicalPath = realpathSync(loadedFile.physicalPath);
          } catch {
            throw new Error(
              `Loaded source physical path is unavailable: ${item.caseId} ${loadedFile.sourcePath}`,
            );
          }
          const currentComparable =
            process.platform === 'win32'
              ? currentPhysicalPath.toLowerCase()
              : currentPhysicalPath;
          const recordedComparable =
            process.platform === 'win32'
              ? recordedPhysicalPath.toLowerCase()
              : recordedPhysicalPath;
          if (currentComparable !== recordedComparable)
            throw new Error(
              `Loaded source physical path differs from current catalog source: ${item.caseId} ${loadedFile.sourcePath}`,
            );
        }
      }
    }
    if (item.rendererPath || accepted) {
      if (!item.rendererPath || !item.rendererSha256)
        throw new Error(`Missing renderer fingerprint: ${item.caseId}`);
      const path = rendererFilePath(repoRoot, item.rendererPath);
      await check(dirname(path), basename(path), item.rendererSha256);
    }
    if (item.browserRuntime) {
      const runtimeErrors = browserRuntimeErrors(
        item.browserRuntime,
        selectedBrowserExecutablePath(),
      );
      if (runtimeErrors.length)
        throw new Error(runtimeErrors.join('; '));
      await check(
        dirname(item.browserRuntime.executablePath),
        basename(item.browserRuntime.executablePath),
        item.browserRuntime.executableSha256,
      );
    }
    for (const [path, hash] of item.runtimeAssetFingerprints ?? []) {
      const absolute = isAbsolute(path) ? path : containedPath(repoRoot, path);
      await check(dirname(absolute), basename(absolute), hash);
    }
    for (const [path, hash] of item.captureProgramFingerprints ?? [])
      await check(repoRoot, path, hash);
  }
  if (accepted) {
    const catalogInputDocument = entry.penpotInputPath
      ? parseZuiDocument(
          await readFile(
            containedPath(catalogRoot, entry.penpotInputPath),
            'utf8',
          ),
        ).document
      : undefined;
    const preparedDocument = catalogInputDocument
      ? reviewHost(catalogInputDocument) ?? catalogInputDocument
      : undefined;
    if (
      catalogInputDocument &&
      (entry.sourceKind === 'component' || entry.sourceKind === 'view')
    ) {
      const missing = sourceScenarioCoverageErrors(
        catalogInputDocument,
        entry.cases ?? [],
      );
      if (missing.length) throw new Error(missing.join('; '));
    }
    for (const reviewCase of entry.cases ?? []) {
      const load = async (
        items: LayoutRenderEvidence[],
        kind: 'geometry' | 'text',
      ) => {
        const item = items.find((item) => item.caseId === reviewCase.id)!;
        return JSON.parse(
          await readFile(
            containedPath(catalogRoot, item[`${kind}Path`]!),
            'utf8',
          ),
        );
      };
      const penpotGeometry = await load(entry.penpotEvidence!, 'geometry');
      const engineGeometry = await load(entry.engineEvidence!, 'geometry');
      const penpotRender = entry.penpotEvidence!.find(
        (item) => item.caseId === reviewCase.id,
      )!;
      const engineRender = entry.engineEvidence!.find(
        (item) => item.caseId === reviewCase.id,
      )!;
      const workbenchPresentationSourceFingerprint = isWorkbenchStateReviewCase(
        reviewCase,
      )
        ? (
            reviewCase.data['workbenchPresentation'] as
              | { sourceFingerprint?: { sourcePath?: unknown; sha256?: unknown } }
              | undefined
          )?.sourceFingerprint
        : undefined;
      const authoredSources: Array<[string, string]> = [
        [entry.sourcePath, entry.sourceSha256],
        ...(entry.dependencyFingerprints ?? []).map(
          ({ sourcePath, sha256 }) => [sourcePath, sha256] as [string, string],
        ),
      ];
      if (
        typeof workbenchPresentationSourceFingerprint?.sourcePath === 'string' &&
        typeof workbenchPresentationSourceFingerprint.sha256 === 'string'
      )
        authoredSources.push([
          workbenchPresentationSourceFingerprint.sourcePath,
          workbenchPresentationSourceFingerprint.sha256,
        ]);
      const runtimeAssets: RendererRuntimeAssets = {
        penpot: penpotRender.runtimeAssetFingerprints,
        engine: engineRender.runtimeAssetFingerprints,
        authoredSources,
        engineCapturePrograms: engineRender.captureProgramFingerprints,
        requireRuntimeProvenance: isWorkbenchStateReviewCase(reviewCase),
      };
      const sourceInventoryErrors: string[] = [];
      if (isWorkbenchStateReviewCase(reviewCase)) {
        runtimeAssets.expectedHostOverlayIds =
          expectedWorkbenchOverlayIds(reviewCase);
        if (!preparedDocument) {
          sourceInventoryErrors.push(
            'Source-derived workbench inventory is unavailable without Penpot input',
          );
        } else {
          try {
            const caseDocument = reviewDocumentForCase(
              preparedDocument,
              reviewCase as Parameters<typeof reviewDocumentForCase>[1],
            );
            const sourceInventory = deriveSourceRenderInventory(
              caseDocument,
              reviewCase,
              [
                {
                  sourcePath: entry.sourcePath,
                  sha256: entry.sourceSha256,
                },
                ...(entry.dependencyFingerprints ?? []).map(
                  ({ sourcePath, sha256 }) => ({ sourcePath, sha256 }),
                ),
              ],
            );
            sourceInventoryErrors.push(...sourceInventory.errors);
            runtimeAssets.expectedSemanticNodes =
              sourceInventory.semanticNodeIdentities;
            runtimeAssets.expectedResourceUses = sourceInventory.resourceUses;
            const sourceClosure = deriveExpectedRuntimeSourceFiles(
              caseDocument,
              [
                entry.sourcePath,
                ...(typeof reviewCase.themeSourcePath === 'string'
                  ? [reviewCase.themeSourcePath]
                  : []),
              ],
              authoredSources.map(([sourcePath, sha256]) => ({
                sourcePath,
                sha256,
              })),
            );
            sourceInventoryErrors.push(...sourceClosure.errors);
            runtimeAssets.expectedSourceClosure = sourceClosure.sources;
          } catch (error) {
            sourceInventoryErrors.push(
              `Cannot derive case-projected source inventory: ${error instanceof Error ? error.message : String(error)}`,
            );
          }
        }
      }
      const result = compareSemanticGeometry(
        penpotGeometry,
        engineGeometry,
        reviewCase,
        runtimeAssets,
      );
      result.errors.push(...sourceInventoryErrors);
      if (preparedDocument)
        for (const [renderer, geometry] of [
          ['penpot', penpotGeometry],
          ['engine', engineGeometry],
        ] as const)
          result.errors.push(
            ...sourceCellCoverageErrors(preparedDocument, geometry).map(
              (error) => `${renderer}: ${error}`,
            ),
          );
      result.errors.push(
        ...compareTextEvidence(
          await load(entry.penpotEvidence!, 'text'),
          await load(entry.engineEvidence!, 'text'),
          penpotGeometry,
          reviewCase,
          runtimeAssets,
          engineGeometry,
        ),
      );
      if (
        penpotGeometry.layout?.semanticNodes?.some(
          (node: { visible: boolean; text?: string }) =>
            Boolean(node.text?.trim()) &&
            isSemanticNodePaintVisible(node, penpotGeometry),
        )
      )
        for (const renderer of ['penpotEvidence', 'engineEvidence'] as const) {
          const item = entry[renderer]!.find(
            (item) => item.caseId === reviewCase.id,
          )!;
          if (
            !item.runtimeAssetFingerprints?.some(([path]) =>
              /\.(ttf|otf|woff2?|ttc)$/i.test(path),
            )
          )
            result.errors.push(
              `${renderer}: missing actual font file provenance`,
            );
        }
      if (result.errors.length)
        throw new Error(`${reviewCase.id}: ${result.errors.join('; ')}`);
    }
  }
}

export function selectedBrowserExecutablePath(): string {
  const candidates = [
    process.env['PLAYWRIGHT_CHROMIUM_EXECUTABLE'],
    process.env['ProgramFiles']
      ? resolve(process.env['ProgramFiles'], 'Google/Chrome/Application/chrome.exe')
      : undefined,
    process.env['ProgramFiles(x86)']
      ? resolve(
          process.env['ProgramFiles(x86)'],
          'Microsoft/Edge/Application/msedge.exe',
        )
      : undefined,
  ];
  return resolve(
    candidates.find((candidate) => Boolean(candidate && existsSync(candidate))) ??
      chromium.executablePath(),
  );
}

export function rendererFilePath(repoRoot: string, path: string): string {
  const error = () =>
    new Error(
      `Renderer path is outside repository and approved build roots: ${path}`,
    );
  if (!path.trim() || path.split(/[\\/]/).some((part) => part === '.' || part === '..'))
    throw error();
  const repository = resolve(repoRoot);
  const candidate = resolve(repository, path);
  const repositoryRelative = relative(repository, candidate);
  const insideRepository =
    !isAbsolute(repositoryRelative) &&
    repositoryRelative !== '..' &&
    !/^\.\.[/\\]/.test(repositoryRelative);
  const isRepositoryStorageAlias = (value: string): boolean =>
    value
      .split(/[\\/]/)
      .some((part) =>
        ['target', 'targets', 'cargo-targets', 'zirconbuilds'].includes(
          part.toLowerCase(),
        ),
      );
  if (insideRepository) {
    if (isRepositoryStorageAlias(repositoryRelative)) throw error();
    try {
      const physicalRepository = realpathSync(repository);
      const physicalCandidate = realpathSync(candidate);
      const physicalRelative = relative(physicalRepository, physicalCandidate);
      if (
        isAbsolute(physicalRelative) ||
        physicalRelative === '..' ||
        /^\.\.[/\\]/.test(physicalRelative)
      )
        throw error();
      return physicalCandidate;
    } catch (cause) {
      if (cause instanceof Error && cause.message.startsWith('Renderer path'))
        throw cause;
      throw error();
    }
  }

  const absolute = resolve(path);
  const match = absolute.match(/^([DEF]):\\cargo-targets\\/i);
  if (!isAbsolute(path) || !match) throw error();
  const approvedRoot = resolve(`${match[1]}:\\cargo-targets`);
  const rootRelative = relative(approvedRoot, absolute);
  if (
    isAbsolute(rootRelative) ||
    rootRelative === '..' ||
    /^\.\.[/\\]/.test(rootRelative) ||
    rootRelative
      .split(/[\\/]/)
      .some((part) =>
        ['targets', 'cargo-targets', 'zirconbuilds'].includes(
          part.toLowerCase(),
        ),
      )
  )
    throw error();
  try {
    const physicalRoot = realpathSync(approvedRoot);
    const physical = realpathSync(absolute);
    const physicalRelative = relative(physicalRoot, physical);
    if (
      physicalRoot.toLowerCase() !== approvedRoot.toLowerCase() ||
      isAbsolute(physicalRelative) ||
      physicalRelative === '..' ||
      /^\.\.[/\\]/.test(physicalRelative)
    )
      throw error();
    return physical;
  } catch (cause) {
    if (cause instanceof Error && cause.message.startsWith('Renderer path'))
      throw cause;
    throw error();
  }
}

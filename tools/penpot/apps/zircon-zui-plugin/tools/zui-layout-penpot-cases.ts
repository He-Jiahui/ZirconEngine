import { resolvePluginBundlePath } from './zui-layout-penpot-session';
import { resolveLayoutCatalogPath } from './zui-layout-paths';
import assert from 'node:assert/strict';
import { readFile, realpath, writeFile } from 'node:fs/promises';
import { isAbsolute, relative, resolve, sep } from 'node:path';
import type { FrameLocator, Locator, Page } from 'playwright';
import type { CatalogEntry } from './zui-layout-catalog';
import {
  bytesSha256,
  containedPath,
  fileSha256,
  selectedBrowserExecutablePath,
} from './zui-layout-evidence';
import { screenshotPreview } from './zui-layout-screenshot';
import { requestPreviewLayoutAudit } from './penpot-preview-layout-audit';
import { clickAndWaitForDownload, downloadPath } from './zui-layout-download';
import { currentCaptureProgram } from './zui-layout-capture-provenance';
import { capturePenpotRuntimeReceipts } from './zui-layout-penpot-receipts';
import type { FontResourceRecord } from './zui-layout-font-resources';
import {
  semanticTextIdentity,
  mapMeasuredTexts,
} from './zui-layout-text-mapping';
import type { RenderedLayoutAudit } from '../src/penpot-render-layout';
import {
  auditTextStructure,
  type MeasuredText,
  type TextStructureNode,
} from './zui-layout-text-structure';
import { parseZuiDocument, type ZuiDocument } from '../src/bridge/zui-document';
import { reviewDocumentForCase } from '../src/bridge/zui-review-case';
import { reviewHost } from '../src/bridge/zui-review-host';
import {
  deriveSourceRenderInventory,
  type SourceRenderInventory,
} from './zui-layout-source-render-inventory';
import { isSemanticNodePaintVisible } from './zui-layout-semantic-parity';
import { browserRuntimeErrors } from './zui-layout-browser-runtime';
import {
  buildBrowserRuntimeLoadedSourcesAudit,
  capturePreparedEditorTokenReceipt,
  capturePreparedWorkbenchLocaleAudit,
  deriveExpectedRuntimeSourceFiles,
  expectedWorkbenchRuntimeDocumentPaths,
  type BrowserRuntimeSourceFile,
  type RuntimeSourceFingerprint,
} from './zui-layout-runtime-provenance';
import {
  caseSha256,
  isWorkbenchStateReviewCase,
  type LayoutRenderEvidence,
  type LayoutReviewCase,
  type BrowserRuntimeRecord,
} from './zui-layout-review-contract';

export interface LayoutAuditSnapshot {
  semanticNodes: NonNullable<RenderedLayoutAudit['semanticNodes']>;
  totalNodes: number;
  checkedNodes: number;
  overflowCount: number;
  invalidGeometryCount: number;
  overflowNodes: string;
  invalidNodes: string;
  overflowDetails: string;
  invalidDetails: string;
}

interface RawPlatformFont {
  familyName: string;
  postScriptName?: string;
  glyphCount: number;
}

interface RawMeasuredText extends MeasuredText {
  fontFamily: string;
  fontSize: string;
  fontWeight: string;
  lineHeight: string;
  letterSpacing: string;
  lineTexts: string[];
  fontSources: Array<{ familyName: string; urls: string[] }>;
  fonts: RawPlatformFont[];
}

function hostOverlayAuditForCase(reviewCase: LayoutReviewCase): {
  audit: { complete: boolean; windows: Array<{ windowId: string }> };
  pendingReason?: string;
} {
  if (!isWorkbenchStateReviewCase(reviewCase))
    return { audit: { complete: false, windows: [] } };
  const presentation = reviewCase.data['workbenchPresentation'] as
    { layout?: { floating_windows?: unknown } } | undefined;
  const windows = presentation?.layout?.floating_windows;
  if (!Array.isArray(windows))
    return {
      audit: { complete: false, windows: [] },
      pendingReason: 'Workbench floating-window inventory is unavailable',
    };
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
  if (ids.some((id) => !id?.trim()) || new Set(ids).size !== ids.length)
    return {
      audit: { complete: false, windows: [] },
      pendingReason: 'Workbench floating-window identities are invalid',
    };
  if (ids.length)
    return {
      audit: {
        complete: false,
        windows: ids.map((windowId) => ({ windowId: windowId! })),
      },
      pendingReason:
        'Visible floating windows require a factual host overlay capture',
    };
  return { audit: { complete: true, windows: [] } };
}

async function strictPenpotTextEvidence(
  textPath: string,
  reviewCase: LayoutReviewCase,
  nodes: LayoutAuditSnapshot['semanticNodes'],
  fontResources: FontResourceRecord[] | undefined,
): Promise<{
  document: Record<string, unknown>;
  pairs: Array<[string, string]>;
  pendingReasons: string[];
}> {
  const pendingReasons: string[] = [];
  const raw = JSON.parse(await readFile(textPath, 'utf8')) as {
    fontAudit?: { loaded?: boolean };
    nativeSvgAudit?: unknown;
    texts?: RawMeasuredText[];
  };
  if (raw.fontAudit?.loaded !== true)
    pendingReasons.push('Penpot document font audit is incomplete');
  if (!Array.isArray(raw.texts))
    pendingReasons.push('Penpot text measurements are missing');
  if (!fontResources?.length)
    pendingReasons.push(
      'Loaded Penpot font files have no URL/path/SHA receipts',
    );

  const measured = Array.isArray(raw.texts) ? raw.texts : [];
  const mapped = mapMeasuredTexts(
    nodes.map((node) => ({
      ...node,
      sourcePath: node.sourcePath ?? undefined,
      sourceNodeId: node.sourceNodeId ?? undefined,
    })),
    measured,
  );
  const penpotTexts: Array<Record<string, unknown>> = [];
  const usedPairs = new Map<string, string>();
  for (const node of nodes.filter(
    (item) =>
      item.text?.trim() &&
      isSemanticNodePaintVisible(item, {
        windowMetrics: { logicalSize: reviewCase.viewport },
      }),
  )) {
    const identity = semanticTextIdentity({
      nodeId: node.nodeId,
      sourcePath: node.sourcePath ?? undefined,
      sourceNodeId: node.sourceNodeId ?? undefined,
      instancePath: node.instancePath,
    });
    const fragments = mapped.get(identity) ?? [];
    if (fragments.length !== 1) {
      pendingReasons.push(
        `Penpot text does not resolve once to semantic node ${node.nodeId}`,
      );
      continue;
    }
    const text = fragments[0] as RawMeasuredText;
    const fonts: Array<Record<string, unknown>> = [];
    for (const face of text.fonts ?? []) {
      const sources = (text.fontSources ?? []).filter(
        (source) =>
          source.familyName.toLocaleLowerCase() ===
          face.familyName.toLocaleLowerCase(),
      );
      const candidates = (fontResources ?? []).filter((resource) =>
        sources.some((source) => source.urls.includes(resource.url)),
      );
      const unique = new Map(
        candidates.map((resource) => [
          `${resource.path}\0${resource.sha256}`,
          resource,
        ]),
      );
      if (
        !face.familyName ||
        !face.postScriptName ||
        !Number.isFinite(face.glyphCount) ||
        face.glyphCount <= 0 ||
        unique.size !== 1
      ) {
        pendingReasons.push(
          `Penpot font face file proof is incomplete for ${node.nodeId}`,
        );
        continue;
      }
      const resource = [...unique.values()][0];
      fonts.push({
        familyName: face.familyName,
        postScriptName: face.postScriptName,
        glyphCount: face.glyphCount,
        resourcePath: resource.path,
        sha256: resource.sha256,
        sourceUrl: resource.url,
      });
      usedPairs.set(resource.path, resource.sha256);
    }
    if (!fonts.length)
      pendingReasons.push(
        `Penpot text has no file-backed actual font face: ${node.nodeId}`,
      );
    if (
      !text.fontFamily ||
      !text.fontSize ||
      !text.fontWeight ||
      !text.lineHeight ||
      !text.letterSpacing ||
      !Array.isArray(text.lineTexts) ||
      text.lineTexts.length !== text.lines.length ||
      !text.lines.length
    )
      pendingReasons.push(
        `Penpot text style or line evidence is incomplete: ${node.nodeId}`,
      );
    penpotTexts.push({
      shapeId: text.shapeId,
      ancestorShapeIds: text.ancestorShapeIds,
      text: text.text,
      fontFamily: text.fontFamily,
      fontSize: text.fontSize,
      fontWeight: text.fontWeight,
      lineHeight: text.lineHeight,
      letterSpacing: text.letterSpacing,
      lineTexts: text.lineTexts,
      lines: text.lines,
      fonts,
    });
  }
  const document = {
    case: reviewCase,
    caseSha256: caseSha256(reviewCase),
    coordinateSpace: 'logical',
    fontAudit: {
      loaded: raw.fontAudit?.loaded === true && pendingReasons.length === 0,
    },
    nativeSvgAudit: raw.nativeSvgAudit,
    texts: penpotTexts,
  };
  return {
    document,
    pairs: [...usedPairs.entries()].sort(([a], [b]) => a.localeCompare(b)),
    pendingReasons,
  };
}

/** Scroll clipping can hide offscreen glyphs, not justify visible sibling collisions. */
export function blockingPenpotTextFindings(
  nodes: TextStructureNode[],
  texts: MeasuredText[],
  viewport: { width: number; height: number },
): ReturnType<typeof auditTextStructure> {
  return auditTextStructure(nodes, texts, viewport).filter(
    (finding) =>
      finding.kind === 'node-outside-parent' ||
      finding.kind === 'text-outside-node' ||
      finding.kind === 'missing-measurement' ||
      (finding.kind === 'text-overlap' &&
        (finding.semanticContext === 'unbounded' ||
          finding.semanticContext === 'scroll-content')) ||
      (finding.semanticContext === 'unbounded' &&
        finding.kind === 'text-outside-viewport'),
  );
}

/**
 * A targeted recapture must not erase valid evidence for unselected cases.
 * Evidence remains current only when it came from the same product source,
 * Penpot input and renderer bundle as the new capture.
 */
export function retainCompatiblePenpotEvidence(
  existingEvidence: readonly LayoutRenderEvidence[] | undefined,
  selectedCaseIds: ReadonlySet<string>,
  sourceSha256: string,
  inputSha256: string | undefined,
  rendererSha256: string,
): LayoutRenderEvidence[] {
  return (existingEvidence ?? []).filter(
    (item) =>
      typeof item.caseId === 'string' &&
      item.caseId.trim() !== '' &&
      !selectedCaseIds.has(item.caseId) &&
      item.sourceSha256 === sourceSha256 &&
      item.inputSha256 === inputSha256 &&
      item.rendererSha256 === rendererSha256,
  );
}

/** A targeted failure cannot erase unrelated current captures or invent failures for them. */
export function evidenceAfterPenpotFailure(
  entry: Pick<
    CatalogEntry,
    'cases' | 'sourceSha256' | 'penpotInputSha256' | 'dependencySha256'
  >,
  existingEvidence: readonly LayoutRenderEvidence[] | undefined,
  selectedCaseIds: ReadonlySet<string>,
  captureStarted: boolean,
  rendererSha256: string,
  programFingerprints: Array<[string, string]>,
  error: string,
): LayoutRenderEvidence[] {
  const retained = retainCompatiblePenpotEvidence(
    existingEvidence,
    selectedCaseIds,
    entry.sourceSha256,
    entry.penpotInputSha256,
    rendererSha256,
  );
  const selected = (entry.cases ?? []).filter((item) =>
    selectedCaseIds.has(item.id),
  );
  return [
    ...retained,
    ...selected.map((reviewCase) => {
      const matches = captureStarted
        ? (existingEvidence ?? []).filter(
            (item) =>
              item.caseId === reviewCase.id &&
              item.sourceSha256 === entry.sourceSha256 &&
              item.inputSha256 === entry.penpotInputSha256 &&
              item.dependencySha256 === entry.dependencySha256 &&
              item.rendererSha256 === rendererSha256 &&
              item.caseSha256 === caseSha256(reviewCase) &&
              currentCaptureProgram([item], programFingerprints),
          )
        : [];
      if (matches.length === 1) return matches[0];
      return {
        caseId: reviewCase.id,
        status: 'failed' as const,
        screenshotPath: '',
        screenshotSha256: '',
        sourceSha256: entry.sourceSha256,
        inputSha256: entry.penpotInputSha256,
        dependencySha256: entry.dependencySha256 ?? '',
        caseSha256: caseSha256(reviewCase),
        rendererSha256,
        rendererPath:
          resolvePluginBundlePath(),
        rendererKind: 'penpot-official-frontend',
        captureProgramFingerprints: programFingerprints,
        error,
      };
    }),
  ];
}

/** A partial recapture is evidence, but it cannot pass the whole item. */
export function penpotCasesComplete(
  entry: Pick<
    CatalogEntry,
    | 'cases'
    | 'penpotEvidence'
    | 'category'
    | 'name'
    | 'penpotPreviewPath'
    | 'sourceSha256'
    | 'dependencySha256'
    | 'penpotInputSha256'
  >,
  rendererSha256: string,
  captureProgramFingerprints: Array<[string, string]>,
): boolean {
  const cases = entry.cases ?? [];
  const evidence = entry.penpotEvidence ?? [];
  if (!cases.length || cases.length !== evidence.length) return false;
  return cases.every((reviewCase) => {
    const matches = evidence.filter((item) => item.caseId === reviewCase.id);
    const item = matches[0];
    return (
      matches.length === 1 &&
      item.status === 'passed' &&
      item.screenshotPath === penpotScreenshotPathForCase(entry, reviewCase) &&
      Boolean(item.screenshotSha256) &&
      item.sourceSha256 === entry.sourceSha256 &&
      item.dependencySha256 === entry.dependencySha256 &&
      item.inputSha256 === entry.penpotInputSha256 &&
      item.rendererSha256 === rendererSha256 &&
      item.caseSha256 === caseSha256(reviewCase) &&
      browserRuntimeErrors(item.browserRuntime, selectedBrowserExecutablePath())
        .length === 0 &&
      currentCaptureProgram([item], captureProgramFingerprints)
    );
  });
}

export function penpotScreenshotPathForCase(
  entry: Pick<
    CatalogEntry,
    'category' | 'name' | 'penpotPreviewPath' | 'cases'
  >,
  reviewCase: LayoutReviewCase,
): string {
  const directory = `${entry.category}/${entry.name}`;
  return reviewCase.id === entry.cases?.[0]?.id
    ? (entry.penpotPreviewPath ?? `${directory}/penpot.png`)
    : `${directory}/evidence/penpot-${reviewCase.id}.png`;
}

async function readPreparedRuntimeSourceFiles(
  repoRoot: string,
  expectedSources: RuntimeSourceFingerprint[],
): Promise<{ files: BrowserRuntimeSourceFile[]; errors: string[] }> {
  const files: BrowserRuntimeSourceFile[] = [];
  const errors: string[] = [];
  let canonicalRoot: string;
  try {
    canonicalRoot = await realpath(repoRoot);
  } catch (error) {
    return {
      files,
      errors: [
        `Cannot resolve repository root for browser source receipts: ${error instanceof Error ? error.message : String(error)}`,
      ],
    };
  }
  for (const expected of expectedSources) {
    try {
      const candidatePath = containedPath(repoRoot, expected.sourcePath);
      const physicalPath = await realpath(candidatePath);
      const relativePath = relative(canonicalRoot, physicalPath);
      if (
        isAbsolute(relativePath) ||
        relativePath === '..' ||
        relativePath.startsWith(`..${sep}`)
      )
        throw new Error('resolved source escapes the repository root');
      const bytes = await readFile(physicalPath);
      const sourceDocument = parseZuiDocument(bytes.toString('utf8')).document;
      const assetId = sourceDocument.asset?.id;
      if (typeof assetId !== 'string' || !assetId.trim())
        throw new Error('parsed source has no asset ID');
      const assetMarker = '/assets/';
      const markerIndex = expected.sourcePath.indexOf(assetMarker);
      if (markerIndex < 0)
        throw new Error('source path has no package assets root');
      const resourceUri = `res://${expected.sourcePath.slice(markerIndex + assetMarker.length)}`;
      const sha256 = bytesSha256(bytes);
      files.push({
        assetId,
        sourcePath: expected.sourcePath,
        resourceUri,
        physicalPath,
        sha256,
        catalogMatches: sha256 === expected.sha256,
        currentFileMatches: true,
      });
    } catch (error) {
      errors.push(
        `Cannot prove current prepared source ${expected.sourcePath}: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }
  return { files, errors };
}

export async function capturePenpotCases(options: {
  page: Page;
  pluginFrame: FrameLocator;
  statusPanel: Locator;
  pluginModal: Locator;
  previewBoardId: string;
  entry: CatalogEntry;
  writeEvidence: boolean;
  catalogRoot: string;
  pluginBundleSha256: string;
  captureProgramFingerprints: Array<[string, string]>;
  captureFontResources: () => Promise<Array<[string, string]>>;
  captureFontResourceRecords?: () => Promise<FontResourceRecord[]>;
  captureBrowserRuntime?: () => Promise<BrowserRuntimeRecord>;
  caseIds?: ReadonlySet<string>;
  reviewTimeoutMs?: number;
  exportTimeoutMs?: number;
  validateImage: (
    bytes: Buffer,
    reviewCase: LayoutReviewCase,
    layout: LayoutAuditSnapshot,
  ) => Promise<string | undefined>;
}): Promise<{
  primaryBytes: Buffer;
  primaryLayout: LayoutAuditSnapshot;
  evidence: LayoutRenderEvidence[];
}> {
  const { page, pluginFrame, statusPanel, pluginModal, entry } = options;
  assert.ok(entry.cases?.length, 'Missing review cases');
  const cases = entry.cases.filter(
    (reviewCase) => !options.caseIds || options.caseIds.has(reviewCase.id),
  );
  assert.ok(cases.length, 'Case filter selected no review cases');
  const selectedCaseIds = new Set(cases.map((reviewCase) => reviewCase.id));
  const retainedEvidence = retainCompatiblePenpotEvidence(
    entry.penpotEvidence,
    selectedCaseIds,
    entry.sourceSha256,
    entry.penpotInputSha256,
    options.pluginBundleSha256,
  );
  const evidence: LayoutRenderEvidence[] = [...retainedEvidence];
  // Keep partial failure evidence even if a later case cannot be rendered,
  // and keep already-captured cases when a caller intentionally selects only
  // one viewport at a time.
  entry.penpotEvidence = evidence;
  let primaryBytes: Buffer | undefined;
  let primaryLayout: LayoutAuditSnapshot | undefined;
  const errors: string[] = [];
  const session = await page.context().newCDPSession(page);
  const inputDocument = parseZuiDocument(
    await readFile(
      resolveLayoutCatalogPath(options.catalogRoot, entry.penpotInputPath ?? entry.outputPath),
      'utf8',
    ),
  ).document;
  const expected = inputDocument['penpot_original_source'];
  const preparedHostDocument = reviewHost(inputDocument) ?? inputDocument;
  assert.equal(typeof expected, 'string', 'Missing original runtime source');
  try {
    for (const [index, reviewCase] of cases.entries()) {
      const started = Date.now();
      const directory = `${entry.category}/${entry.name}`;
      const screenshotPath = penpotScreenshotPathForCase(entry, reviewCase);
      const item: LayoutRenderEvidence = {
        caseId: reviewCase.id,
        status: 'failed',
        screenshotPath: '',
        screenshotSha256: '',
        sourceSha256: entry.sourceSha256,
        dependencySha256: entry.dependencySha256 ?? '',
        caseSha256: caseSha256(reviewCase),
        rendererSha256: options.pluginBundleSha256,
        rendererPath:
          resolvePluginBundlePath(),
        rendererKind: 'penpot-official-frontend',
        captureProgramFingerprints: options.captureProgramFingerprints,
        inputSha256: entry.penpotInputSha256,
        ...(reviewCase.reviewHost
          ? { reviewHostSha256: reviewCase.reviewHost.sha256 }
          : {}),
      };
      evidence.push(item);
      try {
        let browserPendingReason: string | undefined;
        if (options.captureBrowserRuntime) {
          try {
            item.browserRuntime = await options.captureBrowserRuntime();
            const browserErrors = browserRuntimeErrors(
              item.browserRuntime,
              selectedBrowserExecutablePath(),
            );
            if (browserErrors.length)
              browserPendingReason = `Browser runtime receipt is invalid: ${browserErrors.join('; ')}`;
          } catch (error) {
            browserPendingReason = `Browser runtime receipt is unavailable: ${error instanceof Error ? error.message : String(error)}`;
          }
        } else {
          browserPendingReason = 'Browser runtime receipt is unavailable';
        }
        if (reviewCase.reviewHost)
          assert.equal(
            await fileSha256(
              containedPath(options.catalogRoot, reviewCase.reviewHost.path),
            ),
            reviewCase.reviewHost.sha256,
            'Review consumer host changed; regenerate the catalog',
          );
        await session.send('Emulation.setDeviceMetricsOverride', {
          // Capture the authored review viewport exactly.  The startup page
          // uses a larger workspace so Penpot can boot reliably, but a case
          // must never be enlarged here because that changes responsive
          // layout and invalidates the dual-end comparison.
          width: reviewCase.viewport.width,
          height: reviewCase.viewport.height,
          deviceScaleFactor: reviewCase.dpi,
          mobile: false,
        });
        assert.equal(
          await page.evaluate(() => window.devicePixelRatio),
          reviewCase.dpi,
        );
        await setReviewCase(
          pluginFrame,
          reviewCase,
          options.reviewTimeoutMs ?? 300_000,
        );
        const { layoutAudit } = await requestPreviewLayoutAudit(pluginFrame);
        const layout = readLayoutAudit(layoutAudit);
        if (reviewCase.reviewHost)
          assert.ok(
            layout.checkedNodes > 0 && layout.semanticNodes.length > 0,
            'Theme review requires audited semantic consumer nodes',
          );
        const bounds = layoutAudit.assetBounds;
        assert.ok(bounds, 'Missing requested preview bounds');
        assert.ok(
          Math.abs(bounds.width - reviewCase.viewport.width) <= 1,
          `preview width differs by more than one logical pixel: ${bounds.width}`,
        );
        assert.ok(
          Math.abs(bounds.height - reviewCase.viewport.height) <= 1,
          `preview height differs by more than one logical pixel: ${bounds.height}`,
        );
        const boardId = await statusPanel.getAttribute('data-preview-board-id');
        assert.ok(boardId, 'Missing Penpot preview board ID');
        await page.evaluate(() => document.fonts.ready);
        const geometryPath = `${directory}/evidence/penpot-${reviewCase.id}.geometry.json`;
        const authoredSources = [
          { sourcePath: entry.sourcePath, sha256: entry.sourceSha256 },
          ...(entry.dependencyFingerprints ?? [])
            .map((dependency) => ({
              sourcePath: dependency.sourcePath,
              sha256: dependency.sha256,
            }))
            .filter(({ sha256 }) => /^[0-9a-f]{64}$/.test(sha256)),
        ].filter(({ sha256 }) => /^[0-9a-f]{64}$/.test(sha256));
        let sourceInventory: SourceRenderInventory | undefined;
        let caseDocument: ZuiDocument | undefined;
        let browserTokenReceipt = capturePreparedEditorTokenReceipt(undefined);
        let browserLocaleAudit = capturePreparedWorkbenchLocaleAudit(
          undefined,
          reviewCase.locale,
        );
        const emptySourceAudit = buildBrowserRuntimeLoadedSourcesAudit(
          [],
          [],
          [],
          ['prepared workbench runtime source closure is unavailable'],
        );
        let browserSourceAudit = emptySourceAudit.audit;
        let runtimeTokenPending: string | undefined;
        let runtimeSourcePending: string | undefined;
        let localePending: string | undefined;
        if (isWorkbenchStateReviewCase(reviewCase)) {
          try {
            caseDocument = reviewDocumentForCase(
              preparedHostDocument,
              reviewCase as Parameters<typeof reviewDocumentForCase>[1],
            );
            sourceInventory = deriveSourceRenderInventory(
              caseDocument,
              reviewCase,
              authoredSources,
            );
            const preparedPresentation =
              caseDocument['penpot_review_workbench_presentation'];
            browserTokenReceipt = capturePreparedEditorTokenReceipt(
              caseDocument.tokens,
            );
            browserLocaleAudit = capturePreparedWorkbenchLocaleAudit(
              preparedPresentation,
              reviewCase.locale,
            );
            if (!browserTokenReceipt.receipt.complete)
              runtimeTokenPending = browserTokenReceipt.errors.join('; ');
            if (!browserLocaleAudit.complete)
              localePending =
                'Penpot prepared workbench locale does not match the case';

            const presentationSource =
              preparedPresentation &&
              typeof preparedPresentation === 'object' &&
              !Array.isArray(preparedPresentation)
                ? (preparedPresentation as Record<string, unknown>)[
                    'sourceFingerprint'
                  ]
                : undefined;
            const presentationFingerprint =
              presentationSource &&
              typeof presentationSource === 'object' &&
              !Array.isArray(presentationSource)
                ? (presentationSource as Record<string, unknown>)
                : undefined;
            const catalogSourceFingerprints: RuntimeSourceFingerprint[] = [
              ...authoredSources,
              ...(typeof presentationFingerprint?.['sourcePath'] === 'string' &&
              typeof presentationFingerprint['sha256'] === 'string'
                ? [
                    {
                      sourcePath: presentationFingerprint['sourcePath'],
                      sha256: presentationFingerprint['sha256'],
                    },
                  ]
                : []),
            ];
            const sourceClosure = deriveExpectedRuntimeSourceFiles(
              caseDocument,
              [
                entry.sourcePath,
                ...(typeof reviewCase.themeSourcePath === 'string'
                  ? [reviewCase.themeSourcePath]
                  : []),
              ],
              catalogSourceFingerprints,
            );
            const documentRoots =
              expectedWorkbenchRuntimeDocumentPaths(preparedPresentation);
            const currentSources = await readPreparedRuntimeSourceFiles(
              resolveLayoutCatalogPath(options.catalogRoot, '../..'),
              sourceClosure.sources,
            );
            const sealed = buildBrowserRuntimeLoadedSourcesAudit(
              sourceClosure.sources,
              currentSources.files,
              documentRoots.sourcePaths,
              [
                ...sourceClosure.errors,
                ...documentRoots.errors,
                ...currentSources.errors,
              ],
            );
            browserSourceAudit = sealed.audit;
            if (sealed.errors.length)
              runtimeSourcePending = [...new Set(sealed.errors)].join('; ');
          } catch (error) {
            sourceInventory = {
              semanticNodeIdentities: [],
              resourceUses: [],
              errors: [
                `Cannot derive case-projected source inventory: ${error instanceof Error ? error.message : String(error)}`,
              ],
            };
            runtimeSourcePending = `Cannot capture browser runtime source closure: ${error instanceof Error ? error.message : String(error)}`;
          }
        }
        const receipts = await capturePenpotRuntimeReceipts(
          page,
          boardId,
          layout.semanticNodes,
          authoredSources,
          reviewCase.viewport,
          sourceInventory?.resourceUses,
        );
        const semanticNodes = layout.semanticNodes.map((node) => {
          const style = receipts.styles.get(node.shapeId ?? '');
          const sourceIdentity = node as unknown as {
            sourcePath?: string | null;
            sourceNodeId?: string | null;
            instancePath?: string | null;
            parentSourcePath?: string | null;
            parentSourceNodeId?: string | null;
            parentInstancePath?: string | null;
            controlId?: string | null;
          };
          return {
            ...node,
            nodeId: node.nodeId,
            sourcePath: sourceIdentity.sourcePath ?? null,
            sourceNodeId: sourceIdentity.sourceNodeId ?? null,
            instancePath: sourceIdentity.instancePath ?? '',
            parentSourcePath: sourceIdentity.parentSourcePath ?? null,
            parentSourceNodeId: sourceIdentity.parentSourceNodeId ?? null,
            parentInstancePath: sourceIdentity.parentInstancePath ?? null,
            controlId: sourceIdentity.controlId ?? null,
            styleInventory: style ?? {
              version: 1,
              complete: false,
              properties: {
                foregroundColor: null,
                backgroundColor: null,
                borderColor: null,
                borderWidth: null,
                borderRadius: null,
                opacity: null,
                boxShadow: { complete: false, layers: [] },
              },
              reason: 'semantic shape has no final mounted style receipt',
            },
          };
        });
        const semanticIdentities = semanticNodes.map((node) =>
          JSON.stringify([
            node.sourcePath,
            node.sourceNodeId,
            node.instancePath,
          ]),
        );
        const semanticCoverageComplete = sourceInventory
          ? sourceInventory.errors.length === 0 &&
            new Set(semanticIdentities).size === semanticIdentities.length &&
            semanticIdentities.length ===
              sourceInventory.semanticNodeIdentities.length &&
            semanticIdentities.every((identity) =>
              sourceInventory!.semanticNodeIdentities.includes(identity),
            )
          : semanticNodes.length > 0 &&
            semanticNodes.every(
              (node) =>
                typeof node.sourcePath === 'string' &&
                !!node.sourcePath &&
                typeof node.sourceNodeId === 'string' &&
                !!node.sourceNodeId &&
                typeof node.instancePath === 'string' &&
                node.instancePath.length > 0,
            );
        const hostOverlay = hostOverlayAuditForCase(reviewCase);
        const geometryDocument = {
          case: reviewCase,
          caseSha256: caseSha256(reviewCase),
          coordinateSpace: 'logical',
          windowMetrics: {
            logicalSize: reviewCase.viewport,
            physicalSize: {
              width: reviewCase.viewport.width * reviewCase.dpi,
              height: reviewCase.viewport.height * reviewCase.dpi,
            },
          },
          semanticAudit: {
            complete: semanticCoverageComplete,
            expectedNodeCount:
              sourceInventory?.semanticNodeIdentities.length ??
              semanticNodes.length,
          },
          assetAudit: {
            complete:
              (!sourceInventory || sourceInventory.errors.length === 0) &&
              receipts.pendingReasons.every(
                (reason) => !reason.startsWith('media '),
              ),
            resources: receipts.media,
          },
          hostOverlayAudit: hostOverlay.audit,
          ...(isWorkbenchStateReviewCase(reviewCase)
            ? {
                consumedTokens: browserTokenReceipt.receipt,
                localeAudit: browserLocaleAudit,
              }
            : {}),
          sourceIdentityProvenance: {
            sourceMapFingerprint: null,
            sources: authoredSources.map(({ sourcePath, sha256 }) => [
              sourcePath,
              sha256,
            ]),
            ...(isWorkbenchStateReviewCase(reviewCase)
              ? { runtimeLoadedSources: browserSourceAudit }
              : {}),
          },
          layout: { semanticNodes },
        };
        const geometry = Buffer.from(
          `${JSON.stringify(geometryDocument, null, 2)}\n`,
        );
        if (options.writeEvidence)
          await writeFile(resolveLayoutCatalogPath(options.catalogRoot, geometryPath), geometry);
        item.geometryPath = geometryPath;
        item.geometrySha256 = bytesSha256(geometry);
        // Selection keeps Penpot's native frame active instead of a cached
        // thumbnail. Selection handles live outside the captured shape tree.
        await setPreviewSelection(pluginFrame, true);
        await pluginModal.evaluate((element) => {
          (element as HTMLElement).style.visibility = 'hidden';
        });
        let bytes: Buffer;
        let measuredTexts: MeasuredText[] | undefined;
        const textPath = `${directory}/evidence/penpot-${reviewCase.id}.text.json`;
        try {
          bytes = await screenshotPreview(
            page,
            boardId ?? options.previewBoardId,
            options.writeEvidence
              ? resolveLayoutCatalogPath(options.catalogRoot, screenshotPath)
              : undefined,
            bounds,
            layout.semanticNodes.filter(
              (node) =>
                node.text?.trim() &&
                isSemanticNodePaintVisible(node, {
                  windowMetrics: { logicalSize: reviewCase.viewport },
                }),
            ).length,
            session,
            options.writeEvidence
              ? resolveLayoutCatalogPath(options.catalogRoot, textPath)
              : undefined,
            (texts) => {
              measuredTexts = texts;
            },
          );
        } finally {
          await pluginModal.evaluate((element) => {
            (element as HTMLElement).style.visibility = '';
          });
        }
        item.screenshotPath = screenshotPath;
        item.screenshotSha256 = bytesSha256(bytes);
        const legacyFontPairs = await options.captureFontResources();
        const fontRecords = options.captureFontResourceRecords
          ? await options.captureFontResourceRecords()
          : undefined;
        const strictText = options.writeEvidence
          ? await strictPenpotTextEvidence(
              resolveLayoutCatalogPath(options.catalogRoot, textPath),
              reviewCase,
              layout.semanticNodes,
              fontRecords,
            )
          : {
              document: {},
              pairs: [],
              pendingReasons: [
                'Strict text evidence was not persisted for this capture',
              ],
            };
        if (options.writeEvidence) {
          const strictTextBytes = Buffer.from(
            `${JSON.stringify(strictText.document, null, 2)}\n`,
          );
          await writeFile(
            resolveLayoutCatalogPath(options.catalogRoot, textPath),
            strictTextBytes,
          );
          item.textPath = textPath;
          item.textSha256 = bytesSha256(strictTextBytes);
        }
        const runtimeAssetPairs = new Map<string, string>([
          ...strictText.pairs,
          ...receipts.runtimeAssetFingerprints,
          ...legacyFontPairs.filter(([path, hash]) =>
            strictText.pairs.some(
              (pair) => pair[0] === path && pair[1] === hash,
            ),
          ),
        ]);
        item.runtimeAssetFingerprints = [...runtimeAssetPairs.entries()].sort(
          ([a], [b]) => a.localeCompare(b),
        );
        if (options.writeEvidence) {
          item.geometryPath = geometryPath;
        }
        if (index === 0) {
          primaryBytes = bytes;
          primaryLayout = layout;
        }
        const pendingReason = await options.validateImage(
          bytes,
          reviewCase,
          layout,
        );
        assert.ok(measuredTexts, 'Missing native measured text bounds');
        const blockingText = blockingPenpotTextFindings(
          layout.semanticNodes,
          measuredTexts,
          reviewCase.viewport,
        );
        assert.equal(
          blockingText.length,
          0,
          `measured text boundary defects: ${JSON.stringify(blockingText.slice(0, 8))}`,
        );
        assert.equal(
          layout.overflowCount,
          0,
          `layout overflow: ${layout.overflowDetails}`,
        );
        assert.equal(
          layout.invalidGeometryCount,
          0,
          `invalid geometry: ${layout.invalidDetails}`,
        );
        await setPreviewSelection(pluginFrame, true);
        const download = await clickAndWaitForDownload(
          () =>
            page.waitForEvent('download', {
              timeout: options.exportTimeoutMs ?? 300_000,
            }),
          () =>
            pluginFrame
              .getByRole('button', { name: 'Export selected' })
              .click({ timeout: options.exportTimeoutMs ?? 300_000 }),
        );
        const path = await downloadPath(
          download,
          options.exportTimeoutMs ?? 300_000,
        );
        assert.ok(path, `missing case export: ${reviewCase.id}`);
        const exported = await readFile(path, 'utf8');
        assert.equal(
          exported,
          expected,
          `review case changed original runtime source: ${reviewCase.id}`,
        );
        if (options.writeEvidence)
          await writeFile(
            resolveLayoutCatalogPath(
              options.catalogRoot,
              `${directory}/evidence/penpot-${reviewCase.id}.roundtrip.zui`,
            ),
            exported,
          );
        const receiptPending = receipts.pendingReasons.length
          ? receipts.pendingReasons.join('; ')
          : undefined;
        const inventoryPending = sourceInventory?.errors.length
          ? sourceInventory.errors.join('; ')
          : undefined;
        const hostOverlayPending = hostOverlay.pendingReason;
        const textPending = strictText.pendingReasons.length
          ? strictText.pendingReasons.join('; ')
          : undefined;
        item.status =
          pendingReason ||
          receiptPending ||
          inventoryPending ||
          hostOverlayPending ||
          textPending ||
          browserPendingReason ||
          runtimeTokenPending ||
          runtimeSourcePending ||
          localePending
            ? 'pending'
            : 'passed';
        if (
          pendingReason ||
          receiptPending ||
          inventoryPending ||
          hostOverlayPending ||
          textPending ||
          browserPendingReason ||
          runtimeTokenPending ||
          runtimeSourcePending ||
          localePending
        )
          item.pendingReason = [
            pendingReason,
            receiptPending,
            inventoryPending,
            hostOverlayPending,
            textPending,
            browserPendingReason,
            runtimeTokenPending,
            runtimeSourcePending,
            localePending,
          ]
            .filter((reason): reason is string => Boolean(reason))
            .join('; ');
      } catch (error) {
        const status = await readReviewStatus(statusPanel);
        item.error = `${error instanceof Error ? error.message : String(error)}${status?.level === 'error' ? `; plugin error: ${status.message}` : ''}`;
        errors.push(`${reviewCase.id}: ${item.error}`);
      }
      console.log(
        JSON.stringify({
          sourcePath: entry.sourcePath,
          caseId: reviewCase.id,
          status: item.status,
          elapsedMs: Date.now() - started,
          ...(item.error ? { error: item.error } : {}),
          completedCases: index + 1,
          totalCases: cases.length,
        }),
      );
    }
  } finally {
    await session.send('Emulation.clearDeviceMetricsOverride');
    await session.detach();
    await setPreviewSelection(pluginFrame, true);
  }
  assert.equal(errors.length, 0, errors.join('\n'));
  assert.ok(primaryBytes && primaryLayout, 'Missing primary Penpot case');
  return { primaryBytes, primaryLayout, evidence };
}

async function setPreviewSelection(
  pluginFrame: FrameLocator,
  selected: boolean,
): Promise<void> {
  await pluginFrame.locator('body').evaluate((_, value) => {
    parent.postMessage({ type: 'set-preview-selection', selected: value }, '*');
  }, selected);
}

async function setReviewCase(
  pluginFrame: FrameLocator,
  reviewCase: LayoutReviewCase,
  timeoutMs = 300_000,
): Promise<void> {
  const trace = process.env['ZUI_LAYOUT_REVIEW_TRACE'] === '1';
  if (trace)
    console.error(JSON.stringify({ caseId: reviewCase.id, phase: 'dispatch' }));
  await pluginFrame.locator('body').evaluate((_, value) => {
    parent.postMessage({ type: 'set-review-case', reviewCase: value }, '*');
  }, reviewCase);
  if (trace)
    console.error(
      JSON.stringify({ caseId: reviewCase.id, phase: 'dispatched' }),
    );
  const statusPanel = pluginFrame.locator('.status-panel');
  const deadline = Date.now() + timeoutMs;
  let lastStatus = '';
  while (Date.now() <= deadline) {
    // Angular may replace the status section while the retained board is
    // rebuilt. Read with a short timeout so a transient iframe reflow does not
    // consume the whole acknowledgement budget.
    const status = await readReviewStatus(statusPanel);
    if (trace && status && status.message !== lastStatus) {
      lastStatus = status.message;
      console.error(
        JSON.stringify({
          caseId: reviewCase.id,
          phase: 'status',
          reviewCaseId: status.reviewCaseId,
          level: status.level,
          message: status.message,
        }),
      );
    }
    if (status?.level === 'error')
      throw new Error(status.message || `Review case failed: ${reviewCase.id}`);
    if (status?.reviewCaseId === reviewCase.id && status.level !== 'working')
      return;
    await new Promise((done) => setTimeout(done, 100));
  }
  throw new Error(
    `Review case acknowledgement timed out: ${reviewCase.id}; last status: ${lastStatus || '(unavailable)'}`,
  );
}

async function readReviewStatus(statusPanel: Locator): Promise<{
  reviewCaseId: string | null;
  level: string | null;
  message: string;
} | null> {
  try {
    const [reviewCaseId, level, message] = await Promise.all([
      statusPanel.getAttribute('data-review-case-id', { timeout: 1_000 }),
      statusPanel.getAttribute('data-level', { timeout: 1_000 }),
      statusPanel.innerText({ timeout: 1_000 }),
    ]);
    return { reviewCaseId, level, message: message.trim() };
  } catch (error) {
    if (
      error instanceof Error &&
      /Timeout|detached|closed/i.test(error.message)
    )
      return null;
    throw error;
  }
}

function readLayoutAudit(audit: RenderedLayoutAudit): LayoutAuditSnapshot {
  for (const [name, value] of Object.entries({
    totalNodes: audit.totalNodes,
    checkedNodes: audit.checkedNodes,
    overflowCount: audit.overflowCount,
    invalidGeometryCount: audit.invalidGeometryCount,
  })) {
    assert.ok(Number.isInteger(value) && value >= 0, `Invalid ${name}`);
  }
  return {
    totalNodes: audit.totalNodes,
    semanticNodes: audit.semanticNodes ?? [],
    checkedNodes: audit.checkedNodes,
    overflowCount: audit.overflowCount,
    invalidGeometryCount: audit.invalidGeometryCount,
    overflowNodes: audit.overflowNodeIds.join(','),
    invalidNodes: audit.invalidGeometryNodeIds.join(','),
    overflowDetails: JSON.stringify(audit.overflowDetails ?? []),
    invalidDetails: JSON.stringify(audit.invalidGeometryDetails ?? []),
  };
}

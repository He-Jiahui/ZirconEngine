import { layoutSourceRoot, resolveLayoutCatalogPath } from './zui-layout-paths';
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { embedLayoutImages } from './zui-layout-images';
import { existsSync } from 'node:fs';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { basename, dirname, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { promisify } from 'node:util';

import { parse } from 'smol-toml';
import {
  catalogEntryForSource,
  parseZuiLayoutSource,
} from '../src/bridge/zui-layout-optimizer.js';
import { LayoutDependencies } from './zui-layout-dependencies.js';
import { prepareThemeReviewHost } from './zui-layout-theme-hosts';
import {
  prepareComponentReviewHost,
  requiredComponentSlots,
} from './zui-layout-component-hosts';
import { prepareDynamicReviewState } from './zui-layout-dynamic-hosts';
import { reviewHost } from '../src/bridge/zui-review-host';
import {
  projectZuiDocument,
  type ZuiAssetProjection,
} from '../src/bridge/penpot-projection.js';
import {
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
  type ZuiNode,
} from '../src/bridge/zui-document.js';
import type { ZuiPrefabRole } from '../src/bridge/zui-prefab-system.js';
import {
  appendEmbeddedScrollReviewCases,
  defaultReviewCases,
  casesSha256,
  type LayoutReviewCase,
  type LayoutRenderEvidence,
  type DependencyFingerprint,
  type CatalogReview,
} from './zui-layout-review-contract';
import {
  bytesSha256,
  dependenciesSha256,
  dependencyFingerprints,
  currentDualReview,
} from './zui-layout-evidence';
import { acquireLayoutCatalogLock } from './zui-layout-catalog-lock';
import { retainExpandedCaseEvidence } from './zui-layout-catalog-evidence';
import {
  buildWorkbenchReviewCases,
  readWorkbenchCaseSnapshots,
  WORKBENCH_WINDOW_SOURCE,
} from './zui-layout-workbench-cases';
import {
  catalogSourceSelection,
  mergeSelectedCatalogEntries,
  selectCatalogSources,
} from './zui-layout-catalog-selection';
export {
  caseSha256,
  casesSha256,
  canonicalSha256,
} from './zui-layout-review-contract';
export type {
  LayoutReviewCase,
  LayoutRenderEvidence,
  DependencyFingerprint,
  CatalogReview,
} from './zui-layout-review-contract';

const execFileAsync = promisify(execFile);
const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const penpotRoot = resolve(packageRoot, '../../../../third_party/penpot');
const repoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(penpotRoot, '../..'));
const outputRoot = resolve(
  process.env['ZUI_LAYOUT_OUTPUT_ROOT'] ?? resolve(repoRoot, 'docs/_data/layout'),
);

export interface CatalogManifest {
  schema: 'dev.zircon.zui.layout-catalog';
  version: 1;
  generatedAt: string;
  repoRoot: string;
  sourceCount: number;
  /**
   * M0 provenance for the source inventory.  The bridge fixture lives under
   * `tools/penpot/`, which is intentionally excluded from the normal project scan,
   * so keep its explicit inclusion visible rather than making a changed total
   * look like an accidental generated-file ingestion.
   */
  sourceDiscovery?: CatalogSourceDiscovery;
  entries: CatalogEntry[];
}

export interface CatalogSourceDiscovery {
  scan: 'git-ls-files-cached-others-exclude-standard';
  controlledOrUnignoredSourceCount: number;
  projectOwnedScannedFixtures?: Array<{
    sourcePath: string;
    reason: string;
  }>;
  explicitProjectFixtures: Array<{
    sourcePath: string;
    reason: string;
  }>;
  excludedRoots: string[];
}

export interface CatalogEntry {
  sourcePath: string;
  outputPath: string;
  resultPath: string;
  previewPath: string;
  category: string;
  name: string;
  /** Stable M0 classification used to select the acceptance host. */
  sourceClassification: SourceClassification;
  /** Directory containing the authored source and its local assets. */
  assetRoot: string;
  /** Authored component references, kept separate from file dependencies. */
  componentDependencies: string[];
  /** Resource and dependent .zui paths resolved for this entry. */
  resourceDependencies: string[];
  sourceSha256: string;
  outputSha256: string;
  sourceKind: string;
  sourceVersion: number | string;
  outputKind: string;
  outputVersion: number | string;
  sourceFormat: 'v2' | 'legacy';
  status: 'prepared' | 'failed';
  nodeCount: number;
  projectedShapeCount: number;
  prefabNodeCount: number;
  prefabCoverage: number | null;
  prefabRoleCounts: Partial<Record<ZuiPrefabRole, number>>;
  diagnostics: Array<{
    severity: string;
    code: string;
    message: string;
    path?: string;
  }>;
  changes: string[];
  sourceChanges?: string[];
  visualStatus: 'pending' | 'passed' | 'failed';
  visualPluginSha256?: string;
  visualError?: string;
  visualPendingReason?: string;
  visualScreenshot?: string;
  visualSha256?: string;
  visualPixelRatio?: number;
  visualDetailPixelRatio?: number;
  visualColorBucketCount?: number;
  visualLayoutOverflowCount?: number;
  visualLayoutInvalidGeometryCount?: number;
  dependencyFingerprints?: DependencyFingerprint[];
  dependencySha256?: string;
  cases?: LayoutReviewCase[];
  penpotInputPath?: string;
  penpotInputSha256?: string;
  penpotPreviewPath?: string;
  penpotEvidence?: LayoutRenderEvidence[];
  engineEvidence?: LayoutRenderEvidence[];
  review?: CatalogReview;
  reviewHistory?: CatalogReview[];
}

/** Imported Workbench prefabs expand into native nodes before the catalog records findings. */
export function isWorkbenchWindowReviewDocument(
  document: ZuiDocument | undefined,
): boolean {
  return (
    document?.nodes?.['top_toolbar']?.control_id ===
      'WorkbenchWindowTopToolbarRegion' &&
    document?.nodes?.['component_drawer_shell']?.control_id ===
      'BottomDrawerShellRoot'
  );
}

export type SourceClassification =
  'product-page' | 'component' | 'theme' | 'dynamic-host' | 'test-fixture';

interface SourceHeader {
  kind: string;
  version: number | string;
  id: string;
  displayName: string;
}

interface TableSourceReportRecord {
  sourcePath: string;
  changes?: Array<{ nodeId: string; options: string[] }>;
  unresolved?: Array<{ nodeId: string; reason: string }>;
}

interface TableSourceReport {
  records?: TableSourceReportRecord[];
}

interface MaterialSourceReportRecord {
  sourcePath: string;
  afterSha256: string;
  titleHeight: number;
  metaRowHeight: number;
  height: number;
}

interface MaterialSampleReportRecord {
  sourcePath: string;
  beforeSha256: string;
  afterSha256: string;
  changes: string[];
}

async function readSourceCorrectionReport(name: string) {
  const path = resolve(repoRoot, 'docs/_data/layout/evidence', name);
  const report = existsSync(path)
    ? (JSON.parse(await readFile(path, 'utf8')) as {
        applied: boolean;
        records: MaterialSampleReportRecord[];
      })
    : undefined;
  return new Map(
    report?.applied
      ? report.records.map((record) => [record.sourcePath, record])
      : [],
  );
}

async function main(): Promise<void> {
  const rawArgs = process.argv.slice(2);
  const args = new Set(rawArgs);
  const requestedSources = catalogSourceSelection(rawArgs);
  if (requestedSources.length && args.has('--refresh-records'))
    throw new Error('--source cannot be combined with --refresh-records');
  const shouldWrite = !args.has('--check');
  const lock = shouldWrite
    ? await acquireLayoutCatalogLock(outputRoot, {
        command: ['zui-layout-catalog.ts', ...args].join(' '),
      })
    : undefined;
  try {
    await generateCatalog(args, shouldWrite, requestedSources);
  } finally {
    await lock?.release();
  }
}

async function generateCatalog(
  args: Set<string>,
  shouldWrite: boolean,
  requestedSources: string[],
): Promise<void> {
  const existingManifest = await readExistingManifest();
  if (args.has('--refresh-records')) {
    if (!existingManifest)
      throw new Error('Generate the catalog before refreshing records');
    if (shouldWrite) {
      for (const entry of existingManifest.entries)
        await writeFile(
          resolveLayoutCatalogPath(outputRoot, entry.resultPath),
          renderResult(entry),
          'utf8',
        );
      await writeFile(
        resolveLayoutCatalogPath(outputRoot, 'index.md'),
        renderIndex(existingManifest),
        'utf8',
      );
    }
    console.log(
      JSON.stringify({
        refreshedRecords: existingManifest.entries.length,
        wrote: shouldWrite,
      }),
    );
    return;
  }
  const existingBySource = new Map(
    (existingManifest?.entries ?? []).map((entry) => [entry.sourcePath, entry]),
  );
  const discovery = await discoverZuiSources();
  const sources = selectCatalogSources(discovery.paths, requestedSources);
  const dependencies = new LayoutDependencies();
  await dependencies.load(repoRoot, discovery.paths);
  const tableSourceReport = await readTableSourceReport();
  const materialSourceReport = await readMaterialSourceReport();
  const samplesBySource = await readSourceCorrectionReport(
    'material-sample-applied.json',
  );
  const compositesBySource = await readSourceCorrectionReport(
    'workbench-composite-applied.json',
  );
  const extensionsBySource = await readSourceCorrectionReport(
    'extension-layout-applied.json',
  );
  const headingsBySource = await readSourceCorrectionReport(
    'workbench-heading-applied.json',
  );
  const styleNormalizationBySource = await readSourceCorrectionReport(
    'style-source-normalization-applied.json',
  );
  const showcaseVisualBySource = await readSourceCorrectionReport(
    'showcase-visual-section-source-correction.json',
  );
  const entries: CatalogEntry[] = [];

  for (const sourcePath of sources) {
    const absoluteSourcePath = resolve(repoRoot, sourcePath);
    const sourceBytes = await readFile(absoluteSourcePath);
    const source = sourceBytes.toString('utf8');
    const sourceHeader = readSourceHeader(source);
    const sourceSha256 = bytesSha256(sourceBytes);
    const materialRecord = materialSourceReport.get(sourcePath);
    const sampleRecord = samplesBySource.get(sourcePath);
    const compositeRecord = compositesBySource.get(sourcePath);
    const extensionRecord = extensionsBySource.get(sourcePath);
    const headingRecord = headingsBySource.get(sourcePath);
    const styleNormalizationRecord = styleNormalizationBySource.get(sourcePath);
    const showcaseVisualRecord = showcaseVisualBySource.get(sourcePath);
    let cases = defaultReviewCases(sourcePath, sourceHeader.kind);
    const isV2 =
      sourceHeader.version === 2 &&
      !['layout', 'widget'].includes(sourceHeader.kind);
    let prepared: {
      document?: ZuiDocument;
      sourceFormat: 'v2' | 'legacy';
      changes: string[];
    } = {
      sourceFormat: isV2 ? 'v2' : 'legacy',
      changes: [],
    };
    let projection: ZuiAssetProjection | undefined;
    let serialized: string | undefined;
    let fingerprints: DependencyFingerprint[] = [];
    let preparationError: string | undefined;
    let status: CatalogEntry['status'] = 'prepared';
    let consumerHost:
      | NonNullable<Awaited<ReturnType<typeof prepareThemeReviewHost>>>
      | NonNullable<Awaited<ReturnType<typeof prepareComponentReviewHost>>>
      | null = null;
    let hostFile = 'theme-consumer';
    try {
      const input = isV2
        ? parseZuiDocument(source).document
        : parseZuiLayoutSource(source, sourcePath).document;
      prepared.document = input;
      cases = defaultReviewCases(sourcePath, sourceHeader.kind, input);
      // A structural host is intentionally reviewed as its authored mount
      // contract. Expanding every imported workspace here turns a slot-only
      // source into a multi-thousand-node pseudo page and makes Penpot
      // acknowledgement time out before the host can be marked pending. Keep
      // the imports/fingerprints intact, but do not materialize runtime-only
      // branches into the design projection.
      const structuralHost = isStructuralWorkbenchHostDocument(input);
      const dependencyChanges = structuralHost
        ? []
        : dependencies.embed(input, sourcePath, cases[0]?.themeSourcePath);
      const dynamicReview = prepareDynamicReviewState(input, sourcePath);
      dependencyChanges.push(...dynamicReview.changes);
      // Popup controls often arrive through an imported prefab (for example a
      // WorkbenchDropdown materializes as a native Dropdown node). Generate
      // the review matrix from the final expanded projection so every
      // source-owned popup instance that survives deterministic host selection
      // gets open/closed/focus-return scenes. Recomputing after dynamic branch
      // pruning is important: an inactive module may contain popup prefabs
      // that are removed from the Penpot input and have no review host.
      cases = defaultReviewCases(sourcePath, sourceHeader.kind, input);
      cases = appendEmbeddedScrollReviewCases(cases, input);
      if (sourcePath === WORKBENCH_WINDOW_SOURCE)
        cases = buildWorkbenchReviewCases(
          cases,
          input,
          await readWorkbenchCaseSnapshots(repoRoot),
        );
      consumerHost = structuralHost
        ? null
        : await prepareThemeReviewHost(repoRoot, sourcePath, input);
      if (!consumerHost) {
        consumerHost = structuralHost
          ? null
          : await prepareComponentReviewHost(
              repoRoot,
              sourcePath,
              input,
              dependencies,
              cases[0]?.themeSourcePath,
              cases[0]?.data['workbenchPresentation'] as
                Record<string, unknown> | undefined,
            );
        hostFile = 'component-host';
      }
      // The retained Editor page hosts are assembled from the authored
      // EditorMainFrame/WorkbenchShell/WorkbenchWindow assets.  Keep that
      // projection distinct from a component slot specimen so the native
      // runner can verify it against the page-host contract.
      if (
        consumerHost &&
        cases.length > 0 &&
        cases.every((reviewCase) => reviewCase.host === 'editor')
      ) {
        hostFile = 'editor-host';
      }
      if (consumerHost) {
        input['penpot_review_host'] = consumerHost.projection;
        input['penpot_dependency_sources'] = [
          ...new Set([
            ...((input['penpot_dependency_sources'] as string[]) ?? []),
            ...consumerHost.consumers.map((item) => item.sourcePath),
            ...((consumerHost.projection[
              'penpot_dependency_sources'
            ] as string[]) ?? []),
          ]),
        ];
        dependencyChanges.push(
          hostFile === 'theme-consumer'
            ? 'mount-authored-theme-consumers'
            : hostFile === 'editor-host'
              ? 'mount-authored-editor-host'
              : 'mount-authored-component-host',
        );
      }
      if (await embedLayoutImages(input, sourcePath, repoRoot))
        dependencyChanges.push('embed-authored-image-resources');
      prepared = {
        document: input,
        sourceFormat: isV2 ? ('v2' as const) : ('legacy' as const),
        changes: [...new Set(dependencyChanges)],
      };
      fingerprints = await dependencyFingerprints(repoRoot, sourcePath, input);
      projection = projectZuiDocument(reviewHost(input) ?? input);
      input['penpot_original_source'] = source;
      serialized = serializeZuiDocument(input);
    } catch (error) {
      status = 'failed';
      preparationError = errorMessage(error);
    }
    const entryKey = catalogEntryForSource(sourcePath);
    const relativeDir = `${entryKey.category}/${entryKey.name}`;
    // A consumer projection is useful for Penpot editing, but the native
    // acceptance runners must only substitute a host whose contract matches
    // the case host.  Editor/plugin/page views (including the retained
    // EditorMainFrame) are rendered from their real source/runtime host; a
    // design-only component-host here would silently replace that product
    // boundary and the Editor capture would reject it.  Keep the projection
    // embedded and publish its source for Penpot.  The native runner accepts
    // only the explicitly typed component/toolbar/theme/editor host contracts.
    const nativeReviewHostCompatible =
      consumerHost &&
      cases.every((reviewCase) =>
        hostFile === 'theme-consumer'
          ? reviewCase.host === 'theme'
          : hostFile === 'editor-host'
            ? reviewCase.host === 'editor'
            : reviewCase.host === 'component' || reviewCase.host === 'toolbar',
      );
    if (nativeReviewHostCompatible && consumerHost) {
      for (const reviewCase of cases)
        reviewCase.reviewHost = {
          path: `${relativeDir}/evidence/${hostFile}.zui`,
          sha256: sha256(consumerHost.source),
        };
    }
    const outputPath = `${relativeDir}/${entryKey.name}.zui`;
    const resultPath = `${relativeDir}/result.md`;
    const previewPath = `${relativeDir}/preview.png`;
    const sourceClassification = classifySourcePath(
      sourcePath,
      sourceHeader.kind,
      prepared.document,
    );
    const assetRoot = sourceAssetRoot(sourcePath);
    const componentDependencies = collectComponentDependencies(
      prepared.document,
    );
    const resourceDependencies = fingerprints.map(
      (fingerprint) => fingerprint.sourcePath,
    );
    const penpotInputPath =
      serialized === undefined
        ? undefined
        : `${relativeDir}/evidence/penpot-input.zui`;
    const shapes = projection?.shapes ?? [];
    const nodeCount = Object.keys(prepared.document?.nodes ?? {}).length;
    const tableRecord = tableSourceReport.get(sourcePath);
    const diagnostics = [
      ...(projection?.diagnostics ?? []),
      ...(tableRecord?.unresolved ?? []).map((item) => ({
        severity: 'warning' as const,
        code: 'table-source-cell-unresolved',
        message: `表格节点 ${item.nodeId} 未能从旧 value_text 安全迁移为显式列数据：${item.reason}`,
        path: `nodes.${item.nodeId}.props.value_text`,
      })),
      ...(preparationError
        ? [
            {
              severity: 'error' as const,
              code: 'preparation-failed',
              message: preparationError,
            },
          ]
        : []),
    ];
    const entry: CatalogEntry = {
      sourcePath,
      outputPath,
      resultPath,
      previewPath,
      category: entryKey.category,
      name: entryKey.name,
      sourceClassification,
      assetRoot,
      componentDependencies,
      resourceDependencies,
      sourceSha256,
      outputSha256: sourceSha256,
      dependencyFingerprints: fingerprints,
      dependencySha256: dependenciesSha256(fingerprints),
      cases,
      penpotInputPath,
      penpotInputSha256:
        serialized === undefined ? undefined : sha256(serialized),
      penpotPreviewPath: `${relativeDir}/penpot.png`,
      sourceKind: sourceHeader.kind,
      sourceVersion: sourceHeader.version,
      outputKind: sourceHeader.kind,
      outputVersion: sourceHeader.version,
      sourceFormat: prepared.sourceFormat,
      status,
      nodeCount,
      projectedShapeCount: shapes.length,
      prefabNodeCount: shapes.length,
      prefabCoverage: nodeCount > 0 ? shapes.length / nodeCount : null,
      prefabRoleCounts: shapes.reduce<Partial<Record<ZuiPrefabRole, number>>>(
        (counts, shape) => {
          counts[shape.prefabRole] = (counts[shape.prefabRole] ?? 0) + 1;
          return counts;
        },
        {},
      ),
      diagnostics,
      changes: [
        ...prepared.changes,
        ...((tableRecord?.changes?.length ?? 0) > 0
          ? ['materialize-table-cell-options']
          : []),
        ...((tableRecord?.unresolved?.length ?? 0) > 0
          ? ['preserve-source-diagnostics']
          : []),
      ],
      sourceChanges:
        materialRecord?.afterSha256 === sourceSha256
          ? [
              `nodes.title now uses WorkbenchLabel with the shared 20px title token in a ${materialRecord.titleHeight}px row.`,
              `nodes.meta changed from a single overflowing row to a two-column, two-row GridBox; its four WorkbenchCaption instances use 12px text and ${materialRecord.metaRowHeight}px rows so authored descriptions remain present.`,
              'nodes.state_strip changed from eight narrow 8px pills to a two-column, four-row GridBox with 12px WorkbenchCaption text and 24px rows. State values and semantic colors are preserved.',
              `nodes.root now has a ${materialRecord.height}px content height and the shared 12px gap. Sample control nodes, events, bindings, IDs, children and unknown nonvisual fields are preserved.`,
              'Original source and available baseline images: evidence/before-20260907-workbench/. The viewport remains the case viewport; these edits do not enlarge the capture canvas.',
            ]
          : undefined,
      visualStatus: 'pending',
    };
    if (sampleRecord?.afterSha256 === sourceSha256)
      entry.sourceChanges = [
        'WorkbenchLabel / WorkbenchCaption title, metadata and state-strip reflow retained; see evidence/material-source-applied.json for the earlier source fingerprint.',
        ...sampleRecord.changes,
        'Sample reflow source and screenshots before correction: evidence/before-20260907-sample-reflow/. Events, state, bindings, component identifiers and child order remain intact.',
      ];
    if (compositeRecord?.afterSha256 === sourceSha256)
      entry.sourceChanges = [
        ...compositeRecord.changes,
        'Original source and images: evidence/before-20260907-composite-reflow/. IDs, events, bindings and child order are preserved.',
      ];
    if (extensionRecord?.afterSha256 === sourceSha256)
      entry.sourceChanges = extensionRecord.changes;
    if (headingRecord?.afterSha256 === sourceSha256)
      entry.sourceChanges = headingRecord.changes;
    if (styleNormalizationRecord?.afterSha256 === sourceSha256)
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        `Shared Editor/plugin component style normalization applied (${styleNormalizationRecord.changes.length} authored property change${styleNormalizationRecord.changes.length === 1 ? '' : 's'}).`,
        ...styleNormalizationRecord.changes,
        'The rewrite reparsed the complete source and preserved component IDs, events, bindings, children and unknown runtime fields; the full before/after fingerprint ledger is evidence/style-source-normalization-applied.json.',
      ];
    if (showcaseVisualRecord?.afterSha256 === sourceSha256)
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        ...showcaseVisualRecord.changes,
        'The showcase source correction keeps authored text, deterministic component state, node IDs, child order and runtime fields intact; the before/after fingerprint is evidence/showcase-visual-section-source-correction.json.',
      ];
    if (
      sourcePath.includes('/reactbits_') &&
      prepared.document !== undefined &&
      hasAuthoredReactBitsTypography(prepared.document)
    ) {
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Native baseline defect: generic runtime text used the Fira Mono default while the Penpot projection measured a proportional UI face, changing wrapping and pane widths.',
        'Source correction: the fixture now owns a universal reactbits-ui-typography rule that selects res://fonts/editor-ui.font.toml / Fira Sans; this is part of the .zui source, not a capture-only override.',
        'Shared dependency/validation: editor-ui.font.toml and editor-ui.ttc are fingerprinted, and both Penpot and native cases must be recaptured before this source can be accepted.',
      ];
    }
    if (
      sourcePath.endsWith('/reactbits_auth_onboarding_components.zui') &&
      prepared.document?.nodes?.['onboarding_subtitle']?.props?.['wrap'] ===
        'word'
    ) {
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'At 900px the onboarding subtitle wrapped into a second line but its fixed 18px row clipped the glyph bounds; the source now uses an authored word-wrap label row of 28/30/34px and expands the heading group to 54/58/64px so the full text remains inside its node.',
      ];
    }
    if (
      sourcePath === REACTBITS_AUTH_ONBOARDING_FIXTURE &&
      prepared.document?.nodes?.['onboarding_action_gap']?.component === 'Space'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Onboarding action footer: the authored onboarding_action_gap Space now separates Back and Continue, keeping the primary action at the right edge without an overlay or hidden content.',
        'Shared Penpot bridge defect: an omitted flex maximum resolved to null and was treated as zero; this compressed both the footer gap and the Continue text frame to 0.01/1px. Weighted row width and cross-axis height now treat null as unbounded.',
        '2026-09-21 selected Penpot observations: default 1280x800 had a 270px action gap and a 42px Continue text frame inside its 42px button; scroll-after 640x520 kept both actions reachable. See the case evidence table for current coverage; these observations alone do not constitute native acceptance.',
      ];
    if (
      sourcePath === 'zircon_editor/assets/ui/editor/welcome.zui' &&
      (
        prepared.document?.nodes?.['outer_panel']?.layout?.['width'] as
          { max?: unknown } | undefined
      )?.max === 1000
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Welcome content row: the source-owned left/right Space nodes and 1000px capped outer panel were present, but the same null-maximum bridge defect pinned both margins to 16px in the 1280px Penpot preview. With the shared fix they are 140px each and the outer panel begins at x=140.',
        'The 640px source layout retains the native-host-owned recent-projects panel. Its real data and compact placement require Editor retained-host and engine review; a Penpot-only capture does not establish acceptance.',
      ];
    if (
      sourcePath.endsWith('/composites/chrome/workbench_panel_header.zui') &&
      prepared.document?.nodes?.['root']?.layout?.['padding']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'nodes.title_slot is required, but the catalog classified its structural-only source as a dynamic host and projected an empty 240px-tall panel instead of mounting a component specimen.',
        'The component review host now instantiates the authored WorkbenchSectionTitle and WorkbenchButton in the title/actions slots. These design-only specimen values cannot be written back to the product component on a no-edit Penpot round trip.',
        'nodes.root.layout.padding now owns the same left/right 8px inset already named in the component token properties, so native flow and Penpot both place the title and action inside the panel edges.',
        'Reference: [ReactBits App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell) with the supporting [Navbar](https://pro.reactbits.dev/docs/app-ui/navbar) header/action pattern; retain the local flat gray workbench surface, 30px header, 16px title, 28px action and focus semantics. The 240/360/480px and 150% DPI captures are Penpot-only until native evidence is collected.',
      ];
    if (
      sourcePath.endsWith(
        '/composites/feedback/workbench_diagnostic_row.zui',
      ) &&
      (
        prepared.document?.nodes?.['root']?.layout?.['height'] as
          { preferred?: unknown } | undefined
      )?.preferred === '$editor.control.height.large'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'nodes.severity_slot and nodes.message_slot are required, but structural-host classification rendered a blank row and hid the missing slot specimen.',
        'The component host now instantiates two real WorkbenchStatusItem slots; at 240px the 28px fixed row placed the two-line diagnostic message above/outside its text frame, so the measured glyph audit rejected it.',
        'nodes.root.layout.height now uses the authored 48px large control tier for the two-line diagnostic row, preserving the warning label and the complete message at 240/360/480px without clipping or replacing its content.',
        'Penpot source, geometry and measured text were recaptured in four width/DPI combinations; native Editor retained-host screenshots and dual-end parity remain pending.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_icon_button.zui') &&
      prepared.document?.nodes?.['root']?.component === 'IconButton'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Design projection defect at nodes.root.props.icon: select.svg has literal cyan fill/stroke, while the retained Editor IconButton painter tints its glyph from the authored icon_color and interaction-state palette. Penpot formerly displayed the SVG colors in every state, masking the native gray/hover/accent/disabled hierarchy.',
        'The Penpot-only SVG auxiliary shape now keeps the authored icon geometry but tints its opaque fill/stroke from the same Editor IconButton state and theme tokens. The product .zui and SVG stay unchanged; no-edit Penpot export continues to restore the exact source.',
        'Before evidence: [default cyan](evidence/before-20260921-icon-tint-default-360x520-dpi1.png) SHA-256 a1fd8b5f6b64e60c3e5da1469c0f37d44db90603771c4ec4656ba5bba91a80be; [selected cyan](evidence/before-20260921-icon-tint-selected-360x520-dpi1.png) SHA-256 f5ef8e8ef411def935b9c37e99933f07f9c2a4ac4e5346b85173a7bacd301feb.',
        'The 24 current Penpot state/width/DPI captures require native Editor retained-host evidence and original-size dual-end review before this icon control can be accepted.',
      ];
    if (
      sourcePath.endsWith(
        '/components/workbench/shell/workbench_top_toolbar.zui',
      ) &&
      (
        prepared.document?.nodes?.['toolbar_module_commands']?.layout?.[
          'width'
        ] as { min?: unknown } | undefined
      )?.min === 292
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Source layout defect: at 640x520 the toolbar_module_commands group was assigned about 130px for 292px of visible controls. Compile/Diff/Sim then overpainted the tool icons; the compact context-drawer action began outside the clipped command row.',
        'The product .zui now keeps all five horizontal groups at or above the widths of their visible children, prevents their slot shrinkage inside the authored horizontal ScrollableBox, and orders file, run, and layout/context actions ahead of lower-priority module/tools groups. No control, label, event, icon, or binding was removed.',
        'Before correction: [640x520 Workbench window screenshot](../workbench-window-a5213d36/evidence/before-20260921-toolbar-overlap-640x520-dpi1.png) SHA-256 a1f39c5c7a47acfa99eb778ae011e2a5ab9e33e44f0c4b6abab401ee1644b976. Six current Penpot sizes/DPI/scroll scenes were inspected individually in [the toolbar image record](../../evidence/workbench-toolbar-penpot-review-20260924.json); the Workbench page and native retained-host still need current original-size dual-end review.',
        'Reference: [ReactBits App Shell 3](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-3) keeps global navigation above content; [App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) retains access to an optional context panel. We follow the layout topology but keep local flat dark Workbench tokens.',
        'With the current bridge build, all six Penpot toolbar scenes again passed import/export capture. Original-size 640px [scroll start](evidence/penpot-default-640x520-dpi1.png) SHA-256 e06cebea46de08fc45e312895c3eb6819cd8d9add710e16f7377933e0cc7e294 shows file/run actions and early modules without overlap; [scroll end](evidence/penpot-scroll-after-640x520-dpi1.png) SHA-256 fc14f98bedf73d8f91d10e88549fbef7ec4922f700fd97002c694cfb3d113872 exposes Assets, VFX, HUD and trailing tools. This confirms only the Penpot toolbar projection: the containing window route, native scroll reachability and dual-end pixel comparison remain pending.',
      ];
    if (
      sourcePath.endsWith('/host/workbench_shell.zui') &&
      prepared.document?.nodes?.['activity_rail']?.component === 'ActivityRail'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Reference/implementation gap: [ReactBits App Shell 3](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-3) supplies the topbar and collapsible navigation-rail topology; [App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) supplies an optional context panel. This product .zui currently declares a fixed-width activity_rail and zero-sized drawer shells, with no source-owned rail-to-drawer breakpoint. The reference is an acceptance target, not a completed responsive layout.',
        'Keep the Workbench flat dark tokens, authored rail routes, document priority, and focus return. Add source and host state for 640x520, then compare current Penpot and retained-host screenshots at 1280x800, 900x620 and 640x520 before accepting this shell; a static rail/icon screenshot does not prove the collapse.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_button.zui') &&
      prepared.document?.nodes?.['root']?.component === 'Button'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'On 2026-09-24, the current 24 Penpot import/export state and width scenes were recaptured. Original-size default 240/360px, focused 360px and disabled 360px screenshots retain the authored label, 32px control height, flat dark surface and distinct focus/disabled borders; [focused image](evidence/penpot-focused-360x520-dpi1.png) SHA-256 b82e005bd8e993022e27e9603eb23d6185bb05a49f9bba750f1b4e202cebdb54 and [disabled image](evidence/penpot-disabled-360x520-dpi1.png) SHA-256 6c34664e8fd9f9eaf01e105fbde69323fe160d0819d47f779701b972911fd2c0. Product source is unchanged; native button state screenshots and one-logical-pixel glyph/bounds comparison remain pending.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_field.zui') &&
      prepared.document?.nodes?.['root']?.component === 'InputField'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'On 2026-09-24, all 36 current Penpot field scenes passed import/export and automated text checks. Original-size 240px default/empty/long English/[long Chinese](evidence/penpot-long-zh-240x520-dpi1.png) states keep the 14px text inside the 32px field and elide overflow without hiding the authored value; the long-Chinese PNG SHA-256 is dc694b570513b5a68e2dda9e49d85d65b1cd34376c793e27ec9813f6cfb79e30. The [480px error state](evidence/penpot-error-480x520-dpi1.png) retains a visible semantic red border (SHA-256 cf32e1ca7e62fc42559580ffa5adf345bb4601e1323a66e439eb34d7dbd084b3), while focused 360px shows the focus ring. No product source edit was needed; native glyph bounds (including 150% DPI) and consuming-page comparisons remain pending.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_number_field.zui') &&
      prepared.document?.nodes?.['root']?.component === 'NumberField'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Penpot bridge defect, not an authored .zui defect: a numeric control with a mapped editable value_text painted a second auxiliary text for the same value. The old non-default 360px hover/focus scenes measured two overlapping 42 glyphs under one NumberField. The shared control renderer now uses the mapped editable text and creates an auxiliary numeric value only when no editable text projection exists; the product node, binding, events, tokens, and stepper remain unchanged.',
        'On 2026-09-24 the 20 current Penpot scenes (240/360/480 widths, default 150% DPI, and default/hover/pressed/focused/disabled states) passed capture and text-structure checks. All 20 measured screenshots contain exactly one 42; [hover scene](evidence/penpot-hover-360x520-dpi1.png) SHA-256 3a6fd5170c3b1ca8ee193f70bb6477ec512a336392a23fa135727878e4b98336. Native component state screenshots and dual-end glyph bounds remain pending.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_search_input.zui') &&
      prepared.document?.nodes?.['root']?.component === 'SearchField'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Reference: [ReactBits Filtering](https://pro.reactbits.dev/docs/app-ui/filtering) places the query before its result set; [Command Menu](https://pro.reactbits.dev/docs/app-ui/command-menu) treats quick search and its invoked results as separate regions. This entry is the source SearchField only, not evidence that a result list or command popup exists. Preserve the Workbench dark palette and the authored query, placeholder and clear-action semantics.',
        'On 2026-09-24, all 36 current Penpot scenes passed capture and automated text checks across 240/360/480 logical widths, 150% DPI and default/empty/focused/disabled/error/long bilingual cases. Original-size inspection of [default](penpot.png), [empty](evidence/penpot-empty-240x520-dpi1.png), [focused](evidence/penpot-focused-360x520-dpi1.png), [error](evidence/penpot-error-480x520-dpi1.png), [long English](evidence/penpot-long-en-240x520-dpi1.png), and [long Chinese](evidence/penpot-long-zh-240x520-dpi1.png) shows the leading search icon and trailing clear action remain visible, with long query text elided inside its field. The 240px long-Chinese screenshot SHA-256 is 86d8d39299ee9eba2b6cad7eb17989e740e6c0f3e607f71eed2931e9047966d7. No product source edit was needed here; native state screenshots, glyph-bound comparison and consuming-page results remain pending.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_dropdown.zui') &&
      prepared.document?.nodes?.['root']?.component === 'Dropdown'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Coverage gap at nodes.root.props.popup_open: the prior 20 Penpot scenes covered default, hover, pressed, focused and disabled trigger chrome at 240/360/480 logical widths and 150% DPI. Source-derived open and closed scenes are now registered but remain uncaptured; a changed trigger border is not popup evidence. Implement the real popup host and paired native/Penpot open, closed and focus-return scenes before acceptance. The authored value, option, arrow and route are unchanged.',
        'Reference: [ReactBits App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog) informs the anchored popover and dismiss/focus relation. The product Dropdown and Editor dark tokens remain authoritative; no replacement menu is painted into the current trigger-only screenshot.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_icon_button.zui') &&
      prepared.document?.nodes?.['root']?.component === 'IconButton'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Icon-size follow-up at nodes.root: the source declares a 32px icon button but no semantic icon_size tier, while the current Penpot icon projection caps the asset glyph at 18px. All 24 declared trigger states/sizes passed Penpot capture, including visible focus and disabled states; native asset-backed glyph extents and the correct dense-toolbar versus standard-action tier still need comparison before changing the source or marking this component accepted. Avoid one-off pixel tuning.',
        'Reference: [ReactBits Navbar](https://pro.reactbits.dev/docs/app-ui/navbar) for toolbar action hierarchy; keep the local flat Workbench palette and the project s/m/l/xl semantic icon-size scale rather than copying external artwork.',
      ];
    if (
      sourcePath.endsWith('/primitives/data/workbench_table_row.zui') &&
      prepared.document?.nodes?.['root']?.component === 'Table'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Historical defect: the preserved [240x520 default Penpot screenshot](evidence/before-20260924-hidden-size-revision-240x520-dpi1.png) SHA-256 0561d2631e57571c3b83193b354bd4aa14ab50832387152b10da3d606f3052c4 contains only Item_01 and Mesh; the authored Size (2.4 MB) and Revision (2m ago) cells remained in the source but disappeared from measured visible text. The previous 20 Penpot scenes passed automated geometry checks, so those checks alone did not promote this row.',
        'Source correction: the real WorkbenchTableRow now reserves a 360px minimum content width, while the component review host, Asset Browser table and generated-bottom route table use explicit horizontal ScrollableBox owners. The four authored cells therefore remain in the source and are reachable at 240/360/480px instead of being silently discarded by narrow allocation.',
        'Reference: [ReactBits Data Table 8](https://pro.reactbits.dev/docs/app-ui/data-table/data-table-8) keeps a sticky first column with horizontal access to remaining columns. Rework the actual .zui table/consumer scroll ownership and the paired Penpot/native column allocation together; add 240px scroll-start/end plus selected/long-content scenes. Do not shrink four 14px cells to illegibility, silently discard them, or claim acceptance from 360px-only visibility. Native capture and original-size dual-end review remain pending.',
        'Review bridge correction: source-selected Workbench table rows previously produced the same 240px PNG as default. The scenario list now covers selected at 240/360/480 and 150% DPI, and the Penpot preview projects the authored selected background and accent border from transient state without rewriting source props. The current [selected 240px image](evidence/penpot-selected-240x520-dpi1.png) SHA-256 7b83a5a22771207e169f3b362ff377f96959945caf3ee2dee1bb8a6002fb3f6b differs from default SHA-256 0561d2631e57571c3b83193b354bd4aa14ab50832387152b10da3d606f3052c4; both still expose the same missing-cell defect.',
        'No-edit Penpot export initially failed because the auxiliary Name font-measurement shape was saved as 1x1 before its real 37x20 glyph size settled. Layout stability sampling now includes and awaits these invisible source-derived measurements before writing reversible metadata; unmapped geometry edits still produce an explicit error. The 24 current Penpot import/export captures must be recaptured against the corrected scroll-owned source, and native evidence remains absent. Static native source inspection also shows selected rows using host surface_pressed instead of the authored selected_background_color; reconcile that authority before dual-end acceptance.',
      ];
    if (
      sourcePath.endsWith(
        '/components/workbench/shell/workbench_component_drawer.zui',
      ) &&
      (
        prepared.document?.nodes?.['icon_toggle_segment']?.layout?.['width'] as
          { min?: unknown } | undefined
      )?.min === 216 &&
      (
        prepared.document?.nodes?.['input_segmented']?.layout?.['height'] as
          { min?: unknown } | undefined
      )?.min === 48 &&
      (
        prepared.document?.nodes?.['component_feedback']?.layout?.['height'] as
          { min?: unknown } | undefined
      )?.min === 385 &&
      (
        prepared.document?.nodes?.['component_lower_row']?.layout?.[
          'height'
        ] as { min?: unknown } | undefined
      )?.min === 547
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'The 640x520 Workbench window measured Left/Center/Right at y=842..859 outside the labeled input_segmented node y=818..850, and Columns at x=440..495 outside the icon_toggle_segment node x=332..482. These are authored composite geometry defects; allowing the scroll clip or removing text would hide them.',
        'nodes.input_segmented now reserves 48px for its retained 14px label and a normal 28px segmented option row. nodes.icon_toggle_segment retains all three option values and allocates 216px for the measured Columns glyph plus horizontal insets; its containing sample card is 232px to preserve both 8px side paddings. The unrelated 28/32px inputs remain unchanged.',
        'The authored Table specimen now places its 360px-minimum four-column group inside WorkbenchTableScroll, an explicit horizontal ScrollableBox nested in the drawer body. Narrow component-drawer widths keep the row values reachable without changing the vertical drawer scroll owner or dropping Size/Modified cells.',
        'Before correction: [640x520 screenshot](../workbench-window-a5213d36/evidence/before-20260924-drawer-segments-640x520-dpi1.png) SHA-256 8cc1516217f76f0554a824c09a78dc4f19e1ee411235127d8ec4fbd47874e89a; raw [Penpot geometry](../workbench-window-a5213d36/evidence/before-20260924-drawer-segments-640x520-dpi1.geometry.json) and [measured glyphs](../workbench-window-a5213d36/evidence/before-20260924-drawer-segments-640x520-dpi1.text.json). Recheck this shared component and all dependent windows in both renderers before acceptance.',
        'Original-size 640x520 scroll-after inspection found four 30px alerts overflowing their 61px container, overpainting the 78px tooltip; the 61px toast column also allocated less height than its 135px of children. Automated text boundaries alone did not catch this sibling overlap. The product source now reserves 132px alerts, 135px toast contents, and 385px for the padded sample card, inside the authored scroll area; no alert or toast was hidden. [Pre-fix screenshot](evidence/before-20260924-feedback-overlap-640x520-dpi1.png) SHA-256 feb4fec04fd2b73b4b3437c761b4b0af0e7388a5807b5053ac34cde2eab6f2ea and [measured geometry](evidence/before-20260924-feedback-overlap-640x520-dpi1.geometry.json). Recheck in original-size Penpot and native screenshots, including the shared Workbench window.',
        'After the first correction, original-size geometry showed the 154px table card had collapsed to 0.01px, while the 385px feedback card extended beyond its 240px lower-row parent. This put four table rows underneath feedback. The actual product .zui now reserves 154px table and 547px lower row (154px + 8px gap + 385px feedback), so each card and its contents participate in scroll flow. [Intermediate failing screenshot](evidence/before-20260924-lower-row-collapse-640x520-dpi1.png) SHA-256 b9935ce5dc164068a6924883532b27882e4974ac91dc023428417df57112a8c0 and [measured geometry](evidence/before-20260924-lower-row-collapse-640x520-dpi1.geometry.json).',
        'Layout mapping: [ReactBits App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog) supplies the drawer/scrollable-body relationship; [Data Table](https://pro.reactbits.dev/docs/app-ui/data-table) keeps comparable item columns, and [Notifications](https://pro.reactbits.dev/docs/app-ui/notifications) keeps severity alerts and Toast as separate states. The product keeps its flat dark Workbench palette, authored ScrollableBox and single scroll owner; the references do not replace source labels, focus/route semantics or local tokens.',
        'State-scene gap under review: the old 1280x800 open screenshot passed automation while feedback_toast was at y=1321 outside the first viewport. The current LayoutReviewCase adds open-scroll-after-640x520-dpi1 (open interaction plus explicit end offset). On 2026-09-24, original-size [open](evidence/penpot-open-640x520-dpi1.png) and [open, scroll end](evidence/penpot-open-scroll-after-640x520-dpi1.png) Penpot scenes passed and the latter shows table, four alerts, tooltip and Toast without sibling overlap. Native scroll, focus/route confirmation, and the complete scene matrix remain pending; an offscreen state mutation alone is not acceptance.',
        'New coverage defect after current 11/11 automated Penpot recapture: [scroll-after](evidence/penpot-scroll-after-640x520-dpi1.png) and [open-scroll-after](evidence/penpot-open-scroll-after-640x520-dpi1.png) are pixel-identical (SHA-256 1a4baeada551540c22b7ff79d70684fb45798170fafc50f1b5412ac21f9a56b7). The imported static WorkbenchToast specimen has no authored open property, so applying a transient open state does not prove that an overlay opened or focus returned; even 640px top-of-scroll open leaves the feedback specimen below the viewport. The shared dual-evidence gate now rejects pixel-identical opened/baseline pairs even if automated captures pass. The original-size end scene does show all four table columns, alerts, Tooltip and Toast with no sibling overlap, but it does not fulfill open/closed coverage. Keep this item pending and replace the unobservable open case with a source-owned actual invoked popup/Toast host and paired native evidence before acceptance.',
      ];
    if (
      sourcePath.endsWith(
        '/components/workbench/modules/generated/workbench_generated_bottom_panel.zui',
      ) &&
      record(
        prepared.document?.nodes?.[
          'generated_bottom_route_table_horizontal_viewport'
        ]?.layout?.['container'],
      )?.['axis'] === 'Horizontal' &&
      prepared.document?.nodes?.[
        'generated_bottom_route_table_horizontal_viewport'
      ]?.children?.some(
        ({ node }) => node === 'generated_bottom_route_table',
      ) &&
      record(
        prepared.document?.nodes?.['generated_bottom_route_table']?.layout?.[
          'container'
        ],
      )?.['axis'] === 'Vertical' &&
      prepared.document?.nodes?.[
        'generated_bottom_route_table'
      ]?.children?.some(
        ({ node }) => node === 'generated_bottom_route_table_column',
      ) &&
      prepared.document?.nodes?.['generated_bottom_route_table_column']
        ?.layout?.['width']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'The generated-bottom route list composes two supported single-axis ScrollableBox owners: a finite horizontal viewport around the existing vertical route table and its 360px-minimum, 640px-preferred column. All 37 route rows remain authored. The horizontal owner stretches to the body viewport; the vertical owner stretches to that viewport height while its column retains 1124px of content height. This replaces the unsupported Both axis without discarding either overflow direction.',
        'Reference: local Unreal Engine Slate SScrollBox (dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Layout/SScrollBox.h) declares one EOrientation per scroll owner. The local route IDs, neutral Workbench tokens and retained host remain authoritative. Native wheel routing uses one scalar delta and prefers the inner owner; verify vertical scroll and the outer horizontal scrollbar in the real product and Penpot before acceptance.',
      ];
    if (
      sourcePath.endsWith('/windows/workbench_window.zui') &&
      isWorkbenchWindowReviewDocument(prepared.document)
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Shared dependency defect: the 640x520 Workbench window previously showed overlapping toolbar labels/icons and a context-drawer action beyond the clipped viewport. The product window source is unchanged; the referenced workbench_top_toolbar.zui owns the source fix and dependency fingerprint must trigger this page for re-review.',
        'Before correction: [original-size 640x520 screenshot](evidence/before-20260921-toolbar-overlap-640x520-dpi1.png) SHA-256 a1f39c5c7a47acfa99eb778ae011e2a5ab9e33e44f0c4b6abab401ee1644b976. Neither Penpot-only nor static toolbar tests can establish native page acceptance.',
        'After the toolbar reflow, the full 640x520 page exposed four genuine segmented-control text-boundary defects in its component drawer. The drawer dependency now owns 48px of labeled height and 216px of three-option width without changing business labels; [failed pre-correction capture](evidence/before-20260924-drawer-segments-640x520-dpi1.png) SHA-256 8cc1516217f76f0554a824c09a78dc4f19e1ee411235127d8ec4fbd47874e89a. Recapture this entire page; do not promote the failed screenshot.',
        'The same embedded drawer later exposed overlapping Alert/Tooltip content in the standalone 640x520 scroll-after scene; the shared .zui now allocates its full feedback children. [Original failing image](../workbench-component-drawer-81f5ad59/evidence/before-20260924-feedback-overlap-640x520-dpi1.png) SHA-256 feb4fec04fd2b73b4b3437c761b4b0af0e7388a5807b5053ac34cde2eab6f2ea. This window also needs fresh scroll-state and native screenshots.',
        'Follow-up geometry revealed a 0.01px table card underneath feedback and an undersized lower-row parent; the same product drawer dependency now allocates both cards in its scroll flow. [Intermediate failing image](../workbench-component-drawer-81f5ad59/evidence/before-20260924-lower-row-collapse-640x520-dpi1.png) SHA-256 b9935ce5dc164068a6924883532b27882e4974ac91dc023428417df57112a8c0. Recheck this full page at original size after its dependency fingerprint changes.',
        'ReactBits App Shell 3/4 compact-mode gap: this .zui authors an overflow menu, but its collapsed initial visibility does not establish a narrow-width route into the menu or drawer. Treat the 640x520 rail/toolbar/context transition as unimplemented until the product host, source state, Penpot capture and retained-host capture prove reachable controls without clipping.',
      ];
    if (
      sourcePath.endsWith('/primitives/inputs/workbench_tab_strip.zui') &&
      prepared.document?.nodes?.['root']?.component === 'Tabs'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Design review defect: generic hover/selected states cleared the authored single selection and set the same flag on every Tab in the slot host. The 360px hover image lit the whole row; selected painted all three tabs and one continuous underline, despite automated geometry checks passing.',
        'The scenario adapter now targets one sibling Tab, keeps the authored overview selection for hover/press/focus/disabled, and moves the group value plus selected_index to details only in the selected scene. The product .zui, tab source and three-slot mapping are unchanged; native Editor state targeting still requires separate verification.',
        'Before evidence: [all hovered](evidence/before-20260921-tab-target-hover-360x520-dpi1.png) SHA-256 d69e0b502d7c7f4b1959abe18e295207c49dc36a92d78cd5bacd51adf8e50288; [all selected](evidence/before-20260921-tab-target-selected-360x520-dpi1.png) SHA-256 bcabe7efa6e1979fe9b664969d3c30dcd5259db217616fa1fdc92a80d7b8c475.',
        'Reference: [ReactBits Navbar](https://pro.reactbits.dev/docs/app-ui/navbar) for one active navigation item; preserve the authored flat Workbench tab strip, 14px text and local blue selection underline. Native evidence and original-size dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith('/primitives/feedback/workbench_drag_overlay.zui') &&
      prepared.document?.nodes?.['root']?.component === 'DragOverlay'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Design projection defect: the authored DragOverlay defaults to open=false, so its default component preview is intentionally absent. The closed-popup evidence guard did not include DragOverlay and incorrectly treated the semantically hidden overlay as a blank failed image.',
        'The closed-state contract now recognizes DragOverlay only when its root and every semantic node are hidden; dragging, drop-allowed and drop-blocked states still require their native painter content. The product .zui open/drag/drop flags, payload labels, source/target bindings and z-index remain unchanged, and a regression case prevents visible siblings from being hidden.',
        'On 2026-09-25, all 16 current Penpot scenes passed (default, dragging, drop-allowed and drop-blocked at 240/360/480 logical widths plus 150% DPI). Reference: [ReactBits App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell) for an explicit overlay layer in an application shell; the Workbench drag portal and neutral dark tokens remain authoritative. Native retained-host evidence and original-size dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith('/primitives/feedback/workbench_toast.zui') &&
      prepared.document?.nodes?.['root']?.component === 'Snackbar'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Source layout defect at nodes.root: the 14px authored message wraps at the required 240px review width, but the old 32px Snackbar host positioned the first glyph above its bounds and the second line below them.',
        'The product Snackbar now reserves the tokenized 48px large height. The Penpot native painter uses a host-sized text placement and body panel so two lines remain inside the same semantic root; no message, action, severity, binding or close event was removed.',
        'On 2026-09-25, all 4 current Penpot scenes passed at 240/360/480 logical widths and 150% DPI, including measured text boundaries. Reference: [ReactBits Notifications](https://pro.reactbits.dev/docs/app-ui/notifications) for a distinct notification surface; the local Workbench semantic info colors and flat panel radius remain authoritative. Native retained-host evidence and original-size dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith('/shell/workbench_inspector_panel.zui') &&
      prepared.document?.nodes?.['inspector_layer_label']?.layout?.['width']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the 14px “Layer mask” label wrapping into two glyph fragments outside its 64px semantic node at 900x620; the adjacent field kept a fixed 120px width and left no safe text slot.',
        'The product Inspector source now reserves a 72px fixed label slot and a 96/112/120px stretch field. This keeps the label single-line at compact widths while preserving the value, event routes, binding and shared 4/8/12/16/24 spacing contract.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) for the stable right inspector rail; the local neutral dark Workbench tokens, flat panel split and 14px body scale remain authoritative. Current Penpot recapture passed the main-band default and scroll scenes; native retained-host and dual-end evidence remain pending.',
      ];
    if (
      sourcePath.endsWith('/shell/workbench_main_band.zui') &&
      prepared.document?.['penpot_dynamic_review_state']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Coverage defect: popup scenes were previously retained from an inactive imported module branch even after deterministic Penpot review pruned that branch, causing open/closed/focus cases to request a native painter that was not present in the final host.',
        'The catalog now derives LayoutReviewCase states after dynamic branch selection and records only the retained scene plus its authored scroll-before/scroll-after cases. No popup content is fabricated or hidden in the product source; active module behavior remains a runtime host concern.',
        'Reference: [ReactBits App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell) and [Sidebar](https://pro.reactbits.dev/docs/app-ui/sidebar) for explicit shell regions and responsive rail behavior. Current Penpot capture passed 6/6 cases (4 default sizes plus two scroll positions); native Editor evidence and dual-end image review remain pending.',
      ];
    if (
      sourcePath.endsWith('/shell/workbench_status_bar.zui') &&
      prepared.document?.nodes?.['status_grid']?.layout?.['width']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the wide-only Grid and Snap chips inheriting the shared 14px Chip padding while their fixed widths were too small; Grid: 10 cm split across two lines and painted outside the 24px status row at 1280px and 150% DPI.',
        'The source keeps the status bar at the tokenized 24px height, removes only the wide chip vertical padding, and reserves 96px for Grid plus 80px for Snap. Labels, icon actions, responsive wide-tier visibility and event routes remain unchanged.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) for persistent bottom status/navigation chrome; the local neutral dark Workbench palette and 14px body scale remain authoritative. Penpot recapture and native retained-host evidence are required before dual-end acceptance.',
      ];
    if (
      sourcePath.endsWith('/editor/asset_browser.zui') &&
      (
        prepared.document?.nodes?.['toolbar_title_row']?.layout?.['height'] as
          { preferred?: unknown } | undefined
      )?.preferred === 24 &&
      prepared.document?.nodes?.['content_header_path_text']?.props?.[
        'responsive_min_tier'
      ] === 'wide'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review of the Asset Browser source and ActivityDrawerWindow host found the authored 14px Sources, Details, References and Used By headings extending beyond their 16px rows; the toolbar title also sat against a 20px boundary and its 108px parent had no room for the corrected row. The source now reserves 24px heading rows, expands the toolbar panel to 112px, moves secondary captions to a 32px baseline and starts reference scroll bodies at 28px, preserving every heading and caption inside its flow region.',
        'At the 900px regular App Shell width, the stable left and right rails leave a compact content slot. The secondary “All assets” path now follows an explicit wide-tier responsive rule, so the primary “Content Browser” heading retains the full slot without wrapping outside its 24px row; the path remains present at wide desktop widths. No primary content, data, event, binding or slot is removed.',
        'The authored asset table now keeps its complete four-column rows at a 360px minimum inside an explicit horizontal ScrollableBox. The surrounding content panel remains the single vertical scroll owner for the page flow, while 240/360/480px table access is handled by the nested table scroll instead of clipping Size or Revision.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) for bounded sidebar/main/context relationships and [Sidebar](https://pro.reactbits.dev/docs/app-ui/sidebar) for responsive secondary navigation priority. The local flat dark Workbench tokens, semantic tiers and actual retained host remain authoritative; Penpot and native evidence must be recaptured after this source fingerprint change.',
      ];
    if (
      sourcePath.endsWith('/components/showcase/showcase_state_panel.zui') &&
      (
        prepared.document?.nodes?.['last_control']?.layout?.['height'] as
          { preferred?: unknown } | undefined
      )?.preferred === 48
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the compact 240px showcase panel wrapping retained event values below their fixed 32px PropertyRow bounds; the measured fragments for “event yet”, “Runtime UI event”, “control” and “payload” escaped their semantic rows.',
        'The source keeps the complete event/value strings, 14px body text, bindings and event routes, while the four content-bearing PropertyRow instances reserve a 48px two-line height. This is a flow-driven compact translation, not hidden text or a filename-specific replacement.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) with [Agent Activity 3](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-3) supporting a compact status rail; the local flat neutral Editor tokens remain authoritative. Current Penpot recapture passed all four width/DPI scenes; native host evidence and dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith('/editor/component_showcase.zui') &&
      prepared.document?.nodes?.['component_showcase_body']?.children
        ?.length === 3
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the component showcase body distributing its 640px shell almost evenly across navigation, content and state slots; the main content fell to roughly 139px and long labels escaped their 24px rows.',
        'The product source now gives navigation and state fixed bounded rails, a 220px minimum to the main slot and the remaining width as stretch. The state rail reserves 240px for its complete heading, while authored component text, slots, events and scroll behavior remain intact; no content is hidden or replaced.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) for the bounded sidebar/main/context relationship; local neutral Editor tokens and the actual retained host remain authoritative. Current Penpot capture passed all 31 default, open, closed, focus, empty, selected and scroll cases; native retained-host evidence and dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith(
        '/components/showcase/showcase_collections_section.zui',
      ) &&
      (
        prepared.document?.nodes?.['section_title']?.layout?.['height'] as
          { preferred?: unknown } | undefined
      )?.preferred === 44
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the complete “Collections and Inspector Structures” heading wrapping outside its fixed 24px title row in the compact component-showcase main slot.',
        'The source reserves a 44px two-line heading row and keeps the full authored title, collection examples, inspector structures, child order and scroll semantics intact. This is a flow-sized row correction, not a hidden or abbreviated label.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) for responsive main-slot density; the local flat showcase surface and 14px body token remain authoritative. Current Penpot capture passed all 19 collection-section cases; native component-host evidence and dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith(
        '/components/workbench/composites/animation/workbench_blend_space_details.zui',
      ) &&
      (
        prepared.document?.nodes?.['sample_position']?.layout?.['height'] as
          { preferred?: unknown } | undefined
      )?.preferred === 48
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the 14px “Sample position” label wrapping into a second fragment below its fixed 28px WorkbenchPropertyRow in every width, including scroll-content scenes.',
        'The product source reserves a 48px content-driven row for that semantic label/value pair. The complete label, value, component identity and edit events remain intact; no clipping, abbreviation or overlay is used.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) with [Data Table](https://pro.reactbits.dev/docs/app-ui/data-table) supporting a dense editable detail surface; local neutral Workbench tokens and the single ScrollableBox owner remain authoritative. Current Penpot capture passed all 23 scenes; native retained-host evidence and dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith(
        '/components/workbench/modules/core/assets/workbench_assets_workspace.zui',
      ) &&
      (
        prepared.document?.nodes?.['assets_center_title']?.layout?.['width'] as
          { min?: unknown } | undefined
      )?.min === 220
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the authored Content/Environment/Forest scope title painted past its 180px header slot at every viewport; the compact table also wrapped the Static Mesh cell outside its 30px row when the type column was allocated from header-only measurements.',
        'The source reserves a measured 220px title slot, and the shared Workbench table bridge now includes the widest authored cell in minimum-column allocation before dropping lower-priority columns. Labels, asset values, selection routes and the single table ScrollableBox remain intact; no content is hidden to pass the capture.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) with [Data Table](https://pro.reactbits.dev/docs/app-ui/data-table) for the navigation/detail/table relationship. On 2026-09-25 the rebuilt bundle passed all 6 current Penpot scenes, including both 640px scroll positions; native retained-host evidence and dual-end review remain pending.',
      ];
    const compactWorkbenchRows = [
      {
        suffix:
          '/components/workbench/modules/core/ai/workbench_perception_workspace.zui',
        node: 'perception_map_row_01',
        label: 'Sight Cone',
        value: 'target visible',
      },
      {
        suffix:
          '/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui',
        node: 'ability_graph_row',
        label: 'Graph',
        value: 'Damage -> End',
      },
      {
        suffix:
          '/components/workbench/modules/core/rendering/workbench_render_workspace.zui',
        node: 'render_graph_row_02',
        label: 'Lighting',
        value: '1.84 ms',
      },
    ].find((item) => sourcePath.endsWith(item.suffix));
    if (
      compactWorkbenchRows &&
      (
        prepared.document?.nodes?.[compactWorkbenchRows.node]?.layout?.[
          'height'
        ] as { preferred?: unknown } | undefined
      )?.preferred === 48
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        `Original-size Penpot review found the authored ${compactWorkbenchRows.label} value (including “${compactWorkbenchRows.value}”) wrapping below its 34px WorkbenchPropertyRow at the 640px compact workspace width. The source now reserves a 48px content-bearing row, preserving the full deterministic value, route and row semantics inside the scroll host; no text is hidden or abbreviated.`,
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) for dense bounded main/context surfaces; the local neutral Workbench tokens, 14px body scale and 4/8/12/16/24 spacing remain authoritative. Penpot recapture passed the wide and compact states where applicable; native retained-host and dual-end evidence remain pending.',
      ];
    if (
      sourcePath.endsWith(
        '/modules/extensions/animation/workbench_extension_blend_space_workspace.zui',
      ) &&
      (
        prepared.document?.nodes?.['blend_space_preview_camera']?.layout?.[
          'width'
        ] as { preferred?: unknown } | undefined
      )?.preferred === 96
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the authored “Perspective” WorkbenchChip measured 74px of glyph content inside an 80px preferred slot, so the right side of the label escaped its chip at the preview toolbar. The source now reserves a 96px preferred semantic chip width (112px maximum) while retaining the same camera state and toolbar order.',
        'Reference: [ReactBits App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell) for compact toolbar action grouping; local Workbench chip padding, neutral palette and 14px control typography remain authoritative. Penpot recapture and native retained-host evidence remain required before acceptance.',
      ];
    if (
      sourcePath.endsWith(
        '/modules/extensions/ui/workbench_extension_ui_asset_editor_workspace.zui',
      ) &&
      prepared.document?.nodes?.['ui_asset_editor_workspace']?.component ===
        'Overlay' &&
      prepared.document?.nodes?.['ui_asset_editor_right']?.props?.[
        'responsive_min_tier'
      ] === 'wide'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the UI Asset Editor source keeping its 320px details panel in the 900/640px main slot. The authored layout now follows the App Shell bounded main/context relationship: an Overlay owns the main flow, a wide-only details drawer reserves its width, and the narrow/regular main slot keeps the rail, asset list and layout map reachable without sibling overflow.',
        'The center toolbar and layout table are explicit ScrollableBox owners, matching the UI Binding workspace contract; the complete UI Asset Layout Map, Root Panel, Inventory Grid, Equip Button, Binding and output values remain authored. No labels, controls, routes, bindings or unknown fields were hidden or replaced.',
        'Reference: [ReactBits App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4) for the optional context panel and [Data Table](https://pro.reactbits.dev/docs/app-ui/data-table) for bounded row content; local neutral Workbench tokens, 14px body scale and 4/8/12/16/24 spacing remain authoritative. Penpot recapture is required before native retained-host and dual-end acceptance.',
      ];
    if (
      sourcePath.includes('/material_components/') &&
      (
        prepared.document?.nodes?.['meta']?.layout?.['height'] as
          { preferred?: unknown } | undefined
      )?.preferred !== undefined &&
      Number(
        (
          prepared.document?.nodes?.['meta']?.layout?.['height'] as {
            preferred?: unknown;
          }
        )?.preferred,
      ) >= 100
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Original-size Penpot review found the shared Material prototype metadata grid using fixed 24px rows. At the required 240px component width, complete two-line metadata values escaped their rows and overlapped the following specimen.',
        'The source-owned metadata GridBox now reserves readable rows for complete values, and each authored prototype card grows to its actual metadata/sample/state flow. All labels, state specimens, component props and deterministic content remain present; no capture-only text hiding or canvas enlargement is used.',
        'Reference: [ReactBits App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell) for bounded responsive cards and [Data Table](https://pro.reactbits.dev/docs/app-ui/data-table) for dense, readable content; local Material theme colors and semantic state tokens remain authoritative. Native component-host evidence and dual-end review remain pending.',
      ];
    if (
      sourcePath.endsWith(
        '/material_components/data_display/material_chips.zui',
      ) &&
      prepared.document?.nodes?.['chip_label']?.layout?.['height']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'The authored Chip label contains a deterministic two-line “Styled Warn” specimen when the complete Material metadata card is reviewed at 240/360/480px. Its label slot now reserves a 36px content row, preserving the full label and delete affordance without clipping.',
      ];
    if (
      sourcePath.endsWith(
        '/material_components/data_display/material_timeline.zui',
      ) &&
      prepared.document?.nodes?.['timeline_item']?.layout?.['position']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'The Timeline specimen keeps its authored root label and item content in a non-overlapping flow: the sample reserves a 62px two-part row and the TimelineItem starts at an explicit 24px y offset. The complete Timeline/Build text and separator remain visible.',
      ];
    if (
      sourcePath.endsWith('/material_components/feedback/material_alert.zui') &&
      prepared.document?.nodes?.['alert_message']?.layout?.['position']
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'The Alert specimen keeps the authored root “Warning” label and the message slot as separate flow rows: the sample reserves 96px and alert_message begins at an explicit 48px y offset, removing the prior parent/slot text overlap without hiding either string.',
      ];
    if (
      sourcePath.endsWith('/primitives/feedback/workbench_dialog.zui') &&
      prepared.document?.nodes?.['root']?.component === 'Dialog'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Design review defect at nodes.root: the old projection exempted a component-root Dialog from popup visibility, so default, open and closed 240px previews all painted the same modal even though the product defaults open/popup_open to false.',
        'The bridge now lets explicit popup review state override authored defaults even at a component root. The open case is the informative primary specimen; default and closed remain separate semantically hidden cases, with blank pixels permitted only after the visibility tree confirms that no other content is visible. Product .zui and modal content stay unchanged.',
        'Before evidence: [closed but still painted](evidence/before-20260921-dialog-closed-still-visible-240x520-dpi1.png) SHA-256 3457d68455f87db55bed084cbc13b73e157d868aad6f3779db9666550ea522ef. Penpot open/closed screenshot and semantic checks are current; native modal and focus-return evidence remain pending.',
      ];
    if (
      sourcePath.endsWith(
        '/primitives/feedback/workbench_command_palette.zui',
      ) &&
      prepared.document?.nodes?.['root']?.component === 'CommandPalette'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Design review defect at nodes.root: a component-root CommandPalette ignored authored open=false, so its default preview showed commands; simply hiding the root then concealed the empty and focused review states as well.',
        'The projection now shows the invoked open/focused/empty states but keeps the authored default closed. The 240px empty review explicitly paints “No commands found”; source commands, query bindings and product default are unchanged. Native and focus-return evidence remain pending.',
      ];
    if (
      sourcePath.endsWith(
        '/primitives/feedback/workbench_notification_center.zui',
      ) &&
      prepared.document?.nodes?.['root']?.component === 'NotificationCenter'
    )
      entry.sourceChanges = [
        ...(entry.sourceChanges ?? []),
        'Design review defect at nodes.root: authored open=false was ignored for the component-root NotificationCenter, while applying a root visibility fix without state targeting would hide selected and empty specimens.',
        'The projection now treats open/selected/empty as invoked-panel scenarios and default as an explicitly closed panel. At 240px, “No notifications” remains visible in the empty panel; authored notification data and colors are unchanged. Native and focus-return evidence remain pending.',
      ];
    const previous = existingBySource.get(sourcePath);
    entry.reviewHistory = [...(previous?.reviewHistory ?? [])];
    if (previous?.review) entry.reviewHistory.push(previous.review);
    if (
      previous &&
      status === 'prepared' &&
      previous.sourceSha256 === entry.sourceSha256 &&
      previous.outputSha256 === entry.outputSha256 &&
      previous.dependencySha256 === entry.dependencySha256 &&
      previous.penpotInputSha256 === entry.penpotInputSha256 &&
      casesSha256(previous.cases ?? []) === casesSha256(entry.cases ?? []) &&
      previous.outputPath === entry.outputPath &&
      previous.resultPath === entry.resultPath &&
      previous.previewPath === entry.previewPath
    ) {
      entry.visualStatus = previous.visualStatus;
      entry.visualPluginSha256 = previous.visualPluginSha256;
      entry.visualError = previous.visualError;
      entry.visualPendingReason = previous.visualPendingReason;
      entry.visualScreenshot = previous.visualScreenshot;
      entry.visualSha256 = previous.visualSha256;
      entry.visualPixelRatio = previous.visualPixelRatio;
      entry.visualDetailPixelRatio = previous.visualDetailPixelRatio;
      entry.visualColorBucketCount = previous.visualColorBucketCount;
      entry.visualLayoutOverflowCount = previous.visualLayoutOverflowCount;
      entry.visualLayoutInvalidGeometryCount =
        previous.visualLayoutInvalidGeometryCount;
      entry.penpotEvidence = previous.penpotEvidence;
      entry.engineEvidence = previous.engineEvidence;
      entry.review = previous.review;
      if (!currentDualReview(entry)) delete entry.review;
      else entry.reviewHistory.pop();
    } else if (previous && status === 'prepared') {
      // Adding a case invalidates entry-level acceptance, not unchanged
      // per-case captures. The former primary must move before a new state
      // can use penpot.png or preview.png.
      const retained = await retainExpandedCaseEvidence(
        previous,
        entry,
        outputRoot,
        shouldWrite,
      );
      Object.assign(entry, retained);
    }
    // The catalog may normalize case envelopes (for example when a shared
    // review host gains an explicit mapping) without changing any captured
    // scene. If every current case still has a verified, passed Penpot
    // screenshot, restore the entry-level Penpot status instead of forcing a
    // full recapture. A failed attempt always wins over an older pass.
    if (previous && status === 'prepared') {
      const currentCases = entry.cases ?? [];
      const retainedPenpot = entry.penpotEvidence ?? [];
      const hasFailedAttempt = (previous.penpotEvidence ?? []).some(
        ({ status: evidenceStatus }) => evidenceStatus === 'failed',
      );
      const completePenpotEvidence =
        currentCases.length > 0 &&
        !hasFailedAttempt &&
        currentCases.every((reviewCase) =>
          retainedPenpot.some(
            ({ caseId, status: evidenceStatus }) =>
              caseId === reviewCase.id && evidenceStatus === 'passed',
          ),
        );
      if (completePenpotEvidence) {
        entry.visualStatus = 'passed';
        entry.visualPluginSha256 = previous.visualPluginSha256;
        entry.visualError = undefined;
        entry.visualPendingReason = undefined;
        entry.visualScreenshot = previous.visualScreenshot;
        entry.visualSha256 = previous.visualSha256;
        entry.visualPixelRatio = previous.visualPixelRatio;
        entry.visualDetailPixelRatio = previous.visualDetailPixelRatio;
        entry.visualColorBucketCount = previous.visualColorBucketCount;
        entry.visualLayoutOverflowCount = previous.visualLayoutOverflowCount;
        entry.visualLayoutInvalidGeometryCount =
          previous.visualLayoutInvalidGeometryCount;
      }
    }
    entries.push(entry);

    if (shouldWrite) {
      const directory = resolveLayoutCatalogPath(outputRoot, relativeDir);
      const sourceDirectory = layoutSourceRoot(directory);
      await mkdir(sourceDirectory, { recursive: true });
      await mkdir(resolve(sourceDirectory, "evidence"), { recursive: true });
      await mkdir(directory, { recursive: true });
      await mkdir(resolveLayoutCatalogPath(directory, 'evidence'), { recursive: true });
      if (consumerHost) {
        await writeFile(
          resolveLayoutCatalogPath(directory, `evidence/${hostFile}.zui`),
          consumerHost.source,
          'utf8',
        );
        await writeFile(
          resolveLayoutCatalogPath(
            directory,
            hostFile === 'theme-consumer'
              ? 'evidence/theme-consumers.json'
              : hostFile === 'editor-host'
                ? 'evidence/editor-host-sources.json'
                : 'evidence/component-host-sources.json',
          ),
          `${JSON.stringify(consumerHost.consumers, null, 2)}\n`,
          'utf8',
        );
      }
      if (
        previous &&
        !previous.penpotInputPath &&
        existsSync(resolveLayoutCatalogPath(outputRoot, previous.previewPath))
      ) {
        const legacyPath = resolveLayoutCatalogPath(directory, 'evidence/legacy-penpot.png');
        await rename(
          resolveLayoutCatalogPath(outputRoot, previous.previewPath),
          existsSync(legacyPath)
            ? resolveLayoutCatalogPath(directory, `evidence/legacy-penpot-${Date.now()}.png`)
            : legacyPath,
        );
      }
      await writeFile(resolveLayoutCatalogPath(directory, `${entryKey.name}.zui`), sourceBytes);
      if (penpotInputPath && serialized !== undefined)
        await writeFile(
          resolveLayoutCatalogPath(outputRoot, penpotInputPath),
          serialized,
          'utf8',
        );
      await writeFile(
        resolveLayoutCatalogPath(directory, 'result.md'),
        entry.visualStatus === 'passed'
          ? renderAcceptedResult(entry)
          : renderResult(entry),
        'utf8',
      );
    }
  }

  const catalogEntries = requestedSources.length
    ? mergeSelectedCatalogEntries(existingManifest?.entries ?? [], entries)
    : entries;
  const manifest: CatalogManifest = {
    schema: 'dev.zircon.zui.layout-catalog',
    version: 1,
    generatedAt: new Date().toISOString(),
    repoRoot,
    sourceCount: catalogEntries.length,
    sourceDiscovery: requestedSources.length
      ? (existingManifest?.sourceDiscovery ?? discovery.metadata)
      : discovery.metadata,
    entries: catalogEntries,
  };
  if (shouldWrite) {
    await mkdir(outputRoot, { recursive: true });
    await writeFile(
      resolveLayoutCatalogPath(outputRoot, 'catalog.json'),
      `${JSON.stringify(manifest, null, 2)}\n`,
      'utf8',
    );
    await writeFile(
      resolveLayoutCatalogPath(outputRoot, 'index.md'),
      renderIndex(manifest),
      'utf8',
    );
  }
  console.log(
    JSON.stringify({
      outputRoot,
      sourceCount: entries.length,
      catalogSourceCount: catalogEntries.length,
      prepared: entries.filter(
        ({ status: entryStatus }) => entryStatus === 'prepared',
      ).length,
      failed: entries.filter(
        ({ status: entryStatus }) => entryStatus === 'failed',
      ).length,
      failedPaths: entries
        .filter(({ status: entryStatus }) => entryStatus === 'failed')
        .map(({ sourcePath, diagnostics: entryDiagnostics }) => ({
          sourcePath,
          diagnostics: entryDiagnostics,
        })),
      visualPending: entries.filter(
        ({ visualStatus }) => visualStatus !== 'passed',
      ).length,
      wrote: shouldWrite,
    }),
  );
}

async function readExistingManifest(): Promise<CatalogManifest | null> {
  const path = resolveLayoutCatalogPath(outputRoot, 'catalog.json');
  if (!existsSync(path)) return null;
  try {
    return JSON.parse(await readFile(path, 'utf8')) as CatalogManifest;
  } catch {
    return null;
  }
}

async function readTableSourceReport(): Promise<
  Map<string, TableSourceReportRecord>
> {
  const reportPath = resolve(
    repoRoot,
    'docs/_data/layout/evidence/table-source-cells-applied.json',
  );
  if (!existsSync(reportPath)) return new Map();
  try {
    const report = JSON.parse(
      await readFile(reportPath, 'utf8'),
    ) as TableSourceReport;
    return new Map(
      (report.records ?? []).map((record) => [record.sourcePath, record]),
    );
  } catch {
    return new Map();
  }
}

async function readMaterialSourceReport(): Promise<
  Map<string, MaterialSourceReportRecord>
> {
  const path = resolve(
    repoRoot,
    'docs/_data/layout/evidence/material-source-applied.json',
  );
  if (!existsSync(path)) return new Map();
  const report = JSON.parse(await readFile(path, 'utf8')) as {
    applied: boolean;
    records: MaterialSourceReportRecord[];
  };
  return new Map(
    report.applied
      ? report.records.map((record) => [record.sourcePath, record])
      : [],
  );
}

const BRIDGE_ROUNDTRIP_FIXTURE =
  'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui';
const REACTBITS_REFERENCE_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_agent_workspace.zui';
const REACTBITS_NATIVE_COMPONENT_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_agent_native_components.zui';
const REACTBITS_INTERACTION_SURFACES_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_workbench_interaction_surfaces.zui';
const REACTBITS_AGENT_WORKFLOW_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_agent_workflow_components.zui';
const REACTBITS_DATA_SURFACE_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_data_surface_components.zui';
const REACTBITS_AUTH_ONBOARDING_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_auth_onboarding_components.zui';
const REACTBITS_SETTINGS_FORM_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_settings_form_components.zui';
const REACTBITS_KANBAN_SCHEDULING_FIXTURE =
  'zircon_runtime/tests/fixtures/ui/reactbits_kanban_scheduling_components.zui';

const SOURCE_SCAN_EXCLUDED_ROOTS = [
  'docs/_data/layout/**',
  'docs/ui/zui/**',
  'third_party/**',
  'tools/penpot/**',
  'dev/**',
  'target/**',
  'build/**',
  'dist/**',
];

interface DiscoveredZuiSources {
  paths: string[];
  metadata: CatalogSourceDiscovery;
}

export async function discoverZuiSources(): Promise<DiscoveredZuiSources> {
  const result = await execFileAsync(
    'git',
    [
      '-C',
      repoRoot,
      'ls-files',
      '--cached',
      '--others',
      '--exclude-standard',
      '--',
      '*.zui',
      ...SOURCE_SCAN_EXCLUDED_ROOTS.map((path) => `:(exclude)${path}`),
    ],
    {
      maxBuffer: 1024 * 1024 * 8,
    },
  );
  const controlledOrUnignored = result.stdout
    .split(/\r?\n/)
    .map((path) => normalizeSourcePath(path))
    .filter((path) => path.length > 0)
    .filter(isSourceLayoutPath);
  const paths = [
    ...new Set([...controlledOrUnignored, BRIDGE_ROUNDTRIP_FIXTURE]),
  ]
    .filter((path) => existsSync(resolve(repoRoot, path)))
    .sort((left, right) => left.localeCompare(right, 'en'));
  return {
    paths,
    metadata: {
      scan: 'git-ls-files-cached-others-exclude-standard',
      controlledOrUnignoredSourceCount: new Set(controlledOrUnignored).size,
      projectOwnedScannedFixtures: [
        {
          sourcePath: REACTBITS_REFERENCE_FIXTURE,
          reason:
            'Project-owned ReactBits layout reference fixture; it is part of the normal Git source scan and is not a generated copy.',
        },
        {
          sourcePath: REACTBITS_NATIVE_COMPONENT_FIXTURE,
          reason:
            'Project-owned ReactBits native component probe; it is part of the normal Git source scan and keeps semantic AgentChat/ChatComposer coverage explicit.',
        },
        {
          sourcePath: REACTBITS_INTERACTION_SURFACES_FIXTURE,
          reason:
            'Project-owned ReactBits workbench interaction fixture; it keeps command, dialog, notification, drag, and toast states in one responsive surface.',
        },
        {
          sourcePath: REACTBITS_AGENT_WORKFLOW_FIXTURE,
          reason:
            'Project-owned ReactBits agent workflow fixture; it keeps plan, tool-call, approval, and usage states source-owned while exercising the native painter boundary.',
        },
        {
          sourcePath: REACTBITS_DATA_SURFACE_FIXTURE,
          reason:
            'Project-owned ReactBits data-surface fixture; it keeps filtering, folder navigation, table rows, metrics, and empty-state guidance source-owned while exercising DataGrid and TreeView painters.',
        },
        {
          sourcePath: REACTBITS_AUTH_ONBOARDING_FIXTURE,
          reason:
            'Project-owned ReactBits auth/onboarding fixture; it keeps centered sign-in, provider actions, verification code flow, workspace stepper, slug preview, and a windowed workspace picker source-owned.',
        },
        {
          sourcePath: REACTBITS_SETTINGS_FORM_FIXTURE,
          reason:
            'Project-owned ReactBits settings-form fixture; it keeps section navigation, label-left form rows, windowed API-key data, unsaved actions, notification feedback, and destructive confirmation source-owned.',
        },
        {
          sourcePath: REACTBITS_KANBAN_SCHEDULING_FIXTURE,
          reason:
            'Project-owned ReactBits kanban/scheduling fixture; it keeps WIP columns, capacity summary, date-strip selection, windowed agenda data, and booking actions source-owned.',
        },
      ].filter((fixture) => existsSync(resolve(repoRoot, fixture.sourcePath))),
      explicitProjectFixtures: [
        {
          sourcePath: BRIDGE_ROUNDTRIP_FIXTURE,
          reason:
            'Project-owned Penpot round-trip contract fixture; included explicitly because tools/penpot is excluded from general generated-source and dependency discovery.',
        },
      ],
      excludedRoots: SOURCE_SCAN_EXCLUDED_ROOTS,
    },
  };
}

export async function trackedZuiPaths(): Promise<string[]> {
  return (await discoverZuiSources()).paths;
}

export function isSourceLayoutPath(path: string): boolean {
  path = normalizeSourcePath(path);
  if (path === BRIDGE_ROUNDTRIP_FIXTURE) return true;
  return (
    path.endsWith('.zui') &&
    !/^(docs\/(?:layout|_data\/layout|ui\/zui)|third_party|tools\/penpot|dev|target|build|dist)\//.test(path)
  );
}

/** Classify authored assets independently from their output category. */
export function classifySourcePath(
  sourcePath: string,
  sourceKind?: string,
  document?: ZuiDocument,
): SourceClassification {
  const path = normalizeSourcePath(sourcePath);
  if (
    path === BRIDGE_ROUNDTRIP_FIXTURE ||
    path.includes('/tests/fixtures/') ||
    path.includes('/ui/runtime/fixtures/') ||
    /(?:^|[/_-])fixture(?:[/_-]|\.zui$)/i.test(path)
  )
    return 'test-fixture';
  if (
    sourceKind === 'theme' ||
    sourceKind === 'theme_tokens' ||
    sourceKind === 'style' ||
    /(?:^|\/)(?:theme|themes|tokens)(?:\/|\.zui$)/i.test(path) ||
    /(?:^|[/_-])(?:theme|tokens)(?:[/_-]|\.zui$)/i.test(path)
  )
    return 'theme';
  if (isStructuralWorkbenchHostDocument(document)) return 'dynamic-host';
  if (
    sourceKind === 'component' ||
    /(?:^|\/)(?:components|primitives)(?:\/|\.zui$)/i.test(path)
  )
    return 'component';
  if (
    sourceKind === 'dynamic_host' ||
    /(?:^|\/)(?:workbench|hosts|host|shell|windows)(?:\/|\.zui$)/i.test(path) ||
    path.startsWith('examples/woc/')
  )
    return 'dynamic-host';
  if (
    sourceKind === 'view' &&
    path.startsWith('zircon_plugins/') &&
    isStructuralDynamicHostDocument(document)
  )
    return 'dynamic-host';
  // `view` is the authored page/screen form.  Keep it distinct from host
  // shells so the acceptance queue can choose the product-page viewport.
  return 'product-page';
}

/**
 * Workbench module/index assets and retained host views may contain only
 * structural containers plus custom Workbench mount components. They are not
 * standalone pages: the retained runtime supplies their active branch. Keep
 * these documents in the dynamic-host queue so Penpot evidence records the
 * missing host contract instead of fabricating a replacement screen.
 */
export function isStructuralWorkbenchHostDocument(
  document?: ZuiDocument,
): boolean {
  if (
    !document ||
    (document.asset.kind !== 'component' && document.asset.kind !== 'view')
  )
    return false;
  // ActivityDrawerWindow is authored as an otherwise slot-only component, but
  // it has a real Penpot review host (the shipped editor window consumers) in
  // prepareActivityDrawerReviewHost. Do not classify it as a headless dynamic
  // mount or the catalog will skip that deterministic host and leave every
  // scene pending with a blank Slot-only projection.
  if (
    Object.prototype.hasOwnProperty.call(
      document.components ?? {},
      'ActivityDrawerWindow',
    )
  )
    return false;
  // A component whose caller must provide a slot is not a headless runtime
  // host. Its specimen must mount real accepted children before visual review;
  // otherwise a blank rectangle can pass image/geometry checks as a component.
  if (requiredComponentSlots(document).length) return false;
  const nodes = Object.values(document.nodes ?? {});
  if (!nodes.length) return false;
  const visible = nodes.filter((node) => {
    const visibility = node.props?.['visibility'];
    return (
      visibility !== 'collapsed' &&
      visibility !== 'hidden' &&
      node.props?.['visible'] !== false
    );
  });
  if (!visible.length) return false;
  if (
    visible.some((node) =>
      ['text', 'value_text', 'label', 'placeholder', 'value'].some((key) => {
        const value = node.props?.[key];
        return typeof value === 'string' && value.trim() !== '';
      }),
    )
  )
    return false;
  const structural = visible.every(
    (node) =>
      STRUCTURAL_DYNAMIC_HOST_COMPONENTS.has(node.component) ||
      isWorkbenchMountComponent(node),
  );
  return (
    structural &&
    visible.some(
      (node) =>
        isWorkbenchMountComponent(node) ||
        ['Container', 'Slot', 'Space'].includes(node.component),
    )
  );
}

function isWorkbenchMountComponent(node: ZuiNode): boolean {
  const component = node.component;
  const controlId = node.control_id ?? '';
  if (!component.startsWith('Workbench')) return false;
  return (
    /(Workspaces?|Host|Body|Drawer|Panel|Module|Shell)$/.test(component) ||
    /(Workspaces?|Host|Body|Drawer|Panel|Module|Shell)/.test(controlId)
  );
}

const STRUCTURAL_DYNAMIC_HOST_COMPONENTS = new Set([
  'Container',
  'DockHost',
  'GridBox',
  'HorizontalGroup',
  'Overlay',
  'Panel',
  'ScrollBox',
  'ScrollableBox',
  'Slot',
  'Space',
  'Splitter',
  'VerticalGroup',
]);

/**
 * Identify an authored plugin view whose visible leaves are explicit runtime
 * mounts. Labels and controls deliberately disqualify the document so normal
 * plugin pages remain product pages and must render their real content.
 */
export function isStructuralDynamicHostDocument(
  document?: ZuiDocument,
): boolean {
  if (!document || document.asset.kind !== 'view') return false;
  const nodes = Object.values(document.nodes ?? {});
  if (!nodes.length) return false;
  const visible = nodes.filter((node) => {
    const visibility = node.props?.['visibility'];
    return (
      visibility !== 'collapsed' &&
      visibility !== 'hidden' &&
      node.props?.['visible'] !== false
    );
  });
  if (!visible.length) return false;
  if (
    visible.some((node) => {
      if (!STRUCTURAL_DYNAMIC_HOST_COMPONENTS.has(node.component)) return true;
      const props = node.props ?? {};
      return ['text', 'value_text', 'label', 'placeholder'].some((key) => {
        const value = props[key];
        return typeof value === 'string' && value.trim() !== '';
      });
    })
  )
    return false;
  return visible.some(
    (node) =>
      node.component === 'Space' ||
      node.component === 'Slot' ||
      (!node.children?.length && node.component === 'Container'),
  );
}

export function sourceAssetRoot(sourcePath: string): string {
  const normalized = normalizeSourcePath(sourcePath);
  // Match the resolver and delivery package roots: an authored file under an
  // `assets` tree shares that tree's resource namespace, while Rust fixtures
  // use the enclosing `ui_zui` root.  Other packages keep their local folder
  // as the narrowest safe fallback.
  const assets = normalized.indexOf('/assets/');
  if (assets >= 0) return normalized.slice(0, assets + '/assets'.length);
  const uiZui = normalized.indexOf('/ui_zui/');
  if (uiZui >= 0) return normalized.slice(0, uiZui + '/ui_zui'.length);
  const slash = normalized.lastIndexOf('/');
  return slash < 0 ? '.' : normalized.slice(0, slash);
}

export function collectComponentDependencies(
  document: ZuiDocument | undefined,
): string[] {
  if (!document) return [];
  const values = new Set<string>();
  const nodes = record(document.nodes);
  for (const node of Object.values(nodes ?? {})) {
    const component = record(node)?.['component'];
    if (typeof component === 'string' && component.trim())
      values.add(component.trim());
  }
  const components = record(document.components);
  for (const key of Object.keys(components ?? {})) values.add(key);
  return [...values].sort((left, right) => left.localeCompare(right, 'en'));
}

function normalizeSourcePath(path: string): string {
  return path.trim().replaceAll('\\', '/').replace(/^\.\//, '');
}

function readSourceHeader(source: string): SourceHeader {
  try {
    const parsed = parse(source) as Record<string, unknown>;
    const asset = record(parsed['asset']);
    return {
      kind: stringValue(asset?.['kind']) ?? 'unknown',
      version: scalarVersion(asset?.['version']),
      id: stringValue(asset?.['id']) ?? 'unknown',
      displayName: stringValue(asset?.['display_name']) ?? '',
    };
  } catch {
    return {
      kind: 'unknown',
      version: 'unknown',
      id: 'unknown',
      displayName: '',
    };
  }
}

export function renderResult(entry: CatalogEntry): string {
  const review = currentDualReview(entry) ? entry.review : undefined;
  const link = (path: string, label: string) =>
    `[${label}](${relative(resolve(outputRoot, dirname(entry.resultPath)), resolveLayoutCatalogPath(outputRoot, path)).replaceAll('\\', '/')})`;
  const primary = (renderer: 'penpot' | 'engine', path?: string) => {
    const item = entry[`${renderer}Evidence`]?.find(
      (item) =>
        item.status !== 'failed' &&
        item.screenshotPath === path &&
        item.screenshotSha256 &&
        item.sourceSha256 === entry.sourceSha256 &&
        item.dependencySha256 === entry.dependencySha256,
    );
    return item && path
      ? `${link(path, basename(path))} (${item.status === 'pending' ? `pending: ${item.pendingReason ?? 'capture requires review'}` : 'captured; acceptance is separate'})`
      : `uncaptured${path ? `; reserved: ${path}` : ''}`;
  };
  const caseRows = (entry.cases ?? []).map((reviewCase) => {
    const captures = (['penpot', 'engine'] as const).map((renderer) => {
      const item = entry[`${renderer}Evidence`]?.find(
        (candidate) => candidate.caseId === reviewCase.id,
      );
      return item?.screenshotPath && item.screenshotSha256
        ? link(item.screenshotPath, item.status)
        : (item?.status ?? 'uncaptured');
    });
    const { width, height } = reviewCase.viewport;
    return `| ${reviewCase.id} | ${width}x${height} | ${reviewCase.dpi} | ${reviewCase.locale} | ${captures.join(' | ')} |`;
  });
  const evidence = (['penpot', 'engine'] as const)
    .map((renderer) => {
      const items = entry[`${renderer}Evidence`] ?? [];
      return `- ${renderer}: ${items.filter((item) => item.status === 'passed').length}/${entry.cases?.length ?? 0} cases passed; ${items.filter((item) => item.status === 'pending').length} pending; ${items.filter((item) => item.status === 'failed').length} failed`;
    })
    .join('\n');
  const captureFailures = (['penpot', 'engine'] as const).flatMap((renderer) =>
    (entry[`${renderer}Evidence`] ?? [])
      .filter((item) => item.status === 'failed')
      .map((item) => {
        const geometry = item.geometryPath
          ? `; ${link(item.geometryPath, 'geometry')}`
          : '';
        const detail = (
          item.error ??
          entry.visualError ??
          'Capture did not complete'
        )
          .split('\n    at ')[0]
          .trim();
        return `### ${renderer} / ${item.caseId}\n\n${geometry ? geometry.slice(2) + '\n\n' : ''}\n\n\`\`\`text\n${detail.replaceAll('```', "'''")}\n\`\`\``;
      }),
  );
  const capturePending = (['penpot', 'engine'] as const).flatMap((renderer) =>
    (entry[`${renderer}Evidence`] ?? [])
      .filter((item) => item.status === 'pending')
      .map(
        (item) =>
          `- ${renderer} / ${item.caseId}: ${item.pendingReason ?? 'Capture requires review.'}`,
      ),
  );
  const textAuditPath = `${dirname(entry.outputPath)}/evidence/text-structure.json`;
  const textAudit = existsSync(resolveLayoutCatalogPath(outputRoot, textAuditPath))
    ? `- Current measured text findings: ${link(textAuditPath, 'text-structure.json')} (raw glyph bounds; scroll/overlay context remains subject to visual review).`
    : '- Current measured text audit: not generated for this entry.';
  return `---
source_path: ${entry.sourcePath}
source_sha256: ${entry.sourceSha256}
dependency_sha256: ${entry.dependencySha256 ?? 'unrecorded'}
penpot_input_sha256: ${entry.penpotInputSha256 ?? 'unrecorded'}
review_status: ${review?.status ?? 'pending'}
---

# Layout Review

- Source mirror: ${link(entry.outputPath, entry.name + '.zui')}
- Source kind/version: ${entry.sourceKind} / ${String(entry.sourceVersion)}
- Penpot input: ${entry.penpotInputPath ?? 'unprepared'}
- Review consumer host: ${entry.cases?.[0]?.reviewHost?.path ?? 'source asset'}
- Penpot primary: ${primary('penpot', entry.penpotPreviewPath)}
- Engine primary: ${primary('engine', entry.previewPath)}
- Preparation: ${entry.status}

## Evidence

${evidence}

| Case | Logical Viewport | DPI | Locale | Penpot | Engine |
|---|---|---|---|---|---|
${caseRows.join('\n')}

Scene data, host paths, dependency hashes, renderer hashes and per-image hashes are recorded in the matching source entry in ${link('catalog.json', 'catalog.json')}. A captured image or a passed automated check is not visual acceptance.

## Findings

${review ? review.observations : 'Pending image review. Projection and pixel checks do not establish visual acceptance.'}

## Text Structure Audit

${textAudit}

## Capture Findings

${captureFailures.length ? captureFailures.join('\n\n') : 'No capture failures recorded.'}

## Pending Capture Work

${capturePending.length ? capturePending.join('\n') : 'Missing native or unreviewed evidence remains pending.'}

## Source Corrections

${entry.sourceChanges?.length ? entry.sourceChanges.map((change) => `- ${change}`).join('\n') : 'See the image findings and source history. No additional source correction record is attached to this fingerprint.'}

## Projection Diagnostics

${renderDiagnostics(entry)}

## Projection Transformations

${entry.changes.length ? entry.changes.map((change) => `- ${change}`).join('\n') : '- None'}

- Historical review records: ${entry.reviewHistory?.length ?? 0}
`;
}

export function renderAcceptedResult(entry: CatalogEntry): string {
  return renderResult(entry);
}

const CHANGE_DESCRIPTIONS: Record<string, string> = {
  'embed-authored-image-resources':
    '嵌入页面已声明的仓库图片，通过 Penpot 原生媒体图层显示背景，保留原资源 URI。',
  'fit-anchored-root-to-content':
    '根据锚点、轴心和子面板尺寸扩展根视口，避免居中面板被根画布裁切。',
  'establish-editor-viewport':
    '完整页面设定 1280x800 的首选视口并保留伸展约束；编辑器页面增加一致的 16px 外边距。',
  'establish-component-preview-width':
    '为多控件纵向组件提供 480px 独立预览宽度，避免默认缩成狭窄长列。',
  'align-authored-asset-table-cells':
    '将页面自己的资产名称、类型、大小和版本写入表格列数据，替换继承的预制件示例值。',
  'materialize-shared-prefab-instances':
    '按共享 .zui 预制件定义实例化节点、默认属性和子布局，实例覆盖保持优先；来源写入 penpot_prefab_source，避免预览只显示空框。',
  'readable-control-typography':
    '将过小的标签字体提升到 11px，并同步增加行高和标签宽度，消除状态文字挤压。',
  'isolate-selected-utility-tab':
    '资产浏览器只展示当前 Preview 页，其他工具页设为折叠状态，避免多个标签页内容叠加。',
  'resolve-imported-design-foundation':
    '解析导入的主题令牌和共享预制件样式，保留依赖来源，使颜色、字号、间距使用真实基础组件定义。',
  'migrate-legacy-document':
    '将旧版 .zui 结构迁移为 v2 语义文档，保留原始迁移元数据。',
  'normalize-display-name':
    '补齐稳定的资产显示名称，避免 Penpot 图层标题为空。',
  'normalize-schema-version':
    '统一资产 schema 版本为 v2，确保导入与导出契约一致。',
  'normalize-paint':
    '规范颜色、透明度、圆角和边框数值，避免异常绘制与对比度漂移。',
  'normalize-token-colors':
    '统一设计 token 的颜色表示，保证主题值可被 Penpot 解析。',
  'normalize-geometry': '修正负间距、尺寸上下限和非法几何值，避免子项越界。',
  'normalize-style': '整理节点样式块，使样式覆盖与基础预制件层级一致。',
  'normalize-text-alignment': '将文本对齐方式归一化为 Penpot 支持的方向值。',
  'fit-component-root-to-content':
    '按内容重新计算组件根节点的最小/首选尺寸，消除文本堆叠和空白画布。',
  'fit-auto-layout-to-content':
    '自底向上扩展普通自动布局容器的固有尺寸，保留滚动视口并消除可见子项堆叠。',
  'assign-penpot-design-profile':
    '声明 zircon.penpot.prefabs.v1 设计档案，固定本批资产的 Penpot 解释方式。',
  'embed-penpot-prefab-tokens':
    '嵌入统一画布、表面、边框、文字、强调色和间距 token。',
  'apply-penpot-prefab-classes':
    '为语义节点分配统一 Penpot 预制件类别，替代零散的原始样式。',
  'embed-penpot-prefab-styles':
    '嵌入基础预制件样式表，统一控件圆角、边框、填充和文字层级。',
  'separate-overlay-bottom-actions':
    '下移覆盖层底部操作轨，给角落按钮留出明确安全间距。',
  'constrain-overlay-side-panel':
    '按底部操作轨约束侧面板高度，避免浮层内容互相遮挡。',
  'add-penpot-node-label':
    '补齐稳定的 Penpot 节点标签，便于图层树检查与回写定位。',
  'container-kind-inferred':
    '根据组件语义推断容器类型，保证自动布局方向可复现。',
  'metadata-preserved':
    '保留运行时专属字段到可逆元数据，不把非视觉状态误绘制成控件。',
  'fallback-document':
    '生成可见迁移审查画布，并保留原始解析错误以阻止静默丢失。',
  'preserve-source-diagnostics':
    '保留源文件诊断信息，等待对应 owner 修复后再纳入正式资产。',
  'materialize-table-cell-options':
    '将源表格行的可编辑列显式写入 options，避免 Penpot 预览依赖拼接文本猜测列边界。',
};

export function describeChange(change: string): string {
  return (
    CHANGE_DESCRIPTIONS[change] ??
    `Penpot projection transformation: ${humanizeChangeId(change)}.`
  );
}

export function describeDefects(entry: CatalogEntry): string[] {
  const defects: string[] = [];
  if (
    entry.sourceFormat === 'legacy' ||
    entry.changes.includes('migrate-legacy-document')
  ) {
    defects.push(
      '源文件采用旧版 .zui 格式，存在与当前 Penpot v2 契约不一致的风险。',
    );
  }
  if (entry.changes.includes('fit-component-root-to-content')) {
    defects.push(
      '组件根节点的固有尺寸不足以承载内容，容易造成文本堆叠或无意义空白。',
    );
  }
  if (entry.changes.includes('fit-auto-layout-to-content')) {
    defects.push(
      '自动布局父容器的固有尺寸小于可见内容，导致子项堆叠、裁切或越界。',
    );
  }
  if (
    entry.changes.some((change) =>
      ['normalize-geometry', 'normalize-paint', 'normalize-style'].includes(
        change,
      ),
    )
  ) {
    defects.push('原始布局含有尺寸、间距或样式值不稳定的问题。');
  }
  if (
    entry.changes.includes('embed-penpot-prefab-tokens') ||
    entry.changes.includes('apply-penpot-prefab-classes') ||
    entry.changes.includes('embed-penpot-prefab-styles')
  ) {
    defects.push(
      '原始界面缺少统一的 Penpot 预制件基础样式，需要接入共享 token 与组件类别。',
    );
  }
  if (entry.changes.includes('separate-overlay-bottom-actions')) {
    defects.push('覆盖层底部操作与角落操作存在碰撞风险。');
  }
  if (entry.diagnostics.some(({ severity }) => severity === 'warning')) {
    defects.push('投影仍包含需要人工关注的兼容性警告，已在诊断区保留原路径。');
  }
  if (defects.length === 0)
    defects.push('当前投影记录没有额外诊断；视觉缺陷需根据两端截图逐项复核。');
  return defects;
}

function renderDiagnostics(entry: CatalogEntry): string {
  if (entry.diagnostics.length === 0) return '- 无 Penpot 投影诊断。';
  return entry.diagnostics
    .map((diagnostic) => `- [${diagnostic.severity}] ${diagnostic.message}`)
    .join('\n');
}

function humanizeChangeId(change: string): string {
  return change.replaceAll('-', ' ');
}

export function renderIndex(manifest: CatalogManifest): string {
  const grouped = new Map<string, CatalogEntry[]>();
  for (const entry of manifest.entries) {
    const group = grouped.get(entry.category) ?? [];
    group.push(entry);
    grouped.set(entry.category, group);
  }
  const sections: string[] = [];
  for (const [category, entries] of grouped) {
    sections.push(
      `## ${category}\n\n| 源文件 | 原文镜像 | 分类 | 资产根 | 类型 | 组件依赖 | 资源依赖 | 节点 | 状态 |\n|---|---|---|---|---|---|---|---:|---|\n${entries
        .map(
          (entry) =>
            `| \`${entry.sourcePath}\` | [${entry.name}.zui](${relative(outputRoot, resolveLayoutCatalogPath(outputRoot, entry.outputPath)).replaceAll('\\', '/')}) | ${entry.sourceClassification} | \`${entry.assetRoot}\` | ${entry.sourceKind} v${String(entry.sourceVersion)} | ${entry.componentDependencies.join(', ') || '-'} | ${entry.resourceDependencies.length} | ${entry.nodeCount} | ${entry.status} / ${currentDualReview(entry) ? entry.review?.status : 'review-pending'} |`,
        )
        .join('\n')}`,
    );
  }
  const discovery = manifest.sourceDiscovery
    ? `\n- Git 受控或未忽略源：${manifest.sourceDiscovery.controlledOrUnignoredSourceCount}\n- 扫描到的项目夹具：${(manifest.sourceDiscovery.projectOwnedScannedFixtures ?? []).map((fixture) => `\`${fixture.sourcePath}\``).join(', ') || '无'}\n- 显式项目夹具：${manifest.sourceDiscovery.explicitProjectFixtures.map((fixture) => `\`${fixture.sourcePath}\``).join(', ')}\n- 扫描排除根：${manifest.sourceDiscovery.excludedRoots.map((path) => `\`${path}\``).join(', ')}\n`
    : '';
  return `# ZirconEngine .zui 双端布局目录\n\n- 生成时间：${manifest.generatedAt}\n- 已知源文件：${manifest.sourceCount}${discovery}\n- 布局参考：[ReactBits App UI 布局模式](../ui-and-layout/reactbits-app-shell-layout-modes.md)\n- 原文镜像：\`docs/ui/zui/{category}/{name}/{name}.zui\`\n- 每项验收记录：同目录 \`result.md\`\n- 引擎主图：\`preview.png\`；Penpot 主图：\`penpot.png\`\n- Penpot 展开输入：\`evidence/penpot-input.zui\`\n\n${sections.join('\n\n')}\n`;
}

function record(value: unknown): Record<string, unknown> | undefined {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' ? value : undefined;
}

function hasAuthoredReactBitsTypography(document: ZuiDocument): boolean {
  return (document.stylesheets ?? []).some((stylesheet) =>
    (Array.isArray(stylesheet['rules']) ? stylesheet['rules'] : []).some(
      (ruleValue) => {
        const rule = record(ruleValue);
        const set = record(rule?.['set']);
        const self = record(set?.['self']);
        const font = record(self?.['font']);
        return (
          rule?.['selector'] === '*' &&
          font?.['asset'] === 'res://fonts/editor-ui.font.toml' &&
          font?.['family'] === 'Fira Sans'
        );
      },
    ),
  );
}

function scalarVersion(value: unknown): number | string {
  return typeof value === 'number' || typeof value === 'string'
    ? value
    : 'unknown';
}

function sha256(value: string): string {
  return createHash('sha256').update(value, 'utf8').digest('hex');
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  void main().catch((error: unknown) => {
    console.error(errorMessage(error));
    process.exitCode = 1;
  });
}

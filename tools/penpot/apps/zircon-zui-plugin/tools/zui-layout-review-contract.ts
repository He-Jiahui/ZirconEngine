import { createHash } from 'node:crypto';
import type { ZuiDocument } from '../src/bridge/zui-document';
import {
  componentInputKey,
  componentReviewRoot,
  componentReviewStates,
  nativePainterReviewStates,
} from '../src/bridge/zui-component-review-data';
import { authoredCollectionReviewCases } from '../src/bridge/zui-collection-review';
import { popupReviewStates } from './zui-layout-source-scenario-coverage';
import { validateWorkbenchManagedSceneFingerprint } from './zui-layout-workbench-managed-inputs';
import { validateWorkbenchPresentation } from './zui-layout-workbench-presentation';

export interface LayoutReviewCase {
  id: string;
  sourcePath: string;
  host:
    'editor' | 'plugin' | 'woc' | 'component' | 'toolbar' | 'theme' | 'fixture';
  viewport: { width: number; height: number };
  dpi: number;
  locale: string;
  state: string;
  /** Design/native review offset; does not change the authored document. */
  scrollPosition?: 'start' | 'end';
  data: Record<string, unknown>;
  themeSourcePath?: string;
  reviewHost?: { path: string; sha256: string };
}

export interface WorkbenchStateSelector {
  sourcePath: string;
  controlId: string;
  sourceNodeId?: string;
  instancePath?: string;
  scrollTarget?: WorkbenchSourceTarget;
  textOverrides?: WorkbenchTextOverride[];
}

export interface WorkbenchSourceTarget {
  sourcePath: string;
  controlId: string;
  sourceNodeId?: string;
  instancePath?: string;
}

export interface WorkbenchTextOverride {
  sourcePath: string;
  sourceNodeId: string;
  controlId: string;
  instancePath?: string;
  property: 'text';
  value: string;
}

const REVIEW_CASE_HOSTS = new Set<LayoutReviewCase['host']>([
  'editor',
  'plugin',
  'woc',
  'component',
  'toolbar',
  'theme',
  'fixture',
]);
const REVIEW_CASE_FIELDS = new Set([
  'id',
  'sourcePath',
  'host',
  'viewport',
  'dpi',
  'locale',
  'state',
  'scrollPosition',
  'data',
  'themeSourcePath',
  'reviewHost',
]);

/**
 * Validate the JSON envelope shared by catalog, Penpot and native capture
 * tools.  Keeping this at the tool boundary prevents malformed cases from
 * being hashed and later mistaken for current evidence.
 */
export function validateLayoutReviewCase(value: unknown): LayoutReviewCase {
  if (!isRecord(value)) throw new Error('LayoutReviewCase must be an object');
  rejectUnknownFields(value, REVIEW_CASE_FIELDS, 'LayoutReviewCase');
  requireNonemptyString(value['id'], 'LayoutReviewCase.id');
  requireNonemptyString(value['sourcePath'], 'LayoutReviewCase.sourcePath');
  if (!REVIEW_CASE_HOSTS.has(value['host'] as LayoutReviewCase['host']))
    throw new Error(
      `LayoutReviewCase.host is unsupported: ${String(value['host'])}`,
    );
  if (!isRecord(value['viewport']))
    throw new Error('LayoutReviewCase.viewport must be an object');
  rejectUnknownFields(
    value['viewport'],
    new Set(['width', 'height']),
    'LayoutReviewCase.viewport',
  );
  for (const axis of ['width', 'height'] as const) {
    const dimension = (value['viewport'] as Record<string, unknown>)[axis];
    if (
      typeof dimension !== 'number' ||
      !Number.isInteger(dimension) ||
      dimension < 1 ||
      dimension > 8192
    )
      throw new Error(
        `LayoutReviewCase.viewport.${axis} must be an integer in 1..=8192`,
      );
  }
  if (
    typeof value['dpi'] !== 'number' ||
    !Number.isFinite(value['dpi']) ||
    value['dpi'] <= 0
  )
    throw new Error('LayoutReviewCase.dpi must be positive and finite');
  requireNonemptyString(value['locale'], 'LayoutReviewCase.locale');
  requireNonemptyString(value['state'], 'LayoutReviewCase.state');
  if (
    value['scrollPosition'] !== undefined &&
    !['start', 'end'].includes(value['scrollPosition'] as string)
  )
    throw new Error('LayoutReviewCase.scrollPosition must be start or end');
  if (
    (value['state'] === 'scroll-before' && value['scrollPosition'] === 'end') ||
    (value['state'] === 'scroll-after' && value['scrollPosition'] === 'start')
  )
    throw new Error('LayoutReviewCase.scrollPosition contradicts its state');
  if (!isRecord(value['data']))
    throw new Error('LayoutReviewCase.data must be an object');
  const presentation = (value['data'] as Record<string, unknown>)[
    'workbenchPresentation'
  ];
  if (presentation !== undefined)
    validateWorkbenchPresentation(presentation);
  const workbenchState = (value['data'] as Record<string, unknown>)[
    'workbenchState'
  ];
  if (workbenchState !== undefined) {
    if (presentation === undefined)
      throw new Error(
        'LayoutReviewCase.data.workbenchPresentation is required with workbenchState',
      );
    if (!isRecord(workbenchState))
      throw new Error('LayoutReviewCase.data.workbenchState must be an object');
    rejectUnknownFields(
      workbenchState,
      new Set([
        'sourcePath',
        'controlId',
        'sourceNodeId',
        'instancePath',
        'scrollTarget',
        'textOverrides',
      ]),
      'LayoutReviewCase.data.workbenchState',
    );
    requireNonemptyString(
      workbenchState['sourcePath'],
      'LayoutReviewCase.data.workbenchState.sourcePath',
    );
    requireNonemptyString(
      workbenchState['controlId'],
      'LayoutReviewCase.data.workbenchState.controlId',
    );
    if (workbenchState['sourceNodeId'] !== undefined)
      requireNonemptyString(
        workbenchState['sourceNodeId'],
        'LayoutReviewCase.data.workbenchState.sourceNodeId',
      );
    if (workbenchState['instancePath'] !== undefined)
      requireNonemptyString(
        workbenchState['instancePath'],
        'LayoutReviewCase.data.workbenchState.instancePath',
      );
    if (
      workbenchState['instancePath'] !== undefined &&
      !isCanonicalAuthoredInstancePath(workbenchState['instancePath'])
    )
      throw new Error(
        'LayoutReviewCase.data.workbenchState.instancePath must be canonical authored callsite JSON',
      );
    if (workbenchState['scrollTarget'] !== undefined) {
      const target = workbenchState['scrollTarget'];
      if (!isRecord(target))
        throw new Error(
          'LayoutReviewCase.data.workbenchState.scrollTarget must be an object',
        );
      rejectUnknownFields(
        target,
        new Set(['sourcePath', 'controlId', 'sourceNodeId', 'instancePath']),
        'LayoutReviewCase.data.workbenchState.scrollTarget',
      );
      requireNonemptyString(
        target['sourcePath'],
        'LayoutReviewCase.data.workbenchState.scrollTarget.sourcePath',
      );
      requireNonemptyString(
        target['controlId'],
        'LayoutReviewCase.data.workbenchState.scrollTarget.controlId',
      );
      if (target['sourceNodeId'] !== undefined)
        requireNonemptyString(
          target['sourceNodeId'],
          'LayoutReviewCase.data.workbenchState.scrollTarget.sourceNodeId',
        );
      if (target['instancePath'] !== undefined) {
        requireNonemptyString(
          target['instancePath'],
          'LayoutReviewCase.data.workbenchState.scrollTarget.instancePath',
        );
        if (!isCanonicalAuthoredInstancePath(target['instancePath']))
          throw new Error(
            'LayoutReviewCase.data.workbenchState.scrollTarget.instancePath must be canonical authored callsite JSON',
          );
      }
    }
    if (workbenchState['textOverrides'] !== undefined) {
      const overrides = workbenchState['textOverrides'];
      if (!Array.isArray(overrides) || overrides.length === 0)
        throw new Error(
          'LayoutReviewCase.data.workbenchState.textOverrides must be a nonempty array',
        );
      const seenOverrides = new Set<string>();
      for (const [index, value] of overrides.entries()) {
        if (!isRecord(value))
          throw new Error(
            `LayoutReviewCase.data.workbenchState.textOverrides[${index}] must be an object`,
          );
        rejectUnknownFields(
          value,
          new Set([
            'sourcePath',
            'sourceNodeId',
            'controlId',
            'instancePath',
            'property',
            'value',
          ]),
          `LayoutReviewCase.data.workbenchState.textOverrides[${index}]`,
        );
        requireNonemptyString(
          value['sourcePath'],
          `LayoutReviewCase.data.workbenchState.textOverrides[${index}].sourcePath`,
        );
        requireNonemptyString(
          value['sourceNodeId'],
          `LayoutReviewCase.data.workbenchState.textOverrides[${index}].sourceNodeId`,
        );
        requireNonemptyString(
          value['controlId'],
          `LayoutReviewCase.data.workbenchState.textOverrides[${index}].controlId`,
        );
        if (value['instancePath'] !== undefined)
          requireNonemptyString(
            value['instancePath'],
            `LayoutReviewCase.data.workbenchState.textOverrides[${index}].instancePath`,
          );
        if (
          value['instancePath'] !== undefined &&
          !isCanonicalAuthoredInstancePath(value['instancePath'])
        )
          throw new Error(
            `LayoutReviewCase.data.workbenchState.textOverrides[${index}].instancePath must be canonical authored callsite JSON`,
          );
        if (value['property'] !== 'text')
          throw new Error(
            `LayoutReviewCase.data.workbenchState.textOverrides[${index}].property must be text`,
          );
        requireNonemptyString(
          value['value'],
          `LayoutReviewCase.data.workbenchState.textOverrides[${index}].value`,
        );
        if (Buffer.byteLength(value['value'] as string, 'utf8') > 4096)
          throw new Error(
            `LayoutReviewCase.data.workbenchState.textOverrides[${index}].value exceeds 4096 UTF-8 bytes`,
          );
        if (/\p{Cc}/u.test(value['value'] as string))
          throw new Error(
            `LayoutReviewCase.data.workbenchState.textOverrides[${index}].value contains a control character`,
          );
        const key = JSON.stringify([
          value['sourcePath'],
          value['sourceNodeId'],
          value['controlId'],
          value['instancePath'] ?? null,
          value['property'],
        ]);
        if (seenOverrides.has(key))
          throw new Error(
            `Duplicate workbench text override: ${value['sourcePath']}#${value['sourceNodeId']}`,
          );
        seenOverrides.add(key);
      }
    }
  }
  if (workbenchState !== undefined)
    validateWorkbenchManagedSceneFingerprint(
      value['data']['managedSceneFingerprint'],
    );
  if (value['themeSourcePath'] !== undefined)
    requireNonemptyString(
      value['themeSourcePath'],
      'LayoutReviewCase.themeSourcePath',
    );
  if (value['reviewHost'] !== undefined) {
    if (!isRecord(value['reviewHost']))
      throw new Error('LayoutReviewCase.reviewHost must be an object');
    rejectUnknownFields(
      value['reviewHost'],
      new Set(['path', 'sha256']),
      'LayoutReviewCase.reviewHost',
    );
    requireNonemptyString(
      (value['reviewHost'] as Record<string, unknown>)['path'],
      'LayoutReviewCase.reviewHost.path',
    );
    if (
      typeof (value['reviewHost'] as Record<string, unknown>)['sha256'] !==
        'string' ||
      !/^[0-9a-f]{64}$/i.test(
        (value['reviewHost'] as Record<string, unknown>)['sha256'] as string,
      )
    )
      throw new Error(
        'LayoutReviewCase.reviewHost.sha256 must be a 64-character SHA-256',
      );
  }
  return value as unknown as LayoutReviewCase;
}

export function isWorkbenchStateReviewCase(
  reviewCase: LayoutReviewCase,
): reviewCase is LayoutReviewCase & {
  data: Record<string, unknown> & { workbenchState: WorkbenchStateSelector };
} {
  return Object.hasOwn(reviewCase.data, 'workbenchState');
}

export function validateLayoutReviewCases(value: unknown): LayoutReviewCase[] {
  if (!Array.isArray(value))
    throw new Error('LayoutReviewCase collection must be an array');
  const cases = value.map(validateLayoutReviewCase);
  const ids = new Set<string>();
  for (const reviewCase of cases) {
    if (ids.has(reviewCase.id))
      throw new Error(`Duplicate LayoutReviewCase id: ${reviewCase.id}`);
    ids.add(reviewCase.id);
  }
  return cases;
}

function rejectUnknownFields(
  value: Record<string, unknown>,
  supported: ReadonlySet<string>,
  context: string,
): void {
  const unknown = Object.keys(value).find((key) => !supported.has(key));
  if (unknown) throw new Error(`Unknown ${context} field: ${unknown}`);
}

function requireNonemptyString(value: unknown, name: string): void {
  if (typeof value !== 'string' || value.trim() === '')
    throw new Error(`${name} must be a nonempty string`);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isCanonicalAuthoredInstancePath(value: unknown): value is string {
  if (typeof value !== 'string' || !value) return false;
  try {
    const steps = JSON.parse(value) as unknown;
    return (
      Array.isArray(steps) &&
      steps.every(
        (step) =>
          isRecord(step) &&
          Object.keys(step).join(',') === 'sourcePath,sourceNodeId' &&
          typeof step['sourcePath'] === 'string' &&
          step['sourcePath'].trim() !== '' &&
          typeof step['sourceNodeId'] === 'string' &&
          step['sourceNodeId'].trim() !== '',
      ) &&
      JSON.stringify(steps) === value
    );
  } catch {
    return false;
  }
}

export interface DependencyFingerprint {
  sourcePath: string;
  sha256: string;
}

export type NativeRendererKind =
  'zircon-runtime-wgpu-headless' | 'zircon-editor-retained-host';

export interface BrowserRuntimeRecord {
  product: string;
  userAgent: string;
  executablePath: string;
  executableSha256: string;
}

export function requiredNativeRenderer(
  reviewCase: LayoutReviewCase,
): NativeRendererKind {
  const editorProduct =
    reviewCase.sourcePath.startsWith('zircon_editor/') ||
    reviewCase.sourcePath.startsWith('zircon_plugins/') ||
    reviewCase.host === 'plugin';
  return reviewCase.host !== 'fixture' && editorProduct
    ? 'zircon-editor-retained-host'
    : 'zircon-runtime-wgpu-headless';
}

export interface LayoutRenderEvidence {
  caseId: string;
  /**
   * `pending` is a real capture which is not publishable yet.  In particular,
   * the retained Editor host can produce pixels before font/media readiness
   * and visual review receipts are certified.
   */
  status: 'passed' | 'pending' | 'failed';
  screenshotPath: string;
  screenshotSha256: string;
  sourceSha256: string;
  inputSha256?: string;
  reviewHostSha256?: string;
  dependencySha256: string;
  caseSha256: string;
  rendererSha256?: string;
  rendererPath?: string;
  rendererKind?: string;
  browserRuntime?: BrowserRuntimeRecord;
  captureProgramFingerprints?: Array<[string, string]>;
  runtimeAssetFingerprints?: Array<[string, string]>;
  geometryPath?: string;
  geometrySha256?: string;
  textPath?: string;
  textSha256?: string;
  error?: string;
  pendingReason?: string;
}

export const isRenderEvidencePending = (
  evidence: Pick<LayoutRenderEvidence, 'status'>,
): boolean => evidence.status === 'pending';

export interface LayoutCaseReview {
  caseId: string;
  status: 'accepted' | 'needs_revision';
  observations: string;
  maxGeometryDeltaPx: number;
  textMatches: boolean;
  visibilityMatches: boolean;
}

export interface CatalogReview {
  status: 'accepted' | 'needs_revision';
  screenshotSha256: string;
  outputSha256: string;
  observations: string;
  reviewedAt: string;
  contract?: 'dual-renderer-v1';
  sourceSha256?: string;
  dependencySha256?: string;
  penpotInputSha256?: string;
  casesSha256?: string;
  penpotScreenshotHashes?: Record<string, string>;
  engineScreenshotHashes?: Record<string, string>;
  caseReviews?: LayoutCaseReview[];
  penpotEvidenceSha256?: string;
  engineEvidenceSha256?: string;
}

export function canonicalSha256(value: unknown): string {
  const canonical = (item: unknown): unknown =>
    Array.isArray(item)
      ? item.map(canonical)
      : item !== null && typeof item === 'object'
        ? Object.fromEntries(
            Object.entries(item)
              .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
              .map(([key, child]) => [key, canonical(child)]),
          )
        : item;
  return createHash('sha256')
    .update(JSON.stringify(canonical(value)))
    .digest('hex');
}

export const caseSha256 = (value: LayoutReviewCase): string =>
  canonicalSha256(value);
export const casesSha256 = (values: LayoutReviewCase[]): string =>
  canonicalSha256(
    [...values].sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0)),
  );

function hasScrollableReviewContent(document?: ZuiDocument): boolean {
  return Object.values(document?.nodes ?? {}).some((node) => {
    const component = node.component.toLowerCase();
    if (component === 'scrollablebox' || component === 'scrollbox') return true;
    const container = node.layout?.['container'];
    if (!container || typeof container !== 'object' || Array.isArray(container))
      return false;
    const kind = (container as Record<string, unknown>)['kind'];
    return (
      typeof kind === 'string' &&
      (kind.toLowerCase() === 'scrollablebox' ||
        kind.toLowerCase() === 'scrollbox')
    );
  });
}

function nativePainterDocumentStates(document?: ZuiDocument): string[] {
  const states = new Set<string>(['default']);
  for (const node of Object.values(document?.nodes ?? {})) {
    for (const state of nativePainterReviewStates(node) ?? []) {
      if (state !== 'default') states.add(state);
    }
  }
  return [...states];
}

export function defaultReviewCases(
  sourcePath: string,
  kind: string,
  document?: ZuiDocument,
): LayoutReviewCase[] {
  const fixture =
    /\/tests\/fixtures\/|\/ui\/runtime\/fixtures\/|roundtrip-fixture\.zui$/.test(
      sourcePath,
    );
  const isWoc = sourcePath.startsWith('examples/woc/');
  const isWorkbenchProductSurface =
    sourcePath.includes('/components/workbench/modules/') ||
    sourcePath.includes('/components/workbench/shell/');
  const host: LayoutReviewCase['host'] = fixture
    ? 'fixture'
    : isWoc
      ? 'woc'
      : isWorkbenchProductSurface
        ? 'editor'
        : /theme|style|tokens/.test(kind)
          ? 'theme'
          : /toolbar/.test(sourcePath)
            ? 'toolbar'
            : kind !== 'view' &&
                (/^(component|widget)$/.test(kind) ||
                  /\/components\/|\/widgets\//.test(sourcePath))
              ? 'component'
              : sourcePath.startsWith('zircon_plugins/') ||
                  sourcePath.includes('/plugins/')
                ? 'plugin'
                : 'editor';
  const sizes =
    host === 'woc'
      ? [
          [1920, 1080],
          [1280, 720],
        ]
      : host === 'component'
        ? [
            [360, 520],
            [240, 520],
            [480, 520],
          ]
        : [
            [1280, 800],
            [900, 620],
            [640, 520],
          ];
  const nativePainterStates = nativePainterDocumentStates(document);
  const popupStates = popupReviewStates(document);
  const states =
    host === 'theme' || (host === 'woc' && /theme|style|tokens/.test(kind))
      ? ['default', 'hover', 'pressed', 'focused', 'disabled', 'selected']
      : host === 'component'
        ? [
            ...new Set([
              ...componentReviewStates(document),
              ...nativePainterStates,
              ...popupStates,
            ]),
          ]
        : [...new Set([...nativePainterStates, ...popupStates])];
  // The product default is a closed popup. Review it explicitly, but put
  // the invoked state first so the component's primary preview has content.
  const reviewRoot =
    host === 'component' && document
      ? componentReviewRoot(document)
      : undefined;
  const reviewedStates =
    reviewRoot &&
    popupStates.includes('open') &&
    ['open', 'popup_open'].some((key) =>
      Object.hasOwn(reviewRoot.props ?? {}, key),
    ) &&
    ['open', 'popup_open']
      .filter((key) => Object.hasOwn(reviewRoot.props ?? {}, key))
      .every((key) => reviewRoot.props?.[key] === false) &&
    states.includes('open')
      ? ['open', ...states.filter((state) => state !== 'open')]
      : states;
  const inputKey =
    host === 'component' && document
      ? componentInputKey(componentReviewRoot(document))
      : undefined;
  const defaultData = inputKey
    ? {
        componentInput: {
          [inputKey]: inputKey === 'query' ? 'material' : 'Player',
        },
      }
    : {};
  const scenarios: Array<{
    id: string;
    state: string;
    locale: string;
    data: Record<string, unknown>;
  }> = reviewedStates.map((state) => ({
    id: state,
    state,
    locale: state === 'long-zh' ? 'zh-CN' : 'en-US',
    data: defaultData,
  }));
  if (document && host === 'fixture') {
    const collectionCases = authoredCollectionReviewCases(document);
    for (const collectionCase of collectionCases) {
      scenarios.push({
        id: `collection-${collectionCase}`,
        state: 'default',
        locale: collectionCase === 'long-locale' ? 'zh-CN' : 'en-US',
        data: { collectionCase },
      });
    }
    if (collectionCases.length > 0) {
      for (const state of ['scroll-before', 'scroll-after'] as const) {
        scenarios.push({
          id: `collection-overflow-${state}`,
          state,
          locale: 'en-US',
          data: { collectionCase: 'overflow' },
        });
      }
    }
  }
  if (inputKey) {
    for (const [id, value, locale] of [
      ['empty', '', 'en-US'],
      [
        'long-en',
        'Player_Controller_MainCamera_EditorPreview_Instance_0123456789',
        'en-US',
      ],
      [
        'long-zh',
        '\u89d2\u8272\u63a7\u5236\u5668_\u4e3b\u6444\u50cf\u673a_\u7f16\u8f91\u5668\u9884\u89c8_\u8d85\u957f\u5b9e\u4f8b\u540d\u79f0_0123456789',
        'zh-CN',
      ],
      ['error', 'Invalid/name', 'en-US'],
    ])
      scenarios.push({
        id,
        state: 'default',
        locale,
        data: {
          componentInput: {
            [inputKey]: value,
            ...(id === 'error' ? { validation_level: 'error' } : {}),
          },
        },
      });
  }
  const result: LayoutReviewCase[] = [];
  const themeSourcePath =
    !fixture &&
    (sourcePath.startsWith('zircon_editor/assets/') ||
      sourcePath.startsWith('zircon_plugins/'))
      ? 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui'
      : undefined;
  for (const scenario of scenarios) {
    for (const [index, [width, height]] of sizes.entries()) {
      for (const dpi of index === 0 ? [1, 1.5] : [1]) {
        result.push({
          id: `${scenario.id}-${width}x${height}-dpi${dpi}`,
          sourcePath,
          host,
          viewport: { width, height },
          dpi,
          locale: scenario.locale,
          state: scenario.state,
          data: scenario.data,
          ...(themeSourcePath ? { themeSourcePath } : {}),
        });
      }
    }
  }
  if (
    host === 'component' &&
    sourcePath.endsWith(
      '/components/workbench/primitives/data/workbench_table_row.zui',
    )
  ) {
    for (const [state, scrollPosition] of [
      ['scroll-before', 'start'],
      ['scroll-after', 'end'],
    ] as const) {
      for (const [width, height] of sizes) {
        result.push({
          id: `${state}-${width}x${height}-dpi1`,
          sourcePath,
          host,
          viewport: { width, height },
          dpi: 1,
          locale: 'en-US',
          state,
          scrollPosition,
          data: {},
          ...(themeSourcePath ? { themeSourcePath } : {}),
        });
      }
    }
  }
  if (
    document &&
    host !== 'fixture' &&
    host !== 'theme' &&
    hasScrollableReviewContent(document)
  ) {
    const [width, height] = sizes[sizes.length - 1];
    for (const state of ['scroll-before', 'scroll-after']) {
      result.push({
        id: `${state}-${width}x${height}-dpi1`,
        sourcePath,
        host,
        viewport: { width, height },
        dpi: 1,
        locale: 'en-US',
        state,
        data: {},
        ...(themeSourcePath ? { themeSourcePath } : {}),
      });
    }
    if (reviewedStates.includes('open'))
      result.push({
        id: `open-scroll-after-${width}x${height}-dpi1`,
        sourcePath,
        host,
        viewport: { width, height },
        dpi: 1,
        locale: 'en-US',
        state: 'open',
        scrollPosition: 'end',
        data: defaultData,
        ...(themeSourcePath ? { themeSourcePath } : {}),
      });
  }
  return result;
}

/**
 * An authored page can get its scroll owner through a real component import.
 * Inspect the expanded design projection without changing the original page
 * or replacing its editor/runtime host. The normal source-only case list keeps
 * its hashes; only previously missing before/after and observable open/end
 * scenes are appended.
 */
export function appendEmbeddedScrollReviewCases(
  cases: LayoutReviewCase[],
  expandedDocument: ZuiDocument,
): LayoutReviewCase[] {
  if (
    !hasScrollableReviewContent(expandedDocument) ||
    cases[0]?.host === 'theme' ||
    cases[0]?.host === 'fixture'
  )
    return cases;
  const baseline = cases
    .filter((item) => item.state === 'default' && item.dpi === 1)
    .sort((left, right) => left.viewport.width - right.viewport.width)[0];
  if (!baseline) return cases;
  const missing = (['scroll-before', 'scroll-after'] as const).filter(
    (state) => !cases.some((item) => item.state === state),
  );
  const openAtEnd =
    nativePainterDocumentStates(expandedDocument).includes('open') &&
    !cases.some(
      (item) => item.state === 'open' && item.scrollPosition === 'end',
    );
  if (!missing.length && !openAtEnd) return cases;
  const openData =
    cases.find((item) => item.state === 'open')?.data ?? baseline.data;
  return [
    ...cases,
    ...missing.map((state) => ({
      ...baseline,
      id: `${state}-${baseline.viewport.width}x${baseline.viewport.height}-dpi1`,
      state,
    })),
    ...(openAtEnd
      ? [
          {
            ...baseline,
            id: `open-scroll-after-${baseline.viewport.width}x${baseline.viewport.height}-dpi1`,
            state: 'open',
            scrollPosition: 'end' as const,
            data: openData,
          },
        ]
      : []),
  ];
}

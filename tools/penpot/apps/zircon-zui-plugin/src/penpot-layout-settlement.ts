import type { RenderedLayoutAudit } from './penpot-render-layout';

export type LayoutSettlementPhase =
  | 'initial-refresh'
  | 'first-settle'
  | 'single-line-text-fit'
  | 'final-refresh'
  | 'second-settle';

export interface LayoutSettlementDiagnostic {
  phase: LayoutSettlementPhase;
  event: 'started' | 'sample' | 'completed' | 'failed';
  elapsedMs: number;
  sampleCount?: number;
  sampleMs?: number;
  snapshotMs?: number;
  longestSampleMs?: number;
  lastReady?: boolean;
}

export interface LayoutStabilityOptions<T> {
  intervalMs?: number;
  quietMs?: number;
  timeoutMs?: number;
  /** Decimal places retained for geometry values during stability checks. */
  numericPrecision?: number;
  /** Additional condition required before a quiet layout is considered usable. */
  isReady?: (snapshot: T) => boolean;
  /** Best-effort progress telemetry; it never changes settlement decisions. */
  onDiagnostic?: (diagnostic: LayoutSettlementDiagnostic) => void;
}

/** Allow large boards time for at least two complete Penpot-owned geometry reads. */
export function layoutSettlementOptionsForNodes(nodeCount: number): {
  timeoutMs: number;
} {
  if (!Number.isInteger(nodeCount) || nodeCount < 0)
    throw new Error('Projected node count must be a nonnegative integer');
  return { timeoutMs: Math.max(60_000, Math.min(360_000, nodeCount * 700)) };
}

/**
 * Refresh expensive Penpot-owned content only before and after observing a
 * stable native layout. Repeating the refresh inside every poll can starve
 * the official frontend on large component trees.
 */
function reportLayoutDiagnostic(
  onDiagnostic: ((diagnostic: LayoutSettlementDiagnostic) => void) | undefined,
  diagnostic: LayoutSettlementDiagnostic,
): void {
  try {
    onDiagnostic?.(diagnostic);
  } catch {
    // Optional telemetry cannot change the import result.
  }
}

async function runLayoutDiagnosticPhase(
  phase: LayoutSettlementPhase,
  operation: () => void | Promise<void>,
  onDiagnostic?: (diagnostic: LayoutSettlementDiagnostic) => void,
): Promise<void> {
  const started = Date.now();
  reportLayoutDiagnostic(onDiagnostic, { phase, event: 'started', elapsedMs: 0 });
  try {
    await operation();
  } catch (error) {
    reportLayoutDiagnostic(onDiagnostic, {
      phase,
      event: 'failed',
      elapsedMs: Date.now() - started,
    });
    throw error;
  }
  reportLayoutDiagnostic(onDiagnostic, {
    phase,
    event: 'completed',
    elapsedMs: Date.now() - started,
  });
}

export async function settleAfterLayoutRefresh<T>(
  refresh: () => void,
  snapshot: () => T,
  options: LayoutStabilityOptions<T> = {},
  afterInitialLayout?: () => void | Promise<void>,
): Promise<void> {
  await runLayoutDiagnosticPhase(
    'initial-refresh',
    refresh,
    options.onDiagnostic,
  );
  await waitForStableLayout(() => {}, snapshot, options, 'first-settle');
  if (afterInitialLayout)
    await runLayoutDiagnosticPhase(
      'single-line-text-fit',
      afterInitialLayout,
      options.onDiagnostic,
    );
  await runLayoutDiagnosticPhase(
    'final-refresh',
    refresh,
    options.onDiagnostic,
  );
  await waitForStableLayout(() => {}, snapshot, options, 'second-settle');
}

/**
 * Keep only authored semantic geometry and text measurements in the
 * settlement sample. Component-library IDs and audit detail arrays can change
 * during Penpot materialization without indicating a visible layout change.
 */
export function layoutAuditStabilitySnapshot(audit: RenderedLayoutAudit) {
  return {
    assetBounds: audit.assetBounds,
    totalNodes: audit.totalNodes,
    checkedNodes: audit.checkedNodes,
    overflowNodeIds: [...audit.overflowNodeIds].sort(),
    invalidGeometryNodeIds: [...audit.invalidGeometryNodeIds].sort(),
    contentMeasurements: (audit.contentMeasurements ?? [])
      .map(({ nodeId, text, width, height, styleSignature }) => ({
        nodeId,
        styleSignature,
        text,
        width,
        height,
      }))
      .sort((left, right) => left.nodeId.localeCompare(right.nodeId, 'en')),
    tableMeasurements: (audit.tableMeasurements ?? [])
      .map(({ text, width, height }) => ({ text, width, height }))
      .sort((left, right) => left.text.localeCompare(right.text, 'en')),
    semanticNodes: (audit.semanticNodes ?? [])
      .map((node) => ({
        nodeId: node.nodeId,
        visible: node.visible,
        detached: node.detached,
        bounds: node.bounds,
        text: node.text,
        textParts: (node.textParts ?? []).map((part) => ({
          text: part.text,
          bounds: part.bounds,
        })),
        // Font-measurement helpers are invisible but drive editable table cell
        // positions. Settling only the visible cells can save a 1x1 helper as
        // the export baseline before Penpot finishes shaping its real font.
        tableMeasurements: (node.tableMeasurements ?? [])
          .map(({ text, width, height }) => ({ text, width, height }))
          .sort((left, right) => left.text.localeCompare(right.text, 'en')),
      }))
      .sort((left, right) => left.nodeId.localeCompare(right.nodeId, 'en')),
  };
}

export function tableMeasurementsReady(
  snapshot: ReturnType<typeof layoutAuditStabilitySnapshot>,
): boolean {
  const measurements =
    snapshot.tableMeasurements.length > 0
      ? snapshot.tableMeasurements
      : snapshot.semanticNodes.flatMap((node) => node.tableMeasurements);
  return measurements.every(
    ({ width, height }) =>
      Number.isFinite(width) &&
      Number.isFinite(height) &&
      width > 1 &&
      height > 1,
  );
}

export async function waitForStableLayout<T>(
  update: () => void,
  snapshot: () => T,
  options: LayoutStabilityOptions<T> = {},
  phase: LayoutSettlementPhase = 'first-settle',
): Promise<void> {
  const {
    intervalMs = 100,
    quietMs = 500,
    timeoutMs = 10_000,
    numericPrecision = 3,
    isReady,
  } = options;
  if (
    !Number.isInteger(numericPrecision) ||
    numericPrecision < 0 ||
    numericPrecision > 9
  )
    throw new Error('numericPrecision must be an integer in 0..=9');
  const started = Date.now();
  let unchangedSince = started;
  let previous: string | undefined;
  let previousReady: boolean | undefined;
  let latestChanges: string[] = [];
  let samples = 0;
  let longestSampleMs = 0;
  let latestSampleMs = 0;
  let latestSnapshotMs = 0;
  let lastReady: boolean | undefined;
  const diagnostic = (
    event: LayoutSettlementDiagnostic['event'],
  ): void => {
    reportLayoutDiagnostic(
      options.onDiagnostic,
      event === 'started'
        ? { phase, event, elapsedMs: Date.now() - started }
        : {
            phase,
            event,
            elapsedMs: Date.now() - started,
            sampleCount: samples,
            sampleMs: latestSampleMs,
            snapshotMs: latestSnapshotMs,
            longestSampleMs,
            lastReady,
          },
    );
  };
  diagnostic('started');
  while (Date.now() - started < timeoutMs) {
    const sampleStarted = Date.now();
    update();
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
    samples += 1;
    const snapshotStarted = Date.now();
    const value = snapshot();
    latestSnapshotMs = Date.now() - snapshotStarted;
    const current = stableSnapshot(value, numericPrecision);
    const ready = isReady?.(value) ?? true;
    lastReady = ready;
    let stable = false;
    if (current !== previous || ready !== previousReady) {
      if (previous)
        latestChanges = changedMeasurements(
          JSON.parse(previous),
          JSON.parse(current),
        );
      previous = current;
      previousReady = ready;
      unchangedSince = Date.now();
    } else if (ready && Date.now() - unchangedSince >= quietMs) {
      stable = true;
    }
    latestSampleMs = Date.now() - sampleStarted;
    longestSampleMs = Math.max(longestSampleMs, latestSampleMs);
    if (
      stable ||
      samples === 1 ||
      samples % 50 === 0 ||
      latestSampleMs >= 1_000
    )
      diagnostic('sample');
    if (stable) {
      diagnostic('completed');
      return;
    }
  }
  diagnostic('failed');
  throw new Error(
    'Penpot layout and text measurements did not settle after ' +
      samples +
      ' samples (longest ' +
      longestSampleMs +
      'ms, elapsed ' +
      (Date.now() - started) +
      'ms): ' +
      (latestChanges.join('; ') || 'snapshot changed without a comparable diff') +
      (previousReady === false
        ? '; readiness predicate remained false'
        : ''),
  );
}
function stableSnapshot(value: unknown, numericPrecision: number): string {
  const factor = 10 ** numericPrecision;
  const normalize = (item: unknown): unknown => {
    if (typeof item === 'number') {
      if (!Number.isFinite(item)) return String(item);
      return Math.round(item * factor) / factor;
    }
    if (Array.isArray(item)) return item.map(normalize);
    if (item !== null && typeof item === 'object') {
      return Object.fromEntries(
        Object.entries(item as Record<string, unknown>)
          .sort(([left], [right]) => left.localeCompare(right, 'en'))
          .map(([key, child]) => [key, normalize(child)]),
      );
    }
    return item;
  };
  return JSON.stringify(normalize(value));
}

function changedMeasurements(
  before: unknown,
  after: unknown,
  path = '$',
): string[] {
  if (JSON.stringify(before) === JSON.stringify(after)) return [];
  if (
    before !== null &&
    after !== null &&
    typeof before === 'object' &&
    typeof after === 'object'
  ) {
    const a = before as Record<string, unknown>,
      b = after as Record<string, unknown>;
    const changes: string[] = [];
    for (const key of new Set([...Object.keys(a), ...Object.keys(b)])) {
      changes.push(...changedMeasurements(a[key], b[key], `${path}.${key}`));
      if (changes.length >= 8) break;
    }
    return changes.slice(0, 8);
  }
  return [`${path}: ${String(before)} -> ${String(after)}`];
}

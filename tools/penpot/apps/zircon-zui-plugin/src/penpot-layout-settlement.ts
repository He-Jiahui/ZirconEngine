import type { RenderedLayoutAudit } from './penpot-render-layout';

export interface LayoutStabilityOptions<T> {
  intervalMs?: number;
  quietMs?: number;
  timeoutMs?: number;
  /** Decimal places retained for geometry values during stability checks. */
  numericPrecision?: number;
  /** Additional condition required before a quiet layout is considered usable. */
  isReady?: (snapshot: T) => boolean;
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
export async function settleAfterLayoutRefresh<T>(
  refresh: () => void,
  snapshot: () => T,
  options: LayoutStabilityOptions<T> = {},
  afterInitialLayout?: () => void | Promise<void>,
): Promise<void> {
  refresh();
  await waitForStableLayout(() => {}, snapshot, options);
  await afterInitialLayout?.();
  refresh();
  await waitForStableLayout(() => {}, snapshot, options);
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
  while (Date.now() - started < timeoutMs) {
    const sampleStarted = Date.now();
    update();
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
    samples += 1;
    const value = snapshot();
    longestSampleMs = Math.max(longestSampleMs, Date.now() - sampleStarted);
    const current = stableSnapshot(value, numericPrecision);
    const ready = isReady?.(value) ?? true;
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
      return;
    }
  }
  throw new Error(
    `Penpot layout and text measurements did not settle after ${samples} samples (longest ${longestSampleMs}ms, elapsed ${Date.now() - started}ms): ${latestChanges.join('; ') || 'snapshot changed without a comparable diff'}${previousReady === false ? '; readiness predicate remained false' : ''}`,
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

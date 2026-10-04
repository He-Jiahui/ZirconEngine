import {
  applyReviewScrollPresentation,
  captureReviewScrollLayoutSnapshot,
  restoreReviewScrollPresentation,
  reviewScrollPosition,
} from './penpot-review-scroll';
import type { Board, Shape } from '@penpot/plugin-types';
import { decodeZuiMetadata, type ZuiDocument } from './bridge/zui-document';
import {
  reconcileZuiDocument,
  projectZuiDocument,
  type PenpotAssetSnapshot,
} from './bridge/penpot-projection';
import { reviewHost } from './bridge/zui-review-host';
import { reviewDocumentForCase } from './bridge/zui-review-case';
import type { LayoutReviewCaseMessage } from './model';
import {
  ZUI_METADATA_DOCUMENT,
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_ASSET,
} from './metadata';
import {
  auditRenderedAssetBoard,
  refreshAssetTextLayout,
} from './penpot-asset-renderer';
import { linearContentMeasurementsReady } from './penpot-linear-content';
import { fitNativeSingleLineTexts } from './penpot-text-overflow';
import {
  settleAfterLayoutRefresh,
  layoutAuditStabilitySnapshot,
  layoutSettlementOptionsForNodes,
  tableMeasurementsReady,
} from './penpot-layout-settlement';
export interface AuthoringLayoutDraft {
  finish?: () => void;
  key: string;
  refresh: () => Promise<void>;
}
export interface AuthoringShapeEvents {
  readonly selection: readonly Shape[];
  on(
    type: 'shapechange',
    callback: (shape: Shape) => void,
    props: { shapeId: string },
  ): symbol;
  off(listener: symbol): void;
}
export interface AuthoringLayoutHooks {
  asset: () => Board | null;
  draft: (
    asset: Board,
    keepReviewPresentation: boolean,
  ) => AuthoringLayoutDraft;
  onError: (error: unknown) => void;
}
/** Capture supported current edits before any bridge-owned resize. */
export function captureAuthoringLayoutDraft(
  asset: Board,
  capture: (asset: Board, source: ZuiDocument) => PenpotAssetSnapshot,
  keepReviewPresentation = true,
): AuthoringLayoutDraft {
  const encoded = asset.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_DOCUMENT,
  );
  if (!encoded)
    throw new Error('Imported asset has no reversible source metadata');
  const original = decodeZuiMetadata(encoded),
    source = reviewHost(original) ?? original;
  const reviewValue = asset.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'review-case',
  );
  const reviewCase = reviewValue
    ? (JSON.parse(reviewValue) as LayoutReviewCaseMessage)
    : null;
  const scroll = reviewCase ? reviewScrollPosition(reviewCase) : null;
  let scrollSnapshot = scroll
    ? (restoreReviewScrollPresentation(asset),
      captureReviewScrollLayoutSnapshot(asset))
    : null;
  const finish = () => {
    if (scroll && keepReviewPresentation)
      applyReviewScrollPresentation(asset, scroll, scrollSnapshot ?? undefined);
  };
  let document: ZuiDocument;
  try {
    const snapshot = capture(asset, source);
    const reconciled = reconcileZuiDocument(source, snapshot);
    document = reviewCase
      ? reviewDocumentForCase(reconciled.document, reviewCase)
      : reconciled.document;
  } catch (error) {
    finish();
    throw error;
  }
  const key = JSON.stringify(document);
  return {
    key,
    finish,
    refresh: async () => {
      const projection = projectZuiDocument(document);
      await settleAfterLayoutRefresh(
        () => refreshAssetTextLayout(asset, document, projection),
        () => layoutAuditStabilitySnapshot(auditRenderedAssetBoard(asset)),
        {
          intervalMs: 100,
          quietMs: 700,
          numericPrecision: 3,
          ...layoutSettlementOptionsForNodes(projection.shapes.length),
          isReady: (snapshot) =>
            tableMeasurementsReady(snapshot) &&
            linearContentMeasurementsReady(asset),
        },
        () => fitNativeSingleLineTexts(asset),
      );
      if (scroll) scrollSnapshot = captureReviewScrollLayoutSnapshot(asset);
    },
  };
}
/** Serialize native reflow; delayed self-events become no-ops for the same draft. */
export function createAuthoringLayoutRefresh(
  events: AuthoringShapeEvents,
  hooks: AuthoringLayoutHooks,
) {
  const settled = new Map<string, string>();
  let tail = Promise.resolve();
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pauseDepth = 0;
  let generation = 0;
  let listeners: symbol[] = [];
  const clearTimer = () => {
    if (timer !== undefined) {
      clearTimeout(timer);
      timer = undefined;
    }
  };
  const unwatch = () => {
    for (const id of listeners) events.off(id);
    listeners = [];
  };
  const flush = (
    asset: Board,
    keepReviewPresentation = true,
  ): Promise<void> => {
    clearTimer();
    const requestedGeneration = generation;
    const work = tail
      .catch(() => {})
      .then(async () => {
        if (pauseDepth || requestedGeneration !== generation) return;
        const draft = hooks.draft(asset, keepReviewPresentation);
        try {
          if (settled.get(asset.id) === draft.key) return;
          await draft.refresh();
          settled.set(asset.id, draft.key);
        } finally {
          draft.finish?.();
        }
      });
    tail = work;
    return work;
  };
  const schedule = (asset: Board) => {
    if (pauseDepth) return;
    clearTimer();
    timer = setTimeout(() => {
      timer = undefined;
      void flush(asset).catch(hooks.onError);
    }, 150);
  };
  const bindSelection = () => {
    unwatch();
    if (pauseDepth) return;
    const active = hooks.asset();
    if (!active) return;
    // Scroll review offsets are temporary capture presentation. Their native
    // geometry is flushed explicitly at audit/export barriers, so their own
    // detach/restore events cannot create an authoring feedback loop.
    const review = active.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'review-case',
    );
    if (
      review &&
      reviewScrollPosition(JSON.parse(review) as LayoutReviewCaseMessage)
    )
      return;
    const ids = new Set<string>();
    for (const selected of events.selection) {
      let ancestor: Shape | null = selected;
      while (
        ancestor &&
        ancestor.getSharedPluginData(
          ZUI_METADATA_NAMESPACE,
          ZUI_METADATA_ROLE,
        ) !== ZUI_ROLE_ASSET
      )
        ancestor = ancestor.parent;
      if (ancestor?.id === active.id) ids.add(selected.id);
    }
    for (const shapeId of ids)
      listeners.push(
        events.on('shapechange', () => schedule(active), { shapeId }),
      );
  };
  const suspend = async (): Promise<() => void> => {
    pauseDepth += 1;
    generation += 1;
    clearTimer();
    unwatch();
    await tail.catch(() => {});
    return () => {
      pauseDepth -= 1;
      if (!pauseDepth) bindSelection();
    };
  };
  const dispose = () => {
    generation += 1;
    clearTimer();
    unwatch();
    settled.clear();
  };
  return { flush, bindSelection, suspend, dispose };
}

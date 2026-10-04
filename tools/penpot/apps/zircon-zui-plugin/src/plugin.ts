import { allocatedAuthoredGeometry } from './penpot-layout-allocation';
import {
  captureAuthoringLayoutDraft,
  createAuthoringLayoutRefresh,
} from './penpot-authoring-layout';
import type { Board, CommonLayout, Shape } from '@penpot/plugin-types';
import { captureNodeTexts } from './penpot-text-capture';
import { captureSlotPadding } from './penpot-slot-padding';
import { refreshNativePainterContent } from './penpot-control-content';
import { fieldPaintShape } from './penpot-field-controls';
import { fitNativeSingleLineTexts } from './penpot-text-overflow';
import {
  authoredGeometry,
  validateCapturedWrapAlignment,
  authoredAnchoredPosition,
  layoutSizingIsCompatible,
  resolvedParentContainerKinds,
  singleSolidFill,
  singleSolidStroke,
  stableSemanticChildOrder,
  type CapturedContainerKind,
  type CapturedLayoutSizing,
} from './penpot-capture-validation.js';
import {
  PENPOT_BRIDGE_VERSION,
  assertSnapshotBaselines,
  type PenpotBaselineIndex,
} from './bridge/penpot-asset.js';
import {
  decodeZuiMetadata,
  encodeZuiMetadata,
  parseZuiDocument,
  serializeZuiDocument,
  zuiNodes,
  zuiRootNodeIds,
  type ZuiDocument,
  type ZuiNode,
} from './bridge/zui-document.js';
import {
  projectZuiDocument,
  type PenpotAssetSnapshot,
  type PenpotShapeSnapshot,
  type ProjectionContainer,
  type ProjectionEditableState,
} from './bridge/penpot-projection.js';
import type { ZuiAssetProjection } from './bridge/penpot-projection-model.js';
import { resolveDesignNumber } from './bridge/zui-prefab-system.js';
import {
  materializeNativeComponents,
  removeNativeComponentAsset,
  rollbackNativeComponentAssetTransfer,
  transferNativeComponentAsset,
} from './penpot-native-components';
import { reviewDocumentForCase } from './bridge/zui-review-case.js';
import { canReuseReviewBoard } from './penpot-review-board-reuse';
import { reviewHost, reconcileReviewSource } from './bridge/zui-review-host.js';
import {
  applyReviewScrollPresentation,
  captureReviewScrollLayoutSnapshot,
  type ReviewScrollLayoutSnapshot,
  reviewScrollLayoutDiagnostics,
  reviewScrollLayoutReadyForCapture,
  reviewScrollPosition,
  restoreReviewScrollPresentation,
} from './penpot-review-scroll';
import {
  ZUI_METADATA_BASELINE,
  ZUI_METADATA_BASELINES,
  ZUI_METADATA_BRIDGE_VERSION,
  ZUI_METADATA_CHILD_INDEX,
  ZUI_METADATA_COMPONENT,
  ZUI_METADATA_DOCUMENT,
  ZUI_METADATA_FILE_NAME,
  ZUI_METADATA_LAYOUT_SIZING_GUARD,
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_METADATA_VISUAL_DETACHED,
  ZUI_METADATA_VISUAL_DETACHED_PARENT,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_NODE,
} from './metadata.js';
import type {
  LayoutReviewCaseMessage,
  PluginHostMessage,
  PluginUiMessage,
} from './model.js';
import { restoreVirtualNodes } from './penpot-virtual-nodes';
import {
  auditRenderedAssetBoard,
  createAssetBoard as createRenderedAssetBoard,
  resizeAssetReviewViewport,
  refreshAssetTextLayout,
} from './penpot-asset-renderer.js';
import {
  layoutAuditStabilitySnapshot,
  layoutSettlementOptionsForNodes,
  settleAfterLayoutRefresh,
  tableMeasurementsReady,
  waitForStableLayout,
} from './penpot-layout-settlement.js';
import {
  assertExportShapeGuard,
  writeExportShapeGuard,
} from './penpot-export-guard';
import { originalZuiSource } from './bridge/zui-source-export';

const UI_SIZE = { width: 420, height: 520 };
// Large composite pages can require several Penpot layout passes after their
// component instances and text metrics are materialized. Keep the window
// bounded, but long enough to observe those real passes instead of treating a
// slow official frontend as a successful or synthetic preview.
const REVIEW_LAYOUT_SETTLEMENT = {
  intervalMs: 100,
  quietMs: 700,
  timeoutMs: 60_000,
  numericPrecision: 3,
} as const;
let lastImportedAssetBoard: Board | null = null;
let lastImportedDocument: ZuiDocument | null = null;
// The visual harness can opt a single import into a lighter projection when
// a retained shell contains very deep composite prefab trees. Product users
// never set this flag, so normal imports continue to materialize every native
// component instance.
let pendingNativeComponentMode: 'full' | 'leaf' | 'semantic' = 'full';

penpot.flags.naturalChildOrdering = true;
penpot.ui.open('ZIRCON ZUI', `?theme=${penpot.theme}`, UI_SIZE);

penpot.on('themechange', (theme) =>
  sendMessage({ type: 'theme', content: theme }),
);
const authoringLayout = createAuthoringLayoutRefresh(penpot, {
  asset: () => lastImportedAssetBoard,
  draft: (board, present) =>
    captureAuthoringLayoutDraft(board, captureAssetSnapshot, present),
  onError: (error) =>
    sendMessage({
      type: 'status',
      level: 'error',
      message: errorMessage(error),
    }),
});
penpot.on('selectionchange', () => {
  authoringLayout.bindSelection();
  sendSelectionState();
});
penpot.on('finish', () => authoringLayout.dispose());

penpot.ui.onMessage<PluginUiMessage>((message) => {
  if (message.type === 'ready' || message.type === 'inspect-selection') {
    sendMessage({ type: 'theme', content: penpot.theme });
    sendSelectionState();
    return;
  }
  if (message.type === 'import-zui') {
    void importZui(
      message.fileName,
      message.source,
      message.replaceSelection,
      message.nativeComponentMode ?? pendingNativeComponentMode,
    );
    return;
  }
  if (message.type === 'set-review-materialization') {
    pendingNativeComponentMode = message.mode;
    // Acknowledge the design-only policy before the harness starts the file
    // input flow.  The acknowledgement closes the postMessage race between
    // the Penpot UI iframe and the plugin worker; ordinary product imports do
    // not send this message and remain on the full policy.
    sendMessage({
      type: 'status',
      level: 'idle',
      message: `Review materialization · ${message.mode}`,
    });
    return;
  }
  if (message.type === 'export-zui') {
    void exportSelectedZui();
    return;
  }
  if (message.type === 'set-preview-selection') {
    penpot.selection =
      message.selected && lastImportedAssetBoard
        ? [lastImportedAssetBoard]
        : [];
    return;
  }
  if (message.type === 'audit-preview-layout') {
    const assetBoard = lastImportedAssetBoard;
    if (!assetBoard) return;
    void authoringLayout
      .flush(assetBoard)
      .then(() => {
        sendMessage({
          type: 'preview-audit',
          requestId: message.requestId,
          previewBoardId: assetBoard.id,
          layoutAudit: auditRenderedAssetBoard(assetBoard),
        });
        penpot.viewport.zoomIntoView([assetBoard]);
      })
      .catch((error) => {
        const refreshError = errorMessage(error);
        sendMessage({ type: 'status', level: 'error', message: refreshError });
        sendMessage({
          type: 'preview-audit',
          requestId: message.requestId,
          previewBoardId: assetBoard.id,
          layoutAudit: { ...auditRenderedAssetBoard(assetBoard), refreshError },
        });
      });
    return;
  }
  if (message.type === 'set-review-case') {
    void setReviewCase(message.reviewCase);
  }
});

async function setReviewCase(
  reviewCase: LayoutReviewCaseMessage,
): Promise<void> {
  const resumeAuthoring = await authoringLayout.suspend();
  reportReviewProgress(reviewCase, 'preparing review projection');
  let board: Board | null = null;
  const previous = lastImportedAssetBoard;
  const original = lastImportedDocument;
  let transferred = false;
  try {
    if (!previous || !original)
      throw new Error('Import a ZUI asset before selecting a review case');
    const originalFileName = metadata(previous, ZUI_METADATA_FILE_NAME);
    const baseline = reviewHost(original) ?? original;
    const document = reviewDocumentForCase(baseline, reviewCase);
    const projection = projectZuiDocument(document);
    const reuseNativeBoard = canReuseReviewBoard(
      previous,
      original,
      reviewCase,
    );
    if (reuseNativeBoard) {
      // Reuse only when the authored responsive container geometry is stable.
      // Clone the already-materialized board so Penpot does not recreate
      // hundreds of native component copies for equivalent viewport states.
      // A previous scroll capture temporarily detaches children and clips the
      // semantic container. Restore that capture-only presentation before
      // cloning so the next viewport/state starts from authored geometry.
      restoreReviewScrollPresentation(previous);
      const clone = previous.clone();
      if (clone.type !== 'board')
        throw new Error('Review board clone is not a board');
      board = clone;
      resizeAssetReviewViewport(
        board,
        document,
        projection,
        reviewCase.viewport,
      );
      applyProjectionVisibility(board, projection);
      refreshReviewNativePainterContent(board, document, projection);
      refreshAssetTextLayout(board, document, projection);
      transferNativeComponentAsset(previous, board, { removePrevious: false });
      transferred = true;
      lastImportedAssetBoard = null;
    } else {
      reportReviewProgress(reviewCase, 'creating review board');
      board = await createRenderedAssetBoard(
        originalFileName,
        document,
        projection,
      );
      resizeAssetReviewViewport(
        board,
        document,
        projection,
        reviewCase.viewport,
      );
      reportReviewProgress(reviewCase, 'loading image resources');
      await loadAssetImages(board, document);
      // A review case owns fresh instances, but its component masters are
      // source/structure-stable across viewports and states. Transfer the shelf
      // before materializing the new board so the official frontend does not
      // recreate a whole component library for every capture.
      transferNativeComponentAsset(previous, board, { removePrevious: false });
      transferred = true;
      lastImportedAssetBoard = null;
      reportReviewProgress(
        reviewCase,
        'materializing native component instances',
      );
      await materializeNativeComponents(
        board,
        document,
        ({ index, total, nodeId, source, createdMaster }) => {
          if (createdMaster || index % 4 === 0)
            reportReviewProgress(
              reviewCase,
              `materializing native components ${index + 1}/${total} · ${nodeId || '<unnamed>'} · ${source}`,
            );
        },
      );
    }
    const caseBoard = board;
    reportReviewProgress(reviewCase, 'settling layout and text metrics');
    await settleAfterLayoutRefresh(
      () => refreshAssetTextLayout(caseBoard, document, projection),
      () => layoutAuditStabilitySnapshot(auditRenderedAssetBoard(caseBoard)),
      {
        ...REVIEW_LAYOUT_SETTLEMENT,
        ...layoutSettlementOptionsForNodes(projection.shapes.length),
        isReady: (snapshot) =>
          tableMeasurementsReady(snapshot) &&
          linearContentMeasurementsReady(caseBoard),
      },
      () => fitNativeSingleLineTexts(caseBoard),
    );
    const scrollSnapshot = await waitForScrollReviewLayout(
      caseBoard,
      reviewCase,
      projection.shapes.length,
    );
    reportReviewProgress(reviewCase, 'writing reversible metadata');
    // Current host geometry and state are the baseline; source metadata stays authored.
    setMetadata(board, ZUI_METADATA_DOCUMENT, encodeZuiMetadata(original));
    writeSemanticBaselines(board, baseline);
    if (originalFileName)
      setMetadata(board, ZUI_METADATA_FILE_NAME, originalFileName);
    setMetadata(board, 'review-case', JSON.stringify(reviewCase));
    applyReviewCasePresentation(board, reviewCase, scrollSnapshot);
    previous.remove();
    lastImportedAssetBoard = board;
    penpot.selection = [board];
    penpot.viewport.zoomIntoView([board]);
    sendMessage({
      type: 'status',
      level: 'success',
      message: `${reviewCase.id} · ${reviewCase.state}`,
      previewBoardId: board.id,
      reviewCase,
      layoutAudit: auditRenderedAssetBoard(board),
    });
  } catch (error) {
    if (board) {
      if (transferred && previous) {
        rollbackNativeComponentAssetTransfer(previous, board);
      } else {
        removeNativeComponentAsset(board);
      }
    }
    lastImportedAssetBoard = previous;
    lastImportedDocument = original;
    if (previous) penpot.selection = [previous];
    sendMessage({
      type: 'status',
      level: 'error',
      message: errorMessage(error),
      reviewCase,
    });
  } finally {
    resumeAuthoring();
  }
}

function applyProjectionVisibility(
  assetBoard: Board,
  projection: ZuiAssetProjection,
): void {
  const projected = new Map(
    projection.shapes.map((shape) => [shape.nodeId, shape]),
  );
  const visit = (shape: Shape, inheritedHidden: boolean): void => {
    const nodeId = metadata(shape, ZUI_METADATA_NODE_ID);
    const current = nodeId ? projected.get(nodeId) : undefined;
    const hidden = inheritedHidden || Boolean(current?.previewHidden);
    if (current && shape.type !== 'text') shape.hidden = hidden;
    for (const child of 'children' in shape ? shape.children : [])
      visit(child, hidden || child.hidden);
  };
  for (const child of assetBoard.children) visit(child, assetBoard.hidden);
}

/**
 * Update native Runtime-painter surfaces on a cloned review board.  Native
 * component copies retain their library structure and are deliberately left
 * untouched; only plain semantic boards (for example a previously collapsed
 * command palette) need new panel/text children for the selected state.
 */
function refreshReviewNativePainterContent(
  assetBoard: Board,
  document: ZuiDocument,
  projection: ZuiAssetProjection,
): void {
  const projected = new Map(
    projection.shapes.map((shape) => [shape.nodeId, shape]),
  );
  const visit = (shape: Shape): void => {
    if (shape.type === 'board') {
      const nodeId = metadata(shape, ZUI_METADATA_NODE_ID);
      const node = nodeId ? document.nodes?.[nodeId] : undefined;
      const current = nodeId ? projected.get(nodeId) : undefined;
      const nativeComponentId = metadata(shape, 'native-component-id');
      if (node && current && !nativeComponentId)
        refreshNativePainterContent(shape, node, document, current.text);
    }
    for (const child of 'children' in shape ? shape.children : []) visit(child);
  };
  visit(assetBoard);
}

async function waitForScrollReviewLayout(
  assetBoard: Board,
  reviewCase: LayoutReviewCaseMessage,
  projectedNodeCount: number,
): Promise<ReviewScrollLayoutSnapshot | null> {
  if (!reviewScrollPosition(reviewCase)) return null;
  try {
    await waitForStableLayout(
      () => {},
      () => layoutAuditStabilitySnapshot(auditRenderedAssetBoard(assetBoard)),
      {
        ...REVIEW_LAYOUT_SETTLEMENT,
        ...layoutSettlementOptionsForNodes(projectedNodeCount),
        isReady: () => reviewScrollLayoutReadyForCapture(assetBoard),
      },
    );
  } catch (error) {
    throw new Error(
      `${error instanceof Error ? error.message : String(error)}; diagnostics=${JSON.stringify(reviewScrollLayoutDiagnostics(assetBoard))}`,
      { cause: error },
    );
  }
  const snapshot = captureReviewScrollLayoutSnapshot(assetBoard);
  if (!snapshot.containers.length)
    throw new Error(
      `Scroll review ${reviewCase.id} has no semantic scroll container to snapshot.`,
    );
  return snapshot;
}

function applyReviewCasePresentation(
  assetBoard: Board,
  reviewCase: LayoutReviewCaseMessage,
  scrollSnapshot: ReviewScrollLayoutSnapshot | null,
): void {
  const state = reviewCase.state.toLowerCase();
  const scrollPosition = reviewScrollPosition(reviewCase);
  if (scrollPosition)
    applyReviewScrollPresentation(
      assetBoard,
      scrollPosition,
      scrollSnapshot ?? undefined,
    );
  const normalized = state === 'hover' ? 'hovered' : state;
  walkShapes(assetBoard, (shape) => {
    if (shape.type !== 'board') return;
    setMetadata(shape, 'review-state', normalized);
    const nodeId = metadata(shape, ZUI_METADATA_NODE_ID);
    if (!nodeId) return;
    shape.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'review-state',
      normalized,
    );
  });
}

import { loadAssetImages } from './penpot-image-content';
import { linearContentMeasurementsReady } from './penpot-linear-content';

async function importZui(
  fileName: string,
  source: string,
  replaceSelection = false,
  nativeComponentMode: 'full' | 'leaf' | 'semantic' = 'full',
): Promise<void> {
  sendMessage({
    type: 'status',
    level: 'working',
    message: `Importing ${fileName} · parsing source`,
  });
  const undoBlock = penpot.history.undoBlockBegin();
  const resumeAuthoring = await authoringLayout.suspend();
  const previousBoard = lastImportedAssetBoard;
  const previousDocument = lastImportedDocument;
  const replaced = replaceSelection ? selectedAssetBoard() : null;
  let assetBoard: Board | null = null;
  try {
    const parsed = parseZuiDocument(source);
    if (originalZuiSource(parsed.document) === undefined)
      parsed.document['penpot_original_source'] = source;
    const document = reviewHost(parsed.document) ?? parsed.document;
    reportImportProgress(fileName, 'projecting semantic nodes');
    const projection = projectZuiDocument(document);
    reportImportProgress(fileName, 'creating design board');
    assetBoard = await createRenderedAssetBoard(fileName, document, projection);
    reportImportProgress(fileName, 'loading image resources');
    await loadAssetImages(assetBoard, document);
    reportImportProgress(fileName, 'materializing native component instances');
    await materializeNativeComponents(
      assetBoard,
      document,
      ({ index, total, nodeId, source, createdMaster }) => {
        if (createdMaster || index % 4 === 0)
          reportImportProgress(
            fileName,
            `materializing native components ${index + 1}/${total} · ${nodeId || '<unnamed>'} · ${source}`,
          );
      },
      { mode: nativeComponentMode },
    );
    const importedBoard = assetBoard;
    reportImportProgress(fileName, 'settling layout and text metrics');
    await settleAfterLayoutRefresh(
      () => refreshAssetTextLayout(importedBoard, document, projection),
      () =>
        layoutAuditStabilitySnapshot(auditRenderedAssetBoard(importedBoard)),
      {
        ...REVIEW_LAYOUT_SETTLEMENT,
        ...layoutSettlementOptionsForNodes(projection.shapes.length),
        isReady: (snapshot) =>
          tableMeasurementsReady(snapshot) &&
          linearContentMeasurementsReady(importedBoard),
      },
      () => fitNativeSingleLineTexts(importedBoard),
    );
    reportImportProgress(fileName, 'writing reversible metadata');
    setMetadata(
      assetBoard,
      ZUI_METADATA_DOCUMENT,
      encodeZuiMetadata(parsed.document),
    );
    writeSemanticBaselines(assetBoard, document);
    if (replaced) removeNativeComponentAsset(replaced);
    lastImportedDocument = parsed.document;
    lastImportedAssetBoard = assetBoard;
    penpot.selection = [assetBoard];
    penpot.viewport.zoomIntoView([assetBoard]);
    sendMessage({
      type: 'status',
      level: projection.diagnostics.some(
        ({ severity }) => severity === 'warning',
      )
        ? 'warning'
        : 'success',
      message: `${projection.displayName} · ${projection.shapes.length} nodes`,
      diagnostics: [...parsed.diagnostics, ...projection.diagnostics],
      previewBoardId: assetBoard.id,
    });
    sendSelectionState();
  } catch (error) {
    if (assetBoard) removeNativeComponentAsset(assetBoard);
    lastImportedAssetBoard = previousBoard;
    lastImportedDocument = previousDocument;
    sendMessage({
      type: 'status',
      level: 'error',
      message: errorMessage(error),
    });
  } finally {
    // Scope the design-only policy to one import. A normal UI-triggered import
    // after a review must not inherit the harness optimisation.
    resumeAuthoring();
    pendingNativeComponentMode = 'full';
    penpot.history.undoBlockFinish(undoBlock);
  }
}

function reportImportProgress(fileName: string, phase: string): void {
  sendMessage({
    type: 'status',
    level: 'working',
    message: `Importing ${fileName} · ${phase}`,
  });
}

function reportReviewProgress(
  reviewCase: LayoutReviewCaseMessage,
  phase: string,
): void {
  sendMessage({
    type: 'status',
    level: 'working',
    message: `Rendering ${reviewCase.id} · ${phase}`,
    reviewCase,
  });
}

async function exportSelectedZui(): Promise<void> {
  sendMessage({
    type: 'status',
    level: 'working',
    message: 'Exporting selected ZUI asset',
  });
  try {
    const assetBoard = selectedAssetBoard();
    if (!assetBoard) {
      throw new Error(
        'Select an imported ZUI asset or one of its layers before exporting.',
      );
    }
    const bridgeVersion = assetBoard.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_BRIDGE_VERSION,
    );
    if (bridgeVersion !== String(PENPOT_BRIDGE_VERSION)) {
      throw new Error(
        `The selected asset uses unsupported ZUI bridge metadata version ${bridgeVersion || 'missing'}. Re-import its source .zui file.`,
      );
    }
    const documentValue = assetBoard.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_DOCUMENT,
    );
    if (!documentValue) {
      throw new Error(
        'The selected asset has no reversible ZUI document metadata.',
      );
    }
    const sourceDocument = decodeZuiMetadata(documentValue);
    // Scroll review offsets are a capture-only projection. Restore them before
    // taking the export snapshot so an untouched source round-trips exactly.
    restoreReviewScrollPresentation(assetBoard);
    await authoringLayout.flush(assetBoard, false);
    const snapshot = captureAssetSnapshot(
      assetBoard,
      reviewHost(sourceDocument) ?? sourceDocument,
    );
    const result = reconcileReviewSource(sourceDocument, snapshot);
    const source = result.source ?? serializeZuiDocument(result.document);
    const storedFileName = assetBoard.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_FILE_NAME,
    );
    const fileName = sanitizeFileName(
      storedFileName || sourceDocument.asset.id,
    );
    sendMessage({
      type: 'export-ready',
      fileName,
      source,
      diagnostics: result.diagnostics,
    });
    sendMessage({
      type: 'status',
      level: 'success',
      message: `${fileName} · ${result.changes.length} projected edits`,
      diagnostics: result.diagnostics,
    });
  } catch (error) {
    sendMessage({
      type: 'status',
      level: 'error',
      message: errorMessage(error),
    });
  }
}

function writeSemanticBaselines(
  assetBoard: Board,
  document: ZuiDocument,
): void {
  const baselines = Object.create(null) as PenpotBaselineIndex;
  const virtual = JSON.parse(
    metadata(assetBoard, 'virtual-nodes') || '[]',
  ) as PenpotShapeSnapshot[];
  for (const record of virtual) baselines[record.nodeId] = record.baseline;
  walkShapes(assetBoard, (shape) => {
    if (
      shape.type !== 'board' ||
      metadata(shape, ZUI_METADATA_ROLE) !== ZUI_ROLE_NODE
    ) {
      return;
    }
    const nodeId = metadata(shape, ZUI_METADATA_NODE_ID);
    setMetadata(
      shape,
      ZUI_METADATA_LAYOUT_SIZING_GUARD,
      JSON.stringify(layoutSizing(shape)),
    );
    const baseline = captureEditableState(shape, document);
    baselines[nodeId] = baseline;
    setMetadata(shape, ZUI_METADATA_BASELINE, JSON.stringify(baseline));
  });
  setMetadata(assetBoard, ZUI_METADATA_BASELINES, JSON.stringify(baselines));
  writeExportShapeGuard(assetBoard);
}

function captureAssetSnapshot(
  assetBoard: Board,
  document: ZuiDocument,
): PenpotAssetSnapshot {
  assertExportShapeGuard(assetBoard);
  const shapes: PenpotShapeSnapshot[] = [];
  const recordByNode = new Map<string, PenpotShapeSnapshot>();
  const baselineIndex = parseBaselineIndex(
    metadata(assetBoard, ZUI_METADATA_BASELINES),
    document.asset.id,
  );
  const baselineKinds = new Map<string, CapturedContainerKind>(
    Object.entries(baselineIndex).map(([nodeId, baseline]) => [
      nodeId,
      baseline.container.kind,
    ]),
  );
  const currentKinds = new Map<string, CapturedContainerKind>();
  const siblingIndexByNode = new Map<string, number>();
  const sourceParents = sourceParentIndex(zuiNodes(document));

  const visit = (shape: Shape, parentNodeId: string | null): void => {
    const role = metadata(shape, ZUI_METADATA_ROLE);
    let semanticParent = parentNodeId;
    if (role === ZUI_ROLE_NODE && shape.type === 'board') {
      const nodeId = metadata(shape, ZUI_METADATA_NODE_ID);
      const component = metadata(shape, ZUI_METADATA_COMPONENT);
      const baselineValue = metadata(shape, ZUI_METADATA_BASELINE);
      if (!nodeId || !component || !baselineValue) {
        throw new Error(
          `Semantic board ${shape.name} has incomplete ZUI metadata.`,
        );
      }
      const baseline = parseBaseline(baselineValue, nodeId);
      const visualDetachedParentId =
        metadata(shape, ZUI_METADATA_VISUAL_DETACHED) === 'true'
          ? metadata(shape, ZUI_METADATA_VISUAL_DETACHED_PARENT) || null
          : null;
      const effectiveParentNodeId = visualDetachedParentId ?? parentNodeId;
      const siblingIndex = Number.parseInt(
        metadata(shape, ZUI_METADATA_CHILD_INDEX),
        10,
      );
      if (Number.isInteger(siblingIndex) && siblingIndex >= 0) {
        siblingIndexByNode.set(nodeId, siblingIndex);
      }
      const resolvedParentKinds = resolvedParentContainerKinds(
        sourceParents.get(nodeId) ?? null,
        effectiveParentNodeId,
        baselineKinds,
        currentKinds,
      );
      if (
        resolvedParentKinds.baseline === null ||
        resolvedParentKinds.current === null
      ) {
        throw new Error(
          `Semantic node ${nodeId} has incomplete parent layout metadata; re-import the .zui asset before exporting.`,
        );
      }
      const parentKinds = {
        baseline: resolvedParentKinds.baseline,
        current: resolvedParentKinds.current,
      };
      const current = captureEditableState(
        shape,
        document,
        baseline,
        parentKinds,
      );
      const record: PenpotShapeSnapshot = {
        nodeId,
        component,
        ...(metadata(shape, 'sourcePath')
          ? { sourcePath: metadata(shape, 'sourcePath') }
          : {}),
        ...(metadata(shape, 'sourceNodeId')
          ? { sourceNodeId: metadata(shape, 'sourceNodeId') }
          : {}),
        controlId: metadata(shape, 'controlId') || null,
        ...(metadata(shape, 'instancePath')
          ? { instancePath: metadata(shape, 'instancePath') }
          : {}),
        parentNodeId: effectiveParentNodeId,
        childNodeIds: [],
        baseline,
        current,
      };
      shapes.push(record);
      recordByNode.set(nodeId, record);
      currentKinds.set(nodeId, current.container.kind);
      if (effectiveParentNodeId) {
        const parent = recordByNode.get(effectiveParentNodeId);
        if (!parent)
          throw new Error(
            `Parent semantic node ${effectiveParentNodeId} was not visited.`,
          );
        parent.childNodeIds.push(nodeId);
      }
      semanticParent = nodeId;
    }
    for (const child of childrenOf(shape)) {
      visit(child, semanticParent);
    }
  };
  visit(assetBoard, null);
  const virtual = JSON.parse(
    metadata(assetBoard, 'virtual-nodes') || '[]',
  ) as PenpotShapeSnapshot[];
  const complete = restoreVirtualNodes(shapes, virtual);
  shapes.splice(0, shapes.length, ...complete);
  for (const record of virtual) {
    const parent = sourceParents.get(record.nodeId);
    const index = parent
      ? document.nodes?.[parent]?.children?.findIndex(
          (child) => child.node === record.nodeId,
        )
      : -1;
    if (index !== undefined && index >= 0)
      siblingIndexByNode.set(record.nodeId, index);
  }
  for (const record of shapes) {
    record.childNodeIds = stableSemanticChildOrder(
      record.childNodeIds,
      siblingIndexByNode,
    );
  }

  const rootNodeIds = zuiRootNodeIds(document);
  const snapshot: PenpotAssetSnapshot = {
    assetId: document.asset.id,
    rootNodeIds,
    detachedNodeIds: shapes
      .filter(
        ({ nodeId, parentNodeId }) =>
          parentNodeId === null && !rootNodeIds.includes(nodeId),
      )
      .map(({ nodeId }) => nodeId),
    shapes,
  };
  assertSnapshotBaselines(snapshot, baselineIndex);
  return snapshot;
}

function captureEditableState(
  board: Board,
  document: ZuiDocument,
  baseline?: ProjectionEditableState,
  parentKinds?: {
    baseline: CapturedContainerKind;
    current: CapturedContainerKind;
  },
): ProjectionEditableState {
  const nodeId = metadata(board, ZUI_METADATA_NODE_ID) || board.name;
  const paintShape = fieldPaintShape(board);
  const fill = singleSolidFill(paintShape.fills, `Semantic node ${nodeId}`);
  const stroke = singleSolidStroke(
    paintShape.strokes,
    `Semantic node ${nodeId}`,
  );
  const capturedTexts = captureNodeTexts(board, baseline);
  const flex = board.flex;
  const grid = board.grid;
  validateCapturedWrapAlignment(
    nodeId,
    flex?.wrap === 'wrap',
    flex?.alignItems,
    flex?.alignContent,
  );
  const layout = flex ?? grid;
  const direction = flex?.dir.startsWith('row') ? 'row' : 'column';
  assertLayoutSizingUnchanged(board, nodeId, parentKinds);
  const capturedPosition = capturedAuthoredPosition(board, document, nodeId);
  const geometry = allocatedAuthoredGeometry(
    board,
    authoredGeometry(
      {
        x: capturedPosition.x,
        y: capturedPosition.y,
        width: board.width,
        height: board.height,
      },
      baseline?.geometry,
      hasAutoLayoutParent(board),
      board.layoutChild?.horizontalSizing,
      board.layoutChild?.verticalSizing,
    ),
    baseline?.geometry,
    hasAutoLayoutParent(board),
  );
  const slotPadding = captureSlotPadding(
    board,
    baseline?.slotPadding,
    baseline === undefined &&
      hasAutoLayoutParent(board) &&
      metadata(board.parent!, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE,
  );
  return {
    geometry,
    ...(slotPadding ? { slotPadding } : {}),
    paint: {
      fillColor: fill?.fillColor ?? null,
      fillOpacity: fill?.fillOpacity ?? 1,
      strokeColor: stroke?.strokeColor ?? null,
      strokeOpacity: stroke?.strokeOpacity ?? 1,
      strokeWidth: stroke?.strokeWidth ?? 0,
      borderRadius: paintShape.borderRadius,
      opacity: board.opacity,
    },
    ...capturedTexts,
    container: {
      kind: grid ? 'grid' : flex ? 'flex' : 'free',
      direction,
      wrap: flex?.wrap === 'wrap',
      gap: layout
        ? Math.abs(layout.rowGap - layout.columnGap) < 0.001
          ? layout.rowGap
          : (baseline?.container.gap ??
            Math.min(layout.rowGap, layout.columnGap))
        : 0,
      rowGap: layout?.rowGap ?? 0,
      columnGap: layout?.columnGap ?? 0,
      columns: grid?.columns.length ?? 1,
      rows: grid?.rows.length ?? 1,
      padding: layout
        ? {
            top: layout.topPadding,
            right: layout.rightPadding,
            bottom: layout.bottomPadding,
            left: layout.leftPadding,
          }
        : { top: 0, right: 0, bottom: 0, left: 0 },
      alignItems: normalizeAlignItems(layout?.alignItems),
      justifyContent: normalizeJustifyContent(layout?.justifyContent),
      clip: board.clipContent,
    },
  };
}

function capturedAuthoredPosition(
  board: Board,
  document: ZuiDocument,
  nodeId: string,
): { x: number; y: number } {
  if (hasAutoLayoutParent(board) || board.parent?.type !== 'board') {
    return { x: board.parentX, y: board.parentY };
  }
  const node = zuiNodes(document)[nodeId];
  if (!node) return { x: board.parentX, y: board.parentY };
  return authoredAnchoredPosition(
    { width: board.parent.width, height: board.parent.height },
    { width: board.width, height: board.height },
    capturedLayoutPoint(document, node, 'anchor'),
    capturedLayoutPoint(document, node, 'pivot'),
    { x: board.parentX, y: board.parentY },
  );
}

function capturedLayoutPoint(
  document: ZuiDocument,
  node: ZuiNode,
  key: 'anchor' | 'pivot',
): { x: number; y: number } {
  const value = node.layout?.[key];
  const table =
    typeof value === 'object' && value !== null && !Array.isArray(value)
      ? (value as Record<string, unknown>)
      : undefined;
  return {
    x: resolveDesignNumber(document, table?.['x']) ?? 0,
    y: resolveDesignNumber(document, table?.['y']) ?? 0,
  };
}

function selectedAssetBoard(): Board | null {
  for (const selected of penpot.selection) {
    let shape: Shape | null = selected;
    while (shape) {
      if (
        shape.type === 'board' &&
        metadata(shape, ZUI_METADATA_ROLE) === ZUI_ROLE_ASSET
      ) {
        return shape;
      }
      shape = shape.parent;
    }
  }
  return null;
}

function sendSelectionState(): void {
  const assetBoard = selectedAssetBoard();
  sendMessage({
    type: 'selection',
    canExport: assetBoard !== null,
    label: assetBoard?.name ?? null,
  });
}

function walkShapes(shape: Shape, visitor: (shape: Shape) => void): void {
  visitor(shape);
  childrenOf(shape).forEach((child) => walkShapes(child, visitor));
}

function childrenOf(shape: Shape): Shape[] {
  const children = 'children' in shape ? [...shape.children] : [];
  if (shape.type !== 'board' || !shape.grid) return children;

  // Penpot exposes children in z-index order. Grid cells carry the semantic
  // placement, so use their coordinates when reconstructing ZUI child order.
  return children
    .map((child, index) => ({ child, index, cell: child.layoutCell }))
    .sort((left, right) => {
      const leftCell = left.cell;
      const rightCell = right.cell;
      if (leftCell && rightCell) {
        return (
          (leftCell.row ?? Number.POSITIVE_INFINITY) -
            (rightCell.row ?? Number.POSITIVE_INFINITY) ||
          (leftCell.column ?? Number.POSITIVE_INFINITY) -
            (rightCell.column ?? Number.POSITIVE_INFINITY) ||
          left.index - right.index
        );
      }
      if (leftCell) return -1;
      if (rightCell) return 1;
      return left.index - right.index;
    })
    .map(({ child }) => child);
}

function setMetadata(shape: Shape, key: string, value: string): void {
  shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
}

function metadata(shape: Shape, key: string): string {
  return shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, key) || '';
}

function parseBaseline(value: string, nodeId: string): ProjectionEditableState {
  try {
    return JSON.parse(value) as ProjectionEditableState;
  } catch (error) {
    throw new Error(
      `Node ${nodeId} baseline metadata is invalid: ${errorMessage(error)}`,
      {
        cause: error,
      },
    );
  }
}

function parseBaselineIndex(
  value: string,
  assetId: string,
): PenpotBaselineIndex {
  try {
    const parsed: unknown = JSON.parse(value);
    if (
      typeof parsed !== 'object' ||
      parsed === null ||
      Array.isArray(parsed)
    ) {
      throw new Error('baseline index is not an object');
    }
    return parsed as PenpotBaselineIndex;
  } catch (error) {
    throw new Error(
      `Asset ${assetId} baseline index metadata is invalid: ${errorMessage(error)}`,
      { cause: error },
    );
  }
}

function assertLayoutSizingUnchanged(
  board: Board,
  nodeId: string,
  parentKinds?: {
    baseline: CapturedContainerKind;
    current: CapturedContainerKind;
  },
): void {
  const expected = parseLayoutSizingGuard(
    metadata(board, ZUI_METADATA_LAYOUT_SIZING_GUARD),
    nodeId,
  );
  const current = layoutSizing(board);
  if (
    !layoutSizingIsCompatible(
      expected,
      current,
      parentKinds?.baseline ?? null,
      parentKinds?.current ?? null,
    )
  ) {
    throw new Error(
      `Semantic node ${nodeId} changed an auto-layout sizing mode outside the .zui bridge profile; restore it or re-import the asset.`,
    );
  }
}

function layoutSizing(board: Board): CapturedLayoutSizing {
  return {
    horizontal: board.layoutChild?.horizontalSizing ?? null,
    vertical: board.layoutChild?.verticalSizing ?? null,
  };
}

function parseLayoutSizingGuard(
  value: string,
  nodeId: string,
): CapturedLayoutSizing {
  try {
    const parsed: unknown = JSON.parse(value);
    if (
      typeof parsed !== 'object' ||
      parsed === null ||
      Array.isArray(parsed) ||
      !Object.hasOwn(parsed, 'horizontal') ||
      !Object.hasOwn(parsed, 'vertical')
    ) {
      throw new Error('layout sizing guard is not an object');
    }
    return parsed as CapturedLayoutSizing;
  } catch (error) {
    throw new Error(
      `Semantic node ${nodeId} layout sizing metadata is invalid: ${errorMessage(error)}`,
      { cause: error },
    );
  }
}

function sourceParentIndex(
  nodes: Record<string, ZuiNode>,
): Map<string, string> {
  const parents = new Map<string, string>();
  for (const [parentId, node] of Object.entries(nodes)) {
    for (const child of node.children ?? []) {
      parents.set(child.node, parentId);
    }
  }
  return parents;
}

function hasAutoLayoutParent(board: Board): boolean {
  const parent = board.parent;
  return parent?.type === 'board' && Boolean(parent.flex ?? parent.grid);
}

function normalizeAlignItems(
  value: CommonLayout['alignItems'],
): ProjectionContainer['alignItems'] {
  return value === 'center' || value === 'end' || value === 'stretch'
    ? value
    : 'start';
}

function normalizeJustifyContent(
  value: CommonLayout['justifyContent'],
): ProjectionContainer['justifyContent'] {
  return value === 'center' ||
    value === 'end' ||
    value === 'space-between' ||
    value === 'space-around' ||
    value === 'space-evenly'
    ? value
    : 'start';
}

function sanitizeFileName(value: string): string {
  const tail = value.replaceAll('\\', '/').split('/').pop() || 'zircon-ui.zui';
  const safe = tail.replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '');
  return safe.toLowerCase().endsWith('.zui')
    ? safe
    : `${safe || 'zircon-ui'}.zui`;
}

function sendMessage(message: PluginHostMessage): void {
  penpot.ui.sendMessage(message);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

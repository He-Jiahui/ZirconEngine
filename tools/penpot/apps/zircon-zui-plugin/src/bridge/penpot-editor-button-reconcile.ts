import { ZuiDocumentError, type ZuiDocument } from './zui-document';
import type {
  PenpotAssetSnapshot,
  ZuiAssetProjection,
} from './penpot-projection-model';
import { hasEditorButtonPainter } from './zui-editor-button-projection';

/** Derived native colors may be exported only when the resulting source paints the edit. */
export function validateEditorButtonPaintEdits(
  document: ZuiDocument,
  snapshot: PenpotAssetSnapshot,
  project: (document: ZuiDocument) => ZuiAssetProjection,
): void {
  const edits = snapshot.shapes.filter(
    (shape) =>
      hasEditorButtonPainter(document, document.nodes![shape.nodeId]) &&
      (JSON.stringify(shape.baseline.paint) !==
        JSON.stringify(shape.current.paint) ||
        shape.baseline.text?.color !== shape.current.text?.color ||
        shape.baseline.text?.colorOpacity !== shape.current.text?.colorOpacity),
  );
  if (!edits.length) return;
  const projections = new Map(
    project(document).shapes.map((shape) => [shape.nodeId, shape]),
  );
  const equal = (left: unknown, right: unknown) =>
    typeof left === 'number' && typeof right === 'number'
      ? Math.abs(left - right) < 0.01
      : typeof left === 'string' && typeof right === 'string'
        ? left.toLowerCase() === right.toLowerCase()
        : left === right;
  for (const shape of edits) {
    const projected = projections.get(shape.nodeId)!;
    for (const key of [
      'fillColor',
      'fillOpacity',
      'strokeColor',
      'strokeOpacity',
      'strokeWidth',
    ] as const) {
      if (
        !equal(shape.baseline.paint[key], shape.current.paint[key]) &&
        !equal(projected.paint[key], shape.current.paint[key])
      )
        throw new ZuiDocumentError(
          `Native Editor button ${shape.nodeId} cannot represent ${key} edit; edit the owning theme or button variant.`,
        );
    }
    for (const key of ['color', 'colorOpacity'] as const) {
      if (
        !equal(shape.baseline.text?.[key], shape.current.text?.[key]) &&
        !equal(projected.text?.[key], shape.current.text?.[key])
      )
        throw new ZuiDocumentError(
          `Native Editor button ${shape.nodeId} cannot represent text.${key} edit; edit the owning theme or button variant.`,
        );
    }
  }
}

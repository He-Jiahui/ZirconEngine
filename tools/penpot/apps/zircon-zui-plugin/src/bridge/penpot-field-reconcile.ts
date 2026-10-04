import { ZuiDocumentError, type ZuiDocument } from './zui-document';
import type { PenpotAssetSnapshot, ZuiAssetProjection } from './penpot-projection-model';
import { isEditorField } from './zui-field-projection';

export function validateEditorFieldEdits(document: ZuiDocument, snapshot: PenpotAssetSnapshot,
  project: (value: ZuiDocument) => ZuiAssetProjection): void {
  if (!document['penpot_host_theme_source']) return;
  const edits = snapshot.shapes.filter((shape) => isEditorField(document.nodes![shape.nodeId]) && (
    JSON.stringify(shape.baseline.paint) !== JSON.stringify(shape.current.paint) ||
    JSON.stringify(shape.baseline.text) !== JSON.stringify(shape.current.text)));
  if (!edits.length) return;
  const nodes = new Map(project(document).shapes.map((shape) => [shape.nodeId, shape]));
  const same = (a: unknown, b: unknown, key: string) => typeof a === 'number' && typeof b === 'number' ?
    Math.abs(a - b) < 0.01 : key.toLowerCase().endsWith('color') && typeof a === 'string' && typeof b === 'string' ?
      a.toLowerCase() === b.toLowerCase() : a === b;
  for (const shape of edits) {
    const actual = nodes.get(shape.nodeId)!;
    if (shape.current.text && /[\r\n\u2028\u2029]/u.test(shape.current.text.characters))
      throw new ZuiDocumentError(`Native Editor field ${shape.nodeId} requires a single-line value.`);
    for (const group of ['paint', 'text'] as const) {
      const before = shape.baseline[group];
      const after = shape.current[group];
      if (!before || !after) continue;
      for (const key of Object.keys(after)) {
        const read = (value: unknown) => (value as Record<string, unknown> | null)?.[key];
        if (!same(read(before), read(after), key) && !same(read(actual[group]), read(after), key))
          throw new ZuiDocumentError(`Native Editor field ${shape.nodeId} cannot represent ${group}.${key} edit; edit its owning theme or value property.`);
      }
    }
  }
}

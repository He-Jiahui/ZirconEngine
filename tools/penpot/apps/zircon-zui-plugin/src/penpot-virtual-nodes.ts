import type { PenpotShapeSnapshot } from './bridge/penpot-projection-model';

// Hidden descendants remain in the semantic document without allocating canvas objects.
export function restoreVirtualNodes(
  rendered: PenpotShapeSnapshot[],
  virtual: PenpotShapeSnapshot[],
): PenpotShapeSnapshot[] {
  const result = [...rendered, ...virtual];
  const byId = new Map(result.map((shape) => [shape.nodeId, shape]));
  for (const shape of virtual) {
    if (!shape.parentNodeId) continue;
    const parent = byId.get(shape.parentNodeId);
    if (!parent) throw new Error(`Hidden node ${shape.nodeId} lost its parent`);
    if (!parent.childNodeIds.includes(shape.nodeId)) parent.childNodeIds.push(shape.nodeId);
  }
  return result;
}

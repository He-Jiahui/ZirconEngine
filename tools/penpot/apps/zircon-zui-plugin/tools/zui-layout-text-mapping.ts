export interface SemanticTextOwner {
  nodeId: string;
  sourceNodeId?: string;
  sourcePath?: string;
  instancePath?: string;
  shapeId?: string;
}

export interface ShapeTextMeasurement {
  shapeId: string;
  ancestorShapeIds?: string[];
}

export function semanticTextIdentity(
  node: Pick<SemanticTextOwner, 'nodeId' | 'sourceNodeId' | 'sourcePath' | 'instancePath'>,
): string {
  return node.sourcePath && node.sourceNodeId && node.instancePath
    ? JSON.stringify([node.sourcePath, node.sourceNodeId, node.instancePath])
    : node.nodeId;
}

export function mapMeasuredTexts<T extends ShapeTextMeasurement>(
  nodes: SemanticTextOwner[],
  texts: T[],
): Map<string, T[]> {
  const normalize = (id: string) => id.replace(/^shape-/, '');
  const owners = new Map(
    nodes
      .filter((node) => node.shapeId)
      .map((node) => [normalize(node.shapeId!), semanticTextIdentity(node)]),
  );
  const mapped = new Map<string, T[]>();
  for (const text of texts) {
    // The nearest semantic ancestor owns a text fragment. Never assign a
    // nested component's text to an outer container with the same caption.
    const owner = [text.shapeId, ...(text.ancestorShapeIds ?? [])]
      .map((id) => owners.get(normalize(id)))
      .find((id) => id !== undefined);
    if (owner === undefined) continue;
    const fragments = mapped.get(owner) ?? [];
    fragments.push(text);
    mapped.set(owner, fragments);
  }
  return mapped;
}

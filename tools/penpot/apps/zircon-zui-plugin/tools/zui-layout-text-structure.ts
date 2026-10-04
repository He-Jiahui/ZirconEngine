import { mapMeasuredTexts } from './zui-layout-text-mapping';

export interface TextRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface TextStructureNode {
  nodeId: string;
  shapeId?: string;
  parentNodeId: string | null;
  component: string;
  visible: boolean;
  detached?: boolean;
  /** Explicit retained-host clipping. Scroll containers are clipped by default. */
  clip?: boolean;
  /** Effective clip frame when the renderer exposes one. */
  clipBounds?: TextRect;
  bounds: TextRect;
  text: string | null;
  textParts?: { shapeId: string; text: string; bounds: TextRect }[];
}

export interface MeasuredText {
  shapeId: string;
  ancestorShapeIds?: string[];
  text: string;
  lines: TextRect[];
}

function record(value: unknown): Record<string, unknown> | undefined {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : undefined;
}

function rect(value: unknown): TextRect | undefined {
  const item = record(value);
  if (!item) return undefined;
  const values = ['x', 'y', 'width', 'height'].map((key) => item[key]);
  if (
    !values.every(
      (value) => typeof value === 'number' && Number.isFinite(value),
    )
  )
    return undefined;
  return {
    x: values[0] as number,
    y: values[1] as number,
    width: values[2] as number,
    height: values[3] as number,
  };
}

/**
 * Normalize the Runtime's semantic-node payload to the shared text audit
 * contract. Native nodes use their source node id as both node and shape id;
 * this keeps the mapping explicit without inventing a Penpot shape identity.
 */
export function nativeSemanticNodes(value: unknown): TextStructureNode[] {
  const root = record(value);
  const layout = record(root?.['layout']);
  const raw = Array.isArray(layout?.['semanticNodes'])
    ? layout['semanticNodes']
    : [];
  return raw.flatMap((candidate) => {
    const item = record(candidate);
    const nodeId =
      typeof item?.['nodeId'] === 'string' ? item['nodeId'] : undefined;
    const bounds = rect(item?.['bounds']);
    if (!item || !nodeId || !bounds) return [];
    const parentNodeId =
      typeof item['parentNodeId'] === 'string' ? item['parentNodeId'] : null;
    const component =
      typeof item['component'] === 'string' ? item['component'] : 'Unknown';
    const text = typeof item['text'] === 'string' ? item['text'] : null;
    return [
      {
        nodeId,
        shapeId: typeof item['shapeId'] === 'string' ? item['shapeId'] : nodeId,
        parentNodeId,
        component,
        visible: item['visible'] !== false,
        detached: item['detached'] === true,
        ...(typeof item['clip'] === 'boolean' ? { clip: item['clip'] } : {}),
        ...(rect(item['clipBounds'])
          ? { clipBounds: rect(item['clipBounds']) }
          : {}),
        bounds,
        text,
      },
    ];
  });
}

/** Normalize native text command line frames to measured text records. */
export function nativeTextMeasurements(value: unknown): MeasuredText[] {
  const root = record(value);
  const raw = Array.isArray(root?.['nodes'])
    ? root['nodes']
    : Array.isArray(root?.['texts'])
      ? root['texts']
      : [];
  return raw.flatMap((candidate) => {
    const item = record(candidate);
    const shapeId =
      typeof item?.['nodeId'] === 'string'
        ? item['nodeId']
        : typeof item?.['shapeId'] === 'string'
          ? item['shapeId']
          : undefined;
    if (!item || !shapeId) return [];
    const layout = record(item['layout']);
    const lines = Array.isArray(layout?.['lines'])
      ? layout['lines'].flatMap((line) => {
          const lineRecord = record(line);
          return rect(lineRecord?.['frame'] ?? line) ?? [];
        })
      : [];
    return [
      {
        shapeId,
        text: typeof item['text'] === 'string' ? item['text'] : '',
        lines,
      },
    ];
  });
}

export interface TextStructureFinding {
  kind:
    | 'missing-measurement'
    | 'text-outside-node'
    | 'node-outside-parent'
    | 'text-outside-fragment'
    | 'text-outside-viewport'
    | 'text-overlap';
  nodeIds: string[];
  texts: string[];
  bounds?: TextRect;
  semanticContext:
    'unbounded' | 'overlay' | 'scroll-content' | 'detached-template';
}

export function auditTextStructure(
  nodes: TextStructureNode[],
  texts: MeasuredText[],
  viewport: { width: number; height: number },
): TextStructureFinding[] {
  const byId = new Map(nodes.map((node) => [node.nodeId, node]));
  const mapped = mapMeasuredTexts(nodes, texts);
  const findings: TextStructureFinding[] = [];
  const measured: Array<{
    node: TextStructureNode;
    partId: string;
    text: string;
    line: TextRect;
  }> = [];
  const outside = (a: TextRect, b: TextRect) =>
    a.x < b.x - 1 ||
    a.y < b.y - 1 ||
    a.x + a.width > b.x + b.width + 1 ||
    a.y + a.height > b.y + b.height + 1;
  const intersect = (a: TextRect, b: TextRect): TextRect | null => {
    const x = Math.max(a.x, b.x);
    const y = Math.max(a.y, b.y);
    const right = Math.min(a.x + a.width, b.x + b.width);
    const bottom = Math.min(a.y + a.height, b.y + b.height);
    if (right <= x || bottom <= y) return null;
    return { x, y, width: right - x, height: bottom - y };
  };
  const context = (
    node: TextStructureNode,
  ): TextStructureFinding['semanticContext'] => {
    const visited = new Set<string>();
    let current: TextStructureNode | undefined = node;
    while (current && !visited.has(current.nodeId)) {
      visited.add(current.nodeId);
      if (current.detached) return 'detached-template';
      if (current.component.toLowerCase() === 'overlay') return 'overlay';
      if (
        /^(scrollablebox|scrollview|scroll|virtuallist)$/i.test(
          current.component,
        )
      )
        return 'scroll-content';
      current = current.parentNodeId
        ? byId.get(current.parentNodeId)
        : undefined;
    }
    return 'unbounded';
  };
  const clips = (node: TextStructureNode): TextRect | null | undefined => {
    const visited = new Set<string>();
    let current: TextStructureNode | undefined = node;
    let result: TextRect | null = null;
    let hasClip = false;
    while (current && !visited.has(current.nodeId)) {
      visited.add(current.nodeId);
      const componentIsScroll =
        /^(scrollablebox|scrollview|scroll|virtuallist)$/i.test(
          current.component,
        );
      // An explicit false is authoritative for projections that intentionally
      // expose scroll content without a clip frame. Otherwise authored clip
      // and scroll containers bound the visible glyph area.
      if (
        current.clip === true ||
        (current.clip !== false && componentIsScroll)
      ) {
        hasClip = true;
        const frame = current.clipBounds ?? current.bounds;
        result = result ? intersect(result, frame) : frame;
      }
      current = current.parentNodeId
        ? byId.get(current.parentNodeId)
        : undefined;
    }
    // `undefined` means that no ancestor clips the text. `null` means that
    // two or more clip frames do not intersect, so the glyphs are fully
    // hidden and must not collide with fixed siblings outside the scroll
    // viewport.
    return hasClip ? result : undefined;
  };
  const viewportBounds: TextRect = { x: 0, y: 0, ...viewport };
  for (const node of nodes) {
    const parent = node.parentNodeId ? byId.get(node.parentNodeId) : undefined;
    if (
      !node.visible ||
      !parent?.visible ||
      !/^(verticalgroup|horizontalgroup|container)$/i.test(parent.component) ||
      /^(overlay|popupmenu|popover|dialog|modal)$/i.test(node.component) ||
      context(node) === 'overlay' ||
      context(node) === 'detached-template' ||
      !outside(node.bounds, parent.bounds)
    )
      continue;
    // A scroll frame permits its content to leave the viewport, but an
    // ordinary row/card inside that frame must still contain its children.
    findings.push({
      kind: 'node-outside-parent',
      nodeIds: [node.nodeId, parent.nodeId],
      texts: node.text ? [node.text] : [],
      bounds: node.bounds,
      semanticContext: context(node),
    });
  }
  for (const node of nodes.filter(
    (item) => item.visible && item.text?.trim(),
  )) {
    const parts = mapped.get(node.nodeId) ?? [];
    const fragmentNodes = (node.textParts ?? []).map((part) => ({
      ...part,
      nodeId: part.shapeId,
    }));
    const fragments = mapMeasuredTexts(fragmentNodes, parts);
    for (const fragment of fragmentNodes)
      if (!fragments.get(fragment.nodeId)?.some((part) => part.lines?.length))
        findings.push({
          kind: 'missing-measurement',
          nodeIds: [node.nodeId],
          texts: [fragment.text],
          semanticContext: context(node),
        });
    if (!parts.length || parts.some((part) => !part.lines?.length)) {
      findings.push({
        kind: 'missing-measurement',
        nodeIds: [node.nodeId],
        texts: [node.text!],
        semanticContext: context(node),
      });
      continue;
    }
    for (const part of parts) {
      const fragment = fragmentNodes.find((candidate) =>
        fragments.get(candidate.nodeId)?.includes(part),
      );
      for (const line of part.lines) {
        if (outside(line, node.bounds))
          findings.push({
            kind: 'text-outside-node',
            nodeIds: [node.nodeId],
            texts: [part.text],
            bounds: line,
            semanticContext: context(node),
          });
        if (fragment && outside(line, fragment.bounds))
          findings.push({
            kind: 'text-outside-fragment',
            nodeIds: [node.nodeId],
            texts: [part.text],
            bounds: line,
            semanticContext: context(node),
          });
        if (outside(line, { x: 0, y: 0, ...viewport }))
          findings.push({
            kind: 'text-outside-viewport',
            nodeIds: [node.nodeId],
            texts: [part.text],
            bounds: line,
            semanticContext: context(node),
          });
        // Collision checks operate on pixels that can actually be visible.
        // Keep the raw line for overflow findings above, but intersect it
        // with authored clip frames and the review viewport before comparing
        // it with a sibling. This preserves scroll overflow evidence without
        // treating clipped source content as an overlap with fixed chrome.
        const clipFrame = clips(node);
        if (clipFrame === null) continue;
        let visibleLine: TextRect | null = line;
        for (const frame of [clipFrame ?? viewportBounds, viewportBounds]) {
          if (!visibleLine) break;
          visibleLine = intersect(visibleLine, frame);
        }
        if (!visibleLine) continue;
        measured.push({
          node,
          partId: fragment?.shapeId ?? part.shapeId,
          text: part.text,
          line: visibleLine,
        });
      }
    }
  }
  const pairs = new Set<string>();
  for (let a = 0; a < measured.length; a++)
    for (let b = a + 1; b < measured.length; b++) {
      const left = measured[a],
        right = measured[b];
      if (left.partId === right.partId) continue;
      const x = Math.max(left.line.x, right.line.x),
        y = Math.max(left.line.y, right.line.y);
      const width =
        Math.min(
          left.line.x + left.line.width,
          right.line.x + right.line.width,
        ) - x;
      const height =
        Math.min(
          left.line.y + left.line.height,
          right.line.y + right.line.height,
        ) - y;
      if (width <= 1 || height <= 1) continue;
      const pair = [left.partId, right.partId].sort();
      if (pairs.has(pair.join('\0'))) continue;
      pairs.add(pair.join('\0'));
      const semanticContext =
        context(left.node) === 'detached-template' ||
        context(right.node) === 'detached-template'
          ? 'detached-template'
          : context(left.node) === 'overlay' ||
              context(right.node) === 'overlay'
            ? 'overlay'
            : context(left.node) === 'scroll-content' ||
                context(right.node) === 'scroll-content'
              ? 'scroll-content'
              : 'unbounded';
      findings.push({
        kind: 'text-overlap',
        nodeIds: [...new Set([left.node.nodeId, right.node.nodeId])].sort(),
        texts: [left.text, right.text],
        bounds: { x, y, width, height },
        semanticContext,
      });
    }
  return findings;
}

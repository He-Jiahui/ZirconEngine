import { describe, expect, it } from 'vitest';
import {
  auditTextStructure,
  nativeSemanticNodes,
  nativeTextMeasurements,
  type TextStructureNode,
} from '../../tools/zui-layout-text-structure';

const node = (nodeId: string, y: number): TextStructureNode => ({
  nodeId,
  shapeId: nodeId,
  parentNodeId: null,
  component: 'Label',
  visible: true,
  bounds: { x: 0, y, width: 100, height: 20 },
  text: nodeId,
});
const text = (id: string, x: number, y: number, width = 40) => ({
  shapeId: `shape-${id}`,
  text: id,
  lines: [{ x, y, width, height: 16 }],
});

describe('measured text structure', () => {
  it('detects measured glyph collisions even when layout rectangles do not collide', () => {
    const result = auditTextStructure(
      [node('one', 0), node('two', 24)],
      [text('one', 0, 18), text('two', 8, 24)],
      { width: 100, height: 100 },
    );
    expect(
      result.some(
        (item) =>
          item.kind === 'text-outside-node' && item.nodeIds[0] === 'one',
      ),
    ).toBe(true);
    expect(
      result.some(
        (item) => item.kind === 'text-overlap' && item.nodeIds.includes('two'),
      ),
    ).toBe(true);
  });
  it('keeps explicit scroll context visible for review without accepting overflow', () => {
    const scroll = {
      ...node('scroll', 0),
      component: 'ScrollableBox',
      text: '',
    };
    const child = { ...node('child', 120), parentNodeId: 'scroll' };
    const result = auditTextStructure(
      [scroll, child],
      [text('child', 0, 120)],
      { width: 100, height: 100 },
    );
    expect(result).toEqual([
      expect.objectContaining({
        kind: 'text-outside-viewport',
        semanticContext: 'scroll-content',
      }),
    ]);
  });
  it('clips scroll content before reporting collisions with a sibling', () => {
    const scroll = {
      ...node('scroll', 0),
      component: 'ScrollableBox',
      bounds: { x: 0, y: 0, width: 100, height: 40 },
      text: '',
    };
    const clippedChild = {
      ...node('child', 0),
      parentNodeId: 'scroll',
      bounds: { x: 0, y: 0, width: 100, height: 100 },
    };
    const sibling = node('sibling', 60);
    const result = auditTextStructure(
      [scroll, clippedChild, sibling],
      [text('child', 0, 60), text('sibling', 0, 60)],
      { width: 100, height: 100 },
    );
    expect(
      result.some(
        (item) =>
          item.kind === 'text-overlap' &&
          item.nodeIds.includes('child') &&
          item.nodeIds.includes('sibling'),
      ),
    ).toBe(false);
    expect(
      result.some(
        (item) =>
          item.kind === 'text-outside-viewport' &&
          item.nodeIds.includes('child') &&
          item.semanticContext === 'scroll-content',
      ),
    ).toBe(false);
  });
  it('treats a nested clip intersection as fully hidden scroll content', () => {
    const scroll = {
      ...node('scroll', 0),
      component: 'ScrollableBox',
      bounds: { x: 0, y: 0, width: 100, height: 40 },
      text: '',
    };
    const clippedRow = {
      ...node('clipped-row', 60),
      parentNodeId: 'scroll',
      component: 'Table',
      clip: true,
      clipBounds: { x: 0, y: 60, width: 100, height: 20 },
    };
    const sibling = node('sibling', 60);
    const result = auditTextStructure(
      [scroll, clippedRow, sibling],
      [text('clipped-row', 0, 60), text('sibling', 0, 60)],
      { width: 100, height: 100 },
    );
    expect(
      result.some(
        (item) =>
          item.kind === 'text-overlap' &&
          item.nodeIds.includes('clipped-row') &&
          item.nodeIds.includes('sibling'),
      ),
    ).toBe(false);
  });
  it('ignores invisible text and permits the one logical pixel tolerance', () => {
    const visible = node('visible', 0),
      hidden = { ...node('hidden', 0), visible: false };
    expect(
      auditTextStructure([visible, hidden], [text('visible', -1, 0, 102)], {
        width: 100,
        height: 100,
      }),
    ).toEqual([]);
  });
  it('reports missing measurements independently of nonblank pixels', () => {
    expect(
      auditTextStructure([node('label', 0)], [], { width: 100, height: 100 }),
    ).toEqual([
      expect.objectContaining({
        kind: 'missing-measurement',
        nodeIds: ['label'],
      }),
    ]);
  });
  it('retains findings on detached source templates with explicit context', () => {
    const template = { ...node('template', 0), detached: true, text: '' };
    const child = { ...node('child', 0), parentNodeId: 'template' };
    expect(
      auditTextStructure([template, child], [], { width: 100, height: 100 }),
    ).toEqual([
      expect.objectContaining({
        kind: 'missing-measurement',
        nodeIds: ['child'],
        semanticContext: 'detached-template',
      }),
    ]);
  });
  it('audits internal text using its semantic component bounds', () => {
    expect(
      auditTextStructure(
        [node('button', 0)],
        [
          {
            ...text('caption', 0, 0),
            ancestorShapeIds: ['shape-button'],
          },
        ],
        { width: 100, height: 100 },
      ),
    ).toEqual([]);
  });
  it('detects collisions and overflow between cells inside one table row', () => {
    const row = {
      ...node('row', 0),
      textParts: [
        {
          shapeId: 'cell-a',
          text: 'Alpha',
          bounds: { x: 0, y: 0, width: 40, height: 20 },
        },
        {
          shapeId: 'cell-b',
          text: 'Beta',
          bounds: { x: 40, y: 0, width: 60, height: 20 },
        },
      ],
    };
    const result = auditTextStructure(
      [row],
      [
        { ...text('cell-a', 0, 0, 60), ancestorShapeIds: ['shape-row'] },
        { ...text('cell-b', 40, 0, 30), ancestorShapeIds: ['shape-row'] },
      ],
      { width: 100, height: 100 },
    );
    expect(result.map((item) => item.kind)).toEqual([
      'text-outside-fragment',
      'text-overlap',
    ]);
    expect(result[1].nodeIds).toEqual(['row']);
  });
  it('reports a missing table cell even when another cell is rendered', () => {
    const row = {
      ...node('row', 0),
      textParts: [
        {
          shapeId: 'cell-a',
          text: 'Alpha',
          bounds: { x: 0, y: 0, width: 40, height: 20 },
        },
        {
          shapeId: 'cell-b',
          text: 'Beta',
          bounds: { x: 40, y: 0, width: 60, height: 20 },
        },
      ],
    };
    expect(
      auditTextStructure(
        [row],
        [{ ...text('cell-a', 0, 0), ancestorShapeIds: ['shape-row'] }],
        { width: 100, height: 100 },
      ),
    ).toEqual([
      expect.objectContaining({ kind: 'missing-measurement', texts: ['Beta'] }),
    ]);
  });
  it('normalizes native semantic and measured text evidence into the shared audit shape', () => {
    const nodes = nativeSemanticNodes({
      layout: {
        semanticNodes: [
          {
            nodeId: 'title',
            component: 'Label',
            parentNodeId: null,
            visible: true,
            detached: false,
            clip: true,
            clipBounds: { x: 0, y: 0, width: 60, height: 12 },
            bounds: { x: 4, y: 8, width: 80, height: 20 },
            text: 'Title',
          },
        ],
      },
    });
    const measurements = nativeTextMeasurements({
      nodes: [
        {
          nodeId: 'title',
          text: 'Title',
          layout: {
            lines: [{ frame: { x: 4, y: 8, width: 36, height: 16 } }],
          },
        },
      ],
    });
    expect(nodes[0]).toMatchObject({
      nodeId: 'title',
      shapeId: 'title',
      clip: true,
      clipBounds: { x: 0, y: 0, width: 60, height: 12 },
    });
    expect(measurements).toEqual([
      {
        shapeId: 'title',
        text: 'Title',
        lines: [{ x: 4, y: 8, width: 36, height: 16 }],
      },
    ]);
    expect(
      auditTextStructure(nodes, measurements, { width: 100, height: 100 }),
    ).toEqual([]);
  });
});

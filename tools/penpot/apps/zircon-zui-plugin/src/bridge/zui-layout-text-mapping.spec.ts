import { describe, expect, it } from 'vitest';
import { mapMeasuredTexts } from '../../tools/zui-layout-text-mapping';

describe('native SVG text ownership', () => {
  const nodes = [
    { nodeId: 'panel', shapeId: 'panel-board' },
    { nodeId: 'button', shapeId: 'button-board' },
  ];
  it('maps an internal text shape to its nearest semantic component', () => {
    const fragment = {
      shapeId: 'shape-caption',
      ancestorShapeIds: ['shape-button-board', 'shape-panel-board'],
    };
    expect([...mapMeasuredTexts(nodes, [fragment])]).toEqual([
      ['button', [fragment]],
    ]);
  });
  it('keeps legacy direct IDs while refusing to guess an absent mapping', () => {
    const direct = { shapeId: 'shape-button-board' };
    expect([
      ...mapMeasuredTexts(nodes, [direct, { shapeId: 'caption' }]),
    ]).toEqual([['button', [direct]]]);
  });
  it('preserves each measured fragment once and in rendered order', () => {
    const fragments = ['first', 'second'].map((shapeId) => ({
      shapeId,
      ancestorShapeIds: ['button-board', 'panel-board'],
    }));
    expect([...mapMeasuredTexts(nodes, fragments)]).toEqual([
      ['button', fragments],
    ]);
  });
});

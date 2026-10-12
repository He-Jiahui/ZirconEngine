import { describe, expect, it } from 'vitest';
import { findPenpotShapeAncestor } from './penpot-shape-ancestors';

interface TestShape {
  id: string;
  role: 'asset' | 'shape';
  parent: TestShape | null;
}

describe('Penpot shape ancestor traversal', () => {
  it('stops when different proxy objects repeat a stable shape id', () => {
    const repeated = {
      id: 'same-id',
      role: 'shape',
      get parent(): TestShape | null {
        throw new Error('must stop before following a repeated id');
      },
    } as TestShape;
    const start: TestShape = {
      id: 'same-id',
      role: 'shape',
      parent: repeated,
    };

    expect(
      findPenpotShapeAncestor(start, (shape) => shape.parent, () => false),
    ).toBeNull();
  });

  it('stops before following a shape with a missing id', () => {
    const missingId = {
      id: undefined,
      role: 'shape',
      get parent(): TestShape | null {
        throw new Error('must stop before following a missing id');
      },
    } as unknown as TestShape;

    expect(
      findPenpotShapeAncestor(missingId, (shape) => shape.parent, () => false),
    ).toBeNull();
  });

  it('finds the nearest matching ancestor on a valid parent chain', () => {
    const board: TestShape = { id: 'board', role: 'asset', parent: null };
    const group: TestShape = { id: 'group', role: 'shape', parent: board };
    const text: TestShape = { id: 'text', role: 'shape', parent: group };

    expect(
      findPenpotShapeAncestor(
        text,
        (shape) => shape.parent,
        (shape) => shape.role === 'asset',
      ),
    ).toBe(board);
  });
});

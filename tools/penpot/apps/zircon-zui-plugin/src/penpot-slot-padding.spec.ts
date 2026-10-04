import type { Board } from '@penpot/plugin-types';
import { describe, expect, it } from 'vitest';
import { applySlotPadding, captureSlotPadding } from './penpot-slot-padding';

function layoutBoard(values: Record<string, unknown>) {
  const writes: string[] = [];
  const child = new Proxy(values, {
    set(target, key, value) {
      writes.push(String(key));
      target[String(key)] = value;
      return true;
    },
  });
  return {
    board: { name: 'slot', layoutChild: child } as unknown as Board,
    writes,
    values,
  };
}

describe('Penpot slot margin updates', () => {
  it('keeps equivalent zero margins out of the host reflow queue', () => {
    const { board, writes } = layoutBoard({ marginType: 'simple' });
    const padding = { top: 0, right: 0, bottom: 0, left: 0 };
    applySlotPadding(board, padding);
    expect(captureSlotPadding(board, padding)).toEqual(padding);
    expect(writes).toEqual([]);
  });

  it('updates only changed sides while preserving reversible margins', () => {
    const { board, writes } = layoutBoard({
      marginType: 'multiple',
      topMargin: 4,
      rightMargin: 8,
      bottomMargin: 4,
      leftMargin: 8,
    });
    const padding = { top: 4, right: 12, bottom: 4, left: 8 };
    applySlotPadding(board, padding);
    expect(captureSlotPadding(board, padding)).toEqual(padding);
    expect(writes).toEqual(['rightMargin']);
  });

  it('supports side setters when the official proxy omits marginType', () => {
    const { board, writes, values } = layoutBoard({ topMargin: 2 });
    const padding = { top: 2, right: 5, bottom: 0, left: 0 };
    applySlotPadding(board, padding);
    expect(captureSlotPadding(board, padding)).toEqual(padding);
    expect(writes).toEqual(['rightMargin']);
    expect(values['marginType']).toBeUndefined();
  });

  it('restores asymmetric margin semantics even when side values match', () => {
    const { board, writes, values } = layoutBoard({
      marginType: 'simple',
      topMargin: 4,
      rightMargin: 8,
      bottomMargin: 0,
      leftMargin: 8,
    });
    const padding = { top: 4, right: 8, bottom: 0, left: 8 };
    applySlotPadding(board, padding);
    // Simple mode mirrors top onto bottom regardless of the stored bottom side.
    expect(values['marginType']).toBe('multiple');
    expect(captureSlotPadding(board, padding)).toEqual(padding);
    expect(writes).toEqual(['marginType']);
  });
});

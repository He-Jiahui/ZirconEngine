import {
  solveWeightedFlexWidths,
  stretchedRowChildHeight,
} from './penpot-weighted-flex';

describe('weighted ZUI widths in Penpot flex rows', () => {
  it('matches the Runtime constraint solver for a centered capped panel', () => {
    const widths = [
      { min: 16, preferred: 16, weight: 1, priority: 0 },
      { min: 0, preferred: 1000, max: 1000, weight: 12, priority: 100 },
      { min: 16, preferred: 16, weight: 1, priority: 0 },
    ];

    expect(solveWeightedFlexWidths(640, widths)).toEqual([16, 608, 16]);
    expect(solveWeightedFlexWidths(900, widths)).toEqual([16, 868, 16]);
    expect(solveWeightedFlexWidths(1280, widths)).toEqual([140, 1000, 140]);
  });

  it('treats an unresolved optional max as unbounded in the authored row', () => {
    const widths = [
      { min: 16, preferred: 16, max: null, weight: 1, priority: 0 },
      { min: 0, preferred: 1000, max: 1000, weight: 12, priority: 100 },
      { min: 16, preferred: 16, max: null, weight: 1, priority: 0 },
    ];
    expect(solveWeightedFlexWidths(1280, widths)).toEqual([140, 1000, 140]);
    expect(
      stretchedRowChildHeight(42, 0, 0, 1, {
        stretch: 'Stretch',
        max: null,
      }),
    ).toBe(42);
  });

  it('distributes both growth and shrink between unequal weighted content panes', () => {
    const widths = [
      { min: 136, preferred: 184, max: 240, weight: 1, priority: 0 },
      { min: 0, preferred: 560, max: 760, weight: 4, priority: 0 },
    ];

    const narrow = solveWeightedFlexWidths(608, widths);
    expect(narrow[0]).toBeCloseTo(156.8);
    expect(narrow[1]).toBeCloseTo(451.2);
    expect(solveWeightedFlexWidths(1000, widths)).toEqual([240, 760]);
  });

  it('keeps fixed controls fixed and does not hide a child when minima overflow', () => {
    const widths = [
      {
        min: 28,
        preferred: 28,
        max: 28,
        weight: 1,
        priority: 0,
        stretch: 'Fixed' as const,
      },
      { min: 240, preferred: 320, weight: 3, priority: 0 },
    ];

    expect(solveWeightedFlexWidths(400, widths)).toEqual([28, 372]);
    expect(solveWeightedFlexWidths(200, widths)).toEqual([28, 240]);
  });

  it('resizes a cross-axis fill child after a review viewport shrinks', () => {
    expect(
      stretchedRowChildHeight(488, 0, 0, 608, { stretch: 'Stretch' }),
    ).toBe(488);
    expect(
      stretchedRowChildHeight(488, 12, 12, 608, {
        stretch: 'Stretch',
        min: 100,
      }),
    ).toBe(464);
    expect(stretchedRowChildHeight(488, 0, 0, 32, { stretch: 'Fixed' })).toBe(
      32,
    );
  });
});

import {
  solveWeightedFlexWidths,
  solveNativeLinearFlexWidths,
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

// Values follow the native taffy_layout_pass linear_slots fixtures and the
// ordinary 1280x800 Unreal editor semantic frames (not a synthetic renderer).
describe('current native linear flex slots', () => {
  it('matches the ordinary Unreal viewport and right dock, then freezes the dock minimum', () => {
    const panes = [
      {min: 0, basis: 0, grow: 1, shrink: 1},
      {min: 240, basis: 0, grow: 0.25, shrink: 1},
    ];
    expect(solveNativeLinearFlexWidths(1276, panes)).toEqual([1020.8, 255.2]);
    expect(solveNativeLinearFlexWidths(896, panes)).toEqual([656, 240]);
    expect(solveNativeLinearFlexWidths(636, panes)).toEqual([396, 240]);
  });

  it('matches the native outliner/details split without treating both minima as content bases', () => {
    const panes = [
      {min: 96, basis: 0, grow: 0.4, shrink: 1},
      {min: 96, basis: 0, grow: 1, shrink: 1},
    ];
    const sizes = solveNativeLinearFlexWidths(690, panes);
    expect(sizes[0]).toBeCloseTo(197.1428680419922, 4);
    expect(sizes[1]).toBeCloseTo(492.8571472167969, 4);
  });

  it('matches native Auto and StretchContent measured slots', () => {
    expect(solveNativeLinearFlexWidths(100, [
      {min: 0, basis: 25, grow: 0, shrink: 1},
      {min: 0, basis: 10, grow: 0, shrink: 1},
    ])).toEqual([25, 10]);
    expect(solveNativeLinearFlexWidths(95, [
      {min: 0, basis: 25, grow: 1, shrink: 1},
      {min: 0, basis: 10, grow: 1, shrink: 1},
    ])).toEqual([55, 40]);
  });

  it('redistributes room after the native slot maximum is reached', () => {
    expect(solveNativeLinearFlexWidths(200, [
      {min: 80, max: 90, basis: 80, grow: 1, shrink: 1},
      {min: 0, basis: 0, grow: 1, shrink: 1},
    ])).toEqual([90, 110]);
  });

  it('retains explicit zero grow/shrink and scales shrinking by the content basis', () => {
    expect(solveNativeLinearFlexWidths(300, [
      {min: 0, basis: 80, grow: 0, shrink: 0},
      {min: 0, basis: 0, grow: 1, shrink: 1},
    ])).toEqual([80, 220]);
    expect(solveNativeLinearFlexWidths(100, [
      {min: 0, basis: 100, grow: 0, shrink: 1},
      {min: 0, basis: 100, grow: 0, shrink: 0},
    ])).toEqual([0, 100]);
    expect(solveNativeLinearFlexWidths(100, [
      {min: 0, basis: 50, grow: 0, shrink: 1},
      {min: 0, basis: 150, grow: 0, shrink: 1},
    ])).toEqual([25, 75]);
  });

  it('keeps bounded minima when the available extent is smaller than the retained controls', () => {
    expect(solveNativeLinearFlexWidths(100, [
      {min: 96, basis: 0, grow: 0.4, shrink: 1},
      {min: 96, basis: 0, grow: 1, shrink: 1},
    ])).toEqual([96, 96]);
  });

  it('native linear deficit keeps fixed slot margins outside the content shrink basis', () => {
    const axes = [
      {min: 0, basis: 28, grow: 0, shrink: 1, margin: 0},
      {min: 0, basis: 60, grow: 0, shrink: 1, margin: 12},
    ];
    const sizes = solveNativeLinearFlexWidths(70, axes);
    expect(sizes).toEqual([18.454545, 39.545455]);
    expect(sizes[0] + sizes[1] + axes[1].margin).toBeCloseTo(70, 6);
  });

  it('native linear deficit reserves fixed margins before growing zero-basis Stretch slots', () => {
    const axes = [
      {min: 0, basis: 0, grow: 1, shrink: 1, margin: 0},
      {min: 0, basis: 0, grow: 1, shrink: 1, margin: 12},
    ];
    expect(solveNativeLinearFlexWidths(100, axes)).toEqual([44, 44]);
  });

  it('native linear deficit freezes content bounds while retaining the full slot margin', () => {
    const shrinking = [
      {min: 24, basis: 28, grow: 0, shrink: 1, margin: 0},
      {min: 0, basis: 60, grow: 0, shrink: 1, margin: 12},
    ];
    expect(solveNativeLinearFlexWidths(70, shrinking)).toEqual([24, 34]);
    const growing = [
      {min: 0, max: 30, basis: 0, grow: 1, shrink: 1, margin: 12},
      {min: 0, basis: 0, grow: 1, shrink: 1, margin: 0},
    ];
    expect(solveNativeLinearFlexWidths(100, growing)).toEqual([30, 58]);
    expect(solveNativeLinearFlexWidths(40, shrinking.map((axis, index) => ({
      ...axis,
      min: index === 0 ? 24 : 30,
    })))).toEqual([24, 30]);
  });

  it('native linear deficit retains explicit fractional and zero shrink when fixed margins overflow', () => {
    const fractional = [
      {min: 0, basis: 100, grow: 0, shrink: 0.5, margin: 120},
    ];
    // The negative room after reserving the margin must not be clamped to zero.
    expect(solveNativeLinearFlexWidths(100, fractional)).toEqual([40]);
    const locked = [
      {min: 0, basis: 28, grow: 0, shrink: 0, margin: 0},
      {min: 0, basis: 60, grow: 0, shrink: 1, margin: 12},
    ];
    expect(solveNativeLinearFlexWidths(70, locked)).toEqual([28, 30]);
  });
});

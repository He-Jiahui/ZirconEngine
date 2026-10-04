import {
  EMPTY_TEXT_SENTINEL,
  authoredGeometry,
  capturedMappedTextStyle,
  capturedFontTypography,
  capturedSemanticText,
  gridCellForChildIndex,
  layoutSizingIsCompatible,
  authoredAnchoredPosition,
  resolvedAnchoredPosition,
  resolvedParentContainerKinds,
  singleSolidFill,
  singleSolidStroke,
  stableSemanticChildOrder,
  symmetricLayoutGap,
} from './penpot-capture-validation';

describe('Penpot capture paint validation', () => {
  it('captures editable family and line height while rejecting mixed or invalid text', () => {
    expect(capturedFontTypography('Fira Sans', '1.5', 'title')).toEqual({
      fontFamily: 'Fira Sans',
      lineHeight: 1.5,
    });
    expect(() => capturedFontTypography('mixed', '1.5', 'title')).toThrow(
      'font family',
    );
    expect(() => capturedFontTypography('Fira Sans', 'mixed', 'title')).toThrow(
      'line height',
    );
    expect(() => capturedFontTypography('Fira Sans', '0', 'title')).toThrow(
      'line height',
    );
  });
  it('places semantic grid children with Penpot one-based coordinates', () => {
    expect(gridCellForChildIndex(0)).toEqual({ row: 1, column: 1 });
    expect(gridCellForChildIndex(8)).toEqual({ row: 9, column: 1 });
    expect(gridCellForChildIndex(0, 3)).toEqual({ row: 1, column: 1 });
    expect(gridCellForChildIndex(2, 3)).toEqual({ row: 1, column: 3 });
    expect(gridCellForChildIndex(3, 3)).toEqual({ row: 2, column: 1 });
    expect(() => gridCellForChildIndex(-1)).toThrow(/non-negative integer/i);
    expect(() => gridCellForChildIndex(0, 0)).toThrow(/positive integer/i);
  });

  it('restores semantic sibling order after Penpot grid traversal', () => {
    expect(
      stableSemanticChildOrder(
        ['warlock', 'warrior', 'mage'],
        new Map([
          ['warrior', 0],
          ['mage', 1],
          ['warlock', 2],
        ]),
      ),
    ).toEqual(['warrior', 'mage', 'warlock']);
  });

  it('restores a Penpot placeholder to the authored empty text', () => {
    expect(capturedSemanticText(EMPTY_TEXT_SENTINEL, true)).toBe('');
    expect(capturedSemanticText(`${EMPTY_TEXT_SENTINEL}Edited`, true)).toBe(
      'Edited',
    );
    expect(capturedSemanticText(EMPTY_TEXT_SENTINEL, false)).toBe(
      EMPTY_TEXT_SENTINEL,
    );
  });

  it('rejects mixed mapped text styles and asymmetric layout gaps', () => {
    expect(() =>
      capturedMappedTextStyle('mixed', '400', 'left', 'Editable text'),
    ).toThrow(/mixed or invalid font size/i);
    expect(() =>
      capturedMappedTextStyle('12', 'mixed', 'left', 'Editable text'),
    ).toThrow(/mixed or invalid font weight/i);
    expect(() =>
      capturedMappedTextStyle('12', '400', 'mixed', 'Editable text'),
    ).toThrow(/mixed or missing text alignment/i);
    expect(() => symmetricLayoutGap(8, 12, 'Semantic node root')).toThrow(
      /asymmetric row and column gaps/i,
    );
    expect(symmetricLayoutGap(8, 8, 'Semantic node root')).toBe(8);
  });

  it('suppresses auto-layout geometry while retaining fixed-size edits', () => {
    const baseline = { x: 10, y: 20, width: 100, height: 40 };
    const laidOut = { x: 30, y: 50, width: 180, height: 48 };

    expect(authoredGeometry(laidOut, baseline, true, 'fill', 'fix')).toEqual({
      x: 10,
      y: 20,
      width: 100,
      height: 48,
    });
    expect(authoredGeometry(laidOut, baseline, false, 'fill', 'fill')).toEqual(
      laidOut,
    );
  });

  it('resolves and reverses anchor, pivot, and offset positioning', () => {
    const resolved = resolvedAnchoredPosition(
      { width: 960, height: 640 },
      { width: 260, height: 106 },
      { x: 1, y: 0 },
      { x: 1, y: 0 },
      { x: -24, y: 226 },
    );

    expect(resolved).toEqual({ x: 676, y: 226 });
    expect(
      authoredAnchoredPosition(
        { width: 960, height: 640 },
        { width: 260, height: 106 },
        { x: 1, y: 0 },
        { x: 1, y: 0 },
        resolved,
      ),
    ).toEqual({ x: -24, y: 226 });
  });

  it('accepts only derived sizing availability across parent mode transitions', () => {
    const unavailable = { horizontal: null, vertical: null };
    const fixed = { horizontal: 'fix', vertical: 'fix' };
    const edited = { horizontal: 'fill', vertical: 'fix' };

    expect(layoutSizingIsCompatible(unavailable, fixed, 'free', 'flex')).toBe(
      true,
    );
    expect(layoutSizingIsCompatible(fixed, unavailable, 'grid', 'free')).toBe(
      true,
    );
    expect(layoutSizingIsCompatible(unavailable, edited, 'free', 'grid')).toBe(
      false,
    );
    expect(layoutSizingIsCompatible(fixed, edited, 'flex', 'grid')).toBe(false);
  });

  it('resolves reparented layout contexts from source and current hierarchies', () => {
    const baselineKinds = new Map([
      ['source-free', 'free' as const],
      ['source-auto', 'flex' as const],
    ]);
    const currentKinds = new Map([
      ['target-auto', 'grid' as const],
      ['target-free', 'free' as const],
    ]);

    expect(
      resolvedParentContainerKinds(
        'source-free',
        'target-auto',
        baselineKinds,
        currentKinds,
      ),
    ).toEqual({ baseline: 'free', current: 'grid' });
    expect(
      resolvedParentContainerKinds(
        'source-auto',
        'target-free',
        baselineKinds,
        currentKinds,
      ),
    ).toEqual({ baseline: 'flex', current: 'free' });
  });

  it('accepts the supported single-solid paint profile', () => {
    expect(
      singleSolidFill([{ fillColor: '#112233', fillOpacity: 0.5 }], 'node'),
    ).toMatchObject({ fillColor: '#112233' });
    expect(
      singleSolidStroke(
        [
          {
            strokeColor: '#445566',
            strokeStyle: 'solid',
            strokeAlignment: 'inner',
          },
        ],
        'node',
      ),
    ).toMatchObject({ strokeColor: '#445566' });
  });

  it('accepts null stroke caps returned by the Penpot StrokeProxy', () => {
    expect(
      singleSolidStroke(
        [
          {
            strokeColor: '#445566',
            strokeStyle: 'solid',
            strokeAlignment: 'inner',
            strokeCapStart: null,
            strokeCapEnd: null,
          } as never,
        ],
        'node',
      ),
    ).toMatchObject({ strokeColor: '#445566' });
  });

  it('rejects paint structures that .zui cannot represent', () => {
    expect(() =>
      singleSolidFill(
        [{ fillColor: '#112233' }, { fillColor: '#445566' }],
        'node',
      ),
    ).toThrow(/multiple fills/i);
    expect(() =>
      singleSolidFill([{ fillColorGradient: {} as never }], 'node'),
    ).toThrow(/non-solid/i);
    expect(() =>
      singleSolidStroke(
        [{ strokeColor: '#112233', strokeStyle: 'dashed' }],
        'node',
      ),
    ).toThrow(/unsupported stroke/i);
  });
});

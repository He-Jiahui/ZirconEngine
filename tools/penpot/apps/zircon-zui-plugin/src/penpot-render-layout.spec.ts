import {
  applyResolvedLayoutChildConstraints,
  auditLayoutBounds,
  arrangedFreeParentExtent,
  childDimensionSizing,
  fillsFreeParentDimension,
  previewAnchoredContainerMinimum,
  previewAssetGeometry,
  previewBoardMinimum,
  renderedRelativePosition,
} from './penpot-render-layout';

describe('Penpot preview board layout', () => {
  const document = {
    asset: { kind: 'component', id: 'component', version: 2 },
    tokens: { fitted: 182 },
  } as const;

  it('omits unset child bounds because Penpot rejects null assignments', () => {
    const writes: Partial<
      Record<'minWidth' | 'maxWidth' | 'minHeight' | 'maxHeight', number>
    > = {};
    const target = new Proxy(
      {} as Parameters<typeof applyResolvedLayoutChildConstraints>[0],
      {
        set(_object, key, value) {
          if (value === null)
            throw new Error('Penpot rejects null constraints');
          if (
            key === 'minWidth' ||
            key === 'maxWidth' ||
            key === 'minHeight' ||
            key === 'maxHeight'
          )
            writes[key] = value as number;
          return true;
        },
      },
    );

    expect(() =>
      applyResolvedLayoutChildConstraints(target, {
        minWidth: null,
        maxWidth: 640,
        minHeight: null,
        maxHeight: 480,
      }),
    ).not.toThrow();
    expect(writes).toEqual({ maxWidth: 640, maxHeight: 480 });
  });

  it('keeps reusable component previews compact', () => {
    expect(previewBoardMinimum('component')).toEqual({
      width: 240,
      height: 48,
    });
    expect(previewBoardMinimum('view')).toEqual({ width: 320, height: 240 });
  });

  it('measures content containers without a preferred size and respects explicit boundaries', () => {
    expect(
      childDimensionSizing(document, { stretch: 'Fixed' }, true, undefined),
    ).toBe('auto');
    expect(
      childDimensionSizing(
        document,
        { min: 28, max: 100 },
        true,
        'ContentDriven',
      ),
    ).toBe('auto');
    expect(
      childDimensionSizing(document, { preferred: '$fitted' }, true, undefined),
    ).toBe('fix');
    expect(
      childDimensionSizing(document, { stretch: 'Stretch' }, true, undefined),
    ).toBe('fill');
    expect(childDimensionSizing(document, undefined, true, 'Fixed')).toBe(
      'fix',
    );
    expect(
      childDimensionSizing(document, undefined, true, 'ParentDirected'),
    ).toBe('fix');
    expect(childDimensionSizing(document, undefined, false, undefined)).toBe(
      'fix',
    );
  });

  it('keeps minimum and preferred sizes flexible inside an overlay', () => {
    expect(fillsFreeParentDimension(document, { stretch: 'Stretch' })).toBe(
      true,
    );
    expect(
      fillsFreeParentDimension(document, {
        stretch: 'Stretch',
        preferred: '$fitted',
      }),
    ).toBe(true);
    expect(
      fillsFreeParentDimension(document, {
        stretch: 'Stretch',
        min: 120,
      }),
    ).toBe(true);
    expect(
      fillsFreeParentDimension(document, { stretch: 'Fixed', min: 120 }),
    ).toBe(false);
    expect(fillsFreeParentDimension(document, undefined)).toBe(false);
  });

  it('sizes the asset board for every detached preview node', () => {
    expect(
      previewAssetGeometry(
        'component',
        [{ width: 320, height: 240 }],
        [
          { width: 300, height: 120 },
          { width: 200, height: 80 },
        ],
        null,
      ),
    ).toEqual({
      primaryWidth: 320,
      primaryHeight: 240,
      detachedLaneX: 368,
      detachedLaneWidth: 332,
      detachedLaneHeight: 244,
      width: 700,
      height: 244,
    });
  });

  it('clamps stretched overlays to the available extent and authored bounds', () => {
    const dimension = {
      stretch: 'Stretch',
      min: 120,
      preferred: 360,
      max: '$fitted',
    };
    expect(arrangedFreeParentExtent(document, dimension, 360, 160)).toBe(160);
    expect(arrangedFreeParentExtent(document, dimension, 360, 640)).toBe(182);
    expect(arrangedFreeParentExtent(document, dimension, 360, 80)).toBe(120);
    expect(
      arrangedFreeParentExtent(
        document,
        { ...dimension, stretch: 'Fixed' },
        160,
        640,
      ),
    ).toBe(160);
    expect(
      arrangedFreeParentExtent(document, { stretch: 'Stretch' }, 360, 240),
    ).toBe(240);
  });

  it('expands the preview canvas for anchored root insets', () => {
    expect(
      previewAnchoredContainerMinimum(
        { width: 560, height: 690 },
        { x: 1, y: 0 },
        { x: 1, y: 0 },
        { x: -16, y: 16 },
      ),
    ).toEqual({ width: 576, height: 706 });
  });

  it('derives rendered child coordinates from absolute bounds', () => {
    expect(
      renderedRelativePosition({ x: 512, y: 428 }, { x: 504, y: 420 }),
    ).toEqual({ x: 8, y: 8 });
  });

  it('reports visible auto-layout and clipped overlay children outside their parents', () => {
    expect(
      auditLayoutBounds([
        {
          nodeId: 'inside',
          parentKind: 'flex',
          parentClips: false,
          parentWidth: 100,
          parentHeight: 80,
          x: 8,
          y: 8,
          width: 84,
          height: 64,
        },
        {
          nodeId: 'right-overflow',
          parentKind: 'grid',
          parentClips: false,
          parentWidth: 100,
          parentHeight: 80,
          x: 20,
          y: 10,
          width: 84,
          height: 40,
        },
        {
          nodeId: 'free-overlay',
          parentKind: 'free',
          parentClips: false,
          parentWidth: 100,
          parentHeight: 80,
          x: -24,
          y: 10,
          width: 40,
          height: 40,
        },
        {
          nodeId: 'clipped-free-overlay',
          parentKind: 'free',
          parentClips: true,
          parentWidth: 100,
          parentHeight: 80,
          x: -24,
          y: 10,
          width: 40,
          height: 40,
        },
        {
          nodeId: 'explicit-clipped-overlay',
          explicitOverlay: true,
          parentKind: 'free',
          parentClips: true,
          parentWidth: 100,
          parentHeight: 80,
          x: -24,
          y: 10,
          width: 40,
          height: 40,
        },
        {
          nodeId: 'hidden-flex-variant',
          parentKind: 'flex',
          parentClips: false,
          parentWidth: 100,
          parentHeight: 80,
          x: -200,
          y: -200,
          width: 300,
          height: 300,
          hidden: true,
        },
        {
          nodeId: 'scroll-content',
          parentKind: 'scroll',
          parentClips: true,
          parentWidth: 100,
          parentHeight: 80,
          x: 0,
          y: 0,
          width: 100,
          height: 320,
          scrollContext: { horizontal: false, vertical: true },
        },
        {
          nodeId: 'invalid',
          parentKind: 'asset',
          parentClips: false,
          parentWidth: 100,
          parentHeight: 80,
          x: 0,
          y: 0,
          width: Number.NaN,
          height: 40,
        },
      ]),
    ).toEqual({
      totalNodes: 8,
      checkedNodes: 6,
      overflowCount: 2,
      overflowNodeIds: ['right-overflow', 'clipped-free-overlay'],
      overflowDetails: [
        {
          nodeId: 'right-overflow',
          parentKind: 'grid',
          parentClips: false,
          parentWidth: 100,
          parentHeight: 80,
          x: 20,
          y: 10,
          width: 84,
          height: 40,
        },
        {
          nodeId: 'clipped-free-overlay',
          parentKind: 'free',
          parentClips: true,
          parentWidth: 100,
          parentHeight: 80,
          x: -24,
          y: 10,
          width: 40,
          height: 40,
        },
      ],
      invalidGeometryCount: 1,
      invalidGeometryNodeIds: ['invalid'],
      invalidGeometryDetails: [
        {
          nodeId: 'invalid',
          parentKind: 'asset',
          parentClips: false,
          parentWidth: 100,
          parentHeight: 80,
          x: 0,
          y: 0,
          width: Number.NaN,
          height: 40,
        },
      ],
    });
  });

  it('does not exempt a flex child because an ancestor scrolls', () => {
    const audit = auditLayoutBounds([
      {
        nodeId: 'horizontal-scroll-descendant',
        parentKind: 'flex',
        parentClips: false,
        parentWidth: 100,
        parentHeight: 80,
        x: 96,
        y: 8,
        width: 32,
        height: 48,
        scrollContext: { horizontal: true, vertical: false },
      },
      {
        nodeId: 'horizontal-scroll-cross-axis-leak',
        parentKind: 'flex',
        parentClips: false,
        parentWidth: 100,
        parentHeight: 80,
        x: 8,
        y: 64,
        width: 48,
        height: 24,
        scrollContext: { horizontal: true, vertical: false },
      },
    ]);

    expect(audit.overflowNodeIds).toEqual([
      'horizontal-scroll-descendant',
      'horizontal-scroll-cross-axis-leak',
    ]);
    expect(audit.overflowDetails).toHaveLength(2);
  });

  it('detects measured button rows escaping a card inside a scrolling drawer', () => {
    const audit = auditLayoutBounds([
      ...[38, 78, 118, 158].map((y, index) => ({
        nodeId: `buttons-row-${index}`,
        parentKind: 'flex' as const,
        parentClips: false,
        parentWidth: 204,
        parentHeight: 54,
        x: 8,
        y,
        width: 188,
        height: 32,
        scrollContext: { horizontal: false, vertical: true },
      })),
      {
        nodeId: 'drawer-content',
        parentKind: 'scroll',
        parentClips: true,
        parentWidth: 960,
        parentHeight: 198,
        x: 0,
        y: 0,
        width: 960,
        height: 240,
        scrollContext: { horizontal: false, vertical: true },
      },
    ]);

    expect(audit.overflowNodeIds).toEqual([
      'buttons-row-0',
      'buttons-row-1',
      'buttons-row-2',
      'buttons-row-3',
    ]);
  });

  it('checks the cross axis of a direct scrolling viewport child', () => {
    const audit = auditLayoutBounds([
      {
        nodeId: 'scroll-cross-axis-leak',
        parentKind: 'scroll',
        parentClips: true,
        parentWidth: 100,
        parentHeight: 80,
        x: 0,
        y: 0,
        width: 104,
        height: 320,
        scrollContext: { horizontal: false, vertical: true },
      },
    ]);

    expect(audit.overflowNodeIds).toEqual(['scroll-cross-axis-leak']);
  });
});

import {
  compareLayoutReviewOrder,
  layoutReviewPhase,
} from '../tools/zui-layout-review-order';

describe('milestone layout review order', () => {
  it('reviews foundations before consumers and keeps fixtures last', () => {
    const entries = [
      {
        sourcePath: 'zircon_runtime/assets/ui/runtime/fixtures/hud.zui',
        sourceKind: 'view',
      },
      {
        sourcePath: 'examples/woc/assets/ui/shell/menu.zui',
        sourceKind: 'view',
      },
      {
        sourcePath: 'zircon_plugins/navigation/editor/bake.zui',
        sourceKind: 'view',
      },
      {
        sourcePath:
          'zircon_editor/assets/ui/editor/components/workbench/modules/core/assets.zui',
        sourceKind: 'component',
      },
      {
        sourcePath:
          'zircon_editor/assets/ui/editor/components/workbench/shell/main.zui',
        sourceKind: 'component',
      },
      {
        sourcePath:
          'zircon_editor/assets/ui/editor/components/workbench/composites/properties.zui',
        sourceKind: 'component',
      },
      {
        sourcePath:
          'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui',
        sourceKind: 'component',
      },
    ];
    expect(
      [...entries].sort(compareLayoutReviewOrder).map(layoutReviewPhase),
    ).toEqual([0, 1, 2, 3, 4, 5, 6]);
  });

  it('does not turn product modules or fixture themes into foundations', () => {
    expect(
      layoutReviewPhase({
        sourcePath:
          'zircon_editor/src/tests/fixtures/ui_zui/theme/editor_base.zui',
        sourceKind: 'style',
      }),
    ).toBe(6);
    expect(
      layoutReviewPhase({
        sourcePath:
          'zircon_editor/assets/ui/editor/components/workbench/modules/core/data.zui',
        sourceKind: 'component',
      }),
    ).toBe(3);
    expect(
      layoutReviewPhase({
        sourcePath:
          'tools/penpot/apps/zircon-zui-plugin/src/bridge/roundtrip-fixture.zui',
        sourceKind: 'view',
      }),
    ).toBe(6);
  });
});

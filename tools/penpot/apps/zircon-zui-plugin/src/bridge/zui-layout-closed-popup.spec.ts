import type { ZuiDocument } from './zui-document';
import type { LayoutReviewCase } from '../../tools/zui-layout-review-contract';
import type { LayoutAuditSnapshot } from '../../tools/zui-layout-penpot-cases';
import { isIntentionallyClosedPopupCase } from '../../tools/zui-layout-closed-popup';

const dialog = (open: boolean): ZuiDocument => ({
  asset: { kind: 'component', id: 'res://review/dialog.zui', version: 2 },
  components: { ReviewDialog: { root: 'dialog' } },
  nodes: {
    dialog: {
      component: 'Dialog',
      props: { open, popup_open: open, title: 'Scene Settings' },
    },
  },
});
const reviewCase = (state: string): LayoutReviewCase => ({
  id: `${state}-240x520-dpi1`,
  sourcePath: 'review/dialog.zui',
  host: 'component',
  viewport: { width: 240, height: 520 },
  dpi: 1,
  locale: 'en-US',
  state,
  data: {},
});
const layout = (visible: boolean): LayoutAuditSnapshot => ({
  totalNodes: 1,
  checkedNodes: visible ? 1 : 0,
  overflowCount: 0,
  invalidGeometryCount: 0,
  overflowNodes: '',
  invalidNodes: '',
  overflowDetails: '[]',
  invalidDetails: '[]',
  semanticNodes: [
    {
      nodeId: 'dialog',
      shapeId: 'shape',
      parentNodeId: null,
      component: 'Dialog',
      visible,
      detached: false,
      bounds: { x: 0, y: 0, width: 240, height: 220 },
      text: '',
    },
  ],
});

describe('intentional closed-popup evidence', () => {
  it('accepts only a semantically hidden authored default or explicit closed dialog', () => {
    expect(
      isIntentionallyClosedPopupCase(
        dialog(false),
        reviewCase('default'),
        layout(false),
      ),
    ).toBe(true);
    expect(
      isIntentionallyClosedPopupCase(
        dialog(true),
        reviewCase('closed'),
        layout(false),
      ),
    ).toBe(true);
    expect(
      isIntentionallyClosedPopupCase(
        dialog(false),
        reviewCase('open'),
        layout(false),
      ),
    ).toBe(false);
    expect(
      isIntentionallyClosedPopupCase(
        dialog(true),
        reviewCase('default'),
        layout(false),
      ),
    ).toBe(false);
    expect(
      isIntentionallyClosedPopupCase(
        dialog(false),
        reviewCase('closed'),
        layout(true),
      ),
    ).toBe(false);
    expect(
      isIntentionallyClosedPopupCase(
        dialog(false),
        reviewCase('focus-return'),
        layout(false),
      ),
    ).toBe(true);
  });

  it('accepts a default popup when the component exposes only popup_open', () => {
    const source = dialog(false);
    delete source.nodes!['dialog'].props!.open;
    const hidden = layout(false);
    expect(
      isIntentionallyClosedPopupCase(source, reviewCase('default'), hidden),
    ).toBe(true);
  });

  it('rejects contradictory open and popup_open flags', () => {
    const source = dialog(false);
    source.nodes!['dialog'].props!.popup_open = true;
    expect(
      isIntentionallyClosedPopupCase(
        source,
        reviewCase('default'),
        layout(false),
      ),
    ).toBe(false);
  });

  it.each([
    'CommandPalette',
    'NotificationCenter',
    'DragOverlay',
    'DropdownPopup',
    'ContextMenu',
    'ContextActionMenu',
    'Popup',
    'Popover',
    'Modal',
  ])(
    'recognizes %s default as closed while requiring open content to remain visible',
    (component) => {
      const source = dialog(false);
      source.nodes!['dialog'].component = component;
      const hidden = layout(false);
      hidden.semanticNodes[0].component = component;
      expect(
        isIntentionallyClosedPopupCase(source, reviewCase('default'), hidden),
      ).toBe(true);
      expect(
        isIntentionallyClosedPopupCase(source, reviewCase('empty'), hidden),
      ).toBe(false);
    },
  );

  it('never permits hiding non-popup content or another visible sibling', () => {
    const source = dialog(false);
    source.nodes!['dialog'].component = 'Panel';
    expect(
      isIntentionallyClosedPopupCase(
        source,
        reviewCase('closed'),
        layout(false),
      ),
    ).toBe(false);
    source.nodes!['dialog'].component = 'Dialog';
    const withSibling = layout(false);
    withSibling.semanticNodes.push({
      nodeId: 'sibling',
      shapeId: 'other',
      parentNodeId: null,
      component: 'Label',
      visible: true,
      detached: false,
      bounds: { x: 0, y: 220, width: 20, height: 20 },
      text: 'Still visible',
    });
    expect(
      isIntentionallyClosedPopupCase(source, reviewCase('closed'), withSibling),
    ).toBe(false);
  });
});

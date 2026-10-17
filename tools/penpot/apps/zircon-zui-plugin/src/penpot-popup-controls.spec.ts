import {
  computePopupMenuLayout,
  computePopupLayout,
  popupMenuAnchorNodeId,
  popupMenuPanelPaint,
  popupMenuRowPaint,
  popupIsOpen,
  popupOptions,
  type PopupRect,
} from './penpot-popup-controls';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';

it('keeps authored option values and labels while preserving popup state flags', () => {
  const options = popupOptions({
    component: 'Dropdown',
    props: {
      value: 'assets',
      value_text: 'Assets',
      options: [
        'scene|label=Scene',
        'assets|label=Assets,focused,selected',
        'console|label=Console,disabled',
        '---',
      ],
    },
  });

  expect(options).toEqual([
    {
      value: 'scene',
      label: 'Scene',
      selected: false,
      disabled: false,
      focused: false,
      hovered: false,
      separator: false,
    },
    {
      value: 'assets',
      label: 'Assets',
      selected: true,
      disabled: false,
      focused: true,
      hovered: false,
      separator: false,
    },
    {
      value: 'console',
      label: 'Console',
      selected: false,
      disabled: true,
      focused: false,
      hovered: false,
      separator: false,
    },
    {
      value: '',
      label: '',
      selected: false,
      disabled: false,
      focused: false,
      hovered: false,
      separator: true,
    },
  ]);
});

it('uses value_text for the selected option when a compact value has no label', () => {
  expect(
    popupOptions({
      component: 'Dropdown',
      props: { value: 'default', value_text: 'Default', options: ['default'] },
    })[0]?.label,
  ).toBe('Default');
});

it('parses native PopupMenu action values, labels, shortcuts, and disabled items', () => {
  expect(
    popupOptions({
      component: 'PopupMenu',
      props: {
        menu_items: [
          'New Level|action=menu.item.new_scene',
          'Cut|disabled',
          'Open|action=menu.file.open|Ctrl+O',
        ],
      },
    }),
  ).toMatchObject([
    { value: 'menu.item.new_scene', label: 'New Level', disabled: false },
    { value: 'Cut', label: 'Cut', disabled: true },
    { value: 'menu.file.open', label: 'Open', disabled: false },
  ]);
});

it('paints PopupMenu rows with the ordinary Editor palette and state priority', () => {
  expect(popupMenuPanelPaint()).toEqual({
    fillColor: '#383838',
    strokeColor: '#383838',
    strokeWidth: 1,
    borderRadius: 0,
  });

  const [plain, selected, hovered, focused, danger, disabled, separator] =
    popupOptions({
      component: 'PopupMenu',
      props: {
        menu_items: [
          'Plain',
          'Selected|selected',
          'Hovered|hovered',
          'Focused|focused',
          'Danger|danger,selected,hovered',
          'Disabled|disabled,selected,hovered,danger',
          '---',
        ],
      },
    });

  expect(popupMenuRowPaint(plain!)).toEqual({
    fillColor: null,
    strokeColor: null,
    textColor: '#c0c0c0',
    borderRadius: 3,
    separatorInsetX: 0,
  });
  expect(popupMenuRowPaint(selected!)).toMatchObject({
    fillColor: '#383838',
    strokeColor: '#0070e0',
    textColor: '#c0c0c0',
    borderRadius: 3,
  });
  expect(popupMenuRowPaint(hovered!)).toMatchObject({
    fillColor: '#575757',
    strokeColor: null,
    textColor: '#c0c0c0',
  });
  expect(popupMenuRowPaint(focused!)).toMatchObject({
    fillColor: null,
    strokeColor: '#0070e0',
    textColor: '#c0c0c0',
  });
  expect(danger?.danger).toBe(true);
  expect(popupMenuRowPaint(danger!)).toMatchObject({
    fillColor: '#383838',
    strokeColor: '#0070e0',
    textColor: '#ef3535',
  });
  expect(popupMenuRowPaint(disabled!)).toMatchObject({
    fillColor: null,
    strokeColor: null,
    textColor: '#808080',
  });
  expect(popupMenuRowPaint(separator!)).toEqual({
    fillColor: '#383838',
    strokeColor: null,
    textColor: '#c0c0c0',
    borderRadius: 0,
    separatorInsetX: 8,
  });
});

it('preserves pressed menu states from flags, objects, and pressed_options', () => {
  const nodes: ZuiNode[] = [
    { component: 'PopupMenu', props: { menu_items: ['Pressed|pressed,focused,hovered'] } },
    {
      component: 'PopupMenu',
      props: { menu_items: [{ value: 'Pressed', pressed: true, focused: true }] },
    },
    {
      component: 'PopupMenu',
      props: {
        menu_items: ['Pressed'],
        pressed_options: ['Pressed'],
        focused_options: ['Pressed'],
      },
    },
  ];
  for (const node of nodes) {
    const option = popupOptions(node)[0]!;
    expect(option.pressed).toBe(true);
    expect(popupMenuRowPaint(option)).toMatchObject({
      fillColor: '#383838',
      strokeColor: null,
      textColor: '#c0c0c0',
      borderRadius: 3,
    });
    expect(popupMenuRowPaint({ ...option, selected: true })).toMatchObject({
      fillColor: '#383838',
      strokeColor: '#0070e0',
    });
    expect(
      popupMenuRowPaint({ ...option, disabled: true, danger: true }),
    ).toMatchObject({
      fillColor: null,
      strokeColor: null,
      textColor: '#808080',
    });
  }
});

it('resolves a unique authored control anchor and measures its anchored layout', () => {
  const popup: ZuiNode = {
    component: 'PopupMenu',
    widget: { popup_anchor: { kind: 'control', control_id: 'MenuSlot2' } },
  };
  const document: ZuiDocument = {
    asset: { kind: 'component', id: 'res://review/menu.zui', version: 2 },
    nodes: {
      slot: { component: 'Button', control_id: 'MenuSlot2' },
    },
  };
  expect(popupMenuAnchorNodeId(popup, document)).toBe('slot');

  document.nodes!.duplicate = {
    component: 'Button',
    control_id: 'MenuSlot2',
  };
  expect(popupMenuAnchorNodeId(popup, document)).toBeNull();
  delete document.nodes!.duplicate;
  document.nodes!.slot!.props = { disabled: true };
  expect(popupMenuAnchorNodeId(popup, document)).toBeNull();

  const options = popupOptions({
    component: 'PopupMenu',
    props: { menu_items: ['New Level|action=menu.item.new_scene', 'Cut'] },
  });
  expect(
    computePopupMenuLayout(
      { x: 22, y: 20, width: 40, height: 16 },
      { x: 0, y: 0, width: 240, height: 180 },
      options,
      { width: 128, rowHeight: 24 },
    ),
  ).toMatchObject({
    x: 22,
    y: 40,
    width: 128,
    height: 48,
    opensAbove: false,
    authority: 'source-measured-fallback',
  });
  expect(
    computePopupMenuLayout(
      { x: 22, y: 140, width: 40, height: 16 },
      { x: 0, y: 0, width: 240, height: 180 },
      options,
      { width: 128, rowHeight: 24 },
    ),
  ).toMatchObject({ x: 22, y: 88, opensAbove: true });
});

it('requires an authored open state and never treats a closed default as open', () => {
  expect(
    popupIsOpen({
      component: 'Dropdown',
      props: { popup_open: false },
      state: { popup_open: true },
    }),
  ).toBe(true);
  expect(
    popupIsOpen({ component: 'Dropdown', props: { popup_open: false } }),
  ).toBe(false);
});

it('anchors below when there is room and flips above without growing the viewport', () => {
  const trigger: PopupRect = { x: 16, y: 40, width: 160, height: 32 };
  const viewport: PopupRect = { x: 0, y: 0, width: 240, height: 180 };
  const options = popupOptions({
    component: 'Dropdown',
    props: { options: ['One', 'Two', 'Three'] },
  });

  expect(computePopupLayout(trigger, viewport, options)).toMatchObject({
    x: 16,
    y: 76,
    width: 160,
    height: 100,
    opensAbove: false,
  });

  expect(
    computePopupLayout({ ...trigger, y: 136 }, viewport, options),
  ).toMatchObject({
    x: 16,
    y: 32,
    width: 160,
    height: 100,
    opensAbove: true,
  });
});

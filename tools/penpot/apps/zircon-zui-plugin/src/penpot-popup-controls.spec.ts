import {
  computePopupLayout,
  popupIsOpen,
  popupOptions,
  type PopupRect,
} from './penpot-popup-controls';

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

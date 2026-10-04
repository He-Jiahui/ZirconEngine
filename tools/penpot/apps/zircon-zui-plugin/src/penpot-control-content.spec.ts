import type { Board } from '@penpot/plugin-types';
import type { ProjectionText } from './bridge/penpot-projection-model';
import { createControlContent } from './penpot-control-content';

it('does not invent product pixels for an unfilled slot', () => {
  const children: Board['children'] = [];
  expect(
    createControlContent({ children } as Board, {
      component: 'Slot',
      props: { name: 'value' },
    }),
  ).toBe(false);
  expect(children).toHaveLength(0);
});

it('uses the editable source text instead of painting a second NumberField value', () => {
  const children: Board['children'] = [];
  const board = {
    children,
    width: 100,
    height: 32,
    x: 0,
    y: 0,
    appendChild(child: Board['children'][number]) {
      children.push(child);
    },
  } as unknown as Board;
  const text = {
    characters: '42',
    property: 'value_text',
  } as ProjectionText;
  const font = {
    fontFamily: 'Fira Sans',
    variants: [{ fontWeight: '400', fontStyle: 'normal' }],
    applyToText: () => undefined,
  };
  vi.stubGlobal('penpot', {
    fonts: { all: [font] },
    createText: (characters: string) => {
      const metadata = new Map<string, string>();
      return {
        characters,
        width: 1,
        height: 1,
        setSharedPluginData: (_namespace: string, key: string, value: string) =>
          metadata.set(key, value),
        getSharedPluginData: (_namespace: string, key: string) =>
          metadata.get(key) ?? '',
        resize(width: number, height: number) {
          this.width = width;
          this.height = height;
        },
      };
    },
  });
  try {
    expect(
      createControlContent(
        board,
        { component: 'NumberField', props: { value: 42, value_text: '42' } },
        undefined,
        text,
      ),
    ).toBe(false);
    expect(children).toHaveLength(0);
  } finally {
    vi.unstubAllGlobals();
  }
});

import { describe, expect, it } from 'vitest';
import {
  measureNativeLeafContent,
  measureNativeMaterialContent,
} from './penpot-content-material';
import type { ZuiDocument } from './bridge/zui-document';
const document: ZuiDocument = {
  asset: { kind: 'view', id: 'test', version: 2 },
};
describe('native material desired size', () => {
  it('uses authored skin padding, icon spacing and slots before minima', () => {
    const node = {
      component: 'Button',
      props: {
        icon: 'res://x.svg',
        layout_padding_left: 4,
        layout_padding_right: 6,
        layout_padding_top: 2,
        layout_padding_bottom: 2,
        layout_icon_size: 16,
        layout_spacing: 8,
        layout_leading_slot_width: 10,
        layout_trailing_slot_width: 14,
        layout_min_width: 80,
        layout_min_height: 24,
      },
    };
    expect(
      measureNativeLeafContent(document, node, { width: 50, height: 18 }),
    ).toEqual({ width: 108, height: 24 });
    expect(
      measureNativeLeafContent(document, node, { width: 0, height: 0 }),
    ).toEqual({ width: 80, height: 24 });
  });
  it('resolves design tokens and does not add the fallback button padding twice', () => {
    const doc = { ...document, tokens: { pad: 12 } };
    expect(
      measureNativeLeafContent(
        doc,
        { component: 'Button', props: { layout_padding_left: '$pad' } },
        { width: 30, height: 18 },
      ),
    ).toEqual({ width: 42, height: 18 });
    expect(
      measureNativeLeafContent(
        document,
        { component: 'Button' },
        { width: 30, height: 18 },
      ),
    ).toEqual({ width: 48, height: 26 });
  });
  it('keeps native unsupported painter aliases and empty buttons without inferred geometry', () => {
    expect(
      measureNativeMaterialContent(
        document,
        { component: 'Table', props: { layout_min_width: 100 } },
        { width: 0, height: 0 },
      ),
    ).toBeUndefined();
    expect(
      measureNativeLeafContent(
        document,
        { component: 'Button' },
        { width: 0, height: 0 },
      ),
    ).toEqual({ width: 0, height: 0 });
    expect(
      measureNativeLeafContent(
        document,
        { component: 'IconButton', props: { layout_icon_size: 16 } },
        { width: 0, height: 0 },
      ),
    ).toEqual({ width: 16, height: 16 });
  });
});

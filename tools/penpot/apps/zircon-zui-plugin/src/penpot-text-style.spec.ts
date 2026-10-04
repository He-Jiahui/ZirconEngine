import type { Text } from '@penpot/plugin-types';
import {
  applyPenpotTextRuns,
  applyPenpotTextStyle,
  resolvePenpotTextStyle,
} from './penpot-text-style';
import type { ZuiDocument } from './bridge/zui-document';

const document: ZuiDocument = {
  asset: { id: 'res://ui/test.zui', kind: 'view', version: 2 },
  tokens: {
    'editor.typography.ui.family': 'Fira Sans',
    'editor.typography.code.family': 'Fira Mono',
    'editor.typography.body.size': 14,
    'editor.typography.line_height': 1.4,
    caption: 12,
  },
};

describe('source-owned Penpot typography', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('selects the exact family when condensed fonts occur first in Penpot', () => {
    const variant = {
      fontWeight: '400',
      fontStyle: 'normal',
      fontVariantId: 'regular',
    };
    const condensed = {
      fontFamily: 'Fira Sans Extra Condensed',
      variants: [variant],
      applyToText: vi.fn(),
    };
    const regular = {
      fontFamily: 'Fira Sans',
      variants: [variant],
      applyToText: vi.fn(),
    };
    vi.stubGlobal('penpot', { fonts: { all: [condensed, regular] } });
    const text = {} as Text;
    applyPenpotTextStyle(
      text,
      resolvePenpotTextStyle(document, { component: 'Label' }),
    );
    expect(regular.applyToText).toHaveBeenCalledWith(text, variant);
    expect(condensed.applyToText).not.toHaveBeenCalled();
    expect(text.fontSize).toBe('14');
  });

  it('applies strong ranges inside one text shape', () => {
    const regular = {
      fontWeight: '400',
      fontStyle: 'normal',
      fontVariantId: 'regular',
    };
    const bold = {
      fontWeight: '700',
      fontStyle: 'normal',
      fontVariantId: 'bold',
    };
    const applyToRange = vi.fn();
    const font = {
      fontFamily: 'Fira Sans',
      variants: [regular, bold],
      applyToText: vi.fn(),
      applyToRange,
    };
    vi.stubGlobal('penpot', { fonts: { all: [font] } });
    const range = { characters: 'rich-text' };
    const text = {
      name: 'one text shape',
      characters: 'Verify rich-text runs',
      getRange: vi.fn(() => range),
    } as unknown as Text;

    applyPenpotTextRuns(
      text,
      {
        family: 'Fira Sans',
        id: 'firasans',
        size: 14,
        weight: '400',
        lineHeight: 1.4,
      },
      [{ start: 7, end: 17, fontWeight: '700' }],
    );

    expect(text.getRange).toHaveBeenCalledWith(7, 17);
    expect(applyToRange).toHaveBeenCalledWith(range, bold);
  });

  it('chooses the nearest available normal variant when bold is not shipped', () => {
    const applyToRange = vi.fn();
    const medium = {
      fontWeight: '600',
      fontStyle: 'normal',
      fontVariantId: 'semibold',
    };
    const font = {
      fontFamily: 'Fira Sans',
      variants: [medium],
      applyToText: vi.fn(),
      applyToRange,
    };
    vi.stubGlobal('penpot', { fonts: { all: [font] } });
    const text = {
      name: 'fallback text',
      characters: 'strong',
      getRange: vi.fn(() => ({})),
    } as unknown as Text;

    applyPenpotTextRuns(
      text,
      {
        family: 'Fira Sans',
        id: 'firasans',
        size: 14,
        weight: '400',
        lineHeight: 1.4,
      },
      [{ start: 0, end: 6, fontWeight: '700' }],
    );

    expect(applyToRange).toHaveBeenCalledWith({}, medium);
  });

  it('rejects absent families and unsupported weights instead of substituting fonts', () => {
    const style = resolvePenpotTextStyle(document, { component: 'Label' });
    vi.stubGlobal('penpot', { fonts: { all: [] } });
    expect(() => applyPenpotTextStyle({} as Text, style)).toThrow(
      'exact Penpot font family',
    );
    vi.stubGlobal('penpot', {
      fonts: { all: [{ fontFamily: 'Fira Sans', variants: [] }] },
    });
    expect(() => applyPenpotTextStyle({} as Text, style)).toThrow(
      'no normal weight 400',
    );
  });

  it('uses the product font, density and line height', () => {
    expect(resolvePenpotTextStyle(document, { component: 'Label' })).toEqual({
      family: 'Fira Sans',
      id: 'firasans',
      size: 14,
      weight: '400',
      lineHeight: 1.4,
    });
  });
  it('keeps code and authored typography distinct', () => {
    expect(
      resolvePenpotTextStyle(document, {
        component: 'Label',
        props: {
          component_variant: 'code',
          font_size: '$caption',
          line_height_ratio: 1.5,
        },
      }),
    ).toMatchObject({ family: 'Fira Mono', size: 12, lineHeight: 1.5 });
    expect(
      resolvePenpotTextStyle(document, {
        component: 'Label',
        props: {
          font_family: 'Noto Sans SC',
          font_size: 18,
        },
      }),
    ).toMatchObject({ id: 'notosanssc', size: 18 });
  });
  it('rejects recursive font aliases', () => {
    expect(() =>
      resolvePenpotTextStyle(
        { ...document, tokens: { a: '$b', b: '$a' } },
        { component: 'Label', props: { font_family: '$a' } },
      ),
    ).toThrow('Cyclic font token');
  });

  it('uses the nested font family and absolute line height before flat aliases', () => {
    expect(
      resolvePenpotTextStyle(document, {
        component: 'Label',
        props: {
          font: { family: 'Fira Mono', size: 16, weight: 500, line_height: 28 },
          font_family: 'Fira Sans',
          font_size: 12,
          font_weight: 600,
          line_height: 24,
          line_height_ratio: 1.2,
        },
      }),
    ).toMatchObject({
      family: 'Fira Mono',
      size: 16,
      weight: '500',
      lineHeight: 1.75,
    });
  });

  it.each([
    { font_family: '$missing.family' },
    { font_weight: '$editor.typography.strong.weight' },
    { font_size: 'huge' },
    { line_height: 0 },
  ])('rejects unresolvable typography %j', (props) => {
    expect(() =>
      resolvePenpotTextStyle(document, { component: 'Label', props }),
    ).toThrow();
  });
});

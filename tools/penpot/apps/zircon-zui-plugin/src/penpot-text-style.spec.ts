import type { Shape } from '@penpot/plugin-types';
import { bindFontResources,copyFontResources,fontResourcesForDocument,fontResourcesForShape,writeFontResources,parseFontResourceManifest,selectRegisteredFont } from './bridge/penpot-font-selection';
import { encodeZuiMetadata,decodeZuiMetadata } from './bridge/zui-document';
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
  ])('rejects unresolvable typography %j', (props: Record<string, string | number>) => {
    expect(() =>
      resolvePenpotTextStyle(document, { component: 'Label', props }),
    ).toThrow();
  });
});


describe('explicit engine font resource selections', () => {
  afterEach(() => vi.unstubAllGlobals());
  const receipt = (weight = '400', variantId = 'official-regular') => ({schema:'zircon.penpot.font-resources.v1',selections:[{
    source:{family:'Fira Sans',weight,style:'normal'},
    engine:{assetPath:'res://fonts/engine.ttc',assetSha256:'a'.repeat(64),faceIndex:7,sfntSha256:'b'.repeat(64)},
    penpot:{fontId:'official-engine-font',variantId,sfntSha256:'b'.repeat(64)},
  }]});
  const setup = () => {
    const regular = {fontVariantId:'official-regular',fontWeight:'400',fontStyle:'normal'};
    const bold = {fontVariantId:'official-bold',fontWeight:'700',fontStyle:'normal'};
    const engine = {fontId:'official-engine-font',fontFamily:'Fira Sans',variants:[regular,bold],applyToText:vi.fn(),applyToRange:vi.fn()};
    const google = {fontId:'google-family',fontFamily:'Fira Sans',variants:[regular],applyToText:vi.fn(),applyToRange:vi.fn()};
    vi.stubGlobal('penpot',{fonts:{all:[google,engine]}});
    return {engine,google,regular,bold};
  };
  it('keeps ambiguity refusal without a receipt and selects the verified exact IDs with it', () => {
    const {engine,google,regular} = setup();
    const original = {...document};
    const untouched = {...document};
    const style = resolvePenpotTextStyle(original,{component:'Label'});
    expect(() => applyPenpotTextStyle({} as Text,style)).toThrow('found 2');
    const manifest = parseFontResourceManifest(receipt());
    bindFontResources(original,manifest);
    const selected = resolvePenpotTextStyle(original,{component:'Label'});
    applyPenpotTextStyle({} as Text,selected);
    expect(engine.applyToText).toHaveBeenCalledWith(expect.anything(),regular);
    expect(google.applyToText).not.toHaveBeenCalled();
    expect(() => applyPenpotTextStyle({} as Text,resolvePenpotTextStyle(untouched,{component:'Label'}))).toThrow('found 2');
    expect(JSON.stringify(original)).toBe(JSON.stringify(untouched));
    expect(encodeZuiMetadata(original)).toBe(encodeZuiMetadata(untouched));
    expect(fontResourcesForDocument({...original})).toBeUndefined();
    const clone = {...original};
    copyFontResources(original,clone);
    expect(fontResourcesForDocument(clone)).toBe(manifest);
  });
  it('uses the exact mapped inline variant instead of a nearest weight', () => {
    const {engine,bold} = setup();
    const raw = receipt();
    raw.selections.push(receipt('700','official-bold').selections[0]);
    const mapped = {...document};
    bindFontResources(mapped,parseFontResourceManifest(raw));
    const range = {};
    const text = {characters:'strong',getRange:vi.fn(() => range)} as unknown as Text;
    const style = resolvePenpotTextStyle(mapped,{component:'Label'});
    applyPenpotTextRuns(text,style,[{start:0,end:6,fontWeight:'700'}]);
    expect(engine.applyToRange).toHaveBeenCalledWith(range,bold);
    expect(() => applyPenpotTextRuns(text,style,[{start:0,end:6,fontWeight:'600'}])).toThrow('explicit font resource selection');
  });
  it('restores separate board receipts for document refresh without changing source metadata', () => {
    setup();
    const manifest = parseFontResourceManifest(receipt());
    const stored = new Map<string,string>();
    const board = {parent:null,setSharedPluginData:(_ns:string,key:string,value:string) => stored.set(key,value),getSharedPluginData:(_ns:string,key:string) => stored.get(key) ?? ''} as unknown as Shape;
    writeFontResources(board,manifest);
    const refreshSource: ZuiDocument = {
      ...document,
      root: { node: 'receipt-root' },
      nodes: { 'receipt-root': { component: 'Label' } },
    };
    const refreshed = decodeZuiMetadata(encodeZuiMetadata(refreshSource));
    bindFontResources(refreshed,fontResourcesForShape(board));
    expect(fontResourcesForDocument(refreshed)).toEqual(manifest);
    expect(encodeZuiMetadata(refreshed)).toBe(encodeZuiMetadata(refreshSource));
  });
  it.each(['missing-id','wrong-family','duplicate-id','missing-variant','wrong-weight','wrong-style','duplicate-variant'])('rejects stale or inconsistent registry mapping: %s', (failure: string) => {
    const {engine,google,regular} = setup();
    if (failure==='missing-id') engine.fontId='wrong-id';
    if (failure==='wrong-family') engine.fontFamily='Other family';
    if (failure==='duplicate-id') google.fontId=engine.fontId;
    if (failure==='missing-variant') regular.fontVariantId='stale';
    if (failure==='wrong-weight') regular.fontWeight='500';
    if (failure==='wrong-style') regular.fontStyle='italic';
    if (failure==='duplicate-variant') engine.variants.push({...regular});
    expect(() => selectRegisteredFont('Fira Sans','400','normal',parseFontResourceManifest(receipt()))).toThrow();
    expect(engine.applyToText).not.toHaveBeenCalled();
  });
  it('rejects duplicate source tuples and unequal SFNT receipts while preserving valid provenance', () => {
    const raw = receipt();
    const manifest = parseFontResourceManifest(raw);
    expect(manifest.selections[0]).toEqual(raw.selections[0]);
    raw.selections.push(raw.selections[0]);
    expect(() => parseFontResourceManifest(raw)).toThrow('Duplicate');
    const mismatch = receipt();
    mismatch.selections[0].penpot.sfntSha256='c'.repeat(64);
    expect(() => parseFontResourceManifest(mismatch)).toThrow('SFNT hash');
  });
});

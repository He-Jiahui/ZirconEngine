import type { Board, Shape, Text } from '@penpot/plugin-types';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { layoutTreeRowText } from './penpot-tree-row-controls';
import { captureNodeTexts, unsupportedTextStyle } from './penpot-text-capture';
import { fitNativeSingleLineTexts, semanticTextCharacters } from './penpot-text-overflow';

function treeRowFixture(value: string) {
  const shared = (initial: Record<string, string> = {}) => ({
    getSharedPluginData: (namespace: string, key: string) =>
      namespace === ZUI_METADATA_NAMESPACE ? initial[key] ?? '' : '',
    setSharedPluginData: (namespace: string, key: string, entry: string) => {
      if (namespace === ZUI_METADATA_NAMESPACE) initial[key] = entry;
    },
  });
  const asset = { id: 'asset', type: 'board', children: [] as Shape[],
    ...shared(), appendChild(child: Shape) { this.children.push(child); Object.assign(child, { parent: this }); },
  } as unknown as Board;
  const row = { id: 'linked-row', type: 'board', x: 30, y: 50, width: 240, height: 28,
    children: [] as Shape[], parent: asset,
    ...shared({ role: 'node', 'node-id': 'row', sourcePath: 'res://tree_row.zui',
      sourceNodeId: 'root', instancePath: 'consumer/row', controlId: 'TreeRowRoot' }),
  } as unknown as Board;
  asset.appendChild(row);
  const text = { id: 'linked-label', type: 'text', name: 'TreeRow caption',
    characters: value, x: 0, y: 0, width: 1, height: 1, hidden: false,
    fontId: 'firasans', fontFamily: 'Fira Sans', fontVariantId: 'regular', fontWeight: '400',
    fontSize: '14', lineHeight: '1.2', fontStyle: 'normal', verticalAlign: 'center',
    growType: 'fixed', letterSpacing: '0', align: 'left', fills: [{ fillColor: '#b3b3b3' }], strokes: [],
    parent: row, layoutChild: { absolute: true },
    ...shared({ role: 'text', 'node-id': 'row', 'text-property': 'text',
      'text-base-weight': '400', 'empty-text': 'false' }),
    resize(width: number, height: number) { this.width = width; this.height = height; },
  } as unknown as Text;
  text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-style-guard', JSON.stringify(unsupportedTextStyle(text)));
  (row.children as Shape[]).push(text);
  const node: ZuiNode = { component: 'TreeRow', props: { text: value, text_overflow: '$overflow' } };
  const document: ZuiDocument = { asset: { kind: 'view', id: 'consumer.zui', version: 2 },
    nodes: { row: node }, tokens: { overflow: 'ellipsis' }, penpot_host_theme_source: 'editor_tokens.zui' };
  const probes: Text[] = [];
  const createText = vi.fn((characters: string) => {
    const probe = { characters, type: 'text', ...shared(),
      get width() { return Array.from(new Intl.Segmenter().segment(this.characters)).length * 10; },
      get height() { return 17; },
      remove() { const index = asset.children.indexOf(this as unknown as Text);
        if (index >= 0) (asset.children as Shape[]).splice(index, 1); },
    } as unknown as Text;
    probes.push(probe);
    return probe;
  });
  vi.stubGlobal('penpot', { createText, fonts: { all: [{ fontFamily: 'Fira Sans',
    variants: [{ id: 'regular', fontWeight: '400', fontStyle: 'normal' }],
    applyToText(target: Text) { Object.assign(target, { fontId: 'firasans', fontFamily: 'Fira Sans',
      fontVariantId: 'regular', fontWeight: '400' }); },
  }] } });
  return { asset, row, text, node, document, createText, probes,
    layout: () => layoutTreeRowText(row, text, node, document) };
}

describe('TreeRow source-owned single-line captions', () => {
  afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); });

  it.each(['发射器／sprite_emitter', 'Emitter / sprite_emitter_with_a_long_name'])(
    'fits %s in the native label lane and captures the complete value', async (value) => {
      vi.useFakeTimers();
      const f = treeRowFixture(value), source = structuredClone(f.document);
      const owner = [f.row.x, f.row.y, f.row.width, f.row.height];
      const identity = ['sourcePath', 'sourceNodeId', 'instancePath', 'controlId']
        .map((key) => f.row.getSharedPluginData(ZUI_METADATA_NAMESPACE, key));
      expect(f.layout()).toBe(true);
      const lane = [f.text.x, f.text.y, f.text.width, f.text.height];
      const fitting = fitNativeSingleLineTexts(f.asset);
      await vi.runAllTimersAsync();
      await fitting;
      expect(f.text.characters).toMatch(/…$/u);
      expect(f.text.characters).not.toMatch(/[\r\n]/u);
      expect(semanticTextCharacters(f.text)).toBe(value);
      expect(captureNodeTexts(f.row).text?.characters).toBe(value);
      expect(f.text.growType).toBe('fixed');
      expect([f.text.x, f.text.y, f.text.width, f.text.height]).toEqual(lane);
      expect([f.row.x, f.row.y, f.row.width, f.row.height]).toEqual(owner);
      expect(['sourcePath', 'sourceNodeId', 'instancePath', 'controlId']
        .map((key) => f.row.getSharedPluginData(ZUI_METADATA_NAMESPACE, key))).toEqual(identity);
      expect([f.row.id, f.text.id]).toEqual(['linked-row', 'linked-label']);
      expect(f.asset.children).toEqual([f.row]);
      expect(f.document).toEqual(source);
      f.layout();
      expect(captureNodeTexts(f.row).text?.characters).toBe(value);
    });

  it('restores the complete value when source state disables ellipsis, then accepts a full override', async () => {
    vi.useFakeTimers();
    const value = 'Particle emitter / source text', f = treeRowFixture(value);
    f.layout();
    const initial = fitNativeSingleLineTexts(f.asset);
    await vi.runAllTimersAsync();
    await initial;
    f.node.state = { text_overflow: 'clip' };
    f.layout();
    expect(f.text.characters).toBe(value);
    expect(f.text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-display')).toBe('');
    expect(f.text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-overflow')).toBe('');
    f.node.state = { text_overflow: 'ellipsis' };
    f.text.characters = 'Camera';
    f.layout();
    const overridden = fitNativeSingleLineTexts(f.asset);
    await vi.runAllTimersAsync();
    await overridden;
    expect(f.text.characters).toBe('Camera');
    expect(captureNodeTexts(f.row).text?.characters).toBe('Camera');
    expect(f.node.props!['text']).toBe(value);
  });

  it('rejects multiline source and ambiguous abbreviated edits instead of exporting a lost suffix', async () => {
    const f = treeRowFixture('Emitter\nNested');
    f.layout();
    await expect(fitNativeSingleLineTexts(f.asset)).rejects.toThrow('Multiline');
    expect(f.createText).not.toHaveBeenCalled();
    f.text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-source', 'Emitter / sprite_emitter');
    f.text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-display', 'Emitter…');
    f.text.characters = 'Edited…';
    f.node.state = { text_overflow: 'clip' };
    expect(f.layout).toThrow('complete value');
    expect(f.text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-source')).toBe('Emitter / sprite_emitter');
    expect(f.text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-overflow')).toBe('ellipsis');
  });

  it('leaves unrelated native text and rows without the host source outside this mapping', () => {
    const f = treeRowFixture('Caption');
    f.node.component = 'Label';
    expect(f.layout()).toBe(false);
    expect(f.text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-overflow')).toBe('');
    f.node.component = 'TreeRow';
    delete f.document['penpot_host_theme_source'];
    expect(f.layout()).toBe(false);
  });

  it('rejects unknown overflow metadata and unresolved source tokens before rewriting the row', () => {
    const f = treeRowFixture('Caption'), geometry = [f.text.x, f.text.y, f.text.width, f.text.height];
    f.text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-overflow', 'unknown');
    expect(f.layout).toThrow('Unknown native single-line overflow mapping');
    expect(f.text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-overflow')).toBe('unknown');
    f.text.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'single-line-overflow', '');
    delete f.document.tokens!['overflow'];
    expect(f.layout).toThrow('Unresolved TreeRow text overflow token');
    expect([f.text.x, f.text.y, f.text.width, f.text.height]).toEqual(geometry);
    expect(f.createText).not.toHaveBeenCalled();
  });
});

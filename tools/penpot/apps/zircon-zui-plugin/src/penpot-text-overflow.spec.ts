import type { Text } from '@penpot/plugin-types';
import { EMPTY_TEXT_SENTINEL } from './penpot-capture-validation';
import {
  cachedTextMeasure,
  measuredEllipsis,
  semanticTextCharacters,
} from './penpot-text-overflow';

describe('native single-line overflow', () => {
  it('keeps the zero-width empty-field sentinel without probing a visible glyph', async () => {
    const measure = vi.fn(async () => {
      throw new Error('should not measure an empty field');
    });
    expect(await measuredEllipsis(EMPTY_TEXT_SENTINEL, 1, measure)).toBe(
      EMPTY_TEXT_SENTINEL,
    );
    expect(measure).not.toHaveBeenCalled();
  });

  it('retains fitting content and measures the complete candidate with its marker', async () => {
    const measure = async (value: string) => Array.from(value).length * 10;
    expect(await measuredEllipsis('Name', 40, measure)).toBe('Name');
    expect(await measuredEllipsis('Character', 50, measure)).toBe('Char\u2026');
    await expect(measuredEllipsis('Long', 5, measure)).rejects.toThrow(
      'ellipsis marker',
    );
  });

  it('never separates a combining mark from its grapheme', async () => {
    const measure = async (value: string) =>
      Array.from(new Intl.Segmenter().segment(value)).length * 10;
    expect(await measuredEllipsis('e\u0301abcdef', 30, measure)).toBe(
      'e\u0301a\u2026',
    );
  });

  it('reuses a native measurement only for the same text style and candidate', async () => {
    const cache = new Map<string, number>();
    let calls = 0;
    const measure = async (value: string) => {
      calls += 1;
      return value.length * 10;
    };
    const regular = cachedTextMeasure(cache, 'Fira Sans/14/400', measure);

    expect(await regular('Character')).toBe(90);
    expect(await regular('Character')).toBe(90);
    expect(calls).toBe(1);

    const bold = cachedTextMeasure(cache, 'Fira Sans/14/700', measure);
    expect(await bold('Character')).toBe(90);
    expect(calls).toBe(2);
  });

  it('preserves hidden suffixes and rejects ambiguous edits of abbreviated text', () => {
    const shape = {
      characters: 'Char\u2026',
      getSharedPluginData: (_namespace: string, key: string) =>
        key === 'single-line-source' ? 'CharacterController' : 'Char\u2026',
    } as Text;
    expect(semanticTextCharacters(shape)).toBe('CharacterController');
    shape.characters = 'Cbar\u2026';
    expect(() => semanticTextCharacters(shape)).toThrow('complete value');
    shape.characters = 'CameraController';
    expect(semanticTextCharacters(shape)).toBe('CameraController');
  });
});

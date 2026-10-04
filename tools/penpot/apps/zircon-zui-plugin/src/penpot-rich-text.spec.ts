import { MARKDOWN_INLINE_V1, parseProjectionText } from './penpot-rich-text';

describe('same-shape inline text projection', () => {
  it('removes strong markers while keeping one contiguous string and ranges', () => {
    expect(
      parseProjectionText(
        'Verify **rich-text** runs and **review** them',
        MARKDOWN_INLINE_V1,
      ),
    ).toEqual({
      characters: 'Verify rich-text runs and review them',
      runs: [
        { start: 7, end: 16, fontWeight: '700' },
        { start: 26, end: 32, fontWeight: '700' },
      ],
    });
  });

  it('uses UTF-16 offsets so Penpot getRange receives JavaScript string indices', () => {
    expect(parseProjectionText('看 **视图**', MARKDOWN_INLINE_V1)).toEqual({
      characters: '看 视图',
      runs: [{ start: 2, end: 4, fontWeight: '700' }],
    });
  });

  it('preserves unsupported formats and malformed markers verbatim', () => {
    expect(parseProjectionText('A **bold** B', 'unsupported')).toEqual({
      characters: 'A **bold** B',
      runs: [],
    });
    expect(parseProjectionText('A **unfinished', MARKDOWN_INLINE_V1)).toEqual({
      characters: 'A **unfinished',
      runs: [],
    });
    expect(parseProjectionText('A **** B', MARKDOWN_INLINE_V1)).toEqual({
      characters: 'A **** B',
      runs: [],
    });
    expect(
      parseProjectionText(String.raw`A \**literal**`, MARKDOWN_INLINE_V1),
    ).toEqual({
      characters: String.raw`A \**literal**`,
      runs: [],
    });
  });
});

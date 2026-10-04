import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import { parseZuiDocument, zuiNodes } from './zui-document';
import { serializeZuiDocument } from './zui-document';
import { applyZuiExportToCanonicalSource } from './zui-export-apply';
import { isSupportedVisualPath } from './zui-export-apply-contract';

function fixture(kind: string, fields = 'gap = 8') {
  return parseZuiDocument(`[asset]
kind = "component"
id = "res://review/container-gap.zui"
version = 2

[components.GapReview]
root = "root"

[nodes.root]
component = "${kind}"
layout = { container = { kind = "${kind}"${fields ? `, ${fields}` : ''} }, width = { preferred = 320 }, height = { preferred = 240 } }
children = [{ node = "first" }, { node = "second" }]

[nodes.first]
component = "Label"
props = { text = "First" }

[nodes.second]
component = "Label"
props = { text = "Second" }
`).document;
}

describe('Penpot gaps follow the engine container contract', () => {
  it.each(['Masonry', 'MasonryBox'])(
    'uses the native scalar zero-gap default for %s',
    (kind) => {
      expect(
        projectZuiDocument(fixture(kind, '')).rootNodes[0].container,
      ).toMatchObject({
        rowGap: 0,
        columnGap: 0,
      });
    },
  );

  it('does not expose grid-only gap overrides on native MasonryBox', () => {
    const document = fixture(
      'MasonryBox',
      'gap = 8, row_gap = 10, column_gap = 6',
    );
    expect(projectZuiDocument(document).rootNodes[0].container).toMatchObject({
      rowGap: 8,
      columnGap: 8,
    });
  });

  it('projects the native MasonryBox spacing alias', () => {
    expect(
      projectZuiDocument(fixture('MasonryBox', 'spacing = 6')).rootNodes[0]
        .container,
    ).toMatchObject({
      rowGap: 6,
      columnGap: 6,
    });
  });

  it('rejects independent native MasonryBox axis gap edits', () => {
    const document = fixture('MasonryBox');
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.container.rowGap = 9;
    expect(() => reconcileZuiDocument(document, snapshot)).toThrow(
      /scalar.*gap/i,
    );
  });

  it('exports matching MasonryBox axis edits as its native scalar gap', () => {
    const document = fixture('MasonryBox');
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    Object.assign(snapshot.shapes[0].current.container, {
      rowGap: 9,
      columnGap: 9,
    });
    const result = reconcileZuiDocument(document, snapshot);
    expect(zuiNodes(result.document)['root'].layout!['container']).toEqual({
      kind: 'MasonryBox',
      gap: 9,
    });
    expect(result.changes).toEqual(['nodes.root.layout.container.gap']);
  });

  it.each(['\n', '\r\n'])(
    'applies a new axis field to a dotted TOML table ending without %j',
    (lineEnding) => {
      const canonical = [
        '[asset]',
        'kind = "component"',
        'id = "res://review/no-newline.zui"',
        'version = 2',
        '',
        '[components.Review]',
        'root = "root"',
        '',
        '[nodes.root]',
        'component = "FlowBox"',
        '',
        '[nodes.root.layout.container]',
        'kind = "FlowBox"',
        'gap = 8 # retained comment',
      ].join(lineEnding);
      const document = parseZuiDocument(canonical).document;
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      snapshot.shapes[0].current.container.rowGap = 9;
      const exported = serializeZuiDocument(
        reconcileZuiDocument(document, snapshot).document,
      );
      const applied = applyZuiExportToCanonicalSource(exported, canonical, {
        expectedCanonicalSha256: createHash('sha256')
          .update(canonical)
          .digest('hex'),
      });
      expect(applied.source).toContain(
        `gap = 8 # retained comment${lineEnding}vertical_gap = 9`,
      );
      expect(
        zuiNodes(parseZuiDocument(applied.source).document)['root'].layout![
          'container'
        ],
      ).toEqual({
        kind: 'FlowBox',
        gap: 8,
        vertical_gap: 9,
      });
    },
  );

  it.each(['VerticalBox', 'HorizontalBox', 'GridBox', 'FlowBox', 'WrapBox'])(
    'uses the native zero-gap default for %s',
    (kind) => {
      expect(
        projectZuiDocument(fixture(kind, '')).rootNodes[0].container,
      ).toMatchObject({ rowGap: 0, columnGap: 0 });
    },
  );

  it('does not use shared gap for WrapBox, whose native parser only reads axis fields', () => {
    expect(
      projectZuiDocument(fixture('WrapBox', 'gap = 8')).rootNodes[0].container,
    ).toMatchObject({ rowGap: 0, columnGap: 0 });
  });
  it.each([
    ['VerticalBox', 'rowGap'],
    ['HorizontalBox', 'columnGap'],
  ] as const)('exports a normal single-axis %s edit as gap', (kind, active) => {
    const document = fixture(kind);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.container[active] = 9;
    const result = reconcileZuiDocument(document, snapshot);
    expect(zuiNodes(result.document)['root'].layout!['container']).toEqual({
      kind,
      gap: 9,
    });
    expect(result.changes).toEqual(['nodes.root.layout.container.gap']);
    expect(
      projectZuiDocument(result.document).rootNodes[0].container[active],
    ).toBe(9);
  });

  it.each([
    ['VerticalBox', 'columnGap'],
    ['HorizontalBox', 'rowGap'],
  ] as const)(
    'rejects an unsupported isolated cross-axis %s edit',
    (kind, inactive) => {
      const document = fixture(kind);
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      snapshot.shapes[0].current.container[inactive] = 9;
      expect(() => reconcileZuiDocument(document, snapshot)).toThrow(
        /inactive.*gap/i,
      );
    },
  );

  it.each(['VerticalBox', 'HorizontalBox'])(
    'preserves a scalar-only %s edit',
    (kind) => {
      const document = fixture(kind);
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      snapshot.shapes[0].current.container.gap = 9;
      expect(
        zuiNodes(reconcileZuiDocument(document, snapshot).document)['root']
          .layout!['container'],
      ).toEqual({ kind, gap: 9 });
    },
  );

  it.each(['VerticalBox', 'HorizontalBox'])(
    'does not project grid axis keys for %s',
    (kind) => {
      const document = fixture(kind, 'gap = 8, row_gap = 99, column_gap = 77');
      const projection = projectZuiDocument(document);
      expect(projection.rootNodes[0].container).toMatchObject({
        rowGap: 8,
        columnGap: 8,
      });
      expect(
        reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
          .document,
      ).toEqual(document);
    },
  );

  it.each(['FlowBox', 'WrapBox'])(
    'projects engine horizontal and vertical %s gaps',
    (kind) => {
      const document = fixture(kind, 'horizontal_gap = 6, vertical_gap = 10');
      const projection = projectZuiDocument(document);
      expect(projection.rootNodes[0].container).toMatchObject({
        wrap: true,
        columnGap: 6,
        rowGap: 10,
      });
      expect(
        reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
          .document,
      ).toEqual(document);
    },
  );

  it.each(['FlowBox', 'WrapBox'])(
    'exports a %s row-gap edit with the engine vertical key',
    (kind) => {
      const document = fixture(kind, 'horizontal_gap = 6, vertical_gap = 10');
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      snapshot.shapes[0].current.container.rowGap = 12;
      const result = reconcileZuiDocument(document, snapshot);
      expect(zuiNodes(result.document)['root'].layout!['container']).toEqual({
        kind,
        horizontal_gap: 6,
        vertical_gap: 12,
      });
      expect(result.changes).toEqual([
        'nodes.root.layout.container.vertical_gap',
      ]);
      const canonical = serializeZuiDocument(document);
      const applied = applyZuiExportToCanonicalSource(
        serializeZuiDocument(result.document),
        canonical,
        {
          expectedCanonicalSha256: createHash('sha256')
            .update(canonical)
            .digest('hex'),
        },
      );
      expect(applied.appliedPaths).toEqual([
        'nodes.root.layout.container.vertical_gap',
      ]);
      expect(
        isSupportedVisualPath([
          'nodes',
          'root',
          'layout',
          'container',
          'vertical_gap',
        ]),
      ).toBe(true);
      expect(
        isSupportedVisualPath([
          'nodes',
          'root',
          'layout',
          'container',
          'horizontal_gap',
        ]),
      ).toBe(true);
    },
  );

  it('exports a grid row-gap edit without modifying its column gap', () => {
    const document = fixture(
      'GridBox',
      'columns = 2, rows = 1, column_gap = 6, row_gap = 10',
    );
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.container.rowGap = 12;
    const result = reconcileZuiDocument(document, snapshot);
    expect(zuiNodes(result.document)['root'].layout!['container']).toEqual({
      kind: 'GridBox',
      columns: 2,
      rows: 1,
      column_gap: 6,
      row_gap: 12,
    });
    expect(result.changes).toEqual(['nodes.root.layout.container.row_gap']);
  });

  it('keeps the visible main-axis gap when wrapping is disabled', () => {
    const document = fixture(
      'FlowBox',
      'gap = 8, horizontal_gap = 6, vertical_gap = 10',
    );
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.container.wrap = false;
    const result = reconcileZuiDocument(document, snapshot);
    expect(
      zuiNodes(result.document)['root'].layout!['container'],
    ).toMatchObject({
      kind: 'HorizontalBox',
      gap: 6,
    });
    expect(
      projectZuiDocument(result.document).rootNodes[0].container,
    ).toMatchObject({
      kind: 'flex',
      direction: 'row',
      wrap: false,
      columnGap: 6,
    });
  });

  it.each([
    ['FlowBox', 'horizontal_gap', 'vertical_gap'],
    ['GridBox', 'column_gap', 'row_gap'],
  ] as const)(
    'keeps visible gaps when entering %s with previously inactive source overrides',
    (kind, columnKey, rowKey) => {
      const document = fixture(
        'HorizontalBox',
        `gap = 8, ${columnKey} = 6, ${rowKey} = 10`,
      );
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      Object.assign(snapshot.shapes[0].current.container, {
        kind: kind === 'GridBox' ? 'grid' : 'flex',
        wrap: kind === 'FlowBox',
      });
      const result = reconcileZuiDocument(document, snapshot);
      expect(
        zuiNodes(result.document)['root'].layout!['container'],
      ).toMatchObject({
        kind,
        [columnKey]: 8,
        [rowKey]: 8,
      });
      expect(
        projectZuiDocument(result.document).rootNodes[0].container,
      ).toMatchObject({
        rowGap: 8,
        columnGap: 8,
      });
    },
  );

  it('rejects vertical wrapping that the native container cannot represent', () => {
    const document = fixture('FlowBox');
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.container.direction = 'column';
    expect(() => reconcileZuiDocument(document, snapshot)).toThrow(/wrap/i);
  });

  it('projects native FlexBox as a horizontal wrapping container', () => {
    const document = fixture(
      'FlexBox',
      'horizontal_gap = 6, vertical_gap = 10',
    );
    expect(projectZuiDocument(document).rootNodes[0].container).toMatchObject({
      kind: 'flex',
      direction: 'row',
      wrap: true,
      rowGap: 10,
      columnGap: 6,
    });
  });

  it('uses the native zero-gap default for FlexBox', () => {
    expect(
      projectZuiDocument(fixture('FlexBox', '')).rootNodes[0].container,
    ).toMatchObject({
      rowGap: 0,
      columnGap: 0,
    });
  });

  it.each([
    ['GridBox', 'column_gap', 'row_gap'],
    ['FlowBox', 'horizontal_gap', 'vertical_gap'],
  ])(
    'updates explicit axis overrides when the shared %s gap changes',
    (kind, columnKey, rowKey) => {
      const document = fixture(
        kind,
        `gap = 8, ${columnKey} = 6, ${rowKey} = 10`,
      );
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      Object.assign(snapshot.shapes[0].current.container, {
        gap: 9,
        rowGap: 9,
        columnGap: 9,
      });
      const result = reconcileZuiDocument(document, snapshot);
      expect(zuiNodes(result.document)['root'].layout!['container']).toEqual({
        kind,
        gap: 9,
        [columnKey]: 9,
        [rowKey]: 9,
      });
      expect(
        projectZuiDocument(result.document).rootNodes[0].container,
      ).toMatchObject({ rowGap: 9, columnGap: 9 });
    },
  );
});
import { createHash } from 'node:crypto';

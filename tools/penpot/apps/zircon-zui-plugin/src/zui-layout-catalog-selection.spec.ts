import {
  catalogSourceSelection,
  mergeSelectedCatalogEntries,
  selectCatalogSources,
} from '../tools/zui-layout-catalog-selection';

describe('focused catalog refresh', () => {
  it('supports repeated exact sources and normalizes Windows separators', () => {
    expect(
      catalogSourceSelection([
        '--source',
        'zircon_editor\\assets\\workbench.zui',
        '--source=zircon_editor/assets/toolbar.zui',
        '--source',
        'zircon_editor/assets/workbench.zui',
      ]),
    ).toEqual([
      'zircon_editor/assets/workbench.zui',
      'zircon_editor/assets/toolbar.zui',
    ]);
  });

  it.each([
    '',
    '../workbench.zui',
    '/workbench.zui',
    'C:/workbench.zui',
    './workbench.zui',
  ])('rejects noncanonical source paths: %s', (path) =>
    expect(() => catalogSourceSelection(['--source', path])).toThrow(),
  );

  it('rejects a missing option value before another flag', () => {
    expect(() => catalogSourceSelection(['--source', '--check'])).toThrow();
  });

  it('requires exact inventory membership without broad substring matching', () => {
    const inventory = ['ui/workbench.zui', 'ui/workbench_toolbar.zui'];
    expect(selectCatalogSources(inventory, ['ui/workbench.zui'])).toEqual([
      'ui/workbench.zui',
    ]);
    expect(() => selectCatalogSources(inventory, ['workbench.zui'])).toThrow();
  });

  it('preserves unselected reviews, case captures, and record order', () => {
    const unrelated = {
      sourcePath: 'ui/other.zui',
      review: { status: 'accepted' },
      evidence: [{ screenshotSha256: 'existing-capture' }],
    };
    const prior = {
      sourcePath: 'ui/workbench.zui',
      review: { status: 'pending' },
      evidence: [],
    };
    const refreshed = { ...prior, sourceSha256: 'current-source' };
    const result = mergeSelectedCatalogEntries([unrelated, prior], [refreshed]);
    expect(result).toEqual([unrelated, refreshed]);
    expect(result[0]).toBe(unrelated);
    expect(result[1]).toBe(refreshed);
  });

  it('adds a newly selected source without deleting existing records', () => {
    expect(
      mergeSelectedCatalogEntries(
        [{ sourcePath: 'ui/old.zui' }],
        [{ sourcePath: 'ui/new.zui' }],
      ),
    ).toEqual([{ sourcePath: 'ui/old.zui' }, { sourcePath: 'ui/new.zui' }]);
  });

  it('rejects duplicate updates instead of silently losing a selected record', () => {
    expect(() =>
      mergeSelectedCatalogEntries(
        [],
        [
          { sourcePath: 'ui/workbench.zui' },
          { sourcePath: 'ui/workbench.zui' },
        ],
      ),
    ).toThrow();
  });
});

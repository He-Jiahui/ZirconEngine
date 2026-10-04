import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { parseZuiDocument } from './bridge/zui-document';
import {
  caseSha256,
  defaultReviewCases,
} from '../tools/zui-layout-review-contract';
import {
  buildWorkbenchReviewCases,
  LONG_CHINESE_NAME,
  LONG_ENGLISH_NAME,
  WORKBENCH_WINDOW_SOURCE,
  type WorkbenchCaseSnapshots,
} from '../tools/zui-layout-workbench-cases';
import { canonicalSha256 } from '../tools/zui-layout-workbench-presentation';
import { projectWorkbenchHierarchyRows } from '../tools/zui-layout-workbench-projection';

const repoRoot = resolve(
  fileURLToPath(new URL('.', import.meta.url)),
  '../../../../..',
);
const source = readFileSync(resolve(repoRoot, WORKBENCH_WINDOW_SOURCE), 'utf8');
const document = parseZuiDocument(source).document;
const hierarchyRows = [
  {
    id: '80',
    parentId: null,
    name: 'Camera',
    kind: 'Camera',
    depth: 0,
    active: true,
    hasChildren: false,
    generation: '7',
    subtreeHash: '101',
  },
  {
    id: '70',
    parentId: null,
    name: 'Sun',
    kind: 'DirectionalLight',
    depth: 0,
    active: true,
    hasChildren: true,
    generation: '7',
    subtreeHash: '202',
  },
  {
    id: '71',
    parentId: '70',
    name: 'Cube',
    kind: 'Mesh',
    depth: 1,
    active: true,
    hasChildren: false,
    generation: '7',
    subtreeHash: '303',
  },
];
const longEnglishName = LONG_ENGLISH_NAME;
const longChineseName = LONG_CHINESE_NAME;
const hierarchyForName = (name: string) =>
  hierarchyRows.map((row) =>
    row.name === 'Cube' ? { ...row, name } : { ...row },
  );
const marker = (
  name: string,
  hierarchy?: unknown,
  activeLocale = 'en',
  inspectorName?: string,
) => ({
  schema: 'dev.zircon.editor.workbench-presentation' as const,
  version: 1 as const,
  activeLocale,
  stateFingerprint: name,
  sourceFingerprint: {
    sourcePath: WORKBENCH_WINDOW_SOURCE,
    sha256: '0'.repeat(64),
  },
  ...(hierarchy === undefined ? {} : { hierarchy }),
  ...(inspectorName === undefined
    ? {}
    : { inspector: { entityId: '71', name: inspectorName } }),
});
const snapshots = {
  schema: 'dev.zircon.editor.workbench-case-snapshots',
  version: 1,
  sourceFingerprint: {
    sourcePath: WORKBENCH_WINDOW_SOURCE,
    sha256: '0'.repeat(64),
  },
  managedSceneFingerprint: {
    projectRoot: 'D:\\cargo-targets\\workbench-test-project',
    sources: [
      { sourcePath: 'zircon-project.toml', sha256: '0'.repeat(64) },
      { sourcePath: 'assets/scenes/main.scene.toml', sha256: '1'.repeat(64) },
      {
        sourcePath: 'assets/scenes/workbench-review-empty.scene.toml',
        sha256: '2'.repeat(64),
      },
    ],
  },
  states: {
    default: marker('default'),
    empty: marker('empty'),
    selected: marker('selected'),
    hierarchy: marker('hierarchy', {
      filterQuery: '',
      expandedIds: [],
      selectedIds: [],
      rows: hierarchyRows,
    }),
    longEnglish: marker(
      'longEnglish',
      {
        filterQuery: '',
        expandedIds: [],
        selectedIds: ['71'],
        rows: hierarchyForName(longEnglishName),
      },
      'en',
      longEnglishName,
    ),
    longChinese: marker(
      'longChinese',
      {
        filterQuery: '',
        expandedIds: [],
        selectedIds: ['71'],
        rows: hierarchyForName(longChineseName),
      },
      'zh-CN',
      longChineseName,
    ),
  },
} as WorkbenchCaseSnapshots;

describe('product WorkbenchWindow case matrix', () => {
  const cases = buildWorkbenchReviewCases(
    defaultReviewCases(WORKBENCH_WINDOW_SOURCE, 'view', document),
    document,
    snapshots,
  );

  it('gives every requested size/state a factual product snapshot and one authored control', () => {
    expect(cases).toHaveLength(81);
    expect(new Set(cases.map((item) => item.id)).size).toBe(cases.length);
    expect(cases.every((item) => item.data['workbenchPresentation'])).toBe(
      true,
    );
    expect(cases.every((item) => item.data['workbenchState'])).toBe(true);
    expect(
      cases.find((item) => item.id === 'selected-1280x800-dpi1.5')?.data,
    ).toMatchObject({
      workbenchPresentation: { stateFingerprint: 'selected' },
    });
    expect(
      cases.find((item) => item.id === 'empty-640x520-dpi1')?.data,
    ).toMatchObject({ workbenchPresentation: { stateFingerprint: 'empty' } });
    expect(
      cases.find((item) => item.id === 'focused-900x620-dpi1')?.data,
    ).toMatchObject({
      workbenchState: {
        sourceNodeId: 'scene_search_field',
        controlId: 'WorkbenchSceneSearchField',
      },
    });
    expect(
      cases.find((item) => item.id === 'scroll-after-640x520-dpi1')?.data,
    ).toMatchObject({
      workbenchState: {
        sourceNodeId: 'scene_tree',
        controlId: 'WorkbenchSceneTree',
      },
    });
    expect(
      cases.find((item) => item.id === 'open-scroll-after-640x520-dpi1')?.data,
    ).toMatchObject({
      workbenchState: {
        sourceNodeId: 'toolbar_main_menu',
        scrollTarget: { sourceNodeId: 'scene_tree' },
      },
    });
    expect(
      cases.find((item) => item.id === 'inspector-scroll-after-640x520-dpi1')
        ?.data,
    ).toMatchObject({
      workbenchPresentation: { stateFingerprint: 'selected' },
      workbenchState: { sourceNodeId: 'inspector_content' },
    });
  });

  it('binds every case and its case hash to the managed scene inputs', () => {
    expect(
      cases.every(
        (item) =>
          JSON.stringify(item.data['managedSceneFingerprint']) ===
          JSON.stringify(snapshots.managedSceneFingerprint),
      ),
    ).toBe(true);
    const changed = structuredClone(cases[0]!);
    const fingerprint = changed.data[
      'managedSceneFingerprint'
    ] as WorkbenchCaseSnapshots['managedSceneFingerprint'];
    fingerprint.sources[0]!.sha256 = '3'.repeat(64);
    expect(caseSha256(changed)).not.toBe(caseSha256(cases[0]!));
  });

  it('opens one real popup at a time and uses product snapshots for long locales', () => {
    expect(
      cases.find((item) => item.id === 'settings_window-open-900x620-dpi1')
        ?.data,
    ).toMatchObject({
      workbenchState: {
        sourceNodeId: 'settings_window',
        controlId: 'WorkbenchPreferences',
      },
    });
    expect(
      cases.find(
        (item) => item.id === 'toolbar_layout_menu-focus-return-900x620-dpi1',
      )?.data,
    ).toMatchObject({
      workbenchState: {
        sourceNodeId: 'toolbar_layout_menu',
        controlId: 'WorkbenchLayoutMenu',
      },
    });
    const english = cases.find((item) => item.id === 'long-en-640x520-dpi1');
    const chinese = cases.find((item) => item.id === 'long-zh-640x520-dpi1');
    expect(english?.locale).toBe('en-US');
    expect(chinese?.locale).toBe('zh-CN');

    for (const [item, snapshotState, activeLocale, expectedName] of [
      [english, 'longEnglish', 'en', longEnglishName],
      [chinese, 'longChinese', 'zh-CN', longChineseName],
    ] as const) {
      const presentation = item?.data['workbenchPresentation'] as Record<
        string,
        unknown
      >;
      expect(presentation['activeLocale']).toBe(activeLocale);
      expect(presentation['stateFingerprint']).toBe(snapshotState);
      expect(presentation['inspector']).toMatchObject({ name: expectedName });
      expect(
        (presentation['hierarchy'] as { rows: typeof hierarchyRows }).rows.find(
          (row) => row.name === expectedName,
        ),
      ).toBeDefined();
      expect(item?.data['workbenchState']).not.toHaveProperty('textOverrides');
    }
  });

  it('filters by the runtime query rules and honors expansion by entity ID', () => {
    const rows = [
      { id: '10', parentId: null, depth: 0, name: 'Root', hasChildren: true },
      { id: '11', parentId: '10', depth: 1, name: 'Sun', hasChildren: true },
      { id: '12', parentId: '11', depth: 2, name: 'Cube', hasChildren: false },
      { id: '20', parentId: null, depth: 0, name: 'Camera', hasChildren: false },
    ];

    expect(projectWorkbenchHierarchyRows(rows, '', ['10'])).toEqual([
      rows[0],
      rows[1],
      rows[3],
    ]);
    expect(projectWorkbenchHierarchyRows(rows, '  cube  ', [])).toEqual([
      rows[0],
      rows[1],
      rows[2],
    ]);
    expect(projectWorkbenchHierarchyRows(rows, 'SUN', [])).toEqual([
      rows[0],
      rows[1],
    ]);
  });

  it('builds actual nested hierarchy cases and rehashes each view state', () => {
    const caseByKind = (kind: string) =>
      cases.find((item) => item.id.startsWith(`hierarchy-${kind}-`))!;
    const stateFor = (kind: string) => {
      const presentation = caseByKind(kind).data['workbenchPresentation'] as Record<
        string,
        unknown
      >;
      const { stateFingerprint, ...state } = presentation;
      expect(stateFingerprint).toBe(canonicalSha256(state));
      return state['hierarchy'] as {
        filterQuery: string;
        expandedIds: string[];
        rows: typeof hierarchyRows;
      };
    };

    const collapsed = stateFor('collapsed');
    const expanded = stateFor('expanded');
    const matching = stateFor('search-match');
    const noMatch = stateFor('search-no-match');
    const cleared = stateFor('search-cleared');
    const cube = hierarchyRows.find((row) => row.name === 'Cube')!;
    const sun = hierarchyRows.find((row) => row.id === cube.parentId)!;

    expect(cube.parentId).toBe(sun.id);
    expect(sun.hasChildren).toBe(true);
    expect(collapsed.expandedIds).toEqual([]);
    expect(expanded.expandedIds).toEqual([sun.id]);
    expect(matching.filterQuery).toBe('Cube');
    expect(matching.expandedIds).toEqual([]);
    expect(noMatch.filterQuery).toBe('__zircon_no_matching_scene_entity__');
    expect(cleared.filterQuery).toBe('');
    expect(cleared.expandedIds).toEqual([sun.id]);

    expect(
      projectWorkbenchHierarchyRows(
        collapsed.rows,
        collapsed.filterQuery,
        collapsed.expandedIds,
      ).map((row) => row.id),
    ).toEqual(['80', sun.id]);
    expect(
      projectWorkbenchHierarchyRows(
        expanded.rows,
        expanded.filterQuery,
        expanded.expandedIds,
      ).map((row) => row.id),
    ).toEqual(['80', sun.id, cube.id]);
    expect(
      projectWorkbenchHierarchyRows(
        matching.rows,
        matching.filterQuery,
        matching.expandedIds,
      ).map((row) => row.id),
    ).toEqual([sun.id, cube.id]);
    expect(
      projectWorkbenchHierarchyRows(
        noMatch.rows,
        noMatch.filterQuery,
        noMatch.expandedIds,
      ),
    ).toEqual([]);
    expect(
      projectWorkbenchHierarchyRows(
        cleared.rows,
        cleared.filterQuery,
        cleared.expandedIds,
      ).map((row) => row.id),
    ).toEqual(['80', sun.id, cube.id]);

    expect(caseByKind('collapsed').viewport).toEqual({ width: 1280, height: 800 });
    expect(caseByKind('expanded').viewport).toEqual({ width: 640, height: 520 });
    expect(caseByKind('search-match').viewport).toEqual({ width: 900, height: 620 });
    expect(caseByKind('search-no-match').dpi).toBe(1.5);
  });
});

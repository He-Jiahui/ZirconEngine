import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { lstat, mkdir, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import fixtureSource from './roundtrip-fixture.zui?raw';
import {
  applyZuiExportFileToCanonicalSourceOwners,
  applyZuiExportFileToSourceRoot,
  type ZuiSourceExportManifest,
  type ZuiSourceFingerprint,
} from './zui-export-apply';
import { parseZuiDocument, serializeZuiDocument } from './zui-document';
import { applyZuiExportToSourceFiles } from './zui-export-apply-sources';
import { cloneProjectionSnapshot, projectZuiDocument } from './penpot-projection';
import { reconcileReviewSource } from './zui-review-host';
import { requireApprovedPenpotArtifactDirectory } from '../../tools/zui-layout-artifact-path';
import { vi } from 'vitest';

const ownerWriteFaults = vi.hoisted(() => ({
  corruptTemporary: false,
  changeDependencyAfterRename: undefined as { ownerPath: string; dependencyPath: string; source: string } | undefined,
}));

vi.mock('node:fs/promises', async (importOriginal) => {
  const actual = await importOriginal<typeof import('node:fs/promises')>();
  return {
    ...actual,
    lstat: async (path: string) => {
      if (ownerWriteFaults.corruptTemporary && path.includes('.zui-apply-')) {
        ownerWriteFaults.corruptTemporary = false;
        await actual.writeFile(path, 'changed after staging', 'utf8');
      }
      return actual.lstat(path);
    },
    rename: async (from: string, to: string) => {
      await actual.rename(from, to);
      const change = ownerWriteFaults.changeDependencyAfterRename;
      if (change && to === change.ownerPath) {
        ownerWriteFaults.changeDependencyAfterRename = undefined;
        await actual.writeFile(change.dependencyPath, change.source, 'utf8');
      }
    },
  };
});

const pluginRoot = fileURLToPath(new URL('../..', import.meta.url));
const sourcePath = 'zircon_editor/assets/ui/workbench_window.zui';
const componentPath =
  'zircon_editor/assets/ui/editor/components/button.zui';

function sha256(source: string): string {
  return createHash('sha256').update(source, 'utf8').digest('hex');
}

function clone<T>(value: T): T {
  return structuredClone(value);
}

function sourceFingerprint(
  path: string,
  source: string,
): ZuiSourceFingerprint {
  return { sourcePath: path, sha256: sha256(source) };
}

function manifest(
  rootSourcePath: string,
  baselineProjection: string,
  exportedProjection: string,
  identityMap: ZuiSourceExportManifest['identityMap'],
  sources: ZuiSourceFingerprint[],
  edits: ZuiSourceExportManifest['edits'],
): string {
  return JSON.stringify({
    schema: 'dev.zircon.zui.penpot-source-exports',
    version: 1,
    rootSourcePath,
    baselineProjection,
    exportedProjection,
    identityMap,
    sources,
    edits,
  } satisfies ZuiSourceExportManifest);
}

function expandedExport(args: {
  baselineSource?: string;
  sourcePath?: string;
  sourceNodeId?: string;
  instancePath?: string;
  exportNodeId?: string;
  fieldPath?: string[];
  dependencySource?: string;
} = {}) {
  const ownerSource = `# Workbench owner comment survives.\r\n${fixtureSource}`;
  const rootSource = fixtureSource;
  const dependencySource = args.dependencySource ?? fixtureSource;
  const baselineDocument = parseZuiDocument(ownerSource).document;
  const document = clone(baselineDocument);
  const ownerNode = document.nodes!.root;
  delete document.nodes!.root;
  const exportNodeId = args.exportNodeId ?? 'expanded_window_content';
  const fieldPath = args.fieldPath ?? ['layout', 'container', 'gap'];
  document.nodes![exportNodeId] = ownerNode;
  if (document.root?.node === 'root') document.root.node = exportNodeId;
  const ownerSourcePath = args.sourcePath ?? sourcePath;
  const ownerInstancePath = args.instancePath ?? '[]';
  for (const [nodeId, node] of Object.entries(document.nodes ?? {})) {
    const isTarget = nodeId === exportNodeId;
    node['penpot_review_source_path'] = ownerSourcePath;
    node['penpot_review_source_node_id'] = isTarget
      ? (args.sourceNodeId ?? 'root')
      : nodeId;
    node['penpot_review_instance_path'] = ownerInstancePath;
  }
  const baselineProjection = serializeZuiDocument(document);
  const before = Number((ownerNode.layout!['container'] as Record<string, unknown>)['gap']);
  (
    document.nodes![exportNodeId].layout!['container'] as Record<
      string,
      unknown
    >
  )['gap'] = before + 2;
  const sources = [
    sourceFingerprint(sourcePath, ownerSource),
    sourceFingerprint(componentPath, dependencySource),
  ];
  const edit = {
    exportNodeId,
    sourcePath: args.sourcePath ?? sourcePath,
    sourceNodeId: args.sourceNodeId ?? 'root',
    controlId:
      typeof ownerNode.control_id === 'string' ? ownerNode.control_id : null,
    instancePath: args.instancePath ?? '[]',
    fieldPath,
    value: before + 2,
  };
  const identityMap = Object.entries(document.nodes ?? {}).map(
    ([nodeId, node]) => ({
      exportNodeId: nodeId,
      sourcePath: String(node['penpot_review_source_path']),
      sourceNodeId: String(node['penpot_review_source_node_id']),
      controlId: typeof node.control_id === 'string' ? node.control_id : null,
      instancePath: String(node['penpot_review_instance_path']),
    }),
  );
  const exportedProjection = serializeZuiDocument(document);
  const rootDocument = parseZuiDocument(rootSource).document;
  rootDocument['penpot_source_exports'] = manifest(
    sourcePath,
    baselineProjection,
    exportedProjection,
    identityMap,
    sources,
    [edit],
  );
  return {
    ownerSource,
    rootSource,
    dependencySource,
    sources,
    exportedSource: serializeZuiDocument(rootDocument),
    baselineProjection,
    exportedProjection,
    identityMap,
    before,
  };
}

function heightMinCallsiteExport() {
  const rootPath = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
  const calleePath = 'zircon_editor/assets/ui/editor/unreal/components/outliner.zui';
  const shellPath = 'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
  const rootSource = `# Keep the authored dock ratio.
[asset]
kind = "view"
id = "res://ui/editor/unreal/level_editor_window.zui"
version = 2
[root]
node = "right_dock"
[imports]
widgets = ["res://ui/editor/unreal/components/outliner.zui#UnrealOutlinerPanel"]
[nodes.right_dock]
component = "VerticalGroup"
layout = { container = { kind = "VerticalBox", gap = 4 }, height = { preferred = 343, stretch = "Fixed" } }
children = [{ node = "outliner", slot = { layout = { linear_size = { rule = "Stretch", value = 0.4 } } } }, { node = "details", slot = { layout = { linear_size = { rule = "Stretch", value = 1.0 } } } }]
[nodes.outliner]
component = "UnrealOutlinerPanel"
control_id = "WorkbenchMainBandSceneTreePanel"
layout = { width = { stretch = "Stretch" }, height = { min = 96, stretch = "Stretch" } }
[nodes.details]
component = "VerticalGroup"
layout = { height = { min = 96, stretch = "Stretch" } }
`;
  const calleeSource = `[asset]
kind = "component"
id = "res://ui/editor/unreal/components/outliner.zui"
version = 2
[components.UnrealOutlinerPanel]
root = "panel"
[nodes.panel]
component = "VerticalGroup"
layout = { height = { stretch = "Stretch" } }
`;
  const shellSource = `[asset]
kind = "view"
id = "res://ui/editor/unreal/editor_shell.zui"
version = 2
[root]
node = "pane_surface"
[nodes.pane_surface]
component = "PaneSurface"
`;
  const canonicalSources: Record<string, string> = {
    [rootPath]: rootSource, [calleePath]: calleeSource, [shellPath]: shellSource,
  };
  const sources = Object.entries(canonicalSources).map(([path, text]) => sourceFingerprint(path, text));
  const projection = parseZuiDocument(rootSource).document;
  const host = clone(projection);
  for (const [id, node] of Object.entries(host.nodes!)) {
    node['penpot_review_source_path'] = rootPath;
    node['penpot_review_source_node_id'] = id;
    node['penpot_review_instance_path'] = '[]';
  }
  Object.assign(host.nodes!.outliner, {
    component: 'VerticalGroup', penpot_review_source_path: calleePath,
    penpot_review_source_node_id: 'panel',
    penpot_review_instance_path: JSON.stringify([
      { sourcePath: shellPath, sourceNodeId: 'pane_surface' },
      { sourcePath: rootPath, sourceNodeId: 'outliner' },
    ]),
    penpot_prefab_source: `${calleePath}#UnrealOutlinerPanel`,
  });
  Object.assign(projection, { penpot_review_host: host, penpot_original_source: rootSource,
    penpot_original_source_path: rootPath, penpot_source_fingerprints: sources });
  const snapshot = cloneProjectionSnapshot(projectZuiDocument(host));
  snapshot.shapes.find(({ nodeId }) => nodeId === 'outliner')!.current.heightMin = 128;
  const exportedSource = reconcileReviewSource(projection, snapshot).source!;
  const envelope = JSON.parse(
    parseZuiDocument(exportedSource).document['penpot_source_exports'] as string,
  ) as ZuiSourceExportManifest;
  return { rootPath, calleePath, canonicalSources, sources, exportedSource, envelope };
}

describe('apply root callsite height.min exports', () => {
  it('writes only the proven original callsite leaf and preserves the component and slot weights', () => {
    const s = heightMinCallsiteExport();
    const original = clone(s.canonicalSources);
    const applied = applyZuiExportToSourceFiles(s.exportedSource, s.canonicalSources, s.sources);
    expect(applied.appliedPathsBySource).toEqual({ [s.rootPath]: ['nodes.outliner.layout.height.min'] });
    expect(applied.outputs.map(({ sourcePath: path }) => path)).toEqual([s.rootPath]);
    const expected = parseZuiDocument(original[s.rootPath]).document;
    (expected.nodes!.outliner.layout!['height'] as Record<string, unknown>)['min'] = 128;
    expect(parseZuiDocument(applied.outputs[0].source).document).toEqual(expected);
    expect(applied.outputs[0].source).toContain('# Keep the authored dock ratio.');
    expect(s.canonicalSources).toEqual(original);
  });

  it.each(['missing proof', 'forged owner', 'wrong callee root', 'rebound callsite',
    'missing leaf', 'ambiguous binding', 'stale baseline', 'duplicate invocation', 'wrong final invocation'])(
    'rejects %s without falling back to the component owner', (kind) => {
      const s = heightMinCallsiteExport();
      const envelope = s.envelope;
      const edit = envelope.edits[0];
      const setSource = (path: string, document: ReturnType<typeof parseZuiDocument>['document']) => {
        const source = serializeZuiDocument(document);
        s.canonicalSources[path] = source;
        s.sources.find((row) => row.sourcePath === path)!.sha256 = sha256(source);
        envelope.sources.find((row) => row.sourcePath === path)!.sha256 = sha256(source);
      };
      if (kind === 'missing proof') delete edit.heightMinOwner;
      if (kind === 'forged owner') edit.heightMinOwner!.sourceNodeId = 'details';
      if (kind === 'wrong callee root') {
        const callee = parseZuiDocument(s.canonicalSources[s.calleePath]).document;
        callee.components!.UnrealOutlinerPanel.root = 'different_panel';
        callee.nodes!.different_panel = { component: 'VerticalGroup' };
        setSource(s.calleePath, callee);
      }
      if (['rebound callsite', 'missing leaf', 'ambiguous binding'].includes(kind)) {
        const root = parseZuiDocument(s.canonicalSources[s.rootPath]).document;
        if (kind === 'rebound callsite') {
          root.nodes!.outliner.component = 'DifferentPanel';
          root.imports!['widgets'] = ['res://ui/editor/unreal/components/outliner.zui#DifferentPanel'];
        }
        if (kind === 'missing leaf')
          delete (root.nodes!.outliner.layout!['height'] as Record<string, unknown>)['min'];
        if (kind === 'ambiguous binding')
          (root.imports!['widgets'] as string[]).push('res://ui/editor/unreal/components/other.zui#UnrealOutlinerPanel');
        setSource(s.rootPath, root);
      }
      if (['stale baseline', 'duplicate invocation', 'wrong final invocation'].includes(kind)) {
        const baseline = parseZuiDocument(envelope.baselineProjection).document;
        const exported = parseZuiDocument(envelope.exportedProjection).document;
        if (kind === 'stale baseline')
          (baseline.nodes!.outliner.layout!['height'] as Record<string, unknown>)['min'] = 95;
        else {
          const steps = kind === 'duplicate invocation'
            ? [{ sourcePath: s.rootPath, sourceNodeId: 'outliner' }, { sourcePath: s.rootPath, sourceNodeId: 'outliner' }]
            : [{ sourcePath: s.calleePath, sourceNodeId: 'panel' }];
          const instancePath = JSON.stringify(steps);
          baseline.nodes!.outliner['penpot_review_instance_path'] = instancePath;
          exported.nodes!.outliner['penpot_review_instance_path'] = instancePath;
          envelope.identityMap.find((row) => row.exportNodeId === 'outliner')!.instancePath = instancePath;
          edit.instancePath = instancePath;
        }
        envelope.baselineProjection = serializeZuiDocument(baseline);
        envelope.exportedProjection = serializeZuiDocument(exported);
      }
      const root = parseZuiDocument(s.canonicalSources[s.rootPath]).document;
      root['penpot_source_exports'] = JSON.stringify(envelope);
      const before = clone(s.canonicalSources);
      expect(() => applyZuiExportToSourceFiles(serializeZuiDocument(root), s.canonicalSources, s.sources))
        .toThrow(/height.min/);
      expect(s.canonicalSources).toEqual(before);
    },
  );

  it('rejects a stale current source hash before resolving the callsite owner', () => {
    const s = heightMinCallsiteExport();
    s.canonicalSources[s.calleePath] += '\n# Changed after import.\n';
    expect(() => applyZuiExportToSourceFiles(s.exportedSource, s.canonicalSources, s.sources))
      .toThrow(/stale compared with the pre-import fingerprint/);
  });

  it('keeps nonvisual event edits rejected alongside a supported height.min edit', () => {
    const s = heightMinCallsiteExport();
    const exported = parseZuiDocument(s.envelope.exportedProjection).document;
    exported.nodes!.outliner.events = [{ id: 'UiHostWindow/ActivateMainPage', event: 'Change',
      route: 'workbench.dock.activate_main_page' }];
    s.envelope.exportedProjection = serializeZuiDocument(exported);
    const root = parseZuiDocument(s.canonicalSources[s.rootPath]).document;
    root['penpot_source_exports'] = JSON.stringify(s.envelope);
    expect(() => applyZuiExportToSourceFiles(serializeZuiDocument(root), s.canonicalSources, s.sources))
      .toThrow(/nonvisual/);
  });
});

async function withCanonicalFixture(
  run: (
    fixture: ReturnType<typeof expandedExport>,
    paths: { sourceRootPath: string; exportedPath: string; ownerPath: string; componentAssetPath: string },
  ) => Promise<void>,
): Promise<void> {
  const registeredRoot = process.env['ZUI_LAYOUT_TEST_ARTIFACT_DIR'];
  if (!registeredRoot) throw new Error('Register ZUI_LAYOUT_TEST_ARTIFACT_DIR before running owner-write regressions.');
  const root = requireApprovedPenpotArtifactDirectory(registeredRoot, 'canonical owner test fixtures');
  const directory = await mkdtemp(join(root, 'zui-canonical-owner-'));
  const identity = await lstat(directory);
  const sourceRootPath = join(directory, 'canonical');
  const paths = {
    sourceRootPath,
    exportedPath: join(directory, 'penpot-download.zui'),
    ownerPath: join(sourceRootPath, sourcePath),
    componentAssetPath: join(sourceRootPath, componentPath),
  };
  const fixture = expandedExport();
  try {
    await mkdir(dirname(paths.componentAssetPath), { recursive: true });
    await writeFile(paths.ownerPath, fixture.ownerSource, 'utf8');
    await writeFile(paths.componentAssetPath, fixture.dependencySource, 'utf8');
    await writeFile(paths.exportedPath, fixture.exportedSource, 'utf8');
    await run(fixture, paths);
  } finally {
    requireApprovedPenpotArtifactDirectory(directory, 'owned canonical test fixture');
    const current = await lstat(directory);
    if (dirname(resolve(directory)) !== resolve(root) || current.isSymbolicLink() ||
        current.dev !== identity.dev || current.ino !== identity.ino)
      throw new Error('Canonical test fixture ownership changed; preserved for review.');
    await rm(directory, { recursive: true, force: true });
  }
}

describe('applyZuiExportFileToCanonicalSourceOwners', () => {
  it('writes the original owner and preserves all unrelated source fields', async () => {
    await withCanonicalFixture(async (fixture, paths) => {
      const result = await applyZuiExportFileToCanonicalSourceOwners({
        ...paths,
        expectedSources: fixture.sources,
        expectedRootSha256: sha256(fixture.ownerSource),
        writableSourcePaths: [sourcePath],
      });
      const writtenSource = await readFile(paths.ownerPath, 'utf8');
      const expected = parseZuiDocument(fixture.ownerSource).document;
      (expected.nodes!.root.layout!['container'] as Record<string, unknown>)['gap'] = fixture.before + 2;
      expect(parseZuiDocument(writtenSource).document).toEqual(expected);
      expect(writtenSource).toContain('# Workbench owner comment survives.');
      expect(await readFile(paths.componentAssetPath, 'utf8')).toBe(fixture.dependencySource);
      expect(result.outputs.map((item) => item.outputPath)).toEqual([paths.ownerPath]);
      expect(result.appliedPathsBySource[sourcePath]).toEqual(['nodes.root.layout.container.gap']);
    });
  });

  it('rejects an empty owner allowlist without writes', async () => {
    await withCanonicalFixture(async (fixture, paths) => {
      await expect(applyZuiExportFileToCanonicalSourceOwners({
        ...paths, expectedSources: fixture.sources,
        expectedRootSha256: sha256(fixture.ownerSource), writableSourcePaths: [],
      })).rejects.toThrow('Explicit writable source owners are required.');
      expect(await readFile(paths.ownerPath, 'utf8')).toBe(fixture.ownerSource);
      expect(await readFile(paths.componentAssetPath, 'utf8')).toBe(fixture.dependencySource);
    });
  });

  it('rejects unauthorized, out-of-closure, and duplicate owners without writes', async () => {
    await withCanonicalFixture(async (fixture, paths) => {
      for (const writableSourcePaths of [[componentPath], ['zircon_editor/assets/ui/not-in-manifest.zui'], [sourcePath, sourcePath]]) {
        await expect(applyZuiExportFileToCanonicalSourceOwners({
          ...paths, expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource), writableSourcePaths,
        })).rejects.toThrow(/explicit writable owner list|trusted source closure|more than once/);
        expect(await readFile(paths.ownerPath, 'utf8')).toBe(fixture.ownerSource);
        expect(await readFile(paths.componentAssetPath, 'utf8')).toBe(fixture.dependencySource);
      }
    });
  });

  it('rejects a stale dependency before changing the owner', async () => {
    await withCanonicalFixture(async (fixture, paths) => {
      const stale = fixture.dependencySource + '\n# changed after import';
      await writeFile(paths.componentAssetPath, stale, 'utf8');
      await expect(applyZuiExportFileToCanonicalSourceOwners({
        ...paths, expectedSources: fixture.sources,
        expectedRootSha256: sha256(fixture.ownerSource), writableSourcePaths: [sourcePath],
      })).rejects.toThrow(/stale|fingerprint/i);
      expect(await readFile(paths.ownerPath, 'utf8')).toBe(fixture.ownerSource);
      expect(await readFile(paths.componentAssetPath, 'utf8')).toBe(stale);
    });
  });

  it('rejects invalid UTF-8 before changing any owner bytes', async () => {
    await withCanonicalFixture(async (fixture, paths) => {
      const invalidUtf8 = Buffer.from([0xff, 0xfe]);
      await writeFile(paths.componentAssetPath, invalidUtf8);
      await expect(applyZuiExportFileToCanonicalSourceOwners({
        ...paths, expectedSources: fixture.sources,
        expectedRootSha256: sha256(fixture.ownerSource), writableSourcePaths: [sourcePath],
      })).rejects.toThrow('Canonical source must round-trip as UTF-8 before owner application.');
      expect(await readFile(paths.ownerPath, 'utf8')).toBe(fixture.ownerSource);
      expect(await readFile(paths.componentAssetPath)).toEqual(invalidUtf8);
    });
  });

  it('rejects a changed temporary file before replacing the original owner', async () => {
    await withCanonicalFixture(async (fixture, paths) => {
      ownerWriteFaults.corruptTemporary = true;
      try {
        await expect(applyZuiExportFileToCanonicalSourceOwners({
          ...paths, expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource), writableSourcePaths: [sourcePath],
        })).rejects.toThrow('Canonical owner temporary file identity or content changed.');
        expect(await readFile(paths.ownerPath, 'utf8')).toBe(fixture.ownerSource);
        expect(await readFile(paths.componentAssetPath, 'utf8')).toBe(fixture.dependencySource);
        expect((await readdir(dirname(paths.ownerPath))).some((name) => name.includes('.zui-apply-'))).toBe(true);
      } finally {
        ownerWriteFaults.corruptTemporary = false;
      }
    });
  });

  it('restores the owner when a dependency changes after replacement and preserves that change', async () => {
    await withCanonicalFixture(async (fixture, paths) => {
      const changed = fixture.dependencySource + '\n# changed during owner application';
      ownerWriteFaults.changeDependencyAfterRename = {
        ownerPath: paths.ownerPath, dependencyPath: paths.componentAssetPath, source: changed,
      };
      try {
        await expect(applyZuiExportFileToCanonicalSourceOwners({
          ...paths, expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource), writableSourcePaths: [sourcePath],
        })).rejects.toThrow(/stale|fingerprint/i);
        expect(await readFile(paths.ownerPath, 'utf8')).toBe(fixture.ownerSource);
        expect(await readFile(paths.componentAssetPath, 'utf8')).toBe(changed);
        expect((await readdir(dirname(paths.ownerPath))).some((name) => name.includes('.zui-apply-'))).toBe(false);
      } finally {
        ownerWriteFaults.changeDependencyAfterRename = undefined;
      }
    });
  });
});

describe('applyZuiExportFileToSourceRoot', () => {
  it('routes a Penpot edit from an expanded node to its authored owner and preserves source contracts', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-owner-apply-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const ownerPath = join(sourceRootPath, sourcePath);
    const componentAssetPath = join(sourceRootPath, componentPath);
    const fixture = expandedExport();

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(ownerPath, fixture.ownerSource, 'utf8');
      await writeFile(componentAssetPath, fixture.dependencySource, 'utf8');
      await writeFile(exportedPath, fixture.exportedSource, 'utf8');

      const result = await applyZuiExportFileToSourceRoot({
        exportedPath,
        sourceRootPath,
        outputRootPath,
        expectedSources: fixture.sources,
        expectedRootSha256: sha256(fixture.ownerSource),
      });
      const appliedSource = await readFile(join(outputRootPath, sourcePath), 'utf8');
      const applied = parseZuiDocument(appliedSource).document;
      const original = parseZuiDocument(fixture.ownerSource).document;

      expect(
        (applied.nodes!.root.layout!['container'] as Record<string, unknown>)[
          'gap'
        ],
      ).toBe(fixture.before + 2);
      expect(applied.asset.id).toBe(original.asset.id);
      expect(applied.nodes!.root.control_id).toBe(
        original.nodes!.root.control_id,
      );
      expect(applied.nodes!.root.events).toEqual(original.nodes!.root.events);
      expect(applied.nodes!.root.children).toEqual(original.nodes!.root.children);
      expect(applied.nodes!.root['zircon_extension']).toEqual(
        original.nodes!.root['zircon_extension'],
      );
      expect(applied.imports).toEqual(original.imports);
      expect(applied.nodes!.submit.events).toEqual(original.nodes!.submit.events);
      expect(appliedSource).toContain('# Workbench owner comment survives.');
      expect(await readFile(ownerPath, 'utf8')).toBe(fixture.ownerSource);
      expect(result.outputs.map(({ sourcePath: path }) => path)).toEqual([
        sourcePath,
      ]);
      expect(result.appliedPathsBySource[sourcePath]).toEqual([
        'nodes.root.layout.container.gap',
      ]);
      expect(result.appliedPathsBySource[componentPath]).toBeUndefined();
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('routes a full-host edit on an imported component to the owning component asset', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-component-owner-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const ownerPath = join(sourceRootPath, sourcePath);
    const componentAssetPath = join(sourceRootPath, componentPath);
    const fixture = expandedExport({
      sourcePath: componentPath,
      sourceNodeId: 'root',
      exportNodeId: 'expanded_button_instance',
      instancePath:
        '[{"sourcePath":"zircon_editor/assets/ui/workbench_window.zui","sourceNodeId":"submit"}]',
    });

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(ownerPath, fixture.ownerSource, 'utf8');
      await writeFile(componentAssetPath, fixture.dependencySource, 'utf8');
      await writeFile(exportedPath, fixture.exportedSource, 'utf8');

      const result = await applyZuiExportFileToSourceRoot({
        exportedPath,
        sourceRootPath,
        outputRootPath,
        expectedSources: fixture.sources,
        expectedRootSha256: sha256(fixture.ownerSource),
      });
      const applied = parseZuiDocument(
        await readFile(join(outputRootPath, componentPath), 'utf8'),
      ).document;
      const original = parseZuiDocument(fixture.dependencySource).document;

      expect(
        (applied.nodes!.root.layout!['container'] as Record<string, unknown>)[
          'gap'
        ],
      ).toBe(fixture.before + 2);
      expect(applied.nodes!.root.events).toEqual(original.nodes!.root.events);
      expect(applied.nodes!.root.children).toEqual(original.nodes!.root.children);
      expect(applied.imports).toEqual(original.imports);
      expect(await readFile(ownerPath, 'utf8')).toBe(fixture.ownerSource);
      expect(result.appliedPathsBySource[componentPath]).toEqual([
        'nodes.root.layout.container.gap',
      ]);
      expect(result.appliedPathsBySource[sourcePath]).toBeUndefined();
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('rejects stale dependency sources before writing any staged output', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-stale-owner-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const fixture = expandedExport();

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        `${fixture.dependencySource}\n# changed after import`,
        'utf8',
      );
      await writeFile(exportedPath, fixture.exportedSource, 'utf8');

      await expect(
        applyZuiExportFileToSourceRoot({
          exportedPath,
          sourceRootPath,
          outputRootPath,
          expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource),
        }),
      ).rejects.toThrow('is stale compared with the pre-import fingerprint');
      await expect(readFile(join(outputRootPath, sourcePath), 'utf8')).rejects.toThrow();
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('requires the trusted root SHA to match the downloaded owner manifest', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-root-sha-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const fixture = expandedExport();

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        fixture.dependencySource,
        'utf8',
      );
      await writeFile(exportedPath, fixture.exportedSource, 'utf8');

      await expect(
        applyZuiExportFileToSourceRoot({
          exportedPath,
          sourceRootPath,
          outputRootPath,
          expectedSources: fixture.sources,
          expectedRootSha256: '0'.repeat(64),
        }),
      ).rejects.toThrow('expected root SHA-256 does not match');
      await expect(readFile(join(outputRootPath, sourcePath), 'utf8')).rejects.toThrow();
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('rejects a forged owner identity that differs from the imported host projection', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-forged-owner-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const fixture = expandedExport();
    const exported = parseZuiDocument(fixture.exportedSource).document;
    const metadata = JSON.parse(
      exported['penpot_source_exports'] as string,
    ) as ZuiSourceExportManifest;
    metadata.identityMap.find(
      ({ exportNodeId }) => exportNodeId === 'expanded_window_content',
    )!.sourcePath = componentPath;
    exported['penpot_source_exports'] = JSON.stringify(metadata);

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        fixture.dependencySource,
        'utf8',
      );
      await writeFile(exportedPath, serializeZuiDocument(exported), 'utf8');

      await expect(
        applyZuiExportFileToSourceRoot({
          exportedPath,
          sourceRootPath,
          outputRootPath,
          expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource),
        }),
      ).rejects.toThrow('does not match the imported projection identity');
      await expect(readFile(join(outputRootPath, componentPath), 'utf8')).rejects.toThrow();
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('rejects visual edits that have no owner record', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-unmapped-owner-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const fixture = expandedExport();
    const exported = parseZuiDocument(fixture.exportedSource).document;
    const metadata = JSON.parse(
      exported['penpot_source_exports'] as string,
    ) as Record<string, unknown>;
    metadata['edits'] = [];
    exported['penpot_source_exports'] = JSON.stringify(metadata);

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        fixture.dependencySource,
        'utf8',
      );
      await writeFile(exportedPath, serializeZuiDocument(exported), 'utf8');

      await expect(
        applyZuiExportFileToSourceRoot({
          exportedPath,
          sourceRootPath,
          outputRootPath,
          expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource),
        }),
      ).rejects.toThrow('explicitly mapped visual edit');
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('rejects a changed event contract in the exported host projection', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-event-owner-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const fixture = expandedExport();
    const exported = parseZuiDocument(fixture.exportedSource).document;
    const metadata = JSON.parse(
      exported['penpot_source_exports'] as string,
    ) as ZuiSourceExportManifest;
    const host = parseZuiDocument(metadata.exportedProjection).document;
    host.nodes!.expanded_window_content.events![0]!.route =
      'foreign.route.change';
    metadata.exportedProjection = serializeZuiDocument(host);
    exported['penpot_source_exports'] = JSON.stringify(metadata);

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        fixture.dependencySource,
        'utf8',
      );
      await writeFile(exportedPath, serializeZuiDocument(exported), 'utf8');

      await expect(
        applyZuiExportFileToSourceRoot({
          exportedPath,
          sourceRootPath,
          outputRootPath,
          expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource),
        }),
      ).rejects.toThrow('Unsupported nonvisual export change');
      await expect(readFile(join(outputRootPath, sourcePath), 'utf8')).rejects.toThrow();
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('rejects conflicting instance edits to the same source field', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-instance-conflict-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const fixture = expandedExport();
    const exported = parseZuiDocument(fixture.exportedSource).document;
    const metadata = JSON.parse(
      exported['penpot_source_exports'] as string,
    ) as ZuiSourceExportManifest;
    const baseline = parseZuiDocument(metadata.baselineProjection).document;
    const secondBaselineNode = clone(baseline.nodes!.expanded_window_content);
    secondBaselineNode.children = [];
    secondBaselineNode['penpot_review_instance_path'] =
      '[{"sourcePath":"zircon_editor/assets/ui/workbench_window.zui","sourceNodeId":"cancel"}]';
    baseline.nodes!['second_expanded_instance'] = secondBaselineNode;
    const secondNode = clone(secondBaselineNode);
    const secondContainer = secondNode.layout!['container'] as Record<
      string,
      unknown
    >;
    secondContainer['gap'] = Number(secondContainer['gap']) + 3;
    const exportedProjection = parseZuiDocument(
      metadata.exportedProjection,
    ).document;
    exportedProjection.nodes!['second_expanded_instance'] = secondNode;
    metadata.baselineProjection = serializeZuiDocument(baseline);
    metadata.exportedProjection = serializeZuiDocument(exportedProjection);
    metadata.identityMap.push({
      ...metadata.identityMap.find(
        ({ exportNodeId }) => exportNodeId === 'expanded_window_content',
      )!,
      exportNodeId: 'second_expanded_instance',
      instancePath:
        '[{"sourcePath":"zircon_editor/assets/ui/workbench_window.zui","sourceNodeId":"cancel"}]',
    });
    metadata.edits.push({
      ...metadata.edits[0],
      exportNodeId: 'second_expanded_instance',
      instancePath:
        '[{"sourcePath":"zircon_editor/assets/ui/workbench_window.zui","sourceNodeId":"cancel"}]',
      value: Number(secondContainer['gap']),
    });
    exported['penpot_source_exports'] = JSON.stringify(metadata);

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        fixture.dependencySource,
        'utf8',
      );
      await writeFile(exportedPath, serializeZuiDocument(exported), 'utf8');

      await expect(
        applyZuiExportFileToSourceRoot({
          exportedPath,
          sourceRootPath,
          outputRootPath,
          expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource),
        }),
      ).rejects.toThrow('Conflicting Penpot instances');
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('rejects source path traversal before writing outside the explicit source root', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-path-owner-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const fixture = expandedExport({ sourcePath: '../outside.zui' });

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        fixture.dependencySource,
        'utf8',
      );
      await writeFile(exportedPath, fixture.exportedSource, 'utf8');

      await expect(
        applyZuiExportFileToSourceRoot({
          exportedPath,
          sourceRootPath,
          outputRootPath,
          expectedSources: fixture.sources,
          expectedRootSha256: sha256(fixture.ownerSource),
        }),
      ).rejects.toThrow('Source path must be a repository-relative UI asset');
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('returns no outputs for a byte-identical export with no source-export envelope', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-noop-owner-'));
    const exportedPath = join(directory, 'penpot-download.zui');
    const source = `# Original comments\r\n${fixtureSource}`;

    try {
      await writeFile(exportedPath, source, 'utf8');
      const result = await applyZuiExportFileToSourceRoot({
        exportedPath,
        sourceRootPath: join(directory, 'canonical'),
        outputRootPath: join(directory, 'staged'),
        expectedSources: [],
        expectedRootSha256: sha256(source),
      });
      expect(result.appliedPathsBySource).toEqual({});
      expect(result.outputs).toEqual([]);
      expect(await readFile(exportedPath, 'utf8')).toBe(source);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('runs the owner-aware CLI with a downloaded .zui and explicit roots', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-owner-cli-'));
    const sourceRootPath = join(directory, 'canonical');
    const outputRootPath = join(directory, 'staged');
    const exportedPath = join(directory, 'penpot-download.zui');
    const expectedSourcesPath = join(directory, 'expected-sources.json');
    const fixture = expandedExport();

    try {
      await mkdir(join(sourceRootPath, 'zircon_editor/assets/ui/editor/components'), {
        recursive: true,
      });
      await writeFile(join(sourceRootPath, sourcePath), fixture.ownerSource, 'utf8');
      await writeFile(
        join(sourceRootPath, componentPath),
        fixture.dependencySource,
        'utf8',
      );
      await writeFile(exportedPath, fixture.exportedSource, 'utf8');
      await writeFile(expectedSourcesPath, JSON.stringify(fixture.sources), 'utf8');
      const result = spawnSync(
        process.execPath,
        [
          '--import',
          'tsx',
          resolve(pluginRoot, 'src/cli.ts'),
          'apply-export-sources',
          exportedPath,
          sourceRootPath,
          outputRootPath,
          '--expected-root-sha256',
          sha256(fixture.ownerSource),
          '--expected-sources',
          expectedSourcesPath,
        ],
        { cwd: pluginRoot, encoding: 'utf8' },
      );

      expect(result.error).toBeUndefined();
      expect(result.status, result.stderr).toBe(0);
      expect(result.stdout).toContain(
        `${sourcePath}: nodes.root.layout.container.gap`,
      );
      expect(
        (parseZuiDocument(await readFile(join(outputRootPath, sourcePath), 'utf8'))
          .document.nodes!.root.layout!['container'] as Record<string, unknown>)[
          'gap'
        ],
      ).toBe(fixture.before + 2);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  }, 30000);
});

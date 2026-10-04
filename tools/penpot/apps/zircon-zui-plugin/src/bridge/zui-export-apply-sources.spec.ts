import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import fixtureSource from './roundtrip-fixture.zui?raw';
import {
  applyZuiExportFileToSourceRoot,
  type ZuiSourceExportManifest,
  type ZuiSourceFingerprint,
} from './zui-export-apply';
import { parseZuiDocument, serializeZuiDocument } from './zui-document';

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

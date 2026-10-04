import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import fixtureSource from './roundtrip-fixture.zui?raw';
import {
  applyZuiExportFileToCanonicalSource,
  applyZuiExportToCanonicalSource,
} from './zui-export-apply';
import { parseZuiDocument, serializeZuiDocument } from './zui-document';

const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
const pluginRoot = fileURLToPath(new URL('../..', import.meta.url));
const workbenchSourcePath =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';

function sha256(source: string): string {
  return createHash('sha256').update(source, 'utf8').digest('hex');
}

function canonicalWithComments(): string {
  return `# Canonical file note must survive.\r\n${fixtureSource.replace(
    'events = [{ id = "Roundtrip/Changed"',
    '# Event contract remains source-owned.\r\nevents = [{ id = "Roundtrip/Changed"',
  )}`;
}

describe('applyZuiExportToCanonicalSource', () => {
  it('applies supported visual edits while preserving source contracts and comments', () => {
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    exportedDocument.nodes!['root'].layout!['container'] = {
      ...(exportedDocument.nodes!['root'].layout!['container'] as Record<
        string,
        unknown
      >),
      gap: 18.0,
    };
    exportedDocument.nodes!['root'].props!['border_color'] = '#667788';
    exportedDocument.nodes!['title'].props!['font_size'] = 20.0;
    exportedDocument.nodes!['title'].props!['font_family'] = 'Inter';
    const exported = serializeZuiDocument(exportedDocument);

    const result = applyZuiExportToCanonicalSource(exported, canonical, {
      expectedCanonicalSha256: sha256(canonical),
    });
    const applied = parseZuiDocument(result.source).document;
    const original = parseZuiDocument(canonical).document;

    expect(applied.nodes!['root'].layout!['container']).toMatchObject({
      kind: 'VerticalBox',
      gap: 18,
    });
    expect(applied.nodes!['root'].props!['border_color']).toBe('#667788');
    expect(applied.nodes!['title'].props).toMatchObject({
      font_size: 20,
      font_family: 'Inter',
    });
    expect(applied.nodes!['root'].events).toEqual(
      original.nodes!['root'].events,
    );
    expect(applied.nodes!['submit'].events).toEqual(
      original.nodes!['submit'].events,
    );
    expect(applied.nodes!['virtual_rows'].repeat).toEqual(
      original.nodes!['virtual_rows'].repeat,
    );
    expect(applied.nodes!['root'].children).toEqual(
      original.nodes!['root'].children,
    );
    expect(applied.imports).toEqual(original.imports);
    expect(applied.nodes!['root']['zircon_extension']).toEqual(
      original.nodes!['root']['zircon_extension'],
    );
    expect(result.source).toContain('# Canonical file note must survive.');
    expect(result.source).toContain('# Event contract remains source-owned.');
    expect(result.appliedPaths).toEqual(
      expect.arrayContaining([
        'nodes.root.layout.container.gap',
        'nodes.root.props.border_color',
        'nodes.title.props.font_size',
        'nodes.title.props.font_family',
      ]),
    );
  });

  it('applies a spacing edit against the canonical workbench source in memory', async () => {
    const canonical = await readFile(resolve(repo, workbenchSourcePath), 'utf8');
    const canonicalDocument = parseZuiDocument(canonical).document;
    const exportedDocument = parseZuiDocument(canonical).document;
    const target = Object.entries(exportedDocument.nodes ?? {}).find(
      ([, node]) => {
        const container = node.layout?.['container'];
        return (
          container !== null &&
          typeof container === 'object' &&
          !Array.isArray(container) &&
          !(container instanceof Date) &&
          typeof container['gap'] === 'number'
        );
      },
    );
    expect(target).toBeDefined();
    const [nodeId, node] = target!;
    const container = node.layout!['container'] as Record<string, unknown>;
    const beforeGap = container['gap'] as number;
    container['gap'] = beforeGap + 1;

    const result = applyZuiExportToCanonicalSource(
      serializeZuiDocument(exportedDocument),
      canonical,
      { expectedCanonicalSha256: sha256(canonical) },
    );
    const applied = parseZuiDocument(result.source).document;

    expect(
      (applied.nodes![nodeId].layout!['container'] as Record<string, unknown>)[
        'gap'
      ],
    ).toBe(beforeGap + 1);
    expect(applied.asset).toEqual(canonicalDocument.asset);
    for (const [id, originalNode] of Object.entries(
      canonicalDocument.nodes ?? {},
    )) {
      expect(applied.nodes![id].events).toEqual(originalNode.events);
      expect(applied.nodes![id].children).toEqual(originalNode.children);
    }
    expect(result.appliedPaths).toContain(
      `nodes.${nodeId}.layout.container.gap`,
    );
  });

  it('returns the original bytes on a no-edit export', () => {
    const canonical = canonicalWithComments();

    const result = applyZuiExportToCanonicalSource(canonical, canonical, {
      expectedCanonicalSha256: sha256(canonical),
    });

    expect(result.source).toBe(canonical);
    expect(result.appliedPaths).toEqual([]);
  });

  it('applies a mapped style self font edit without rewriting neighboring fields', () => {
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    exportedDocument.nodes!['title'].style = {
      self: { font: { size: 22.0, family: 'Inter' } },
    };

    const result = applyZuiExportToCanonicalSource(
      serializeZuiDocument(exportedDocument),
      canonical,
      { expectedCanonicalSha256: sha256(canonical) },
    );
    const applied = parseZuiDocument(result.source).document;

    expect(applied.nodes!['title'].style).toEqual(exportedDocument.nodes!['title'].style);
    expect(applied.nodes!['title'].props).toEqual(
      parseZuiDocument(canonical).document.nodes!['title'].props,
    );
    expect(result.source).toContain('# Event contract remains source-owned.');
    expect(result.appliedPaths).toEqual([
      'nodes.title.style.self.font.size',
      'nodes.title.style.self.font.family',
    ]);
  });

  it('reads a downloaded .zui and writes an explicit output without changing the canonical file', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-export-apply-'));
    const exportedPath = join(directory, 'penpot-download.zui');
    const canonicalPath = join(directory, 'canonical.zui');
    const outputPath = join(directory, 'applied.zui');
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    (exportedDocument.nodes!['root'].layout!['container'] as Record<
      string,
      unknown
    >)['gap'] = 19.0;

    try {
      await writeFile(canonicalPath, canonical, 'utf8');
      await writeFile(
        exportedPath,
        serializeZuiDocument(exportedDocument),
        'utf8',
      );
      const result = await applyZuiExportFileToCanonicalSource({
        exportedPath,
        canonicalPath,
        outputPath,
        expectedCanonicalSha256: sha256(canonical),
      });

      expect(await readFile(outputPath, 'utf8')).toBe(result.source);
      expect(await readFile(canonicalPath, 'utf8')).toBe(canonical);
      expect(result.appliedPaths).toContain(
        'nodes.root.layout.container.gap',
      );
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  });

  it('runs the apply-export CLI against a downloaded .zui file', async () => {
    const directory = await mkdtemp(join(tmpdir(), 'zui-export-apply-cli-'));
    const exportedPath = join(directory, 'penpot-download.zui');
    const canonicalPath = join(directory, 'canonical.zui');
    const outputPath = join(directory, 'applied.zui');
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    (exportedDocument.nodes!['root'].layout!['container'] as Record<
      string,
      unknown
    >)['gap'] = 21.0;

    try {
      await writeFile(canonicalPath, canonical, 'utf8');
      await writeFile(
        exportedPath,
        serializeZuiDocument(exportedDocument),
        'utf8',
      );
      const result = spawnSync(
        process.execPath,
        [
          '--import',
          'tsx',
          resolve(pluginRoot, 'src/cli.ts'),
          'apply-export',
          exportedPath,
          canonicalPath,
          outputPath,
          '--expected-sha256',
          sha256(canonical),
        ],
        { cwd: pluginRoot, encoding: 'utf8' },
      );

      expect(result.error).toBeUndefined();
      expect(result.status).toBe(0);
      expect(result.stdout).toContain('nodes.root.layout.container.gap');
      expect(
        (parseZuiDocument(await readFile(outputPath, 'utf8')).document.nodes![
          'root'
        ].layout!['container'] as Record<string, unknown>)['gap'],
      ).toBe(21);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  }, 30000);

  it('rejects a stale canonical source hash before applying edits', () => {
    const canonical = canonicalWithComments();

    expect(() =>
      applyZuiExportToCanonicalSource(canonical, canonical, {
        expectedCanonicalSha256: '0'.repeat(64),
      }),
    ).toThrow('Canonical source hash changed');
  });

  it('rejects event edits instead of replacing runtime behavior', () => {
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    exportedDocument.nodes!['root'].events![0]['route'] =
      'tests.roundtrip.changed_elsewhere';

    expect(() =>
      applyZuiExportToCanonicalSource(
        serializeZuiDocument(exportedDocument),
        canonical,
        { expectedCanonicalSha256: sha256(canonical) },
      ),
    ).toThrow('Unsupported nonvisual export change');
  });

  it('rejects layout changes outside mapped visual fields', () => {
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    const container = exportedDocument.nodes!['root'].layout![
      'container'
    ] as Record<string, unknown>;
    container['runtime_policy'] = 'replace';

    expect(() =>
      applyZuiExportToCanonicalSource(
        serializeZuiDocument(exportedDocument),
        canonical,
        { expectedCanonicalSha256: sha256(canonical) },
      ),
    ).toThrow('Unsupported nonvisual export change');
  });

  it('rejects added expanded nodes without a canonical source mapping', () => {
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    exportedDocument.nodes!['expanded_internal'] = {
      component: 'Label',
      props: { text: 'Generated internal' },
    };

    expect(() =>
      applyZuiExportToCanonicalSource(
        serializeZuiDocument(exportedDocument),
        canonical,
        { expectedCanonicalSha256: sha256(canonical) },
      ),
    ).toThrow('Expanded prefab node edit requires an explicit source mapping');
  });

  it('rejects an asset identity change', () => {
    const canonical = canonicalWithComments();
    const exportedDocument = parseZuiDocument(canonical).document;
    exportedDocument.asset.id = 'res://different.zui';

    expect(() =>
      applyZuiExportToCanonicalSource(
        serializeZuiDocument(exportedDocument),
        canonical,
        { expectedCanonicalSha256: sha256(canonical) },
      ),
    ).toThrow('Asset identity changed');
  });
});

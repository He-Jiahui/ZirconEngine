import { mkdir, mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { beforeAll, describe, expect, it } from 'vitest';
import { chromium } from 'playwright';
import type {
  CatalogEntry,
  CatalogManifest,
} from '../tools/zui-layout-catalog';
import {
  defaultReviewCases,
  caseSha256,
  requiredNativeRenderer,
} from '../tools/zui-layout-review-contract';
import {
  bytesSha256,
  dependenciesSha256,
  dualEvidenceErrors,
  currentDualReview,
  fileSha256,
  missingPenpotCaptureProgramPaths,
  selectedBrowserExecutablePath,
  verifyCurrentFiles,
} from '../tools/zui-layout-evidence';
import { recordReview } from '../tools/zui-layout-review';
import { compareSemanticGeometry } from '../tools/zui-layout-semantic-parity';
import { compareTextEvidence } from '../tools/zui-layout-text-parity';
import { importEngineReport } from '../tools/zui-layout-import-engine';
import { parseZuiDocument } from './bridge/zui-document';
import { sourceScenarioCoverageErrors } from '../tools/zui-layout-source-scenario-coverage';
import type { TextStructureNode } from '../tools/zui-layout-text-structure';
import {
  assertCheckpointSources,
  archiveReview,
  mergeCheckpointReviews,
} from '../tools/zui-layout-review-history';

const reviewCase = defaultReviewCases('source.zui', 'component')[0];
const fingerprint = bytesSha256('file');
const captureProgramPaths = missingPenpotCaptureProgramPaths([]);
let browserRuntime: {
  product: string;
  userAgent: string;
  executablePath: string;
  executableSha256: string;
};

beforeAll(async () => {
  const executablePath = selectedBrowserExecutablePath();
  const browser = await chromium.launch({ executablePath, headless: true });
  try {
    const page = await browser.newPage();
    const session = await page.context().newCDPSession(page);
    const version = await session.send('Browser.getVersion');
    browserRuntime = {
      product: version.product,
      userAgent: version.userAgent,
      executablePath,
      executableSha256: await fileSha256(executablePath),
    };
  } finally {
    await browser.close();
  }
});

async function writeCaptureProgramFixtures(root: string): Promise<void> {
  await mkdir(
    resolve(root, 'tools/penpot/apps/zircon-zui-plugin/tools'),
    { recursive: true },
  );
  await Promise.all(
    captureProgramPaths.map((path) => writeFile(resolve(root, path), 'file')),
  );
}

const entryFixture = (): CatalogEntry => {
  const evidence = {
    caseId: reviewCase.id,
    status: 'passed' as const,
    screenshotPath: 'image.png',
    screenshotSha256: fingerprint,
    geometryPath: 'geometry.json',
    geometrySha256: fingerprint,
    textPath: 'text.json',
    textSha256: fingerprint,
    rendererPath: 'renderer.js',
    rendererSha256: fingerprint,
    inputSha256: fingerprint,
    sourceSha256: fingerprint,
    dependencySha256: dependenciesSha256([]),
    caseSha256: caseSha256(reviewCase),
  };
  return {
    sourcePath: 'source.zui',
    sourceSha256: fingerprint,
    outputPath: 'output.zui',
    outputSha256: fingerprint,
    previewPath: 'image.png',
    penpotPreviewPath: 'image.png',
    penpotInputPath: 'input.zui',
    penpotInputSha256: fingerprint,
    dependencyFingerprints: [],
    dependencySha256: dependenciesSha256([]),
    status: 'prepared',
    cases: [reviewCase],
    penpotEvidence: [
      {
        ...evidence,
        rendererKind: 'penpot-official-frontend',
        browserRuntime,
        captureProgramFingerprints: captureProgramPaths.map(
          (path) => [path, fingerprint] as [string, string],
        ),
      },
    ],
    engineEvidence: [
      { ...evidence, rendererKind: 'zircon-runtime-wgpu-headless' },
    ],
  } as unknown as CatalogEntry;
};
const conclusions = [
  {
    caseId: reviewCase.id,
    status: 'accepted' as const,
    observations: 'Text and shapes align without clipping.',
    maxGeometryDeltaPx: 0,
    textMatches: true,
    visibilityMatches: true,
  },
];
const geometry = (): {
  case: typeof reviewCase;
  layout: { semanticNodes: TextStructureNode[] };
} => ({
  case: reviewCase,
  layout: {
    semanticNodes: [
      {
        nodeId: 'label',
        shapeId: 'label-shape',
        parentNodeId: null,
        component: 'Label',
        text: 'Hello',
        visible: true,
        detached: false,
        bounds: { x: 10, y: 20, width: 80, height: 30 },
      },
    ],
  },
});

describe('fail-closed dual evidence acceptance', () => {
  it('requires capture program provenance in addition to the plugin bundle', () => {
    const entry = entryFixture();
    delete entry.penpotEvidence![0].captureProgramFingerprints;
    expect(dualEvidenceErrors(entry)).toContain(
      `penpotEvidence: missing capture program fingerprint ${reviewCase.id}`,
    );
  });
  it('refuses a capture checkpoint after source or scene regeneration while allowing reviews', () => {
    const capturing = { entries: [entryFixture()] } as CatalogManifest;
    const disk = structuredClone(capturing);
    disk.entries[0].visualStatus = 'failed';
    recordReview(disk.entries[0], {
      sourcePath: 'source.zui',
      decision: 'needs_revision',
      observations: 'Missing native image.',
    });
    expect(() => assertCheckpointSources(capturing, disk)).not.toThrow();
    disk.entries[0].sourceSha256 = bytesSha256('new source');
    expect(() => assertCheckpointSources(capturing, disk)).toThrow(
      'checkpoint refused',
    );
    disk.entries[0].sourceSha256 = capturing.entries[0].sourceSha256;
    disk.entries[0].cases![0].viewport.width += 1;
    expect(() => assertCheckpointSources(capturing, disk)).toThrow(
      'checkpoint refused',
    );
  });
  it.each([
    [
      'zircon_editor/assets/ui/editor/components/button.zui',
      'component',
      'zircon-editor-retained-host',
    ],
    [
      'zircon_plugins/assets/ui/plugin_panel.zui',
      'view',
      'zircon-editor-retained-host',
    ],
    [
      'zircon_editor/assets/ui/editor/theme/tokens.zui',
      'theme_tokens',
      'zircon-editor-retained-host',
    ],
    ['examples/woc/assets/ui/menu.zui', 'view', 'zircon-runtime-wgpu-headless'],
    [
      'zircon_runtime/assets/ui/runtime/fixtures/button.zui',
      'view',
      'zircon-runtime-wgpu-headless',
    ],
    [
      'zircon_editor/tests/fixtures/button.zui',
      'view',
      'zircon-runtime-wgpu-headless',
    ],
  ])('requires the product host for %s', (sourcePath, kind, rendererKind) => {
    expect(
      requiredNativeRenderer(defaultReviewCases(sourcePath, kind)[0]),
    ).toBe(rendererKind);
  });

  it.each([undefined, 'zircon-editor-retained-host', 'unverified-screenshot'])(
    'rejects a missing or wrong native renderer even with current evidence hashes: %s',
    (rendererKind) => {
      const entry = entryFixture();
      recordReview(entry, {
        sourcePath: entry.sourcePath,
        decision: 'accepted',
        observations: 'Reviewed both renderers.',
        caseReviews: conclusions,
      });
      entry.engineEvidence![0].rendererKind = rendererKind;
      expect(dualEvidenceErrors(entry).join(' ')).toContain(
        'wrong native host',
      );
      expect(currentDualReview(entry)).toBe(false);
    },
  );

  it('refuses Runtime evidence for an Editor product before touching evidence files or history', async () => {
    const entry = entryFixture();
    entry.sourcePath = 'zircon_editor/assets/ui/editor/components/button.zui';
    entry.cases = [{ ...reviewCase, sourcePath: entry.sourcePath }];
    const before = structuredClone(entry.engineEvidence);
    const item = {
      ...entry.engineEvidence![0],
      sourcePath: entry.sourcePath,
      caseSha256: caseSha256(entry.cases[0]),
    };
    await expect(
      importEngineReport(
        { entries: [entry], repoRoot: '.' } as CatalogManifest,
        {
          schema: 'dev.zircon.zui.native-evidence',
          version: 1,
          rendererSha256: fingerprint,
          evidence: [item],
        },
        '.',
      ),
    ).rejects.toThrow('Wrong native host');
    expect(entry.engineEvidence).toEqual(before);
  });

  it('archives failed recapture conclusions without duplicating history', () => {
    const entry = entryFixture();
    recordReview(entry, {
      sourcePath: entry.sourcePath,
      decision: 'needs_revision',
      observations: 'Original image has a clipped label.',
    });
    const disk = structuredClone(entry);
    entry.visualStatus = 'failed';
    archiveReview(entry);
    mergeCheckpointReviews(entry, disk);
    mergeCheckpointReviews(entry, disk);
    expect(entry.review).toBeUndefined();
    expect(entry.reviewHistory).toEqual([disk.review]);
  });

  it('does not restore a review when a secondary evidence file changed but the primary image did not', () => {
    const entry = entryFixture();
    entry.visualStatus = 'passed';
    recordReview(entry, {
      sourcePath: entry.sourcePath,
      decision: 'accepted',
      observations: 'Reviewed both renderers.',
      caseReviews: conclusions,
    });
    const disk = structuredClone(entry);
    entry.penpotEvidence![0].textSha256 = bytesSha256('new text geometry');
    mergeCheckpointReviews(entry, disk);
    expect(entry.review).toBeUndefined();
    expect(entry.reviewHistory).toEqual([disk.review]);
  });

  it('retains a concurrent conclusion only when its complete evidence fingerprint matches', () => {
    const entry = entryFixture();
    entry.visualStatus = 'passed';
    const disk = structuredClone(entry);
    recordReview(disk, {
      sourcePath: disk.sourcePath,
      decision: 'accepted',
      observations: 'All current evidence reviewed.',
      caseReviews: conclusions,
    });
    mergeCheckpointReviews(entry, disk);
    expect(entry.review).toEqual(disk.review);
    expect(currentDualReview(entry)).toBe(true);
  });

  it.each([
    'rendererSha256',
    'rendererPath',
    'geometryPath',
    'geometrySha256',
    'textPath',
    'textSha256',
  ] as const)('rejects missing %s despite a valid PNG', (field) => {
    const entry = entryFixture();
    delete entry.penpotEvidence![0][field];
    expect(dualEvidenceErrors(entry).length).toBeGreaterThan(0);
  });
  it('rejects duplicate and unexpected evidence cases', () => {
    const entry = entryFixture();
    entry.engineEvidence!.push({ ...entry.engineEvidence![0] });
    expect(dualEvidenceErrors(entry).join(' ')).toContain('missing or failed');
    entry.engineEvidence![1].caseId = 'unrequested';
    expect(dualEvidenceErrors(entry).join(' ')).toContain('unexpected');
  });
  it('requires an individual conclusion for every case before mutating review history', () => {
    const entry = entryFixture();
    expect(() =>
      recordReview(entry, {
        sourcePath: entry.sourcePath,
        decision: 'accepted',
        observations: 'Reviewed.',
      }),
    ).toThrow('individual');
    expect(entry.review).toBeUndefined();
    expect(entry.reviewHistory).toBeUndefined();
    expect(() =>
      recordReview(entry, {
        sourcePath: entry.sourcePath,
        decision: 'accepted',
        observations: 'Reviewed.',
        caseReviews: [{ ...conclusions[0], maxGeometryDeltaPx: 1.01 }],
      }),
    ).toThrow('revision');
  });
  it.each(['geometrySha256', 'textSha256', 'rendererSha256'] as const)(
    'invalidates old decisions when %s changes with the PNG unchanged',
    (field) => {
      const entry = entryFixture();
      recordReview(entry, {
        sourcePath: entry.sourcePath,
        decision: 'accepted',
        observations: 'Reviewed.',
        caseReviews: conclusions,
      });
      expect(currentDualReview(entry)).toBe(true);
      entry.engineEvidence![0][field] = bytesSha256('changed');
      expect(currentDualReview(entry)).toBe(false);
    },
  );
  it('allows a failed capture without a PNG to receive revision while preserving failed provenance', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-failed-evidence-'));
    try {
      const entry = entryFixture();
      for (const path of [
        'source.zui',
        'output.zui',
        'input.zui',
        'renderer.js',
      ])
        await writeFile(resolve(root, path), 'file');
      await writeCaptureProgramFixtures(root);
      entry.engineEvidence = [];
      entry.penpotEvidence = [
        {
          ...entry.penpotEvidence![0],
          status: 'failed',
          error: 'Text font did not load',
          screenshotPath: '',
          screenshotSha256: '',
          geometryPath: undefined,
          geometrySha256: undefined,
          textPath: undefined,
          textSha256: undefined,
        },
      ];
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).resolves.toBeUndefined();
      recordReview(entry, {
        sourcePath: entry.sourcePath,
        decision: 'needs_revision',
        observations: 'Font loading failed before the screenshot.',
      });
      expect(currentDualReview(entry)).toBe(true);
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'missing or failed',
      );
      await writeFile(resolve(root, 'renderer.js'), 'changed');
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).rejects.toThrow('renderer.js');
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
  it('treats a real pending capture as current evidence but never as accepted', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-pending-evidence-'));
    try {
      const entry = entryFixture();
      for (const path of [
        'source.zui',
        'output.zui',
        'input.zui',
        'renderer.js',
        'image.png',
        'geometry.json',
        'text.json',
      ])
        await writeFile(resolve(root, path), 'file');
      await writeCaptureProgramFixtures(root);
      entry.engineEvidence = [
        {
          ...entry.engineEvidence![0],
          status: 'pending',
          pendingReason: 'Font/media readiness is not certified yet.',
        },
      ];
      expect(dualEvidenceErrors(entry).join(' ')).toContain('pending');
      recordReview(entry, {
        sourcePath: entry.sourcePath,
        decision: 'needs_revision',
        observations: 'Pending native capture requires visual review.',
      });
      expect(currentDualReview(entry)).toBe(false);
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).resolves.toBeUndefined();
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'pending',
      );
      entry.engineEvidence[0].pendingReason = '';
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).rejects.toThrow('Pending evidence has no explanation');
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
  it('imports pending native captures without promoting them to acceptance', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-pending-import-'));
    try {
      const entry = entryFixture();
      for (const path of [
        'source.zui',
        'output.zui',
        'input.zui',
        'renderer.js',
        'image.png',
        'geometry.json',
        'text.json',
      ])
        await writeFile(resolve(root, path), 'file');
      const item = {
        ...entry.engineEvidence![0],
        sourcePath: entry.sourcePath,
        status: 'pending' as const,
        pendingReason: 'Native capture awaits visual review.',
      };
      await expect(
        importEngineReport(
          { entries: [entry], repoRoot: root } as CatalogManifest,
          {
            schema: 'dev.zircon.zui.native-evidence',
            version: 1,
            rendererSha256: fingerprint,
            evidence: [item],
          },
          root,
        ),
      ).resolves.toBeUndefined();
      expect(entry.engineEvidence?.[0].status).toBe('pending');
      expect(currentDualReview(entry)).toBe(false);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
  it('refuses stale native report entries without partially mutating catalog evidence', async () => {
    const entry = entryFixture();
    const before = structuredClone(entry.engineEvidence);
    const item = {
      ...entry.engineEvidence![0],
      sourcePath: entry.sourcePath,
      rendererKind: 'zircon-runtime-wgpu-headless',
      sourceSha256: bytesSha256('stale'),
    };
    await expect(
      importEngineReport(
        { entries: [entry], repoRoot: '.' } as CatalogManifest,
        {
          schema: 'dev.zircon.zui.native-evidence',
          version: 1,
          rendererSha256: fingerprint,
          evidence: [item],
        },
        '.',
      ),
    ).rejects.toThrow('Stale');
    expect(entry.engineEvidence).toEqual(before);
  });
  it('rejects a changed review host even when screenshots and product source remain current', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-host-evidence-'));
    try {
      const entry = entryFixture();
      for (const path of [
        'source.zui',
        'output.zui',
        'input.zui',
        'renderer.js',
        'image.png',
        'geometry.json',
        'text.json',
        'host.zui',
      ])
        await writeFile(resolve(root, path), 'file');
      await writeCaptureProgramFixtures(root);
      entry.cases = [
        {
          ...reviewCase,
          reviewHost: { path: 'host.zui', sha256: fingerprint },
        },
      ];
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).resolves.toBeUndefined();
      await writeFile(resolve(root, 'host.zui'), 'changed consumer');
      await expect(
        verifyCurrentFiles(entry, root, root, false),
      ).rejects.toThrow('host.zui');
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
  it('checks geometry contents at acceptance and rehashes measured artifacts on disk', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-accepted-evidence-'));
    try {
      const entry = entryFixture();
      const inputSource = [
        '[asset]',
        'kind = "component"',
        'id = "res://ui/editor/components/fixture.zui"',
        'version = 2',
        '[components.Fixture]',
        'root = "root"',
        '[nodes.root]',
        'component = "Panel"',
      ].join('\n');
      const inputHash = bytesSha256(inputSource);
      entry.penpotInputSha256 = inputHash;
      entry.penpotEvidence![0].inputSha256 = inputHash;
      entry.engineEvidence![0].inputSha256 = entry.sourceSha256;
      for (const path of [
        'source.zui',
        'output.zui',
        'renderer.js',
        'image.png',
      ])
        await writeFile(resolve(root, path), 'file');
      await writeFile(resolve(root, 'input.zui'), inputSource);
      await writeCaptureProgramFixtures(root);
      const measured = geometry();
      measured.layout.semanticNodes[0].text = '';
      for (const kind of ['penpot', 'engine'] as const) {
        const evidence = entry[`${kind}Evidence`]![0];
        const geometryBytes = JSON.stringify(measured);
        const textBytes = JSON.stringify(
          kind === 'penpot'
            ? { fontAudit: { loaded: true }, texts: [] }
            : { case: reviewCase, coordinateSpace: 'logical', nodes: [] },
        );
        evidence.geometryPath = `${kind}.geometry.json`;
        evidence.geometrySha256 = bytesSha256(geometryBytes);
        evidence.textPath = `${kind}.text.json`;
        evidence.textSha256 = bytesSha256(textBytes);
        await writeFile(resolve(root, evidence.geometryPath), geometryBytes);
        await writeFile(resolve(root, evidence.textPath), textBytes);
      }
      await expect(
        verifyCurrentFiles(entry, root, root, true),
      ).resolves.toBeUndefined();
      entry.previewPath = 'missing-preview.png';
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'Missing engine primary preview',
      );
      entry.previewPath = 'image.png';
      measured.layout.semanticNodes[0].bounds.width += 2;
      const changed = JSON.stringify(measured);
      entry.engineEvidence![0].geometrySha256 = bytesSha256(changed);
      await writeFile(resolve(root, 'engine.geometry.json'), changed);
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'Geometry differs',
      );
      await writeFile(resolve(root, 'engine.text.json'), '{}');
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'engine.text.json',
      );
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  }, 30_000);
  it('rejects a Workbench table capture that omits source-declared cells', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-table-cell-evidence-'));
    try {
      const entry = entryFixture();
      const source = [
        '[asset]',
        'kind = "component"',
        'id = "res://ui/editor/components/workbench/primitives/data/workbench_table_row.zui"',
        'version = 2',
        '[components.WorkbenchTableRow]',
        'root = "root"',
        '[nodes.root]',
        'component = "Table"',
        'control_id = "WorkbenchTableRowRoot"',
        'props = { options = ["Item_01", "Mesh", "2.4 MB", "2m ago"] }',
      ].join('\n');
      const sourceHash = bytesSha256(source);
      entry.sourceKind = 'component';
      entry.sourceSha256 = sourceHash;
      entry.outputSha256 = sourceHash;
      entry.penpotInputSha256 = sourceHash;
      for (const path of ['source.zui', 'output.zui', 'input.zui'])
        await writeFile(resolve(root, path), source);
      for (const path of ['renderer.js', 'image.png'])
        await writeFile(resolve(root, path), 'file');
      const captured = geometry();
      captured.layout.semanticNodes = [
        {
          ...captured.layout.semanticNodes[0],
          nodeId: 'root',
          component: 'Table',
          text: '',
          textParts: [
            {
              shapeId: 'name',
              text: 'Item_01',
              bounds: { x: 0, y: 0, width: 50, height: 20 },
            },
            {
              shapeId: 'type',
              text: 'Mesh',
              bounds: { x: 60, y: 0, width: 40, height: 20 },
            },
          ],
        },
      ];
      const geometryBytes = JSON.stringify(captured);
      const penpotText = JSON.stringify({
        fontAudit: { loaded: true },
        texts: [],
      });
      const engineText = JSON.stringify({
        case: reviewCase,
        coordinateSpace: 'logical',
        nodes: [],
      });
      for (const [renderer, textBytes] of [
        ['penpot', penpotText],
        ['engine', engineText],
      ] as const) {
        const item = entry[`${renderer}Evidence`]![0];
        item.sourceSha256 = sourceHash;
        item.inputSha256 = sourceHash;
        item.geometryPath = `${renderer}.geometry.json`;
        item.geometrySha256 = bytesSha256(geometryBytes);
        item.textPath = `${renderer}.text.json`;
        item.textSha256 = bytesSha256(textBytes);
        await writeFile(resolve(root, item.geometryPath), geometryBytes);
        await writeFile(resolve(root, item.textPath), textBytes);
      }
      await writeCaptureProgramFixtures(root);
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'missing rendered cells: 2.4 MB, 2m ago',
      );
      captured.layout.semanticNodes = [
        {
          ...captured.layout.semanticNodes[0],
          textParts: [
            {
              shapeId: 'name',
              text: 'Item_01',
              bounds: { x: 0, y: 0, width: 50, height: 20 },
            },
            {
              shapeId: 'type',
              text: 'Mesh',
              bounds: { x: 60, y: 0, width: 40, height: 20 },
            },
            {
              shapeId: 'size',
              text: '2.4 MB',
              bounds: { x: 100, y: 0, width: 50, height: 20 },
            },
            {
              shapeId: 'revision',
              text: '2m ago',
              bounds: { x: 160, y: 0, width: 50, height: 20 },
            },
          ],
        },
      ];
      const completeGeometry = JSON.stringify(captured);
      entry.penpotEvidence![0].geometrySha256 = bytesSha256(completeGeometry);
      await writeFile(resolve(root, 'penpot.geometry.json'), completeGeometry);
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'engine: root: missing rendered cells: 2.4 MB, 2m ago',
      );
      for (const renderer of ['penpot', 'engine'] as const) {
        const item = entry[`${renderer}Evidence`]![0];
        item.geometrySha256 = bytesSha256(completeGeometry);
        await writeFile(resolve(root, item.geometryPath!), completeGeometry);
      }
      await expect(
        verifyCurrentFiles(entry, root, root, true),
      ).resolves.toBeUndefined();
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  }, 30_000);
  it('requires invoked and closed scenes for a component with an authored popup switch', async () => {
    const root = await mkdtemp(resolve(tmpdir(), 'zui-popup-scene-evidence-'));
    try {
      const entry = entryFixture();
      const source = [
        '[asset]',
        'kind = "component"',
        'id = "res://ui/editor/components/workbench/primitives/inputs/workbench_dropdown.zui"',
        'version = 2',
        '[components.WorkbenchDropdown]',
        'root = "root"',
        '[nodes.root]',
        'component = "Dropdown"',
        'props = { popup_open = false, options = ["default"] }',
      ].join('\n');
      const sourceHash = bytesSha256(source);
      entry.sourceKind = 'component';
      entry.sourceSha256 = sourceHash;
      entry.outputSha256 = sourceHash;
      entry.penpotInputSha256 = sourceHash;
      for (const path of ['source.zui', 'output.zui', 'input.zui'])
        await writeFile(resolve(root, path), source);
      for (const path of ['renderer.js', 'image.png'])
        await writeFile(resolve(root, path), 'file');
      const captured = geometry();
      captured.layout.semanticNodes[0].nodeId = 'root';
      captured.layout.semanticNodes[0].component = 'Dropdown';
      captured.layout.semanticNodes[0].text = '';
      const geometryBytes = JSON.stringify(captured);
      for (const renderer of ['penpot', 'engine'] as const) {
        const item = entry[`${renderer}Evidence`]![0];
        item.sourceSha256 = sourceHash;
        item.inputSha256 = sourceHash;
        item.geometryPath = `${renderer}.geometry.json`;
        item.geometrySha256 = bytesSha256(geometryBytes);
        item.textPath = `${renderer}.text.json`;
        const textBytes =
          renderer === 'penpot'
            ? JSON.stringify({ fontAudit: { loaded: true }, texts: [] })
            : JSON.stringify({
                case: reviewCase,
                coordinateSpace: 'logical',
                nodes: [],
              });
        item.textSha256 = bytesSha256(textBytes);
        await writeFile(resolve(root, item.geometryPath), geometryBytes);
        await writeFile(resolve(root, item.textPath), textBytes);
      }
      await writeCaptureProgramFixtures(root);
      await expect(verifyCurrentFiles(entry, root, root, true)).rejects.toThrow(
        'root: missing popup review states: open, closed, focus-return',
      );
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
  it('accepts both explicit popup states without treating drag surfaces as popups', () => {
    const source = parseZuiDocument(
      '[asset]\nkind = "component"\nid = "res://review/popup.zui"\nversion = 2\n' +
        '[components.ReviewPopup]\nroot = "root"\n' +
        '[nodes.root]\ncomponent = "DropdownPopup"\nprops = { open = false, popup_open = false }\n',
    ).document;
    expect(
      sourceScenarioCoverageErrors(source, [
        { state: 'default' },
        { state: 'open' },
        { state: 'closed' },
        { state: 'focus-return' },
      ]),
    ).toEqual([]);
    source.nodes!['root'].component = 'DragOverlay';
    expect(
      sourceScenarioCoverageErrors(source, [{ state: 'default' }]),
    ).toEqual([]);
    source.nodes!['root'].component = 'DropdownPopup';
    source.nodes!['root'].props!['popup_open'] = 'false';
    expect(
      sourceScenarioCoverageErrors(source, [{ state: 'default' }]),
    ).toEqual(['root: unsupported popup switch value']);

    const page = parseZuiDocument(
      '[asset]\nkind = "view"\nid = "res://review/page.zui"\nversion = 2\n' +
        '[root]\nnode = "root"\n' +
        '[nodes.root]\ncomponent = "VerticalGroup"\nchildren = [{ node = "menu" }]\n' +
        '[nodes.menu]\ncomponent = "ContextMenu"\nprops = { popup_open = false }\n',
    ).document;
    expect(sourceScenarioCoverageErrors(page, [{ state: 'default' }])).toEqual([
      'menu: missing popup review states: open, closed, focus-return',
    ]);
  });
  it.each([
    ['zircon-runtime-wgpu-headless', 'source.zui'],
    ['zircon-editor-retained-host', 'zircon_editor/source.zui'],
    ['zircon-editor-retained-host', 'zircon_plugins/source.zui'],
  ])(
    'imports current %s failure reports for %s without blank-path reads',
    async (rendererKind, sourcePath) => {
      const root = await mkdtemp(resolve(tmpdir(), 'zui-native-import-'));
      try {
        const entry = entryFixture();
        if (rendererKind === 'zircon-editor-retained-host') {
          entry.sourcePath = sourcePath;
          entry.cases = entry.cases!.map((item) => ({
            ...item,
            sourcePath: entry.sourcePath,
          }));
          await mkdir(resolve(root, sourcePath.split('/')[0]));
          await writeFile(resolve(root, entry.sourcePath), 'file');
        }
        for (const path of [
          'source.zui',
          'output.zui',
          'input.zui',
          'renderer.js',
        ])
          await writeFile(resolve(root, path), 'file');
        const item = {
          ...entry.engineEvidence![0],
          sourcePath: entry.sourcePath,
          rendererKind,
          caseSha256: caseSha256(entry.cases![0]),
          status: 'failed' as const,
          error: 'Native glyph layout failed',
          screenshotPath: '',
          screenshotSha256: '',
          geometryPath: undefined,
          geometrySha256: undefined,
          textPath: undefined,
          textSha256: undefined,
        };
        await importEngineReport(
          { entries: [entry], repoRoot: root } as CatalogManifest,
          {
            schema: 'dev.zircon.zui.native-evidence',
            version: 1,
            rendererSha256: fingerprint,
            evidence: [item],
          },
          root,
        );
        expect(entry.engineEvidence).toHaveLength(1);
        expect(entry.engineEvidence![0].error).toBe(
          'Native glyph layout failed',
        );
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    },
  );
});

describe('measured semantic parity', () => {
  it('allows exactly one logical pixel and rejects greater drift', () => {
    const native = geometry();
    native.layout.semanticNodes[0].bounds.x += 1;
    expect(
      compareSemanticGeometry(geometry(), native, reviewCase).errors,
    ).toEqual([]);
    native.layout.semanticNodes[0].bounds.x += 0.01;
    expect(
      compareSemanticGeometry(geometry(), native, reviewCase).errors.join(' '),
    ).toContain('Geometry differs');
  });
  it('rejects missing nodes, changed semantics, and stale case geometry', () => {
    const native = geometry();
    native.layout.semanticNodes[0].text = 'Other';
    native.layout.semanticNodes[0].visible = false;
    expect(
      compareSemanticGeometry(geometry(), native, reviewCase).errors,
    ).toEqual(
      expect.arrayContaining([
        'Semantic text differs: label',
        'Visibility differs: label',
      ]),
    );
    native.layout.semanticNodes = [];
    expect(
      compareSemanticGeometry(geometry(), native, reviewCase).errors.join(' '),
    ).toContain('Unmatched');
    expect(
      compareSemanticGeometry(geometry(), geometry(), {
        ...reviewCase,
        dpi: 2,
      }).errors.join(' '),
    ).toContain('stale');
  });
  it('compares actual text-line rectangles and rejects missing native layouts', () => {
    const penpot = {
      fontAudit: { loaded: true },
      texts: [
        {
          shapeId: 'shape-internal-caption',
          ancestorShapeIds: ['shape-label-shape'],
          text: 'Hello',
          fontSize: '14px',
          fonts: [{ familyName: 'Fira Sans', glyphCount: 5 }],
          lines: [{ x: 10, y: 20, width: 40, height: 16 }],
        },
      ],
    };
    const native = {
      case: reviewCase,
      coordinateSpace: 'logical',
      nodes: [
        {
          nodeId: 'label',
          text: 'Hello',
          layout: {
            font_size: 14,
            lines: [
              { text: 'Hello', frame: { x: 10, y: 20, width: 40, height: 16 } },
            ],
          },
        },
      ],
    };
    expect(compareTextEvidence(penpot, native, geometry(), reviewCase)).toEqual(
      [],
    );
    native.nodes[0].layout.lines[0].frame.width += 2;
    expect(
      compareTextEvidence(penpot, native, geometry(), reviewCase).join(' '),
    ).toContain('Text geometry');
    expect(
      compareTextEvidence(
        penpot,
        { ...native, nodes: [] },
        geometry(),
        reviewCase,
      ).join(' '),
    ).toContain('Unmatched');
  });
});

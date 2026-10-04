import { createHash } from 'node:crypto';
import { readFile, mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
  vi,
} from 'vitest';
import type { CatalogEntry } from '../../tools/zui-layout-catalog';
import {
  dependenciesSha256,
  verifyCurrentFiles,
} from '../../tools/zui-layout-evidence';

const sourcePath =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
const inputPaths = [
  'zircon-project.toml',
  'assets/scenes/main.scene.toml',
  'assets/scenes/workbench-review-empty.scene.toml',
] as const;
const projectRootEnv = 'ZIRCON_EDITOR_WORKBENCH_REVIEW_PROJECT_ROOT';
const hash = (value: string) =>
  createHash('sha256').update(value).digest('hex');

describe('managed project scene freshness in an existing workbench catalog', () => {
  const temporaryRoot = resolve(tmpdir());
  let directory: string;
  let projectRoot: string;
  let entry: CatalogEntry;
  const originalInputs = [
    'name = "Workbench input"\n',
    'entities = []\n# main input\n',
    'entities = []\n',
  ];

  beforeAll(async () => {
    directory = await mkdtemp(resolve(temporaryRoot, 'workbench-scene-input-'));
    projectRoot = resolve(directory, 'managed-project');
    await mkdir(resolve(projectRoot, 'assets/scenes'), { recursive: true });
    await mkdir(dirname(resolve(directory, sourcePath)), { recursive: true });
    const source = 'schema = "dev.zircon.ui"\n';
    await writeFile(resolve(directory, sourcePath), source);
    await writeFile(resolve(directory, 'asset.zui'), source);
    entry = {
      sourcePath,
      sourceSha256: hash(source),
      outputPath: 'asset.zui',
      outputSha256: hash(source),
      resultPath: 'result.json',
      previewPath: 'preview.png',
      category: 'workbench',
      name: 'Managed inputs regression',
      sourceClassification: 'product-page',
      assetRoot: dirname(sourcePath),
      componentDependencies: [],
      resourceDependencies: [],
      sourceKind: 'view',
      sourceVersion: 2,
      outputKind: 'view',
      outputVersion: 2,
      sourceFormat: 'v2',
      status: 'prepared',
      nodeCount: 0,
      projectedShapeCount: 0,
      prefabNodeCount: 0,
      prefabCoverage: null,
      prefabRoleCounts: {},
      diagnostics: [],
      changes: [],
      visualStatus: 'failed',
      dependencyFingerprints: [],
      dependencySha256: dependenciesSha256([]),
      engineEvidence: [
        {
          caseId: 'default-1280x800-dpi1',
          screenshotPath: '',
          screenshotSha256: '',
          sourceSha256: hash(source),
          dependencySha256: dependenciesSha256([]),
          caseSha256: '',
          status: 'failed',
          error: 'test input without native capture',
        },
      ],
      cases: [
        {
          id: 'default-1280x800-dpi1',
          sourcePath,
          host: 'editor',
          viewport: { width: 1280, height: 800 },
          dpi: 1,
          locale: 'en-US',
          state: 'default',
          data: {
            workbenchState: {
              sourcePath,
              controlId: 'WorkbenchWindowRoot',
              sourceNodeId: 'root',
            },
            workbenchPresentation: {
              status: { projectPath: projectRoot },
              sourceFingerprint: { sourcePath, sha256: hash(source) },
            },
            managedSceneFingerprint: {
              projectRoot,
              sources: inputPaths.map((sourcePath, index) => ({
                sourcePath,
                sha256: hash(originalInputs[index]!),
              })),
            },
          },
        },
      ],
    };
  }, 60_000);

  beforeEach(async () => {
    vi.stubEnv(projectRootEnv, projectRoot);
    await Promise.all(
      inputPaths.map((path, index) =>
        writeFile(resolve(projectRoot, path), originalInputs[index]!),
      ),
    );
  });

  afterAll(async () => {
    vi.unstubAllEnvs();
    if (directory) {
      if (dirname(resolve(directory)) !== temporaryRoot)
        throw new Error('test directory escaped its temporary root');
      await rm(directory, { recursive: true, force: true, maxRetries: 3 });
    }
  });

  it('retains a current catalog when its bound project inputs are unchanged', async () => {
    await expect(
      verifyCurrentFiles(entry, directory, directory, false),
    ).resolves.toBeUndefined();
  });

  for (const path of inputPaths) {
    it(`rejects changed ${path} while authored ZUI and catalog data stay fixed`, async () => {
      const beforeCatalog = JSON.stringify(entry);
      const beforeZui = await readFile(resolve(directory, sourcePath), 'utf8');
      await writeFile(
        resolve(projectRoot, path),
        'entities = []\n# changed managed input\n',
      );
      await expect(
        verifyCurrentFiles(entry, directory, directory, false),
      ).rejects.toThrow(/managed scene.*changed/i);
      expect(JSON.stringify(entry)).toBe(beforeCatalog);
      expect(await readFile(resolve(directory, sourcePath), 'utf8')).toBe(
        beforeZui,
      );
    });
  }

  it('rejects a snapshot root that differs from the independently configured project', async () => {
    const otherRoot = resolve(directory, 'other-project');
    await mkdir(resolve(otherRoot, 'assets/scenes'), { recursive: true });
    await Promise.all(
      inputPaths.map((path, index) =>
        writeFile(resolve(otherRoot, path), originalInputs[index]!),
      ),
    );
    vi.stubEnv(projectRootEnv, otherRoot);
    await expect(
      verifyCurrentFiles(entry, directory, directory, false),
    ).rejects.toThrow(/bound project root/i);
  });

  it('fails closed when the caller has not bound a managed project', async () => {
    vi.stubEnv(projectRootEnv, undefined);
    await expect(
      verifyCurrentFiles(entry, directory, directory, false),
    ).rejects.toThrow(projectRootEnv);
  });

  it('rejects a lexical alias in the independently configured project root', async () => {
    vi.stubEnv(projectRootEnv, `${projectRoot}/../managed-project`);
    await expect(
      verifyCurrentFiles(entry, directory, directory, false),
    ).rejects.toThrow(/canonical physical path/i);
  });

  it('rejects an old catalog that omitted managed scene fingerprints', async () => {
    const oldEntry = structuredClone(entry);
    delete oldEntry.cases![0]!.data['managedSceneFingerprint'];
    await expect(
      verifyCurrentFiles(oldEntry, directory, directory, false),
    ).rejects.toThrow(/managedSceneFingerprint/i);
  });
});

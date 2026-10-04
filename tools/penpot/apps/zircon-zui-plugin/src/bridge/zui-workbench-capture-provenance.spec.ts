import { afterAll, beforeAll, describe, expect, it } from 'vitest';
import {
  copyFile,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, isAbsolute, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  captureProgramFingerprints,
  currentCaptureProgram,
} from '../../tools/zui-layout-capture-provenance';
import {
  fileSha256,
  missingPenpotCaptureProgramPaths,
} from '../../tools/zui-layout-evidence';
import type { LayoutRenderEvidence } from '../../tools/zui-layout-review-contract';

const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
const appPath = 'tools/penpot/apps/zircon-zui-plugin';
const workbenchPath =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
const workbenchTools = [
  'zui-layout-visual-source-scope.ts',
  'zui-layout-workbench-cases.ts',
  'zui-layout-workbench-managed-inputs.ts',
  'zui-layout-artifact-path.ts',
  'zui-layout-workbench-presentation.ts',
  'zui-layout-workbench-projection.ts',
  'penpot-workbench-canvas-edit.ts',
  'penpot-preview-layout-audit.ts',
];

describe('workbench capture provenance', () => {
  const tempRoot = resolve(tmpdir());
  let directory: string;
  let baseline: Array<[string, string]>;
  let fixedInputs: Array<[string, string]>;

  beforeAll(async () => {
    directory = await mkdtemp(join(tempRoot, 'zui-workbench-provenance-'));
    // The regression inputs include critical helpers even when the producer omits them.
    const sourcePaths = new Set([
      ...(await captureProgramFingerprints(repo))
        .map(([path]) => path)
        .filter((path) => !isAbsolute(path)),
      ...workbenchTools.map((name) => `${appPath}/tools/${name}`),
    ]);
    await Promise.all(
      [...sourcePaths].map(async (path) => {
        const target = resolve(directory, path);
        await mkdir(dirname(target), { recursive: true });
        await copyFile(resolve(repo, path), target);
      }),
    );
    const asset = resolve(directory, workbenchPath);
    await mkdir(dirname(asset), { recursive: true });
    await copyFile(resolve(repo, workbenchPath), asset);
    const screenshot = resolve(directory, 'capture.png');
    await writeFile(
      screenshot,
      Buffer.from(
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jhS0AAAAASUVORK5CYII=',
        'base64',
      ),
    );
    baseline = await captureProgramFingerprints(directory);
    fixedInputs = await Promise.all(
      [asset, screenshot].map(
        async (path) => [path, await fileSha256(path)] as [string, string],
      ),
    );
  }, 60_000);

  afterAll(async () => {
    if (!directory) return;
    if (dirname(resolve(directory)) !== tempRoot)
      throw new Error('Provenance fixture cleanup escapes its temporary root.');
    await rm(directory, { recursive: true, force: true, maxRetries: 3 });
  });

  it('retains a capture while source, renderer, and program bytes are unchanged', async () => {
    const evidence = [
      { captureProgramFingerprints: baseline },
    ] as LayoutRenderEvidence[];
    expect(
      currentCaptureProgram(
        evidence,
        await captureProgramFingerprints(directory),
      ),
    ).toBe(true);
  });

  it.each(workbenchTools)(
    'invalidates an earlier capture after only %s changes',
    async (name) => {
      const path = `${appPath}/tools/${name}`;
      const source = resolve(directory, path);
      const before = await readFile(source);
      const evidence = [
        { captureProgramFingerprints: baseline },
      ] as LayoutRenderEvidence[];
      try {
        await writeFile(
          source,
          Buffer.concat([
            before,
            Buffer.from('\n// Changed capture behavior.\n'),
          ]),
        );
        const current = await captureProgramFingerprints(directory);
        expect(currentCaptureProgram(evidence, current)).toBe(false);
        // Asset, screenshot, compiled plugin, and every other declared input stay fixed.
        expect(current.filter(([candidate]) => candidate !== path)).toEqual(
          baseline.filter(([candidate]) => candidate !== path),
        );
        for (const [input, hash] of fixedInputs)
          expect(await fileSha256(input)).toBe(hash);
      } finally {
        await writeFile(source, before);
      }
    },
  );

  it.each(workbenchTools)(
    'rejects evidence that omits %s even if remaining fingerprints are current',
    (name) => {
      const path = `${appPath}/tools/${name}`;
      expect(missingPenpotCaptureProgramPaths(baseline)).toEqual([]);
      expect(
        missingPenpotCaptureProgramPaths(
          baseline.filter(([candidate]) => candidate !== path),
        ),
      ).toEqual([path]);
    },
  );
});

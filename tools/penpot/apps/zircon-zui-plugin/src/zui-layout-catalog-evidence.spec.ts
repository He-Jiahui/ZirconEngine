import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { afterEach, it, expect } from 'vitest';
import {
  caseSha256,
  type LayoutReviewCase,
} from '../tools/zui-layout-review-contract';
import {
  retainExpandedCaseEvidence,
  type CatalogEvidenceSnapshot,
} from '../tools/zui-layout-catalog-evidence';

const tempRoots: string[] = [];
afterEach(async () => {
  for (const root of tempRoots.splice(0)) await rm(root, { recursive: true });
});

const digest = (bytes: Buffer) =>
  createHash('sha256').update(bytes).digest('hex');
const reviewCase = (state: string, width = 360): LayoutReviewCase => ({
  id: `${state}-${width}x520-dpi1`,
  sourcePath: 'zircon_editor/assets/ui/editor/components/dropdown.zui',
  host: 'component',
  viewport: { width, height: 520 },
  dpi: 1,
  locale: 'en-US',
  state,
  data: {},
});

function snapshots(): [CatalogEvidenceSnapshot, CatalogEvidenceSnapshot] {
  const shared = {
    category: 'editor-ui',
    name: 'dropdown-abcd1234',
    sourceSha256: 'a'.repeat(64),
    outputSha256: 'a'.repeat(64),
    dependencySha256: 'b'.repeat(64),
    penpotInputSha256: 'c'.repeat(64),
    previewPath: 'editor-ui/dropdown-abcd1234/preview.png',
    penpotPreviewPath: 'editor-ui/dropdown-abcd1234/penpot.png',
  };
  const baseline = reviewCase('default');
  const hover = reviewCase('hover');
  const open = reviewCase('open');
  const closed = reviewCase('closed');
  const capture = (item: LayoutReviewCase, path: string, bytes: Buffer) => ({
    caseId: item.id,
    status: 'passed' as const,
    screenshotPath: path,
    screenshotSha256: digest(bytes),
    sourceSha256: shared.sourceSha256,
    inputSha256: shared.penpotInputSha256,
    dependencySha256: shared.dependencySha256,
    caseSha256: caseSha256(item),
  });
  return [
    {
      ...shared,
      cases: [baseline, hover],
      penpotEvidence: [
        capture(baseline, shared.penpotPreviewPath, Buffer.from('default')),
        capture(
          hover,
          'editor-ui/dropdown-abcd1234/evidence/penpot-hover-360x520-dpi1.png',
          Buffer.from('hover'),
        ),
      ],
    },
    { ...shared, cases: [open, baseline, hover, closed] },
  ];
}

it.each([false, true])(
  'retains matching case evidence and %s write mode protects a displaced primary',
  async (write) => {
    const root = await mkdtemp(resolve(tmpdir(), 'zircon-layout-evidence-'));
    tempRoots.push(root);
    const [previous, next] = snapshots();
    const entryDir = resolve(root, previous.category, previous.name);
    await mkdir(resolve(entryDir, 'evidence'), { recursive: true });
    await writeFile(resolve(root, previous.penpotPreviewPath!), 'default');
    await writeFile(
      resolve(entryDir, 'evidence/penpot-hover-360x520-dpi1.png'),
      'hover',
    );

    const retained = await retainExpandedCaseEvidence(
      previous,
      next,
      root,
      write,
    );
    expect(retained.penpotEvidence?.map((item) => item.caseId)).toEqual([
      'default-360x520-dpi1',
      'hover-360x520-dpi1',
    ]);
    const formerPrimary = retained.penpotEvidence?.[0];
    expect(formerPrimary?.screenshotPath).toMatch(
      /^editor-ui\/dropdown-abcd1234\/evidence\/former-primary-penpot-default-360x520-dpi1-[a-f0-9]{12}\.png$/,
    );
    expect(formerPrimary?.screenshotPath).not.toBe(next.penpotPreviewPath);
    expect(
      await readFile(resolve(root, previous.penpotPreviewPath!), 'utf8').catch(
        () => null,
      ),
    ).toBe(write ? null : 'default');
    expect(
      await readFile(
        resolve(root, formerPrimary!.screenshotPath),
        'utf8',
      ).catch(() => null),
    ).toBe(write ? 'default' : null);
  },
);

it('does not carry old evidence across changed source, case, or file bytes', async () => {
  const root = await mkdtemp(resolve(tmpdir(), 'zircon-layout-evidence-'));
  tempRoots.push(root);
  const [previous, next] = snapshots();
  const entryDir = resolve(root, previous.category, previous.name);
  await mkdir(resolve(entryDir, 'evidence'), { recursive: true });
  await writeFile(resolve(root, previous.penpotPreviewPath!), 'default');
  await writeFile(
    resolve(entryDir, 'evidence/penpot-hover-360x520-dpi1.png'),
    'hover',
  );

  expect(
    (
      await retainExpandedCaseEvidence(
        previous,
        { ...next, sourceSha256: 'd'.repeat(64) },
        root,
        true,
      )
    ).penpotEvidence,
  ).toBeUndefined();
  const differentHover = next.cases!.map((item) =>
    item.state === 'hover' ? { ...item, locale: 'zh-CN' } : item,
  );
  const retained = await retainExpandedCaseEvidence(
    previous,
    { ...next, cases: differentHover },
    root,
    false,
  );
  expect(retained.penpotEvidence?.map((item) => item.caseId)).toEqual([
    'default-360x520-dpi1',
  ]);
  await writeFile(
    resolve(entryDir, 'evidence/penpot-hover-360x520-dpi1.png'),
    'corrupt',
  );
  await expect(
    retainExpandedCaseEvidence(previous, next, root, true),
  ).rejects.toThrow(/screenshot hash differs/);
});

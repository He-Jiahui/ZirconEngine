import type { LayoutRenderEvidence } from '../../tools/zui-layout-review-contract';
import {
  caseSha256,
  type LayoutReviewCase,
} from '../../tools/zui-layout-review-contract';
import {
  blockingPenpotTextFindings,
  evidenceAfterPenpotFailure,
  penpotCasesComplete,
  penpotScreenshotPathForCase,
  retainCompatiblePenpotEvidence,
} from '../../tools/zui-layout-penpot-cases';

const reviewCase = (id: string): LayoutReviewCase => ({
  id,
  sourcePath: 'source.zui',
  host: 'fixture',
  viewport: { width: 640, height: 520 },
  dpi: 1,
  locale: 'en-US',
  state: 'default',
  data: {},
});

describe('measured Penpot text acceptance', () => {
  const button = {
    nodeId: 'continue',
    shapeId: 'button-shape',
    parentNodeId: 'scroll',
    component: 'Button',
    visible: true,
    bounds: { x: 0, y: 20, width: 180, height: 42 },
    text: 'Continue',
    textParts: [
      {
        shapeId: 'text-shape',
        text: 'Continue',
        bounds: { x: 10, y: 20, width: 134, height: 1 },
      },
    ],
  };
  const scroll = {
    nodeId: 'scroll',
    shapeId: 'scroll-shape',
    parentNodeId: null,
    component: 'ScrollableBox',
    visible: true,
    bounds: { x: 0, y: 0, width: 200, height: 80 },
    text: null,
  };

  it('blocks a glyph outside its own button even inside an explicit scroll host', () => {
    const findings = blockingPenpotTextFindings(
      [scroll, button],
      [
        {
          shapeId: 'text-shape',
          ancestorShapeIds: ['button-shape', 'scroll-shape'],
          text: 'Continue',
          lines: [{ x: 30, y: 14, width: 60, height: 14 }],
        },
      ],
      { width: 200, height: 80 },
    );
    expect(findings.map((finding) => finding.kind)).toContain(
      'text-outside-node',
    );
  });

  it('allows a correctly bounded scroll item outside the viewport', () => {
    expect(
      blockingPenpotTextFindings(
        [
          scroll,
          {
            ...button,
            bounds: { ...button.bounds, y: 200 },
            textParts: [
              {
                shapeId: 'text-shape',
                text: 'Continue',
                bounds: { x: 10, y: 200, width: 134, height: 42 },
              },
            ],
          },
        ],
        [
          {
            shapeId: 'text-shape',
            ancestorShapeIds: ['button-shape', 'scroll-shape'],
            text: 'Continue',
            lines: [{ x: 30, y: 212, width: 60, height: 14 }],
          },
        ],
        { width: 200, height: 80 },
      ),
    ).toEqual([]);
  });

  it('blocks visible text from separate scroll children painting over each other', () => {
    const first = {
      ...button,
      nodeId: 'first',
      shapeId: 'first-shape',
      text: 'First',
      textParts: [],
      bounds: { x: 0, y: 20, width: 180, height: 28 },
    };
    const second = {
      ...first,
      nodeId: 'second',
      shapeId: 'second-shape',
      text: 'Second',
      bounds: { x: 0, y: 36, width: 180, height: 28 },
    };
    const findings = blockingPenpotTextFindings(
      [scroll, first, second],
      [
        {
          shapeId: 'first-shape',
          text: 'First',
          lines: [{ x: 8, y: 26, width: 40, height: 16 }],
        },
        {
          shapeId: 'second-shape',
          text: 'Second',
          lines: [{ x: 8, y: 38, width: 48, height: 16 }],
        },
      ],
      { width: 200, height: 80 },
    );
    expect(findings).toContainEqual(
      expect.objectContaining({
        kind: 'text-overlap',
        nodeIds: ['first', 'second'],
        semanticContext: 'scroll-content',
      }),
    );
  });

  it('blocks a normal card overflowing its allocated row inside a scroll host', () => {
    const row = {
      ...scroll,
      nodeId: 'row',
      shapeId: 'row-shape',
      parentNodeId: 'scroll',
      component: 'VerticalGroup',
      bounds: { x: 0, y: 10, width: 200, height: 30 },
    };
    const alert = {
      ...row,
      nodeId: 'alert',
      shapeId: 'alert-shape',
      parentNodeId: 'row',
      component: 'WorkbenchToast',
      bounds: { x: 8, y: 30, width: 180, height: 28 },
    };
    expect(
      blockingPenpotTextFindings([scroll, row, alert], [], {
        width: 200,
        height: 80,
      }),
    ).toContainEqual(
      expect.objectContaining({
        kind: 'node-outside-parent',
        nodeIds: ['alert', 'row'],
        semanticContext: 'scroll-content',
      }),
    );
    expect(
      blockingPenpotTextFindings(
        [{ ...row, component: 'Overlay', parentNodeId: null }, alert],
        [],
        { width: 200, height: 80 },
      ),
    ).toEqual([]);
  });
});

function evidence(
  caseId: string,
  overrides: Partial<LayoutRenderEvidence> = {},
): LayoutRenderEvidence {
  return {
    caseId,
    status: 'passed',
    screenshotPath: `${caseId}.png`,
    screenshotSha256: `image-${caseId}`,
    sourceSha256: 'current-source',
    inputSha256: 'current-input',
    dependencySha256: 'current-dependencies',
    caseSha256: `case-${caseId}`,
    rendererSha256: 'current-renderer',
    ...overrides,
  };
}

describe('Penpot case evidence retention', () => {
  it('retains only current, non-selected evidence during a partial recapture', () => {
    const retained = retainCompatiblePenpotEvidence(
      [
        evidence('wide-1280x800-dpi1'),
        evidence('compact-900x620-dpi1'),
        evidence('old-source', { sourceSha256: 'old-source' }),
        evidence('old-input', { inputSha256: 'old-input' }),
        evidence('old-renderer', { rendererSha256: 'old-renderer' }),
        evidence(''),
      ],
      new Set(['wide-1280x800-dpi1']),
      'current-source',
      'current-input',
      'current-renderer',
    );

    expect(retained.map(({ caseId }) => caseId)).toEqual([
      'compact-900x620-dpi1',
    ]);
  });

  it('fails only selected cases when the import fails before a targeted capture', () => {
    const cases = [
      reviewCase('wide'),
      reviewCase('compact'),
      reviewCase('narrow'),
    ];
    const entry = {
      cases,
      sourceSha256: 'current-source',
      penpotInputSha256: 'current-input',
      dependencySha256: 'current-dependencies',
    };
    const result = evidenceAfterPenpotFailure(
      entry,
      [evidence('wide'), evidence('compact')],
      new Set(['wide']),
      false,
      'current-renderer',
      [['capture.ts', 'program']],
      'import failed',
    );
    expect(result.map((item) => [item.caseId, item.status])).toEqual([
      ['compact', 'passed'],
      ['wide', 'failed'],
    ]);
    expect(result[1].error).toBe('import failed');
  });

  it('preserves a selected case captured before a later case failed', () => {
    const cases = [
      reviewCase('wide'),
      reviewCase('compact'),
      reviewCase('narrow'),
    ];
    const entry = {
      cases,
      sourceSha256: 'current-source',
      penpotInputSha256: 'current-input',
      dependencySha256: 'current-dependencies',
    };
    const program: Array<[string, string]> = [['capture.ts', 'program']];
    const captured = evidence('wide', {
      caseSha256: caseSha256(cases[0]),
      captureProgramFingerprints: program,
    });
    const result = evidenceAfterPenpotFailure(
      entry,
      [evidence('compact'), captured],
      new Set(['wide', 'narrow']),
      true,
      'current-renderer',
      program,
      'narrow failed',
    );
    expect(result.map((item) => [item.caseId, item.status])).toEqual([
      ['compact', 'passed'],
      ['wide', 'passed'],
      ['narrow', 'failed'],
    ]);
  });

  it('keeps a targeted capture pending until every required case is current', () => {
    const cases = [reviewCase('wide'), reviewCase('compact')];
    const program: Array<[string, string]> = [['capture.ts', 'program']];
    const current = (id: string) =>
      evidence(id, {
        screenshotPath:
          id === 'wide'
            ? 'runtime-fixtures/auth-review/penpot.png'
            : `runtime-fixtures/auth-review/evidence/penpot-${id}.png`,
        sourceSha256: 'source',
        dependencySha256: 'dependencies',
        inputSha256: 'input',
        rendererSha256: 'renderer',
        caseSha256: caseSha256(cases.find((item) => item.id === id)!),
        captureProgramFingerprints: program,
      });
    const entry = {
      category: 'runtime-fixtures',
      name: 'auth-review',
      penpotPreviewPath: 'runtime-fixtures/auth-review/penpot.png',
      cases,
      sourceSha256: 'source',
      dependencySha256: 'dependencies',
      penpotInputSha256: 'input',
      penpotEvidence: [current('wide')],
    };
    expect(penpotCasesComplete(entry, 'renderer', program)).toBe(false);
    entry.penpotEvidence.push(current('compact'));
    expect(penpotCasesComplete(entry, 'renderer', program)).toBe(true);
    entry.penpotEvidence.push(current('compact'));
    expect(penpotCasesComplete(entry, 'renderer', program)).toBe(false);
    entry.penpotEvidence.pop();
    entry.penpotEvidence[1] = current('compact');
    entry.penpotEvidence[1].caseSha256 = 'stale-case';
    expect(penpotCasesComplete(entry, 'renderer', program)).toBe(false);
    entry.penpotEvidence[1] = current('compact');
    entry.penpotEvidence[1].status = 'pending';
    expect(penpotCasesComplete(entry, 'renderer', program)).toBe(false);
    entry.penpotEvidence[1] = current('compact');
    entry.penpotEvidence[1].captureProgramFingerprints = [
      ['capture.ts', 'old-program'],
    ];
    expect(penpotCasesComplete(entry, 'renderer', program)).toBe(false);
  });

  it('never overwrites the canonical primary screenshot with a targeted state', () => {
    const entry = {
      category: 'runtime-fixtures',
      name: 'auth-review',
      penpotPreviewPath: 'runtime-fixtures/auth-review/penpot.png',
      cases: [reviewCase('default'), reviewCase('scroll-after')],
    };
    expect(penpotScreenshotPathForCase(entry, entry.cases[1])).toBe(
      'runtime-fixtures/auth-review/evidence/penpot-scroll-after.png',
    );
    expect(penpotScreenshotPathForCase(entry, entry.cases[0])).toBe(
      entry.penpotPreviewPath,
    );
  });
});

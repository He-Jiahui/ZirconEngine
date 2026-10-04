import {
  runReviewPhases,
  serializedWriter,
} from '../tools/zui-layout-worker-pool';
import { currentCaptureProgram } from '../tools/zui-layout-capture-provenance';
import type { LayoutRenderEvidence } from '../tools/zui-layout-review-contract';

describe('isolated Penpot capture scheduling', () => {
  it('claims every entry once and finishes a phase before starting the next', async () => {
    const started: number[] = [],
      finished: number[] = [];
    await runReviewPhases(
      [
        [0, 1, 2, 3, 4],
        [5, 6],
      ],
      4,
      async (take) => {
        for (let item = take(); item !== undefined; item = take()) {
          if (item >= 5)
            expect(finished.filter((value) => value < 5)).toHaveLength(5);
          started.push(item);
          await new Promise((resolve) =>
            setTimeout(resolve, (5 - (item % 5)) * 3),
          );
          finished.push(item);
        }
      },
    );
    expect([...finished].sort()).toEqual([0, 1, 2, 3, 4, 5, 6]);
  });

  it('serializes immutable checkpoints and retains the first write failure', async () => {
    const saved: number[] = [];
    const write = serializedWriter(async (value: { count: number }) => {
      await new Promise((resolve) => setTimeout(resolve, 5));
      saved.push(value.count);
    });
    const state = { count: 1 };
    const first = write(state);
    state.count = 2;
    await Promise.all([first, write(state)]);
    expect(saved).toEqual([1, 2]);
    const failed = serializedWriter(async () => {
      throw new Error('disk failure');
    });
    await expect(failed(null)).rejects.toThrow('disk failure');
    await expect(failed(null)).rejects.toThrow('disk failure');
  });

  it('invalidates a passed capture when a capture tool changes', () => {
    const current: Array<[string, string]> = [['capture.ts', 'new']];
    const evidence = [
      { captureProgramFingerprints: [['capture.ts', 'old']] },
    ] as LayoutRenderEvidence[];
    expect(currentCaptureProgram(evidence, current)).toBe(false);
    evidence[0].captureProgramFingerprints = current;
    expect(currentCaptureProgram(evidence, current)).toBe(true);
  });
});

import { describe, expect, it } from 'vitest';
import { prepareVisualSourceEntries } from '../tools/zui-layout-visual-source-scope';

function entry(sourcePath: string) {
  return {
    sourcePath,
    visualStatus: 'passed',
    review: { status: 'accepted' },
    evidence: [{ screenshotSha256: 'prior-capture' }],
  };
}

async function invalidate(item: ReturnType<typeof entry>) {
  item.visualStatus = 'pending';
  item.review.status = 'archived';
}

describe('visual source preparation', () => {
  it('keeps excluded capture evidence byte equivalent and keeps selected identity', async () => {
    const foreign = entry('ui/foreign.zui');
    const workbench = entry('ui/workbench.zui');
    const before = JSON.stringify(foreign);
    const selected = await prepareVisualSourceEntries(
      [foreign, workbench],
      ['--source', 'workbench.zui'],
      invalidate,
    );
    expect(JSON.stringify(foreign)).toBe(before);
    expect(selected).toEqual([workbench]);
    expect(selected[0]).toBe(workbench);
    expect(workbench.visualStatus).toBe('pending');
  });

  it('makes a stale selected passed capture available to the pending batch', async () => {
    const workbench = entry('ui/workbench.zui');
    const selected = await prepareVisualSourceEntries(
      [workbench],
      ['--source', 'workbench.zui', '--pending'],
      invalidate,
    );
    expect(selected.filter((item) => item.visualStatus !== 'passed')).toEqual([
      workbench,
    ]);
  });

  it('prepares the full inventory in order when no source is requested', async () => {
    const entries = [entry('ui/second.zui'), entry('ui/first.zui')];
    const selected = await prepareVisualSourceEntries(
      entries,
      ['--pending'],
      invalidate,
    );
    expect(selected).toEqual(entries);
    expect(selected.every((item) => item.visualStatus === 'pending')).toBe(
      true,
    );
  });

  it('rejects a missing source value before touching any capture', async () => {
    const item = entry('ui/workbench.zui');
    const before = JSON.stringify(item);
    await expect(
      prepareVisualSourceEntries([item], ['--source', '--pending'], invalidate),
    ).rejects.toThrow('--source requires a path');
    expect(JSON.stringify(item)).toBe(before);
  });
});

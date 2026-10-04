import { describe, expect, it, vi } from 'vitest';
import { fileURLToPath } from 'node:url';
import {
  clickAndWaitForDownload,
  downloadPath,
} from '../../tools/zui-layout-download';
import { captureProgramFingerprints } from '../../tools/zui-layout-capture-provenance';

const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));

describe('layout download coordination', () => {
  it('registers the download listener before initiating the export click', async () => {
    const calls: string[] = [];

    await expect(
      clickAndWaitForDownload(
        async () => {
          calls.push('wait');
          return 'download';
        },
        async () => {
          calls.push('click');
        },
      ),
    ).resolves.toBe('download');

    expect(calls).toEqual(['wait', 'click']);
  });

  it('keeps a late download rejection observed when the click fails first', async () => {
    let rejectDownload: (reason?: unknown) => void = () => {};
    const download = new Promise<string>((_, reject) => {
      rejectDownload = reject;
    });

    await expect(
      clickAndWaitForDownload(
        () => download,
        async () => {
          throw new Error('click failed');
        },
      ),
    ).rejects.toThrow('click failed');

    // Promise.all must already be observing this rejection.  The old
    // sequential pattern leaked it as an unhandled rejection 30 seconds
    // after the failed click.
    rejectDownload(new Error('download timed out'));
    await new Promise((resolve) => setTimeout(resolve, 0));
  });

  it('fingerprints the download coordinator as part of capture provenance', async () => {
    const fingerprints = await captureProgramFingerprints(repo);
    expect(fingerprints.map(([path]) => path)).toContain(
      'tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-download.ts',
    );
  });

  it('bounds download-file materialization after the download event', async () => {
    vi.useFakeTimers();
    try {
      let rejectPath: (reason?: unknown) => void = () => {};
      const path = new Promise<string | null>((_, reject) => {
        rejectPath = reject;
      });
      const pending = downloadPath({ path: () => path }, 30);
      const timeout = expect(pending).rejects.toThrow(
        'Timed out waiting for exported download file',
      );

      await vi.advanceTimersByTimeAsync(30);
      await timeout;

      // The late I/O failure must remain observed after the timeout rather
      // than terminating the whole batch as an unhandled rejection.
      rejectPath(new Error('download file failed late'));
      await Promise.resolve();
    } finally {
      vi.useRealTimers();
    }
  });
});

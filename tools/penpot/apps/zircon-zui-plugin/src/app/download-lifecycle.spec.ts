import { describe, expect, it, vi } from 'vitest';
import {
  DOWNLOAD_URL_REVOCATION_DELAY_MS,
  deferDownloadUrlRevocation,
} from './download-lifecycle';

describe('download URL lifecycle', () => {
  it('keeps the object URL alive through the browser download handoff', () => {
    vi.useFakeTimers();
    try {
      const revoke = vi.fn();

      deferDownloadUrlRevocation('blob:zircon-export', revoke);

      expect(revoke).not.toHaveBeenCalled();
      vi.advanceTimersByTime(DOWNLOAD_URL_REVOCATION_DELAY_MS - 1);
      expect(revoke).not.toHaveBeenCalled();
      vi.advanceTimersByTime(1);
      expect(revoke).toHaveBeenCalledOnce();
      expect(revoke).toHaveBeenCalledWith('blob:zircon-export');
    } finally {
      vi.useRealTimers();
    }
  });
});

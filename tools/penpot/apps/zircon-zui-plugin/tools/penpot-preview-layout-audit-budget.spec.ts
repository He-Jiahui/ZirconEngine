import type { FrameLocator } from 'playwright';
import {
  previewLayoutAuditTimeoutForNodes,
  requestPreviewLayoutAudit,
} from './penpot-preview-layout-audit';

afterEach(() => vi.useRealTimers());

it('allows a large native audit past the old deadline and still rejects a missing reply', async () => {
  vi.useFakeTimers();
  const budget = previewLayoutAuditTimeoutForNodes(170);
  const evaluate = vi.fn(
    (_callback: unknown, _request: unknown, _options: { timeout: number }) =>
      new Promise<never>(() => {}),
  );
  const frame = {
    locator: () => ({ evaluate }),
  } as unknown as FrameLocator;
  const operation = requestPreviewLayoutAudit(frame, budget);
  const settled = vi.fn();
  void operation.then(settled, settled);
  const rejection = expect(operation).rejects.toThrow(
    'Timed out waiting for preview audit',
  );

  await vi.advanceTimersByTimeAsync(120_000);
  expect(settled).not.toHaveBeenCalled();
  expect(evaluate.mock.calls[0]?.[2]).toEqual({ timeout: budget });
  await vi.advanceTimersByTimeAsync(budget - 120_000);
  await rejection;
  expect(vi.getTimerCount()).toBe(0);
});

it('caps large-board work and rejects an invalid node count before starting a request', () => {
  expect(previewLayoutAuditTimeoutForNodes(100_000)).toBe(
    previewLayoutAuditTimeoutForNodes(1_000),
  );
  expect(Number.isFinite(previewLayoutAuditTimeoutForNodes(100_000))).toBe(true);
  expect(() => previewLayoutAuditTimeoutForNodes(-1)).toThrow(
    'Projected node count must be a nonnegative integer',
  );
});

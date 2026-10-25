import type { FrameLocator } from 'playwright';
import {
  requestPreviewLayoutAudit,
  type PreviewLayoutAuditReply,
} from '../tools/penpot-preview-layout-audit';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

function iframeAudit() {
  type Request = { requestId: string; timeoutMs: number };
  type Reply = PreviewLayoutAuditReply & { type: 'preview-audit' };
  const listeners = new Set<(event: MessageEvent<Reply>) => void>();
  const parentWindow = { postMessage: vi.fn() };
  vi.stubGlobal('parent', parentWindow);
  vi.stubGlobal('window', {
    setTimeout,
    clearTimeout,
    addEventListener: (
      _type: string,
      listener: (event: MessageEvent<Reply>) => void,
    ) => listeners.add(listener),
    removeEventListener: (
      _type: string,
      listener: (event: MessageEvent<Reply>) => void,
    ) => listeners.delete(listener),
  });
  const evaluate = vi.fn(
    (
      callback: (
        body: unknown,
        request: Request,
      ) => Promise<PreviewLayoutAuditReply>,
      request: Request,
      _options: { timeout: number },
    ) => callback(null, request),
  );
  const frame = {
    locator: () => ({ evaluate }),
  } as unknown as FrameLocator;
  const reply = (requestId: string) => {
    for (const listener of listeners)
      listener({
        source: parentWindow,
        data: {
          type: 'preview-audit',
          requestId,
          previewBoardId: 'asset',
          layoutAudit: { overflowNodeIds: [], invalidGeometryNodeIds: [] },
        },
      } as unknown as MessageEvent<Reply>);
  };
  return { frame, evaluate, parentWindow, listeners, reply };
}

it('uses the requested audit budget for iframe body resolution as well as the reply', async () => {
  const budget = 1234;
  const frame = {
    locator: () => ({
      evaluate: async (
        _callback: unknown,
        request: { requestId: string },
        options?: { timeout: number },
      ) => {
        if (options?.timeout !== budget)
          throw new Error(
            'Iframe resolution used an implicit shorter deadline',
          );
        return {
          requestId: request.requestId,
          previewBoardId: 'asset',
          layoutAudit: { overflowNodeIds: [], invalidGeometryNodeIds: [] },
        };
      },
    }),
  } as unknown as FrameLocator;
  await expect(requestPreviewLayoutAudit(frame, budget)).resolves.toMatchObject(
    { previewBoardId: 'asset' },
  );
});

it('accepts the matching audit reply after 30 seconds within the default 120-second budget', async () => {
  vi.useFakeTimers();
  const iframe = iframeAudit();
  const operation = requestPreviewLayoutAudit(iframe.frame);
  const settled = vi.fn();
  void operation.then(settled, settled);
  expect(iframe.evaluate.mock.calls[0]?.[2]).toEqual({ timeout: 120_000 });
  await vi.advanceTimersByTimeAsync(60_000);
  expect(settled).not.toHaveBeenCalled();
  const request = iframe.parentWindow.postMessage.mock.calls[0]?.[0] as {
    type: string;
    requestId: string;
  };
  expect(request.type).toBe('audit-preview-layout');
  iframe.reply(request.requestId);
  await expect(operation).resolves.toMatchObject({
    requestId: request.requestId,
    previewBoardId: 'asset',
  });
  expect(iframe.listeners.size).toBe(0);
  expect(vi.getTimerCount()).toBe(0);
});

it('rejects at the deadline when the only audit reply has another request ID', async () => {
  vi.useFakeTimers();
  const iframe = iframeAudit();
  const operation = requestPreviewLayoutAudit(iframe.frame);
  const rejection = expect(operation).rejects.toThrow(
    'Timed out waiting for preview audit',
  );
  iframe.reply('another-request');
  await vi.advanceTimersByTimeAsync(120_000);
  await rejection;
  expect(iframe.listeners.size).toBe(0);
  expect(vi.getTimerCount()).toBe(0);
});

it('enforces the default Node deadline when iframe evaluation never resolves', async () => {
  vi.useFakeTimers();
  const evaluate = vi.fn(() => new Promise<never>(() => {}));
  const frame = {
    locator: () => ({ evaluate }),
  } as unknown as FrameLocator;
  const operation = requestPreviewLayoutAudit(frame);
  const settled = vi.fn();
  void operation.then(settled, settled);
  const rejection = expect(operation).rejects.toThrow(
    'Timed out waiting for preview audit',
  );
  await vi.advanceTimersByTimeAsync(119_999);
  expect(settled).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(1);
  await rejection;
  expect(vi.getTimerCount()).toBe(0);
});

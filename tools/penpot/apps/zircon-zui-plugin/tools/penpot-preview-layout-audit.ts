import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import type { FrameLocator } from 'playwright';
import type { RenderedLayoutAudit } from '../src/penpot-render-layout';

export interface PreviewLayoutAuditReply {
  requestId: string;
  previewBoardId: string;
  layoutAudit: RenderedLayoutAudit;
}

/** Read the response to this request through the plugin's public UI channel. */
export async function requestPreviewLayoutAudit(
  pluginFrame: FrameLocator,
  timeoutMs = 120_000,
): Promise<PreviewLayoutAuditReply> {
  const requestId = randomUUID();
  let timeout: ReturnType<typeof setTimeout> | undefined;
  try {
    const response = await Promise.race([
      pluginFrame.locator('body').evaluate(
        (_body, request) =>
          new Promise<PreviewLayoutAuditReply>((resolve, reject) => {
            // Object methods preserve their name without an injected tsx helper
            // when Playwright serializes this callback into the browser realm.
            const handlers = {
              message(
                event: MessageEvent<
                  PreviewLayoutAuditReply & { type?: string }
                >,
              ) {
                if (
                  event.source !== parent ||
                  event.data?.type !== 'preview-audit' ||
                  event.data.requestId !== request.requestId
                )
                  return;
                window.clearTimeout(timer);
                window.removeEventListener('message', handlers.message);
                resolve(event.data);
              },
            };
            const timer = window.setTimeout(() => {
              window.removeEventListener('message', handlers.message);
              reject(
                new Error(
                  `Timed out waiting for preview audit ${request.requestId}`,
                ),
              );
            }, request.timeoutMs);
            window.addEventListener('message', handlers.message);
            parent.postMessage(
              { type: 'audit-preview-layout', requestId: request.requestId },
              '*',
            );
          }),
        { requestId, timeoutMs },
        { timeout: timeoutMs },
      ),
      new Promise<never>((_resolve, reject) => {
        timeout = setTimeout(
          () =>
            reject(
              new Error(`Timed out waiting for preview audit ${requestId}`),
            ),
          timeoutMs,
        );
      }),
    ]);
    assert.equal(response.requestId, requestId);
    assert.ok(response.previewBoardId, 'Missing audited preview board ID');
    assert.ok(response.layoutAudit, 'Missing requested layout audit');
    return response;
  } finally {
    clearTimeout(timeout);
  }
}

import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import type { Page } from 'playwright';
import { requireRealSessionDirectory, sha256 } from './zui-layout-real-session-contract.mjs';

interface RequestEvent {
  requestId: string;
  request: { url: string; method: string; postData?: string };
}
interface ResponseEvent {
  requestId: string;
  response: { status: number; headers: Record<string, string | number>; mimeType: string };
}
interface LoadingEvent { requestId: string; encodedDataLength: number }
interface FailedEvent { requestId: string; errorText: string }
export interface ActualSaveHttp {
  url: string; requestId: string; httpStatus: number; contentType: string;
  requestBody: Buffer; responseBody: Buffer; requestFile: string; responseFile: string;
}
interface ObserverOptions {
  fileId: string; output: string; pending: Set<Promise<void>>;
  diagnostics: Record<string, unknown>[];
  onResponse(response: ActualSaveHttp): Promise<void>;
  onError(error: unknown): void;
}

/** Read the real browser exchange in this CDP session; never replay or intercept it. */
export async function observeRealSaveHttp(page: Page, options: ObserverOptions): Promise<Record<string, unknown>> {
  requireRealSessionDirectory(options.output);
  const session = await page.context().newCDPSession(page);
  // Actual protected-source responses are about 20 MiB. Keep bounded headroom.
  const limits = { maxTotalBufferSize: 128 * 1024 * 1024,
    maxResourceBufferSize: 64 * 1024 * 1024, maxPostDataSize: 64 * 1024 * 1024 };
  const requests = new Map<string, { url: string; requestFile: string; responseFile: string;
    capture: Promise<Buffer>; diagnostic: Record<string, unknown>; response?: ResponseEvent['response'] }>();
  let sequence = 0;
  const track = (operation: Promise<void>) => {
    const handled: Promise<void> = operation.catch(options.onError).finally(() => options.pending.delete(handled));
    options.pending.add(handled);
  };
  session.on('Network.requestWillBeSent', (event: RequestEvent) => {
    const url = new URL(event.request.url);
    if (event.request.method !== 'POST' || !/^\/api\/(?:main\/methods|rpc\/command)\/update-file$/.test(url.pathname) ||
      url.searchParams.get('id') !== options.fileId) return;
    const requestFile = `save-request-${++sequence}.transit.json`;
    const responseFile = `save-response-${sequence}.transit.json`;
    const diagnostic: Record<string, unknown> = { url: url.href, requestId: event.requestId,
      capture: 'CDP Network.loadingFinished/getResponseBody', requestFile, responseFile };
    options.diagnostics.push(diagnostic);
    const capture = (async () => {
      const postData = event.request.postData ?? (await session.send('Network.getRequestPostData', {
        requestId: event.requestId,
      }) as { postData: string }).postData;
      assert.ok(postData, 'The real save request body must be available');
      const bytes = Buffer.from(postData, 'utf8');
      diagnostic['requestByteLength'] = bytes.length;
      diagnostic['requestSha256'] = sha256(bytes);
      await writeFile(resolve(options.output, requestFile), bytes);
      assert.ok(bytes.length <= limits.maxPostDataSize, 'The actual save request exceeds its explicit observation limit');
      return bytes;
    })();
    requests.set(event.requestId, { url: url.href, requestFile, responseFile, capture, diagnostic });
    track(capture.then(() => undefined));
  });
  session.on('Network.responseReceived', (event: ResponseEvent) => {
    const request = requests.get(event.requestId);
    if (!request) return;
    request.response = event.response;
    request.diagnostic['httpStatus'] = event.response.status;
    request.diagnostic['contentType'] = Object.entries(event.response.headers)
      .find(([key]) => key.toLowerCase() === 'content-type')?.[1] ?? event.response.mimeType;
  });
  session.on('Network.loadingFinished', (event: LoadingEvent) => {
    const request = requests.get(event.requestId);
    if (!request) return;
    request.diagnostic['encodedDataLength'] = event.encodedDataLength;
    track((async () => {
      try {
        // Read immediately on completion in the same session whose buffers we configured.
        const [body, requestBody] = await Promise.all([
          session.send('Network.getResponseBody', { requestId: event.requestId }) as Promise<{ body: string; base64Encoded: boolean }>,
          request.capture,
        ]);
        const responseBody = Buffer.from(body.body, body.base64Encoded ? 'base64' : 'utf8');
        request.diagnostic['responseByteLength'] = responseBody.length;
        request.diagnostic['responseSha256'] = sha256(responseBody);
        await writeFile(resolve(options.output, request.responseFile), responseBody);
        assert.ok(responseBody.length <= limits.maxResourceBufferSize,
          'The actual save response exceeds its explicit observation limit');
        assert.ok(request.response, 'Actual response headers must accompany the completed body');
        await options.onResponse({ url: request.url, requestId: event.requestId,
          httpStatus: request.response.status, contentType: String(request.diagnostic['contentType']),
          requestBody, responseBody, requestFile: request.requestFile, responseFile: request.responseFile });
      } catch (error) {
        request.diagnostic['captureError'] = String(error);
        throw error;
      }
    })());
  });
  session.on('Network.loadingFailed', (event: FailedEvent) => {
    const request = requests.get(event.requestId);
    if (!request) return;
    request.diagnostic['captureError'] = event.errorText;
    options.onError(new Error(`The actual save request failed: ${event.errorText}`));
  });
  await session.send('Network.enable', limits);
  // Official CDP uses this supported command for bodies outside the renderer.
  await session.send('Network.configureDurableMessages', {
    maxTotalBufferSize: limits.maxTotalBufferSize, maxResourceBufferSize: limits.maxResourceBufferSize,
  });
  return { ...limits, networkEnableCommandAccepted: true, configureDurableMessagesCommandAccepted: true,
    bodyReader: 'Network.getResponseBody in the configured CDP session at Network.loadingFinished',
    protocolDocumentation: 'https://chromedevtools.github.io/devtools-protocol/tot/Network/' };
}

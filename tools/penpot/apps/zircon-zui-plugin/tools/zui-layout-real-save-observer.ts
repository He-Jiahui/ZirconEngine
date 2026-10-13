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
interface PausedEvent {
  requestId: string; networkId?: string; request: RequestEvent['request'];
  responseStatusCode?: number; responseErrorReason?: string;
  responseHeaders?: { name: string; value: string }[];
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

/** Read the actual save response while paused, then continue the exchange unchanged. */
export async function observeRealSaveHttp(page: Page, options: ObserverOptions): Promise<Record<string, unknown>> {
  requireRealSessionDirectory(options.output);
  const session = await page.context().newCDPSession(page);
  const origin = new URL(page.url()).origin;
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
    if (url.origin !== origin || event.request.method !== 'POST' || !/^\/api\/(?:main\/methods|rpc\/command)\/update-file$/.test(url.pathname) ||
      url.searchParams.get('id') !== options.fileId) return;
    const requestFile = `save-request-${++sequence}.transit.json`;
    const responseFile = `save-response-${sequence}.transit.json`;
    const diagnostic: Record<string, unknown> = { url: url.href, requestId: event.requestId,
      capture: 'CDP Fetch.Response/getResponseBody/continueResponse', requestFile, responseFile };
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
    if (request) request.diagnostic['encodedDataLength'] = event.encodedDataLength;
  });
  session.on('Fetch.requestPaused', (event: PausedEvent) => {
    const url = new URL(event.request.url);
    const actualSave = url.origin === origin && event.request.method === 'POST' &&
      /^\/api\/(?:main\/methods|rpc\/command)\/update-file$/.test(url.pathname) &&
      url.searchParams.get('id') === options.fileId;
    if (!actualSave) {
      track(session.send('Fetch.continueResponse', { requestId: event.requestId }).then(() => undefined));
      return;
    }
    const request = event.networkId ? requests.get(event.networkId) : undefined;
    track((async () => {
      try {
        let actual: ActualSaveHttp;
        try {
          assert.ok(request && event.networkId, 'The paused save response needs its captured actual Network request');
          assert.equal(request.url, url.href, 'The paused response and captured save request must identify the same exchange');
          assert.ok(!event.responseErrorReason && Number.isSafeInteger(event.responseStatusCode),
            'The actual save response must reach the Response stage without a network error');
          const contentType = event.responseHeaders?.find(({ name }) => name.toLowerCase() === 'content-type')?.value ?? '';
          request.diagnostic['fetchRequestId'] = event.requestId;
          request.diagnostic['httpStatus'] = event.responseStatusCode;
          request.diagnostic['contentType'] = contentType;
          const body = await session.send('Fetch.getResponseBody', { requestId: event.requestId }) as
            { body: string; base64Encoded: boolean };
          const requestBody = await request.capture;
          const responseBody = Buffer.from(body.body, body.base64Encoded ? 'base64' : 'utf8');
          request.diagnostic['responseByteLength'] = responseBody.length;
          request.diagnostic['responseSha256'] = sha256(responseBody);
          assert.ok(responseBody.length <= limits.maxResourceBufferSize,
            'The actual save response exceeds its explicit observation limit');
          await writeFile(resolve(options.output, request.responseFile), responseBody);
          actual = { url: request.url, requestId: event.networkId,
            httpStatus: event.responseStatusCode!, contentType, requestBody, responseBody,
            requestFile: request.requestFile, responseFile: request.responseFile };
        } finally {
          // Resume only after the real body read has settled; leave status, headers and body intact.
          await session.send('Fetch.continueResponse', { requestId: event.requestId });
        }
        await options.onResponse(actual);
      } catch (error) {
        if (request) request.diagnostic['captureError'] = String(error);
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
  await session.send('Fetch.enable', { patterns: [
    { urlPattern: origin + '/api/main/methods/update-file*id=' + options.fileId + '*', requestStage: 'Response' },
    { urlPattern: origin + '/api/rpc/command/update-file*id=' + options.fileId + '*', requestStage: 'Response' },
  ] });
  return { ...limits, networkEnableCommandAccepted: true, configureDurableMessagesCommandAccepted: true,
    fetchResponseCaptureCommandAccepted: true,
    bodyReader: 'Fetch.getResponseBody at Response stage, continued without response modification',
    protocolDocumentation: 'https://chromedevtools.github.io/devtools-protocol/tot/Fetch/' };
}

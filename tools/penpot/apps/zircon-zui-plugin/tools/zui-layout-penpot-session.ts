import { resolveLayoutCatalogPath } from './zui-layout-paths';
import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { realpathSync, statSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import {
  dirname,
  extname,
  isAbsolute,
  relative,
  resolve,
  sep,
} from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  requireApprovedPenpotArtifactDirectory,
  requireApprovedPenpotArtifactFile,
} from './zui-layout-artifact-path';
import type {
  Browser,
  BrowserContext,
  FrameLocator,
  Locator,
  Page,
  Route,
} from 'playwright';
import type { BrowserRuntimeRecord } from './zui-layout-review-contract';
import { collectFontResources } from './zui-layout-font-resources';
import {
  navigateWithRetry,
  withPenpotStartupRetry,
} from './penpot-browser-navigation';

const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const penpotRoot = resolve(appRoot, '../../../../third_party/penpot');
const frontendRoot = resolve(penpotRoot, 'frontend');
const apiSuiteRoot = resolve(penpotRoot, 'plugins/apps/plugin-api-test-suite');
const pluginDistRoot = resolvePluginDistRoot();
const penpotStartupTimeoutMs = 300_000;

/** Resolve the staged plugin distribution from an approved physical artifact root. */
export function resolvePluginDistRoot(): string {
  const configured = process.env['ZUI_PLUGIN_DIST_ROOT'];
  if (!configured)
    throw new Error(
      'Set ZUI_PLUGIN_DIST_ROOT to the staged plugin distribution under an approved drive-root cargo-targets directory.',
    );
  return requireApprovedPenpotArtifactDirectory(
    configured,
    'ZUI_PLUGIN_DIST_ROOT',
  );
}

/** Resolve the esbuild bundle captured independently from Angular's staged distribution. */
export function resolvePluginBundlePath(): string {
  const configured = process.env['ZUI_PLUGIN_BUNDLE_PATH'];
  if (!configured)
    throw new Error(
      'Set ZUI_PLUGIN_BUNDLE_PATH to the compiled plugin.js under an approved drive-root cargo-targets directory.',
    );
  return requireApprovedPenpotArtifactFile(
    configured,
    'ZUI_PLUGIN_BUNDLE_PATH',
  );
}

const mockTeamId = 'c7ce0794-0992-8105-8004-38e630f7920a';
const mockFileId = 'c7ce0794-0992-8105-8004-38f280443849';
const mockPageId = '66697432-c33d-8055-8006-2c62cc084cad';

export async function recordPenpotSessionFailure(
  runtime: WorkerRuntime,
  directory: string,
  error: unknown,
): Promise<void> {
  const { page, statusPanel, pluginFrame } = runtime;
  const status = await statusPanel
    .innerText({ timeout: 2_000 })
    .catch(() => null);
  const pluginBodyText = await pluginFrame
    .locator('body')
    .innerText({ timeout: 2_000 })
    .catch(() => null);
  const screenshot = 'penpot-session-error.png';
  const captureError = await page
    .screenshot({
      path: resolve(directory, screenshot),
      timeout: 15_000,
    })
    .then(
      () => null,
      (cause: unknown) => String(cause),
    );
  await writeFile(
    resolve(directory, 'penpot-session-error.json'),
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        error: error instanceof Error ? error.stack : String(error),
        url: page.url(),
        status,
        pluginBodyText: pluginBodyText?.replace(/\s+/g, ' ').trim().slice(0, 2_000) ?? null,
        pageErrors: runtime.pageErrors.slice(0, 40),
        unexpectedRpcs: [...runtime.unexpectedRpcs],
        frames: page.frames().map((frame) => frame.url()),
        screenshot: captureError ? null : screenshot,
        captureError,
      },
      null,
      2,
    )}\n`,
  );
}

const mockRpcFixtures = new Map<string, string>([
  ['get-profile', 'logged-in-user/get-profile-logged-in.json'],
  ['get-teams', 'get-teams.json'],
  ['get-team', 'workspace/get-team-default.json'],
  ['get-team-members', 'logged-in-user/get-team-members-your-penpot.json'],
  ['get-team-users', 'logged-in-user/get-team-users-single-user.json'],
  ['get-project', 'workspace/get-project-default.json'],
  ['get-comment-threads', 'workspace/get-comment-threads-empty.json'],
  [
    'get-profiles-for-file-comments',
    'workspace/get-profile-for-file-comments.json',
  ],
  [
    'get-file-object-thumbnails',
    'workspace/get-file-object-thumbnails-blank.json',
  ],
  ['get-font-variants', 'workspace/get-font-variants-empty.json'],
  ['get-file-fragment', 'workspace/get-file-fragment-blank.json'],
  ['get-file-libraries', 'workspace/get-file-libraries-empty.json'],
  ['update-profile-props', 'workspace/update-profile-empty.json'],
]);
const mockNoopRpcs = new Set([
  'get-enabled-flags',
  'push-audit-events',
  'create-file-object-thumbnail',
  'delete-file-object-thumbnails',
]);

export type WorkerRuntime = {
  context: BrowserContext;
  page: Page;
  pluginFrame: FrameLocator;
  fileInput: Locator;
  statusPanel: Locator;
  exportButton: Locator;
  fontResources: ReturnType<typeof collectFontResources>;
  pageErrors: string[];
  unexpectedRpcs: Set<string>;
};
export async function openPenpotSession(
  ...args: Parameters<typeof openPenpotSessionAttempt>
): Promise<WorkerRuntime> {
  return withPenpotStartupRetry(
    () => openPenpotSessionAttempt(...args),
    (attempt, error) =>
      console.warn(
        JSON.stringify({
          phase: 'penpot-session-startup',
          attempt,
          status: 'retry',
          error: error.message,
        }),
      ),
  );
}

async function openPenpotSessionAttempt(
  browser: Browser,
  options: {
    repoRoot: string;
    catalogRoot: string;
    writeEvidence: boolean;
    penpotBaseUrl: string;
    pluginBundleSha256: string;
    pageErrors: string[];
    unexpectedRpcs: Set<string>;
    captureProgramFingerprints?: Array<[string, string]>;
    captureBrowserRuntime?: (page: Page) => Promise<BrowserRuntimeRecord>;
  },
): Promise<WorkerRuntime> {
  const {
    repoRoot,
    catalogRoot,
    writeEvidence,
    penpotBaseUrl,
    pluginBundleSha256,
    pageErrors,
    unexpectedRpcs,
    captureProgramFingerprints,
    captureBrowserRuntime,
  } = options;
  const pluginHost = new URL('/__zircon-zui-plugin__/', penpotBaseUrl).href;
  const context = await browser.newContext({
    acceptDownloads: true,
    ignoreHTTPSErrors: true,
    locale: 'en-US',
    serviceWorkers: 'block',
    viewport: { width: 1920, height: 1200 },
  });
  const page = await context.newPage();
  const fontResources = collectFontResources(
    page,
    repoRoot,
    catalogRoot,
    writeEvidence,
  );
  const startupNetwork: string[] = [];
  page.on('pageerror', (error) => pageErrors.push(`pageerror: ${error.message}`));
  page.on('console', (message) => {
    if (message.type() === 'error')
      pageErrors.push(`console: ${message.text()}`);
  });
  page.on('requestfailed', (request) => {
    pageErrors.push(
      `requestfailed: ${request.method()} ${request.url()} (${request.failure()?.errorText ?? 'unknown'})`,
    );
  });
  page.on('response', (response) => {
    if (response.status() >= 400)
      pageErrors.push(`http ${response.status()}: ${response.url()}`);
  });
  page.on('request', (request) => {
    if (['document', 'script', 'stylesheet'].includes(request.resourceType()))
      startupNetwork.push(`request ${request.resourceType()}: ${request.url()}`);
  });
  page.on('response', (response) => {
    if (
      ['document', 'script', 'stylesheet'].includes(
        response.request().resourceType(),
      )
    )
      startupNetwork.push(
        `response ${response.status()}: ${response.url()}`,
      );
  });
  let officialAssetReplay: OfficialScriptAssetReplay | undefined;
  let replayReceiptPath: string | undefined;
  try {
    const sockets = await installWebSocketMock(page);
    officialAssetReplay = await installOfficialAssetReplay(
      page,
      penpotBaseUrl,
      startupNetwork,
    );
    if (officialAssetReplay)
      replayReceiptPath = nextOfficialSessionReplayReceiptPath(
        officialAssetReplay.cacheRoot,
      );
    await installMockBackend(page, unexpectedRpcs);
    await installPluginAssetRoute(page);
    const workspaceUrl = `${penpotBaseUrl}/#/workspace?team-id=${mockTeamId}&file-id=${mockFileId}&page-id=${mockPageId}&wasm=false`;
    await navigateWithRetry(page, workspaceUrl);
    await openNotificationsWebSocket(page, sockets);
    await page.getByTestId('viewport').waitFor({ timeout: 30_000 });
    await page.waitForFunction(
      () =>
        typeof (globalThis as unknown as { ɵloadPlugin?: unknown })
          .ɵloadPlugin === 'function',
      undefined,
      { timeout: 30_000 },
    );
    await page.evaluate(
      async ({ host, pluginCode, permissions }) => {
        await (
          globalThis as unknown as {
            ɵloadPlugin: (
              manifestValue: Record<string, unknown>,
            ) => Promise<void>;
          }
        ).ɵloadPlugin({
          pluginId: '00000000-0000-0000-0000-000000000000',
          name: 'Zircon ZUI Layout Visual Validation',
          description: 'All tracked ZUI assets on the official Penpot canvas',
          version: 2,
          host,
          code: pluginCode,
          permissions,
        });
      },
      {
        host: pluginHost,
        pluginCode: `assets/plugin.js?sha256=${pluginBundleSha256}`,
        permissions: JSON.parse(
          await readFile(resolve(pluginDistRoot, 'manifest.json'), 'utf8'),
        ).permissions as string[],
      },
    );
    const pluginFrame = page.frameLocator('plugin-modal iframe');
    const fileInput = pluginFrame.locator('input[type="file"]');
    const statusPanel = pluginFrame.locator('.status-panel');
    const exportButton = pluginFrame.getByRole('button', {
      name: 'Export selected',
    });
    await fileInput.waitFor({ timeout: 30_000 });
    if (officialAssetReplay) {
      const browserRuntime = captureBrowserRuntime
        ? await captureBrowserRuntime(page).catch((error) => {
            pageErrors.push(
              `browser runtime receipt: ${error instanceof Error ? error.message : String(error)}`,
            );
            return null;
          })
        : null;
      const [title, readyState] = await Promise.all([
        page.title().catch(() => ''),
        page
          .evaluate(() => document.readyState)
          .catch(() => 'unavailable'),
      ]);
      assert.ok(replayReceiptPath);
      const receipt = {
        schema: 'dev.zircon.penpot.official-frontend-session-replay',
        version: 1,
        status: 'pending',
        accepted: false,
        pendingReason:
          'Official frontend startup replay only; no source import, capture, or export/apply validation is recorded by this receipt.',
        createdAtUtc: new Date().toISOString(),
        backend: 'repository-mock',
        persistenceClaim: false,
        sessionProcessPid: process.pid,
        pluginBundleSha256,
        sourceFingerprints: captureProgramFingerprints ?? [],
        officialAssetManifest: officialAssetManifestEvidence(
          officialAssetReplay,
        ),
        replayedAssets: officialAssetReplay.servedAssets,
        browserRuntime,
        page: { url: page.url(), title, readyState },
        startupNetwork,
        pageErrors: pageErrors.slice(0, 40),
        unexpectedRpcs: [...unexpectedRpcs],
      };
      await writeFile(
        replayReceiptPath,
        `${JSON.stringify(receipt, null, 2)}\n`,
        'utf8',
      );
    }
    return {
      context,
      page,
      pluginFrame,
      fileInput,
      statusPanel,
      exportButton,
      fontResources,
      pageErrors,
      unexpectedRpcs,
    };
  } catch (error) {
    fontResources.stop();
    const failureScreenshot = resolveLayoutCatalogPath(
      catalogRoot,
      'penpot-session-startup-failure.png',
    );
    await page
      .screenshot({ path: failureScreenshot, fullPage: true, timeout: 10_000 })
      .catch(() => undefined);
    const [title, bodyText, documentState] = await Promise.all([
      page.title().catch(() => ''),
      page
        .locator('body')
        .innerText({ timeout: 1_000 })
        .catch(() => ''),
      page
        .evaluate(() => ({
          readyState: document.readyState,
          scripts: Array.from(document.scripts, (script) => script.src),
          bodyHtml: document.body?.innerHTML.slice(0, 1_200) ?? '',
        }))
        .catch(() => ({ readyState: 'unavailable', scripts: [], bodyHtml: '' })),
    ]);
    const diagnostic = {
      url: page.url(),
      title,
      pageErrors: pageErrors.slice(0, 40),
      startupNetwork: startupNetwork.slice(0, 80),
      bodyText: bodyText.replace(/\s+/g, ' ').trim().slice(0, 1_000),
      documentState,
      failureScreenshot,
    };
    await writeFile(
      resolveLayoutCatalogPath(catalogRoot, 'penpot-session-startup-failure.json'),
      `${JSON.stringify(diagnostic, null, 2)}\n`,
      'utf8',
    ).catch(() => undefined);
    if (officialAssetReplay && replayReceiptPath) {
      const browserRuntime = captureBrowserRuntime
        ? await captureBrowserRuntime(page).catch((runtimeError) => {
            pageErrors.push(
              `browser runtime receipt: ${runtimeError instanceof Error ? runtimeError.message : String(runtimeError)}`,
            );
            return null;
          })
        : null;
      const failedReceipt = {
        schema: 'dev.zircon.penpot.official-frontend-session-replay',
        version: 1,
        status: 'pending',
        accepted: false,
        pendingReason:
          'The verified official frontend bundles were replayed, but Penpot startup did not complete; this is diagnostic evidence only.',
        createdAtUtc: new Date().toISOString(),
        backend: 'repository-mock',
        persistenceClaim: false,
        sessionProcessPid: process.pid,
        pluginBundleSha256,
        sourceFingerprints: captureProgramFingerprints ?? [],
        officialAssetManifest: officialAssetManifestEvidence(
          officialAssetReplay,
        ),
        replayedAssets: officialAssetReplay.servedAssets,
        browserRuntime,
        page: {
          url: page.url(),
          title,
          readyState: documentState.readyState,
        },
        startupNetwork,
        pageErrors: pageErrors.slice(0, 40),
        unexpectedRpcs: [...unexpectedRpcs],
        startupDiagnostic: diagnostic,
      };
      await writeFile(
        replayReceiptPath,
        `${JSON.stringify(failedReceipt, null, 2)}\n`,
        'utf8',
      ).catch(() => undefined);
    }
    await context.close();
    throw new Error(
      `${error instanceof Error ? error.message : String(error)}; Penpot startup diagnostics: ${JSON.stringify(diagnostic)}`,
      { cause: error },
    );
  }
}

interface OfficialScriptAsset {
  name: string;
  url: string;
  bodyPath: string;
  headersPath: string;
  byteLength: number;
  sha256: string;
  headersSha256: string;
  responseStatus: number;
  responseContentType: string;
  responseCacheControl: string | null;
  responseLastModified: string | null;
}

interface OfficialScriptAssetManifest {
  schema: 'dev.zircon.penpot.official-frontend-assets';
  version: 1;
  status: 'complete';
  origin: string;
  appVersion: string;
  assets: OfficialScriptAsset[];
}

interface OfficialScriptAssetReplay {
  cacheRoot: string;
  manifestPath: string;
  manifestSha256: string;
  manifest: OfficialScriptAssetManifest;
  servedAssets: Array<{
    name: string;
    url: string;
    byteLength: number;
    sha256: string;
    servedAtUtc: string;
  }>;
}

function nextOfficialSessionReplayReceiptPath(cacheRoot: string): string {
  return resolve(
    cacheRoot,
    `official-frontend-session-replay-receipt-${process.pid}-${randomUUID()}.json`,
  );
}

function officialAssetManifestEvidence(replay: OfficialScriptAssetReplay) {
  return {
    path: replay.manifestPath,
    sha256: replay.manifestSha256,
    origin: replay.manifest.origin,
    appVersion: replay.manifest.appVersion,
    assets: replay.manifest.assets.map(
      ({
        name,
        url,
        bodyPath,
        headersPath,
        byteLength,
        sha256,
        headersSha256,
        responseStatus,
        responseContentType,
      }) => ({
        name,
        url,
        bodyPath,
        headersPath,
        byteLength,
        sha256,
        headersSha256,
        responseStatus,
        responseContentType,
      }),
    ),
  };
}

/** Replay hash-verified official JS bodies to diagnose CDN delivery stalls. */
async function installOfficialAssetReplay(
  page: Page,
  penpotBaseUrl: string,
  startupNetwork: string[],
): Promise<OfficialScriptAssetReplay | undefined> {
  const configuredRoot = process.env['ZUI_PENPOT_OFFICIAL_ASSET_CACHE_ROOT'];
  if (!configuredRoot) return undefined;

  const cacheRoot = requireApprovedPenpotArtifactDirectory(
    configuredRoot,
    'ZUI_PENPOT_OFFICIAL_ASSET_CACHE_ROOT',
  );
  const physicalCacheRoot = realpathSync.native(cacheRoot);
  const configuredManifestName =
    process.env['ZUI_PENPOT_OFFICIAL_ASSET_MANIFEST_NAME'] ??
    'official-frontend-assets-manifest.json';
  assert.ok(
    !isAbsolute(configuredManifestName),
    'ZUI_PENPOT_OFFICIAL_ASSET_MANIFEST_NAME must be relative to the verified cache root',
  );
  const receiptPath = resolve(cacheRoot, configuredManifestName);
  const physicalReceiptPath = realpathSync.native(receiptPath);
  const physicalReceiptRelative = relative(
    physicalCacheRoot,
    physicalReceiptPath,
  );
  assert.ok(
    physicalReceiptRelative &&
      physicalReceiptRelative !== '..' &&
      !physicalReceiptRelative.startsWith(`..${sep}`),
    `Official asset manifest resolves outside its cache root: ${configuredManifestName}`,
  );
  const receiptBytes = await readFile(receiptPath);
  const manifest = JSON.parse(
    receiptBytes.toString('utf8'),
  ) as OfficialScriptAssetManifest;
  assert.equal(
    manifest.schema,
    'dev.zircon.penpot.official-frontend-assets',
    'Official asset replay receipt has an unsupported schema',
  );
  assert.equal(manifest.version, 1);
  assert.equal(manifest.status, 'complete');
  assert.equal(manifest.origin, new URL(penpotBaseUrl).origin);

  const requiredNames = ['libs.js', 'main.js', 'translation.en.js'];
  const allowedNames = [
    ...requiredNames,
    'shared.js',
    'main-workspace.js',
    'rasterizer.js',
  ];
  const assetNames = manifest.assets.map((asset) => asset.name);
  assert.equal(
    new Set(assetNames).size,
    assetNames.length,
    'Official asset replay receipt cannot contain duplicate bundle names',
  );
  assert.ok(
    requiredNames.every((name) => assetNames.includes(name)) &&
      assetNames.every((name) => allowedNames.includes(name)) &&
      assetNames.length >= requiredNames.length &&
      assetNames.length <= allowedNames.length,
    'Official asset replay receipt must contain the three app bundles and may include captured Penpot workspace or rasterizer bundles',
  );

  const verifiedAssets = new Map<string, OfficialScriptAsset & { body: Buffer }>();
  for (const asset of manifest.assets) {
    const assetUrl = new URL(asset.url);
    assert.equal(assetUrl.origin, manifest.origin);
    assert.equal(assetUrl.searchParams.get('version'), manifest.appVersion);
    assert.equal(assetUrl.pathname, `/js/${asset.name}`);
    assert.equal(asset.responseStatus, 200);
    assert.ok(asset.responseContentType.startsWith('application/javascript'));

    const resolveCacheFile = (relativePath: string): string => {
      assert.ok(relativePath && !isAbsolute(relativePath));
      const candidate = resolve(cacheRoot, relativePath);
      const lexicalRelative = relative(cacheRoot, candidate);
      assert.ok(
        lexicalRelative &&
          lexicalRelative !== '..' &&
          !lexicalRelative.startsWith(`..${sep}`),
        `Official asset replay path escapes its cache root: ${relativePath}`,
      );
      const physical = realpathSync.native(candidate);
      const physicalRelative = relative(physicalCacheRoot, physical);
      assert.ok(
        physicalRelative &&
          physicalRelative !== '..' &&
          !physicalRelative.startsWith(`..${sep}`),
        `Official asset replay path resolves outside its cache root: ${relativePath}`,
      );
      assert.ok(statSync(physical).isFile());
      return physical;
    };

    const body = await readFile(resolveCacheFile(asset.bodyPath));
    const headerBytes = await readFile(resolveCacheFile(asset.headersPath));
    assert.equal(body.byteLength, asset.byteLength);
    assert.equal(
      createHash('sha256').update(body).digest('hex'),
      asset.sha256,
      `Official asset replay body hash mismatch: ${asset.name}`,
    );
    assert.equal(
      createHash('sha256').update(headerBytes).digest('hex'),
      asset.headersSha256,
      `Official asset replay response-header hash mismatch: ${asset.name}`,
    );
    const responseHeaders = headerBytes.toString('utf8');
    assert.match(responseHeaders, /^HTTP\/\S+ 200 OK$/m);
    assert.doesNotMatch(responseHeaders, /^content-range:/im);
    assert.match(
      responseHeaders,
      /^content-type:\s*application\/javascript/im,
    );
    verifiedAssets.set(asset.url, { ...asset, body });
  }

  const replay: OfficialScriptAssetReplay = {
    cacheRoot,
    manifestPath: receiptPath,
    manifestSha256: createHash('sha256').update(receiptBytes).digest('hex'),
    manifest,
    servedAssets: [],
  };

  await page.route(
    (url) => verifiedAssets.has(url.href),
    async (route) => {
      const asset = verifiedAssets.get(route.request().url());
      assert.ok(asset, 'Verified official asset route lost its mapping');
      startupNetwork.push(
        `official asset replay ${asset.name}: ${asset.byteLength} bytes sha256=${asset.sha256}`,
      );
      replay.servedAssets.push({
        name: asset.name,
        url: asset.url,
        byteLength: asset.byteLength,
        sha256: asset.sha256,
        servedAtUtc: new Date().toISOString(),
      });
      await route.fulfill({
        status: asset.responseStatus,
        contentType: asset.responseContentType,
        headers: {
          ...(asset.responseCacheControl
            ? { 'cache-control': asset.responseCacheControl }
            : {}),
          ...(asset.responseLastModified
            ? { 'last-modified': asset.responseLastModified }
            : {}),
        },
        body: asset.body,
      });
    },
  );
  return replay;
}

async function installMockBackend(
  page: Page,
  unexpectedRpcs: Set<string>,
): Promise<void> {
  const media = new Map<string, { bytes: Buffer; mime: string }>();
  await page.route('**/assets/by-file-media-id/**', async (route) => {
    const id = new URL(route.request().url()).pathname.split('/').pop() ?? '';
    const image = media.get(id);
    if (!image)
      return route.fulfill({ status: 404, body: 'Unknown fixture image' });
    return route.fulfill({
      status: 200,
      contentType: image.mime,
      body: image.bytes,
    });
  });
  await page.route('**/js/config.js*', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/javascript',
      body: 'var penpotFlags = "";\n',
    }),
  );
  await page.route('**/api/main/methods/**', async (route) => {
    const url = new URL(route.request().url());
    const command = url.pathname.split('/').pop() ?? '';
    if (command === 'upload-file-media-object') {
      const request = route.request();
      const payload = request.postDataBuffer();
      assert.ok(payload, 'Missing multipart image payload');
      const form = await new Response(Uint8Array.from(payload), {
        headers: { 'content-type': request.headers()['content-type'] },
      }).formData();
      const file = form.get('content');
      assert.ok(file instanceof Blob, 'Missing multipart image content');
      const bytes = Buffer.from(await file.arrayBuffer());
      const size = await page.evaluate(
        async ({ bytes, mime }) => {
          const bitmap = await createImageBitmap(
            new Blob([new Uint8Array(bytes)], { type: mime }),
          );
          const size = { width: bitmap.width, height: bitmap.height };
          bitmap.close();
          return size;
        },
        { bytes: [...bytes], mime: file.type },
      );
      const id = randomUUID();
      media.set(id, { bytes, mime: file.type });
      await route.fulfill({
        status: 200,
        contentType: 'application/transit+json',
        body: JSON.stringify({
          '~:id': `~u${id}`,
          '~:file-id': `~u${mockFileId}`,
          '~:name': String(form.get('name') ?? 'Image'),
          '~:width': size.width,
          '~:height': size.height,
          '~:mtype': file.type,
        }),
      });
      return;
    }
    if (command === 'get-file') {
      await route.fulfill({
        status: 200,
        contentType: 'application/transit+json',
        path: resolve(apiSuiteRoot, 'ci/fixtures/get-file.json'),
      });
      return;
    }
    if (command === 'update-file') {
      await route.fulfill({
        status: 200,
        contentType: 'application/transit+json',
        body: JSON.stringify({ '~:revn': 1, '~:lagged': [] }),
      });
      return;
    }
    if (mockNoopRpcs.has(command)) {
      await route.fulfill({
        status: 200,
        contentType: 'application/transit+json',
        body: '{}',
      });
      return;
    }
    const fixture = mockRpcFixtures.get(command);
    if (fixture) {
      await route.fulfill({
        status: 200,
        contentType: 'application/transit+json',
        path: resolve(frontendRoot, 'playwright/data', fixture),
      });
      return;
    }
    unexpectedRpcs.add(`${route.request().method()} ${url.pathname}`);
    await route.fulfill({ status: 501, body: 'Unexpected mocked RPC' });
  });
}

async function installWebSocketMock(page: Page): Promise<Set<string>> {
  const created = new Set<string>();
  await page.exposeFunction('onMockWebSocketConstructor', (url: string) =>
    created.add(url),
  );
  await page.addInitScript({
    path: resolve(frontendRoot, 'playwright/scripts/MockWebSocket.js'),
  });
  return created;
}

async function openNotificationsWebSocket(
  page: Page,
  created: Set<string>,
): Promise<void> {
  await waitFor(
    () => [...created].some((url) => url.includes('/ws/notifications')),
    'Penpot notifications WebSocket',
    penpotStartupTimeoutMs,
  );
  const url = [...created].find((candidate) =>
    candidate.includes('/ws/notifications'),
  );
  assert.ok(url);
  await page.evaluate((socketUrl) => {
    (
      WebSocket as unknown as {
        getByURL: (candidate: string) => { mockOpen: () => void } | undefined;
      }
    )
      .getByURL(socketUrl)
      ?.mockOpen();
  }, url);
}

async function installPluginAssetRoute(page: Page): Promise<void> {
  await page.route('**/__zircon-zui-plugin__/**', async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    const prefix = '/__zircon-zui-plugin__/';
    const relativePath = pathname.startsWith(prefix)
      ? pathname.slice(prefix.length)
      : '';
    const candidate = resolve(pluginDistRoot, relativePath || 'index.html');
    assert.ok(
      candidate === pluginDistRoot ||
        candidate.startsWith(`${pluginDistRoot}${sep}`),
      `Plugin asset escaped dist root: ${pathname}`,
    );
    await fulfillFile(route, candidate);
  });
}

async function fulfillFile(route: Route, path: string): Promise<void> {
  const contentTypes: Record<string, string> = {
    '.css': 'text/css',
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.json': 'application/json',
    '.svg': 'image/svg+xml',
  };
  await route.fulfill({
    status: 200,
    contentType: contentTypes[extname(path)] ?? 'application/octet-stream',
    headers: { 'cache-control': 'no-store' },
    body: await readFile(path),
  });
}

async function waitFor(
  predicate: () => boolean | Promise<boolean>,
  label: string,
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (!(await predicate())) {
    if (Date.now() >= deadline)
      throw new Error(`Timed out waiting for ${label}`);
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
  }
}

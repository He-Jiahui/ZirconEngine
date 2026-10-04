import assert from 'node:assert/strict';
import { verifySliderTextEdits } from './penpot-slider-browser-contract';
import { verifySegmentedTextEdits } from './penpot-segmented-browser-contract';
import { verifySlotPaddingEdits } from './penpot-slot-padding-browser-contract';
import { verifyNativeComponentEdits } from './penpot-native-components-browser-contract';
import { verifyFieldTextEdits } from './penpot-field-browser-contract';
import { existsSync } from 'node:fs';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { dirname, extname, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

import { chromium, type Page, type Route } from 'playwright';

import { parseZuiDocument, zuiNodes } from '../src/bridge/zui-document.js';
import { navigateWithRetry } from './penpot-browser-navigation';
import { resolvePluginDistRoot } from './zui-layout-penpot-session';
import { requireApprovedPenpotArtifactDirectory } from './zui-layout-artifact-path';

interface BrowserContractSummary {
  contract: 'penpot-browser-canvas-mocked-backend';
  penpotBaseUrl: string;
  semanticBoards: number;
  totalShapes: number;
  exportedFile: string;
  projectedEdits: number;
  preservedRuntimeFields: true;
  screenshot: string;
}

const here = dirname(fileURLToPath(import.meta.url));
const appRoot = resolve(here, '..');
const penpotRoot = resolve(appRoot, '../../../../third_party/penpot');
const frontendRoot = resolve(penpotRoot, 'frontend');
const apiSuiteRoot = resolve(penpotRoot, 'plugins/apps/plugin-api-test-suite');
const pluginDistRoot = resolvePluginDistRoot();
const fixturePath = resolve(appRoot, 'src/bridge/roundtrip-fixture.zui');
const artifactRoot = process.env['ZUI_PLUGIN_ARTIFACT_ROOT'];
if (!artifactRoot) throw new Error('Set ZUI_PLUGIN_ARTIFACT_ROOT to an approved artifact directory.');
const screenshotPath = resolve(artifactRoot, process.env['ZIRCON_PENPOT_SCREENSHOT'] ?? 'a1-penpot-canvas-roundtrip.png');
requireApprovedPenpotArtifactDirectory(dirname(screenshotPath), 'Browser screenshot directory');
const penpotBaseUrl = new URL(
  process.env['PENPOT_BASE_URL'] ?? 'https://design.penpot.app',
).origin;
const pluginHost = new URL('/__zircon-zui-plugin__/', penpotBaseUrl).href;
const editedTitle = 'Edited in Penpot canvas';
const editedRadius = 12;

const browserExecutable = [
  process.env['PLAYWRIGHT_CHROMIUM_EXECUTABLE'],
  process.env['ProgramFiles']
    ? resolve(
        process.env['ProgramFiles'],
        'Google/Chrome/Application/chrome.exe',
      )
    : undefined,
  process.env['ProgramFiles(x86)']
    ? resolve(
        process.env['ProgramFiles(x86)'],
        'Microsoft/Edge/Application/msedge.exe',
      )
    : undefined,
].find((candidate): candidate is string =>
  Boolean(candidate && existsSync(candidate)),
);

const mockTeamId = 'c7ce0794-0992-8105-8004-38e630f7920a';
const mockFileId = 'c7ce0794-0992-8105-8004-38f280443849';
const mockPageId = '66697432-c33d-8055-8006-2c62cc084cad';

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

async function installMockBackend(
  page: Page,
  unexpectedRpcs: Set<string>,
): Promise<void> {
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
  await page.exposeFunction('onMockWebSocketConstructor', (url: string) => {
    created.add(url);
  });
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
    const relative = pathname.startsWith(prefix)
      ? pathname.slice(prefix.length)
      : '';
    const candidate = resolve(pluginDistRoot, relative || 'index.html');
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
    if (Date.now() >= deadline) {
      throw new Error(`Timed out waiting for ${label}`);
    }
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 50));
  }
}

async function main(): Promise<void> {
  const source = await readFile(fixturePath, 'utf8');
  const sourceDocument = parseZuiDocument(source).document;
  const unexpectedRpcs = new Set<string>();
  const pageErrors: string[] = [];
  const browser = await chromium.launch({
    headless: true,
    args: ['--ignore-certificate-errors'],
    ...(browserExecutable ? { executablePath: browserExecutable } : {}),
  });
  const context = await browser.newContext({
    acceptDownloads: true,
    ignoreHTTPSErrors: true,
    locale: 'en-US',
    serviceWorkers: 'block',
    viewport: { width: 1440, height: 900 },
  });
  const page = await context.newPage();
  page.on('pageerror', (error) => pageErrors.push(error.message));

  try {
    const sockets = await installWebSocketMock(page);
    await installMockBackend(page, unexpectedRpcs);
    await installPluginAssetRoute(page);

    await navigateWithRetry(
      page,
      `${penpotBaseUrl}/#/workspace?team-id=${mockTeamId}&file-id=${mockFileId}&page-id=${mockPageId}&wasm=false`,
    );
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
      async ({ host, permissions }) => {
        await (
          globalThis as unknown as {
            ɵloadPlugin: (manifest: Record<string, unknown>) => Promise<void>;
          }
        ).ɵloadPlugin({
          pluginId: '00000000-0000-0000-0000-000000000000',
          name: 'Zircon ZUI Bridge Browser Contract',
          description: 'Real Penpot canvas round-trip contract',
          version: 2,
          host,
          code: 'assets/plugin.js',
          permissions,
        });
      },
      {
        host: pluginHost,
        permissions: JSON.parse(
          await readFile(resolve(pluginDistRoot, 'manifest.json'), 'utf8'),
        ).permissions as string[],
      },
    );

    const pluginFrame = page.frameLocator('plugin-modal iframe');
    const importButton = pluginFrame.getByRole('button', {
      name: 'Import .zui',
    });
    await importButton.waitFor({ timeout: 30_000 });
    await pluginFrame.locator('input[type="file"]').setInputFiles({
      name: 'penpot_roundtrip.zui',
      mimeType: 'text/plain',
      buffer: Buffer.from(source),
    });
    const statusPanel = pluginFrame.locator('.status-panel');
    await waitFor(async () => {
      const level = await statusPanel.getAttribute('data-level');
      return level !== null && level !== 'working' && level !== 'idle';
    }, 'completed ZUI import status');
    const importStatus = (await statusPanel.innerText())
      .replace(/\s+/g, ' ')
      .trim();
    assert.match(
      importStatus,
      /Penpot Roundtrip \u00b7 8 nodes/,
      `Unexpected Penpot import status: ${importStatus}`,
    );

    const layerRows = page.getByTestId('layer-row');
    const assetRow = layerRows.filter({
      hasText: 'ZUI \u00b7 Penpot Roundtrip',
    });
    await assetRow.waitFor({ timeout: 30_000 });
    const assetToggle = assetRow.getByTestId('toggle-content');
    if ((await assetToggle.getAttribute('aria-expanded')) !== 'true') {
      await assetToggle.click({ modifiers: ['Alt'] });
    }
    const titleTextRow = layerRows.filter({ hasText: 'ZUI text \u00b7 title' });
    try {
      await titleTextRow.waitFor({ timeout: 30_000 });
    } catch (error) {
      const diagnostics = await layerRows.evaluateAll((rows) =>
        rows.map((row) => ({
          text: row.textContent?.replace(/\s+/g, ' ').trim(),
          expanded: row
            .querySelector('[aria-expanded]')
            ?.getAttribute('aria-expanded'),
        })),
      );
      throw new Error(
        `${error instanceof Error ? error.message : String(error)}; layer diagnostics=${JSON.stringify(diagnostics)}`,
        { cause: error },
      );
    }

    const rowNames = (await layerRows.allTextContents()).map((name) =>
      name.trim(),
    );
    const semanticBoards = rowNames.filter(
      (name) =>
        name.includes(' \u00b7 ') &&
        !name.startsWith('ZUI \u00b7') &&
        !name.startsWith('ZUI text \u00b7'),
    ).length;
    assert.equal(semanticBoards, 8);
    assert.deepEqual(
      rowNames.filter((name) => name.startsWith('ZUI text \u00b7')).sort(),
      ['cancel', 'detached_template', 'row_template', 'submit', 'title']
        .map((id) => `ZUI text \u00b7 ${id}`)
        .sort(),
    );
    assert.equal(rowNames.length, 15);

    await titleTextRow.click();
    await page.getByTestId('viewport').press('Enter');
    const textEditorSurface = page.locator('[contenteditable="true"]').last();
    await textEditorSurface.waitFor({ timeout: 10_000 });
    await page.keyboard.press('Control+A');
    await page.keyboard.type(editedTitle);
    await page.keyboard.press('Escape');
    await textEditorSurface.waitFor({ state: 'hidden', timeout: 10_000 });

    // Import can leave the workspace in the Frame drawing tool. A layer-row
    // click still updates selection in that mode, but the sidebar renders the
    // drawing preset controls instead of the selected frame's properties.
    const moveButton = page.getByRole('button', { name: 'Move (V)' });
    await moveButton.click();
    await waitFor(
      async () => (await moveButton.getAttribute('aria-pressed')) === 'true',
      'Penpot move tool selection',
    );

    const rootRow = layerRows.filter({ hasText: 'root \u00b7 VerticalBox' });
    await rootRow.click();
    const radiusSection = page.getByRole('region', {
      name: 'Border radius section',
    });
    await radiusSection.waitFor({ timeout: 10_000 });
    const radiusInput = radiusSection.locator('input').first();
    await radiusInput.fill(String(editedRadius));
    await radiusInput.press('Enter');
    await waitFor(
      async () => (await radiusInput.inputValue()) === String(editedRadius),
      'edited Penpot border radius',
    );

    const exportButton = pluginFrame.getByRole('button', {
      name: 'Export selected',
    });
    await waitFor(async () => exportButton.isEnabled(), 'enabled ZUI export');
    const downloadPromise = page.waitForEvent('download');
    await exportButton.click();
    const download = await downloadPromise;
    const downloadedPath = await download.path();
    assert.ok(downloadedPath);
    const exportedSource = await readFile(downloadedPath, 'utf8');
    const exportedDocument = parseZuiDocument(exportedSource).document;
    const sourceNodes = zuiNodes(sourceDocument);
    const exportedNodes = zuiNodes(exportedDocument);

    assert.equal(exportedNodes['title']?.props?.['text'], editedTitle);
    assert.equal(exportedNodes['root']?.props?.['corner_radius'], editedRadius);
    assert.deepEqual(
      exportedNodes['root']?.events,
      sourceNodes['root']?.events,
    );
    assert.deepEqual(
      exportedNodes['root']?.['zircon_extension'],
      sourceNodes['root']?.['zircon_extension'],
    );
    assert.deepEqual(
      exportedNodes['title']?.props?.['runtime_only'],
      sourceNodes['title']?.props?.['runtime_only'],
    );
    await pluginFrame
      .getByText('penpot_roundtrip.zui \u00b7 2 projected edits', {
        exact: true,
      })
      .waitFor({ timeout: 10_000 });

    await mkdir(dirname(screenshotPath), { recursive: true });
    await page.screenshot({ path: screenshotPath });
    const sliderContract = await verifySliderTextEdits(page, pluginFrame);
    const segmentedContract = await verifySegmentedTextEdits(page, pluginFrame);
    await page.screenshot({
      path: resolve(dirname(screenshotPath), 'segmented-browser-contract.png'),
    });
    const slotPaddingContract = await verifySlotPaddingEdits(page, pluginFrame);
    await page.screenshot({
      path: resolve(
        dirname(screenshotPath),
        'slot-padding-browser-contract.png',
      ),
    });
    const nativeComponentContract = await verifyNativeComponentEdits(
      page,
      pluginFrame,
    );
    await page.screenshot({
      path: resolve(
        dirname(screenshotPath),
        'native-components-browser-contract.png',
      ),
    });

    const fieldContract = await verifyFieldTextEdits(page, pluginFrame);
    assert.deepEqual([...unexpectedRpcs], []);
    assert.deepEqual(pageErrors, []);
    const summary: BrowserContractSummary = {
      contract: 'penpot-browser-canvas-mocked-backend',
      penpotBaseUrl,
      semanticBoards,
      totalShapes: rowNames.length,
      exportedFile: download.suggestedFilename(),
      projectedEdits: 2,
      preservedRuntimeFields: true,
      screenshot: screenshotPath,
    };
    console.log(JSON.stringify(summary));
    console.log(JSON.stringify(sliderContract));
    console.log(JSON.stringify(segmentedContract));
    console.log(JSON.stringify(slotPaddingContract));
    console.log(JSON.stringify(nativeComponentContract));
    console.log(JSON.stringify(fieldContract));
  } catch (error) {
    const page = browser.contexts()[0]?.pages()[0];
    if (page) {
      await mkdir(dirname(screenshotPath), { recursive: true });
      await page.screenshot({
        path: resolve(dirname(screenshotPath), 'browser-contract-failure.png'),
      });
      await writeFile(
        resolve(dirname(screenshotPath), 'browser-contract-failure.json'),
        JSON.stringify(
          {
            error: String(error),
            inputs: await page.locator('input').evaluateAll((inputs) =>
              inputs.map((input) => ({
                label: input.getAttribute('aria-label'),
                title: input.closest('[title]')?.getAttribute('title'),
                name: input.getAttribute('data-name'),
                value: input instanceof HTMLInputElement ? input.value : null,
                visible: input.getClientRects().length > 0,
              })),
            ),
            sidebar: await page
              .locator('[data-testid="layer-row"]')
              .allTextContents(),
          },
          null,
          2,
        ),
      );
    }
    throw error;
  } finally {
    await browser.close();
  }
}

void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});

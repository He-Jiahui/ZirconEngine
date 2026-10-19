import { mkdir, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { chromium } from 'playwright';

import { createLayoutCurrentContext, layoutCurrentOutputPath } from './zui-layout-paths';
import { readCurrentDeliveryCatalog } from './zui-layout-delivery';
import { verifyCurrentFiles } from './zui-layout-evidence';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';

export async function runCurrentLayoutContactSheet(args: string[] = []): Promise<void> {
  if (args.length > 2 || args.some((arg) => !/^\d+$/.test(arg)))
    throw new Error('Current contact sheet accepts only integer start/count; historical replay is not available');
  const start = Number(args[0] ?? 0);
  const count = Number(args[1] ?? 4);
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(count) || count <= 0)
    throw new Error('Contact sheet start/count must be finite non-negative integers with positive count');
  const context = createLayoutCurrentContext();
  const preflight = async () => {
    const manifest = await readCurrentDeliveryCatalog(context);
    const entries = manifest.entries.slice(start, start + count);
    if (!entries.length) throw new Error('Contact sheet selection is empty');
    for (const entry of entries) {
      if (entry.penpotEvidence?.length || entry.engineEvidence?.length)
        await verifyCurrentFiles(entry, context.repoRoot, context.outputRoot, entry.review?.status === 'accepted', context, 'strict-capture');
      layoutCurrentOutputPath(context, entry.previewPath);
    }
    layoutCurrentOutputPath(context, `review-sheets/sheet-${start}.html`);
    layoutCurrentOutputPath(context, `review-sheets/sheet-${start}.png`);
    return entries;
  };
  await preflight();
  await withLayoutCatalogLock(context.outputRoot, { command: 'zui-layout-contact-sheet', context }, async () => {
  const entries = await preflight();
  const output = layoutCurrentOutputPath(context, 'review-sheets');
  const htmlPath = layoutCurrentOutputPath(context, `review-sheets/sheet-${start}.html`);
  await mkdir(output, { recursive: true });
const escape = (text: string) =>
  text.replaceAll('&', '&amp;').replaceAll('<', '&lt;');
await writeFile(
  htmlPath,
  `<!doctype html><meta charset="utf-8"><style>*{box-sizing:border-box}body{background:#e8ebed;margin:0;padding:16px;font:14px system-ui}main{display:grid;grid-template-columns:1fr 1fr;gap:16px}section{min-width:0}header{height:40px;overflow:hidden}img{display:block;width:100%;height:420px;object-fit:contain;background:#272b2d}</style><main>${entries.map((entry, index) => `<section><header>${start + index}: ${escape(entry.name)}<br>${entry.visualStatus}</header><img src="${pathToFileURL(layoutCurrentOutputPath(context, entry.previewPath)).href}"></section>`).join('')}</main>`,
);
const browser = await chromium.launch({
  headless: true,
  executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe',
});
try {
  const page = await browser.newPage({
    viewport: { width: 1440, height: Math.ceil(count / 2) * 480 + 16 },
  });
  await page.goto(pathToFileURL(htmlPath).href);
  await page
    .locator('img')
    .evaluateAll(async (images) =>
      Promise.all(images.map((image) => (image as HTMLImageElement).decode())),
    );
  const path = layoutCurrentOutputPath(context, `review-sheets/sheet-${start}.png`);
  await page.screenshot({ path, fullPage: true });
  console.log(
    JSON.stringify({ path, entries: entries.map((entry) => entry.sourcePath) }),
  );
} finally {
  await browser.close();
}
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  void runCurrentLayoutContactSheet(process.argv.slice(2)).catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });

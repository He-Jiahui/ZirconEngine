import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import type { Browser } from 'playwright';
import { createLayoutCurrentContext, layoutCurrentOutputPath, type LayoutCurrentContext } from './zui-layout-paths';
import { readCurrentLayoutOutputCatalog } from './zui-layout-gallery';
import { withLayoutCatalogLock } from './zui-layout-catalog-lock';
import type { CatalogManifest } from './zui-layout-catalog';

async function checkGallery(context: LayoutCurrentContext, catalog: CatalogManifest, launchBrowser?: () => Promise<Browser>): Promise<void> {
const results: {
  width: number;
  height: number;
  entries: number;
  filteredName: string;
  horizontalOverflow: boolean;
}[] = [];
const browser = launchBrowser ? await launchBrowser() : await (await import('playwright')).chromium.launch({
  headless: true,
  executablePath: 'C:/Program Files/Google/Chrome/Application/chrome.exe',
});
try {
  const page = await browser.newPage();
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(error.message));
  for (const viewport of [
    { width: 1440, height: 1000 },
    { width: 390, height: 844 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(pathToFileURL(layoutCurrentOutputPath(context, 'gallery.html')).href);
    assert.equal(await page.locator('article').count(), catalog.sourceCount);
    const firstArticle = page.locator('article').first();
    const name = await firstArticle.locator('h2').innerText();
    await page.locator('#search').fill(name);
    assert.equal(await page.locator('article:visible').count(), 1);
    const visibleImage = page.locator('article:visible img').first();
    if (await visibleImage.count()) {
      await visibleImage.evaluate(async (element) =>
        (element as HTMLImageElement).decode(),
      );
    }
    const fits = await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    );
    assert.ok(fits, `Gallery overflows at ${viewport.width}px`);
    await page.screenshot({
      path: layoutCurrentOutputPath(context, `evidence/gallery-${viewport.width}.png`),
      fullPage: true,
    });
    results.push({
      ...viewport,
      entries: catalog.sourceCount,
      filteredName: name,
      horizontalOverflow: !fits,
    });
    await page.locator('#search').fill('no-matching-layout');
    assert.equal(await page.locator('article:visible').count(), 0);
  }
  assert.deepEqual(errors, []);
  await writeFile(
    layoutCurrentOutputPath(context, 'evidence/gallery-qa.json'),
    `${JSON.stringify({ generatedAt: new Date().toISOString(), results, errors }, null, 2)}\n`,
  );
  console.log(JSON.stringify({ gallery: 'passed', viewports: [1440, 390] }));
} finally {
  await browser.close();
}
}

export async function runCurrentLayoutGalleryCheck(options: { launchBrowser?: () => Promise<Browser> } = {}): Promise<void> {
  const context = createLayoutCurrentContext();
  const validate = async () => {
    const { manifest, catalogSha256 } = await readCurrentLayoutOutputCatalog(context);
    const html = await readFile(layoutCurrentOutputPath(context, 'gallery.html'), 'utf8');
    const bindings = [...html.matchAll(/<meta name="zircon-layout-catalog-sha256" content="([a-f0-9]{64})">/g)];
    if (bindings.length !== 1 || bindings[0]?.[1] !== catalogSha256)
      throw new Error('Gallery HTML does not match the current catalog SHA');
    if (!manifest.sourceCount) throw new Error('Cannot check an empty current gallery');
    return manifest;
  };
  await validate();
  await withLayoutCatalogLock(context.outputRoot, { command: 'zui-layout-gallery-check', context }, async () => {
    const catalog = await validate();
    await mkdir(dirname(layoutCurrentOutputPath(context, 'evidence/gallery-qa.json')), { recursive: true });
    await checkGallery(context, catalog, options.launchBrowser);
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  void runCurrentLayoutGalleryCheck().catch((error: unknown) => { console.error(error); process.exitCode = 1; });

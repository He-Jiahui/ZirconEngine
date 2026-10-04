import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { chromium } from 'playwright';

const root = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../../docs/_data/layout',
);
const catalog = JSON.parse(
  await readFile(resolve(root, 'catalog.json'), 'utf8'),
) as { sourceCount: number };
const results: {
  width: number;
  height: number;
  entries: number;
  filteredName: string;
  horizontalOverflow: boolean;
}[] = [];
const browser = await chromium.launch({
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
    await page.goto(pathToFileURL(resolve(root, 'gallery.html')).href);
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
      path: resolve(root, 'evidence', `gallery-${viewport.width}.png`),
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
    resolve(root, 'evidence/gallery-qa.json'),
    `${JSON.stringify({ generatedAt: new Date().toISOString(), results, errors }, null, 2)}\n`,
  );
  console.log(JSON.stringify({ gallery: 'passed', viewports: [1440, 390] }));
} finally {
  await browser.close();
}

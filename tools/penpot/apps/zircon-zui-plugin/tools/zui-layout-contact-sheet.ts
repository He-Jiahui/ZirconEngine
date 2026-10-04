import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { chromium } from 'playwright';
import type { CatalogManifest } from './zui-layout-catalog';

const root = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../../docs/_data/layout',
);
const manifest = JSON.parse(
  await readFile(resolve(root, 'catalog.json'), 'utf8'),
) as CatalogManifest;
const start = Number(process.argv[2] ?? 0);
const count = Number(process.argv[3] ?? 4);
const entries = manifest.entries.slice(start, start + count);
const output = resolve(root, 'review-sheets');
await mkdir(output, { recursive: true });
const htmlPath = resolve(output, `sheet-${start}.html`);
const escape = (text: string) =>
  text.replaceAll('&', '&amp;').replaceAll('<', '&lt;');
await writeFile(
  htmlPath,
  `<!doctype html><meta charset="utf-8"><style>*{box-sizing:border-box}body{background:#e8ebed;margin:0;padding:16px;font:14px system-ui}main{display:grid;grid-template-columns:1fr 1fr;gap:16px}section{min-width:0}header{height:40px;overflow:hidden}img{display:block;width:100%;height:420px;object-fit:contain;background:#272b2d}</style><main>${entries.map((entry, index) => `<section><header>${start + index}: ${escape(entry.name)}<br>${entry.visualStatus}</header><img src="${pathToFileURL(resolve(root, entry.previewPath)).href}"></section>`).join('')}</main>`,
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
  const path = resolve(output, `sheet-${start}.png`);
  await page.screenshot({ path, fullPage: true });
  console.log(
    JSON.stringify({ path, entries: entries.map((entry) => entry.sourcePath) }),
  );
} finally {
  await browser.close();
}

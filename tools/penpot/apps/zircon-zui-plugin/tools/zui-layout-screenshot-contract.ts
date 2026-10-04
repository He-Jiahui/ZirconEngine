import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium } from 'playwright';
import { screenshotPreview } from './zui-layout-screenshot';
import { mapMeasuredTexts } from './zui-layout-text-mapping';

const directory = await mkdtemp(join(tmpdir(), 'zui-svg-contract-'));
const browser = await chromium.launch({ channel: 'chrome', headless: true });
try {
  const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
  const session = await page.context().newCDPSession(page);
  for (const dpi of [1, 1.5]) {
    await session.send('Emulation.setDeviceMetricsOverride', {
      width: 800,
      height: 600,
      deviceScaleFactor: dpi,
      mobile: false,
    });
    await page.setContent(`<svg width="240" height="120" xmlns="http://www.w3.org/2000/svg">
      <defs><clipPath id="control-clip"><rect width="240" height="120"/></clipPath></defs>
      <g id="shape-board"><g id="shape-surface" clip-path="url(#control-clip)">
        <rect id="surface" width="240" height="120" fill="#ff0000"/>
        <g id="shape-label"><text x="16" y="64" fill="white" font-family="Arial" font-size="14">Stable control</text></g>
        <g id="shape-measurement" opacity="0"><text x="0" y="20">Invisible sizing probe</text></g>
      </g></g></svg>`);
    await page.evaluate(() => {
      setTimeout(
        () =>
          document.getElementById('surface')!.setAttribute('fill', '#123456'),
        250,
      );
    });
    const textPath = join(directory, `text-${dpi}.json`);
    const bytes = await screenshotPreview(
      page,
      'board',
      undefined,
      { x: 0, y: 0, width: 240, height: 120 },
      1,
      session,
      textPath,
    );
    const pixels = await page.evaluate(async (data) => {
      const image = new Image();
      image.src = `data:image/png;base64,${data}`;
      await image.decode();
      const canvas = document.createElement('canvas');
      canvas.width = image.width;
      canvas.height = image.height;
      const context = canvas.getContext('2d')!;
      context.drawImage(image, 0, 0);
      return {
        width: image.width,
        height: image.height,
        pixel: [...context.getImageData(4, 4, 1, 1).data],
      };
    }, bytes.toString('base64'));
    assert.equal(pixels.width, 240 * dpi);
    assert.equal(pixels.height, 120 * dpi);
    assert.deepEqual(pixels.pixel, [18, 52, 86, 255]);
    const evidence = JSON.parse(await readFile(textPath, 'utf8'));
    assert.equal(evidence.texts[0].shapeId, 'shape-label');
    assert.equal(evidence.texts.length, 1);
    assert.deepEqual(evidence.texts[0].ancestorShapeIds, [
      'shape-surface',
      'shape-board',
    ]);
    const ownership = mapMeasuredTexts(
      [{ nodeId: 'control', shapeId: 'surface' }],
      evidence.texts,
    );
    assert.equal(ownership.get('control')?.length, 1);
    assert.match(evidence.nativeSvgAudit.nativeSvgSha256, /^[0-9a-f]{64}$/);
    assert.equal(await page.locator('#zircon-native-svg-capture').count(), 0);
  }
  await session.detach();
  console.log(
    JSON.stringify({
      passed: true,
      dpi: [1, 1.5],
      lateMutation: true,
      sourceTextMapping: true,
      semanticTextOwnership: true,
      invisibleMeasurementExcluded: true,
      resourceIds: 'isolated',
    }),
  );
} finally {
  await browser.close();
  await rm(directory, { recursive: true, force: true });
}

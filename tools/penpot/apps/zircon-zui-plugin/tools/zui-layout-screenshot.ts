import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import type { CDPSession, Page } from 'playwright';
import { captureTextEvidence } from './zui-layout-text-evidence';
import type { MeasuredText } from './zui-layout-text-structure';
import {
  rasterFramesEquivalent,
  readPng,
  type ScreenshotPixels,
} from './zui-layout-png';
import { waitForNativeSvg } from './zui-layout-svg-ready';

export async function screenshotPreview(
  page: Page,
  boardId: string,
  path: string | undefined,
  bounds: { x: number; y: number; width: number; height: number },
  expectedTextNodes: number,
  captureSession?: CDPSession,
  textEvidencePath?: string,
  onTextEvidence?: (texts: MeasuredText[]) => void,
): Promise<Buffer> {
  await page.waitForFunction(
    ({ boardId, expectedTextNodes }) => {
      const root = document.querySelector(`#shape-${boardId}`);
      if (!root) return false;
      const thumbnails = [...root.querySelectorAll('.thumbnail-bitmap')];
      if (
        thumbnails.some((image) => getComputedStyle(image).display !== 'none')
      )
        return false;
      const texts = [
        ...root.querySelectorAll('text, foreignObject span'),
      ].filter(
        (element) =>
          element.textContent?.trim() && !element.querySelector('span'),
      );
      return texts.length >= expectedTextNodes;
    },
    { boardId, expectedTextNodes },
    { timeout: 15_000 },
  );
  await page.evaluate(() => document.fonts.ready);
  const nativeSvgAudit = await waitForNativeSvg(page, boardId);
  const fontAudit = await page.evaluate((boardId) => {
    const root = document.querySelector(`#shape-${boardId}`);
    const texts = [
      ...(root?.querySelectorAll('text, foreignObject span') ?? []),
    ]
      .filter((element) => element.textContent?.trim())
      .map((element) => {
        const style = getComputedStyle(element);
        const font = `${style.fontStyle} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
        return {
          text: element.textContent ?? '',
          font,
          loaded: document.fonts.check(font, element.textContent ?? ''),
        };
      });
    return {
      loaded: texts.every(({ loaded }) => loaded),
      faces: [...document.fonts]
        .map((font) => ({ family: font.family, status: font.status }))
        .slice(0, 8),
      texts,
    };
  }, boardId);
  assert.ok(
    fontAudit.loaded,
    `Penpot preview font failed to load: ${JSON.stringify(fontAudit)}`,
  );
  assert.ok(
    bounds.width > 0 && bounds.height > 0,
    'Missing authored board bounds',
  );
  const overlayId = 'zircon-native-svg-capture';
  await page.evaluate(
    ({ boardId, bounds, overlayId }) => {
      const original = document.querySelector(`#shape-${boardId}`);
      const sourceSvg = original?.closest('svg');
      if (!original || !sourceSvg)
        throw new Error('Missing rendered Penpot SVG');
      const overlay = document.createElement('div');
      overlay.id = overlayId;
      Object.assign(overlay.style, {
        position: 'fixed',
        left: '0',
        top: '0',
        zIndex: '2147483647',
        width: `${bounds.width}px`,
        height: `${bounds.height}px`,
        background: '#151719',
      });
      const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
      svg.setAttribute('width', String(bounds.width));
      svg.setAttribute('height', String(bounds.height));
      svg.setAttribute(
        'viewBox',
        `${bounds.x} ${bounds.y} ${bounds.width} ${bounds.height}`,
      );
      svg.setAttribute('fill', 'none');
      for (const defs of sourceSvg.querySelectorAll(
        ':scope > defs, :scope > style',
      ))
        svg.appendChild(defs.cloneNode(true));
      const clone = original.cloneNode(true) as Element;
      // Canvas text inherits viewport CSS. Preserve it when capturing at 1:1.
      const originals = [original, ...original.querySelectorAll('*')];
      const copies = [clone, ...clone.querySelectorAll('*')];
      originals.forEach((element, index) => {
        if (!element.closest('foreignObject')) return;
        const computed = getComputedStyle(element);
        const target = copies[index] as HTMLElement | SVGElement;
        for (const property of [
          'font-family',
          'font-size',
          'font-weight',
          'font-style',
          'line-height',
          'letter-spacing',
          'white-space',
          'text-align',
          'color',
          'display',
          'align-items',
          'justify-content',
        ]) {
          target.style.setProperty(
            property,
            computed.getPropertyValue(property),
          );
        }
      });
      svg.appendChild(clone);
      // The capture lives beside the canvas. Give its SVG resources distinct
      // IDs so clip paths and fills cannot resolve into the original tree.
      const ids = new Map<string, string>();
      for (const element of svg.querySelectorAll('[id]')) {
        const id = element.getAttribute('id')!;
        if (id.startsWith('shape-'))
          element.setAttribute('data-zui-source-shape-id', id);
        ids.set(id, `${overlayId}-${id}`);
        element.setAttribute('id', ids.get(id)!);
      }
      for (const element of svg.querySelectorAll('*')) {
        for (const attribute of [...element.attributes]) {
          let value = attribute.value.replace(
            /url\(["']?#([^\s)"']+)["']?\)/g,
            (reference, id: string) =>
              ids.has(id) ? `url(#${ids.get(id)})` : reference,
          );
          if (value.startsWith('#') && ids.has(value.slice(1)))
            value = `#${ids.get(value.slice(1))}`;
          if (value !== attribute.value)
            element.setAttribute(attribute.name, value);
        }
      }
      overlay.appendChild(svg);
      document.body.appendChild(overlay);
    },
    { boardId, bounds, overlayId },
  );
  try {
    const overlay = page.locator(`#${overlayId}`);
    await overlay.waitFor({ state: 'visible' });
    const clip = await overlay.boundingBox();
    assert.ok(clip, 'Missing native capture bounds');
    // Playwright retains the context's initial DPR after a CDP metrics override.
    // Capture through Chromium so the current case DPR controls raster dimensions.
    const session =
      captureSession ?? (await page.context().newCDPSession(page));
    try {
      const textEvidence = await captureTextEvidence(page, session, overlayId);
      assert.ok(
        textEvidence.length >= expectedTextNodes,
        `Missing native text evidence: ${textEvidence.length}/${expectedTextNodes}`,
      );
      onTextEvidence?.(textEvidence);
      if (textEvidencePath) {
        await writeFile(
          textEvidencePath,
          `${JSON.stringify({ fontAudit, nativeSvgAudit, texts: textEvidence }, null, 2)}\n`,
        );
      }
      // Require the mounted native SVG to settle before recording evidence.
      let previous: Buffer | undefined;
      let previousPixels: ScreenshotPixels | undefined;
      let penultimate: Buffer | undefined;
      let bytes: Buffer | undefined;
      // CDP metric changes can trigger a late canvas/font repaint, especially
      // at 150% DPI. Give the mounted SVG a quiet period before comparing
      // frames, then keep the strict consecutive-frame equality check.
      await page.waitForTimeout(500);
      for (let attempt = 0; attempt < 20; attempt += 1) {
        await page.evaluate(
          () =>
            new Promise<void>((resolve) =>
              requestAnimationFrame(() =>
                requestAnimationFrame(() => resolve()),
              ),
            ),
        );
        const capture = await session.send('Page.captureScreenshot', {
          format: 'png',
          fromSurface: true,
          captureBeyondViewport: true,
          clip: { ...clip, scale: 1 },
        });
        const current = Buffer.from(capture.data, 'base64');
        const currentPixels = await readPng(current);
        if (
          previous &&
          previousPixels &&
          rasterFramesEquivalent(previousPixels, currentPixels)
        ) {
          bytes = current;
          break;
        }
        penultimate = previous;
        previous = current;
        previousPixels = currentPixels;
        await page.waitForTimeout(200);
      }
      if (!bytes && path) {
        if (penultimate)
          await writeFile(`${path}.unstable-previous.png`, penultimate);
        if (previous) await writeFile(`${path}.unstable-last.png`, previous);
      }
      assert.ok(bytes, 'Penpot raster did not settle to a stable frame');
      if (path) await writeFile(path, bytes);
      return bytes;
    } finally {
      if (!captureSession) await session.detach();
    }
  } finally {
    await page.locator(`#${overlayId}`).evaluate((element) => element.remove());
  }
}

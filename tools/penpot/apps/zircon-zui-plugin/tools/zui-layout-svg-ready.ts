import { createHash } from 'node:crypto';
import type { Page } from 'playwright';

export async function waitForNativeSvg(page: Page, boardId: string) {
  const settled = await page.evaluate(async (id) => {
    const deadline = performance.now() + 15_000;
    let previous = '';
    let unchangedSince = performance.now();
    while (performance.now() < deadline) {
      await new Promise<void>((resolve) =>
        requestAnimationFrame(() => resolve()),
      );
      const root = document.querySelector(`#shape-${id}`);
      if (!root) throw new Error('Native Penpot board disappeared');
      const rect = root.getBoundingClientRect();
      const current = `${rect.x},${rect.y},${rect.width},${rect.height}:${root.outerHTML}`;
      if (current !== previous) {
        previous = current;
        unchangedSince = performance.now();
      }
      const images = [...root.querySelectorAll('image')];
      const resourcesReady = await Promise.all(
        images.map(async (element) => {
          const href =
            element.getAttribute('href') ?? element.getAttribute('xlink:href');
          if (!href) return true;
          const image = new Image();
          image.src = href;
          try {
            await image.decode();
            return true;
          } catch {
            return false;
          }
        }),
      );
      if (
        performance.now() - unchangedSince >= 500 &&
        resourcesReady.every(Boolean)
      )
        return {
          svg: root.outerHTML,
          elementCount: root.querySelectorAll('*').length,
        };
    }
    throw new Error(
      'Original Penpot SVG did not settle or its image resources failed to decode',
    );
  }, boardId);
  return {
    nativeSvgSha256: createHash('sha256').update(settled.svg).digest('hex'),
    nativeElementCount: settled.elementCount,
  };
}

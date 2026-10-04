import type { CDPSession, Page } from 'playwright';

export async function captureTextEvidence(
  page: Page,
  session: CDPSession,
  rootId: string,
) {
  const rendered = await page.evaluate((rootId) => {
    const root = document.getElementById(rootId);
    if (!root) throw new Error('Missing native capture root');
    const origin = root.getBoundingClientRect();
    const texts = [...root.querySelectorAll('text, foreignObject span')].filter(
      (element) => {
        if (!element.textContent?.trim() || element.querySelector('span'))
          return false;
        let ancestor: Element | null = element;
        while (ancestor && root.contains(ancestor)) {
          const style = getComputedStyle(ancestor);
          if (
            style.display === 'none' ||
            style.visibility === 'hidden' ||
            style.visibility === 'collapse' ||
            Number(style.opacity) === 0
          )
            return false;
          ancestor = ancestor.parentElement;
        }
        return true;
      },
    );
    return texts.map((element, index) => {
      element.setAttribute('data-zui-text-evidence', String(index));
      const style = getComputedStyle(element);
      const range = document.createRange();
      range.selectNodeContents(element);
      const rects: DOMRect[] = [];
      for (const fragment of [...range.getClientRects()].sort(
        (a, b) => a.top - b.top || a.left - b.left,
      )) {
        const line = rects.find((candidate) => Math.abs(candidate.top - fragment.top) <= 1);
        if (line) {
          const left = Math.min(line.left, fragment.left);
          const top = Math.min(line.top, fragment.top);
          const right = Math.max(line.right, fragment.right);
          const bottom = Math.max(line.bottom, fragment.bottom);
          line.x = left;
          line.y = top;
          line.width = right - left;
          line.height = bottom - top;
        } else {
          rects.push(new DOMRect(fragment.x, fragment.y, fragment.width, fragment.height));
        }
      }
      const lines = rects.map((rect) => ({
        x: rect.x - origin.x,
        y: rect.y - origin.y,
        width: rect.width,
        height: rect.height,
      }));
      const lineTexts = lines.map(() => '');
      const lineTops = rects.map((rect) => rect.top);
      const segmenter = new Intl.Segmenter(undefined, {
        granularity: 'grapheme',
      });
      let pendingWhitespace = '';
      const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT);
      let textNode = walker.nextNode();
      while (textNode) {
        const content = textNode.textContent ?? '';
        for (const segment of segmenter.segment(content)) {
          if (/^[\r\n]+$/.test(segment.segment)) continue;
          range.setStart(textNode, segment.index);
          range.setEnd(textNode, segment.index + segment.segment.length);
          const glyphRects = [...range.getClientRects()];
          if (!glyphRects.length) {
            if (/^\s+$/.test(segment.segment)) {
              pendingWhitespace += segment.segment;
              continue;
            }
            throw new Error('Penpot text glyph has no measured line frame');
          }
          const glyph = glyphRects[0];
          let lineIndex = 0;
          for (let candidate = 1; candidate < lineTops.length; candidate++)
            if (
              Math.abs(glyph.top - lineTops[candidate]) <
              Math.abs(glyph.top - lineTops[lineIndex])
            )
              lineIndex = candidate;
          if (
            !lineTops.length ||
            Math.abs(glyph.top - lineTops[lineIndex]) > 1
          )
            throw new Error('Penpot glyph could not be assigned to a line');
          lineTexts[lineIndex] += pendingWhitespace + segment.segment;
          pendingWhitespace = '';
        }
        textNode = walker.nextNode();
      }
      if (pendingWhitespace && lineTexts.length)
        lineTexts[lineTexts.length - 1] += pendingWhitespace;
      const shapeIds: string[] = [];
      let ancestor: Element | null = element;
      while (ancestor && root.contains(ancestor)) {
        const id =
          ancestor.getAttribute('data-zui-source-shape-id') ??
          (ancestor.id.startsWith('shape-') ? ancestor.id : null);
        if (id && !shapeIds.includes(id)) shapeIds.push(id);
        ancestor = ancestor.parentElement;
      }
      return {
        shapeId: shapeIds[0] ?? '',
        ancestorShapeIds: shapeIds.slice(1),
        text: element.textContent ?? '',
        fontFamily: style.fontFamily,
        fontSize: style.fontSize,
        fontWeight: style.fontWeight,
        lineHeight: style.lineHeight,
        letterSpacing: style.letterSpacing,
        lineTexts,
        fontSources: collectFontSources(style),
        lines,
      };
    });

    function collectFontSources(style: CSSStyleDeclaration) {
      const familyStack = style.fontFamily
        .split(/,(?=(?:[^\"]*\"[^\"]*\")*[^\"]*$)/)
        .map((family) => family.trim().replace(/^(['\"])(.*)\1$/, '$2').toLowerCase());
      const requestedWeight = Number(style.fontWeight) || 400;
      const requestedStyle = style.fontStyle.toLowerCase();
      const sources: Array<{
        familyName: string;
        weight: string;
        style: string;
        urls: string[];
      }> = [];
      const visitRules = (
        rules: CSSRuleList,
        baseUrl: string,
      ): void => {
        for (const rule of [...rules]) {
          if (rule.type === CSSRule.FONT_FACE_RULE) {
            const face = (rule as CSSFontFaceRule).style;
            const familyName = face
              .getPropertyValue('font-family')
              .trim()
              .replace(/^(['\"])(.*)\1$/, '$2');
            if (!familyStack.includes(familyName.toLowerCase())) continue;
            const weight = face.getPropertyValue('font-weight').trim() || '400';
            const weights = weight.split(/\s+/).map((item) =>
              item === 'normal' ? 400 : item === 'bold' ? 700 : Number(item),
            );
            const lower = weights[0] ?? 400;
            const upper = weights[1] ?? lower;
            if (
              !weights.every(Number.isFinite) ||
              requestedWeight < lower ||
              requestedWeight > upper
            )
              continue;
            const fontStyle =
              face.getPropertyValue('font-style').trim().toLowerCase() ||
              'normal';
            if (
              fontStyle !== requestedStyle &&
              fontStyle !== 'oblique'
            )
              continue;
            const urls = [
              ...face
                .getPropertyValue('src')
                .matchAll(/url\(\s*(['\"]?)(.*?)\1\s*\)/gi),
            ].flatMap((match) => {
              try {
                return [new URL(match[2], baseUrl).href];
              } catch {
                return [];
              }
            });
            if (urls.length)
              sources.push({ familyName, weight, style: fontStyle, urls });
            continue;
          }
          const imported = (rule as CSSImportRule).styleSheet;
          if (imported) {
            try {
              visitRules(imported.cssRules, imported.href ?? baseUrl);
            } catch {
              // Cross-origin font declarations cannot prove local asset bytes.
            }
          }
        }
      };
      for (const sheet of [...document.styleSheets]) {
        try {
          visitRules(sheet.cssRules, sheet.href ?? document.baseURI);
        } catch {
          // A face without inspectable source provenance remains unproven.
        }
      }
      return sources;
    }
  }, rootId);
  await session.send('DOM.enable');
  await session.send('CSS.enable');
  const { root } = await session.send('DOM.getDocument');
  const result = [];
  for (const [index, text] of rendered.entries()) {
    const { nodeId } = await session.send('DOM.querySelector', {
      nodeId: root.nodeId,
      selector: `#${rootId} [data-zui-text-evidence="${index}"]`,
    });
    if (!nodeId) throw new Error(`Missing rendered text node: ${text.shapeId}`);
    const { fonts } = await session.send('CSS.getPlatformFontsForNode', {
      nodeId,
    });
    result.push({ ...text, fonts });
  }
  return result;
}

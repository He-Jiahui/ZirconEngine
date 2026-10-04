import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import type { FrameLocator, Page } from 'playwright';
import {
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
} from '../src/bridge/zui-document';

export async function verifySliderTextEdits(
  page: Page,
  plugin: FrameLocator,
): Promise<{ contract: string; textEdits: number }> {
  const document: ZuiDocument = {
    asset: {
      id: 'res://slider_contract.zui',
      kind: 'component',
      version: 2,
      display_name: 'Slider Text Contract',
    },
    components: { SliderSample: { root: 'slider_sample' } },
    tokens: { 'slider.label': '#b3b3b3' },
    nodes: {
      slider_sample: {
        component: 'RangeSlider',
        props: {
          label_text: 'Range',
          value: 80,
          min: 0,
          max: 100,
          range_min_percent: 0.2,
          value_text: '0.80',
          label_color: '$slider.label',
          font_size: 14,
        },
        layout: { width: { preferred: 360 }, height: { preferred: 32 } },
        events: [
          { id: 'Slider/Change', event: 'Change', route: 'test.update_range' },
        ],
        runtime_extension: { binding: 'model.range', retain: true },
      },
    },
  };
  await plugin.locator('input[type="file"]').setInputFiles({
    name: 'slider_contract.zui',
    mimeType: 'text/plain',
    buffer: Buffer.from(serializeZuiDocument(document)),
  });
  await plugin
    .getByText('Slider Text Contract \u00b7 1 nodes', { exact: true })
    .waitFor({ timeout: 30000 });
  const exportDocument = async () => {
    const downloadPromise = page.waitForEvent('download', { timeout: 30000 });
    await plugin.getByRole('button', { name: 'Export selected' }).click();
    const file = await (await downloadPromise).path();
    assert.ok(file);
    return parseZuiDocument(await readFile(file, 'utf8')).document;
  };
  assert.deepEqual(await exportDocument(), document);
  const rows = page.getByTestId('layer-row');
  const asset = rows.filter({ hasText: 'ZUI \u00b7 Slider Text Contract' });
  const toggle = asset.getByTestId('toggle-content');
  if ((await toggle.getAttribute('aria-expanded')) !== 'true')
    await toggle.click({ modifiers: ['Alt'] });
  const instanceToggle = rows
    .filter({ hasText: 'slider_sample \u00b7 RangeSlider' })
    .getByTestId('toggle-content');
  if ((await instanceToggle.getAttribute('aria-expanded')) !== 'true')
    await instanceToggle.click({ modifiers: ['Alt'] });
  for (const [name, content] of [
    ['ZUI text label \u00b7 slider_sample', 'Level'],
    ['ZUI text \u00b7 slider_sample', '80%'],
  ]) {
    await rows.filter({ hasText: name }).click();
    await page.getByTestId('viewport').press('Enter');
    const editor = page.locator('[contenteditable="true"]').last();
    await editor.waitFor({ timeout: 10000 });
    await page.keyboard.press('Control+A');
    await page.keyboard.type(content);
    await page.keyboard.press('Escape');
    await editor.waitFor({ state: 'hidden', timeout: 10000 });
  }
  const expected = structuredClone(document);
  expected.nodes!['slider_sample'].props!['label_text'] = 'Level';
  expected.nodes!['slider_sample'].props!['value_text'] = '80%';
  assert.deepEqual(await exportDocument(), expected);
  return { contract: 'penpot-slider-text-fragments', textEdits: 2 };
}

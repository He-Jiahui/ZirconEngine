import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import type { FrameLocator, Page } from 'playwright';
import {
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
} from '../src/bridge/zui-document';

export async function verifySegmentedTextEdits(
  page: Page,
  plugin: FrameLocator,
): Promise<{ contract: string; textEdits: number }> {
  const document: ZuiDocument = {
    asset: {
      id: 'res://segmented_contract.zui',
      kind: 'component',
      version: 2,
      display_name: 'Segmented Text Contract',
    },
    components: { SegmentedSample: { root: 'segmented_sample' } },
    tokens: { 'selected.surface': '#243f5a' },
    nodes: {
      segmented_sample: {
        component: 'SegmentedControl',
        props: {
          group_label: 'Mode',
          value: 'center',
          options: [
            'left',
            { label: 'center', value: 'alignment.center', extra: true },
            'right',
          ],
          selected_background_color: '$selected.surface',
          font_size: 14,
        },
        layout: { width: { preferred: 360 }, height: { preferred: 56 } },
        events: [
          { id: 'Mode/Change', event: 'Change', route: 'test.select_mode' },
        ],
        runtime_extension: { binding: 'model.alignment', retain: true },
      },
    },
  };
  await plugin.locator('input[type="file"]').setInputFiles({
    name: 'segmented_contract.zui',
    mimeType: 'text/plain',
    buffer: Buffer.from(serializeZuiDocument(document)),
  });
  await plugin
    .getByText('Segmented Text Contract \u00b7 1 nodes', { exact: true })
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
  const asset = rows.filter({ hasText: 'ZUI \u00b7 Segmented Text Contract' });
  const toggle = asset.getByTestId('toggle-content');
  if ((await toggle.getAttribute('aria-expanded')) !== 'true')
    await toggle.click({ modifiers: ['Alt'] });
  const instanceToggle = rows
    .filter({ hasText: 'segmented_sample \u00b7 SegmentedControl' })
    .getByTestId('toggle-content');
  if ((await instanceToggle.getAttribute('aria-expanded')) !== 'true')
    await instanceToggle.click({ modifiers: ['Alt'] });
  for (const [name, content] of [
    ['ZUI text \u00b7 segmented_sample', 'Alignment'],
    ['ZUI text option-1 \u00b7 segmented_sample', 'Middle'],
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
  expected.nodes!['segmented_sample'].props!['group_label'] = 'Alignment';
  expected.nodes!['segmented_sample'].props!['options'] = [
    'left',
    { label: 'Middle', value: 'alignment.center', extra: true },
    'right',
  ];
  expected.nodes!['segmented_sample'].props!['value'] = 'Middle';
  assert.deepEqual(await exportDocument(), expected);
  return { contract: 'penpot-segmented-text-fragments', textEdits: 2 };
}

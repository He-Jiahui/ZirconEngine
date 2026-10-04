import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import type { FrameLocator, Page } from 'playwright';
import { requestPreviewLayoutAudit } from './penpot-preview-layout-audit';
import {
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
  type ZuiTable,
  type ZuiValue,
} from '../src/bridge/zui-document';

function isZuiTable(value: ZuiValue | undefined): value is ZuiTable {
  return (
    value !== undefined &&
    typeof value === 'object' &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}

export async function verifySlotPaddingEdits(page: Page, plugin: FrameLocator) {
  const document: ZuiDocument = {
    asset: {
      id: 'res://slot_padding_contract.zui',
      kind: 'component',
      version: 2,
      display_name: 'Slot Padding Contract',
    },
    components: { SlotPaddingSample: { root: 'padding_host' } },
    tokens: { 'slot.left': 8 },
    nodes: {
      padding_host: {
        component: 'VerticalBox',
        layout: {
          container: { kind: 'VerticalBox', gap: 0 },
          padding: { top: 0, right: 0, bottom: 0, left: 0 },
          width: { preferred: 360 },
          height: { preferred: 160 },
        },
        children: [
          {
            node: 'padding_label',
            slot: {
              layout: {
                padding: {
                  top: 4,
                  right: 8,
                  bottom: 4,
                  left: '$slot.left',
                  runtime_hint: 'preserve',
                },
              },
            },
          },
        ],
      },
      padding_label: {
        component: 'Label',
        props: { text: 'Property', font_size: 14 },
        layout: { width: { stretch: 'Stretch' }, height: { preferred: 28 } },
        runtime_extension: { binding: 'model.property', retain: true },
      },
    },
  };
  const originalSource = `# Runtime source export contract\n${serializeZuiDocument(document)}`;
  document['penpot_original_source'] = originalSource;
  await plugin.locator('input[type="file"]').setInputFiles({
    name: 'slot_padding_contract.zui',
    mimeType: 'text/plain',
    buffer: Buffer.from(serializeZuiDocument(document)),
  });
  await plugin
    .getByText('Slot Padding Contract \u00b7 2 nodes', { exact: true })
    .waitFor({ timeout: 30000 });
  const exportSource = async () => {
    const download = page.waitForEvent('download', { timeout: 30000 });
    await plugin.getByRole('button', { name: 'Export selected' }).click();
    const file = await (await download).path();
    assert.ok(file);
    return readFile(file, 'utf8');
  };
  const { layoutAudit } = await requestPreviewLayoutAudit(plugin);
  const geometry = layoutAudit.semanticNodes ?? [];
  const host = geometry.find((node) => node.nodeId === 'padding_host');
  const label = geometry.find((node) => node.nodeId === 'padding_label');
  assert.ok(host && label, 'Missing slot host geometry');
  assert.equal(
    label.bounds.x - host.bounds.x,
    8,
    'Slot left padding must move the child',
  );
  assert.equal(
    label.bounds.y - host.bounds.y,
    4,
    'Slot top padding must move the child',
  );
  assert.equal(
    host.bounds.width - label.bounds.width,
    16,
    'Slot padding must reserve both sides',
  );
  assert.equal(await exportSource(), originalSource);
  const rows = page.getByTestId('layer-row');
  const asset = rows.filter({ hasText: 'ZUI \u00b7 Slot Padding Contract' });
  const toggle = asset.getByTestId('toggle-content');
  if ((await toggle.getAttribute('aria-expanded')) !== 'true')
    await toggle.click({ modifiers: ['Alt'] });
  await rows.filter({ hasText: 'padding_label \u00b7 Label' }).click();
  const marginToggle = page.getByTitle('Margin - multiple', { exact: true });
  await marginToggle.waitFor({ timeout: 10000 });
  const margins = marginToggle.locator('..').locator('input');
  if ((await margins.count()) === 2) await marginToggle.click();
  await margins.nth(3).waitFor({ timeout: 10000 });
  assert.equal(await margins.count(), 4);
  const rightMargin = margins.nth(1);
  await rightMargin.waitFor({ timeout: 10000 });
  assert.equal(await rightMargin.inputValue(), '8');
  assert.equal(await margins.nth(3).inputValue(), '8');
  await rightMargin.fill('24');
  await rightMargin.press('Enter');
  const expected = structuredClone(document);
  delete expected['penpot_original_source'];
  const layout =
    expected.nodes?.['padding_host']?.children?.[0]?.slot?.['layout'];
  assert.ok(isZuiTable(layout), 'Missing source-owned slot layout');
  const padding = layout['padding'];
  assert.ok(isZuiTable(padding), 'Missing source-owned slot padding');
  padding['right'] = 24;
  assert.deepEqual(parseZuiDocument(await exportSource()).document, expected);
  return {
    contract: 'penpot-slot-padding',
    noEditRoundtrip: true,
    editedSlotSides: 1,
    preservedTokenAndUnknownFields: true,
  };
}

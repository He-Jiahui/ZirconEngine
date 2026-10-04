import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { FrameLocator, Page } from 'playwright';
import { cloneZuiDocument, parseZuiDocument, serializeZuiDocument } from '../src/bridge/zui-document';
import { LayoutDependencies } from './zui-layout-dependencies';

export async function verifyFieldTextEdits(page: Page, plugin: FrameLocator) {
  const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../', import.meta.url)));
  const sourcePath = 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_search_input.zui';
  const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const original = parseZuiDocument(await readFile(resolve(repo, sourcePath), 'utf8')).document;
  original.asset.display_name = 'Field Browser Contract';
  original.nodes!['root'].props!['query'] = 'Player_Controller_MainCamera_EditorPreview_Selection_0123456789';
  const source = serializeZuiDocument(original);
  const projection = cloneZuiDocument(original);
  const dependencies = new LayoutDependencies();
  await dependencies.load(repo, [sourcePath, theme]);
  dependencies.embed(projection, sourcePath, theme);
  projection['penpot_original_source'] = source;
  await plugin.locator('input[type="file"]').setInputFiles({ name: 'field-contract.zui',
    mimeType: 'text/plain', buffer: Buffer.from(serializeZuiDocument(projection)) });
  await plugin.getByText('Field Browser Contract \u00b7 1 nodes', { exact: true }).waitFor({ timeout: 30000 });
  const exportSource = async () => {
    const pending = page.waitForEvent('download', { timeout: 30000 });
    await plugin.getByRole('button', { name: 'Export selected' }).click();
    const file = await (await pending).path();
    assert.ok(file);
    return readFile(file, 'utf8');
  };
  assert.equal(await exportSource(), source);
  const rows = page.getByTestId('layer-row');
  const asset = rows.filter({ hasText: 'ZUI \u00b7 Field Browser Contract' });
  const toggle = asset.getByTestId('toggle-content');
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') await toggle.click({ modifiers: ['Alt'] });
  const rootToggle = rows.filter({ hasText: 'root \u00b7 SearchField' }).getByTestId('toggle-content');
  if ((await rootToggle.getAttribute('aria-expanded')) !== 'true') await rootToggle.click({ modifiers: ['Alt'] });
  const edit = async (value: string) => {
    await rows.filter({ hasText: 'ZUI text \u00b7 root' }).click();
    await page.getByTestId('viewport').press('Enter');
    const text = page.locator('[contenteditable="true"]').last();
    await text.waitFor({ timeout: 10000 });
    await page.keyboard.press('Control+A');
    await page.keyboard.insertText(value);
    await page.keyboard.press('Escape');
    await text.waitFor({ state: 'hidden', timeout: 10000 });
  };
  await edit('Edited\u2026');
  await plugin.getByRole('button', { name: 'Export selected' }).click();
  await plugin.getByText(/Cannot edit an abbreviated field value/).waitFor({ timeout: 10000 });
  await edit('CameraController');
  const expected = cloneZuiDocument(original);
  expected.nodes!['root'].props!['query'] = 'CameraController';
  assert.deepEqual(parseZuiDocument(await exportSource()).document, expected);
  return { contract: 'penpot-field-full-text-roundtrip', preservedAbbreviatedSource: true,
    rejectedAmbiguousEllipsisEdit: true, completeValueEdit: true };
}

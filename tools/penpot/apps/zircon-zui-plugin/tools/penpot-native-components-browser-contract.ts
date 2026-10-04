import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { FrameLocator, Page } from 'playwright';
import {
  cloneZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
} from '../src/bridge/zui-document';
import { LayoutDependencies } from './zui-layout-dependencies';
import { requestPreviewLayoutAudit } from './penpot-preview-layout-audit';

export async function verifyNativeComponentEdits(
  page: Page,
  plugin: FrameLocator,
) {
  const repo = resolve(
    dirname(fileURLToPath(import.meta.url)),
    '../../../../..',
  );
  const buttonPath =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_button.zui';
  const button = parseZuiDocument(
    await readFile(resolve(repo, buttonPath), 'utf8'),
  ).document;
  const component = Object.keys(button.components!)[0];
  const original: ZuiDocument = {
    asset: {
      id: 'res://ui/native_component_contract.zui',
      kind: 'view',
      version: 2,
      display_name: 'Native Component Contract',
    },
    imports: { widgets: [button.asset.id] },
    root: { node: 'host' },
    nodes: {
      host: {
        component: 'HorizontalBox',
        layout: {
          container: { kind: 'HorizontalBox', gap: 12 },
          width: { preferred: 480 },
          height: { preferred: 64 },
        },
        children: [
          { node: 'button_a' },
          { node: 'button_b' },
          { node: 'button_c' },
        ],
      },
      button_a: {
        component,
        props: { text: 'Apply' },
        layout: {
          width: { preferred: 120, min: 100 },
          height: { preferred: 32 },
        },
      },
      button_b: {
        component,
        props: { text: 'Reset' },
        layout: { width: { preferred: 120 }, height: { preferred: 32 } },
        runtime_extension: { binding: 'model.reset' },
      },
      button_c: {
        component,
        props: { text: 'Save' },
        layout: {
          width: { preferred: 120, min: 100 },
          height: { preferred: 32 },
        },
      },
    },
  };
  const source = serializeZuiDocument(original);
  const projection = cloneZuiDocument(original);
  const catalog = JSON.parse(
    await readFile(resolve(repo, 'docs/_data/layout/catalog.json'), 'utf8'),
  ) as { entries: { sourcePath: string }[] };
  const dependencies = new LayoutDependencies();
  await dependencies.load(
    repo,
    catalog.entries.map((entry) => entry.sourcePath),
  );
  dependencies.embed(
    projection,
    'zircon_editor/assets/ui/native_component_contract.zui',
    'zircon_editor/assets/ui/editor/theme/editor_tokens.zui',
  );
  projection['penpot_original_source'] = source;
  await plugin.locator('input[type="file"]').setInputFiles({
    name: 'native_component_contract.zui',
    mimeType: 'text/plain',
    buffer: Buffer.from(serializeZuiDocument(projection)),
  });
  await plugin
    .getByText('Native Component Contract \u00b7 4 nodes', { exact: true })
    .waitFor({ timeout: 30000 });
  const { layoutAudit } = await requestPreviewLayoutAudit(plugin);
  const nodes = layoutAudit.semanticNodes ?? [];
  const buttons = nodes.filter((node) => node.nodeId.startsWith('button_'));
  assert.equal(buttons.length, 3);
  assert.ok(buttons.every((node) => node.nativeComponent?.copy));
  const byId = new Map(buttons.map((node) => [node.nodeId, node]));
  assert.notEqual(
    byId.get('button_a')!.nativeComponent!.id,
    byId.get('button_b')!.nativeComponent!.id,
  );
  assert.equal(
    byId.get('button_a')!.nativeComponent!.id,
    byId.get('button_c')!.nativeComponent!.id,
  );
  assert.equal(
    buttons[0].nativeComponent!.source,
    `${buttonPath}#${component}`,
  );
  const exportSource = async () => {
    const pending = page.waitForEvent('download', { timeout: 30000 });
    await plugin.getByRole('button', { name: 'Export selected' }).click();
    const file = await (await pending).path();
    assert.ok(file);
    return readFile(file, 'utf8');
  };
  assert.equal(await exportSource(), source);
  const rows = page.getByTestId('layer-row');
  const asset = rows.filter({
    hasText: 'ZUI \u00b7 Native Component Contract',
  });
  const toggle = asset.getByTestId('toggle-content');
  if ((await toggle.getAttribute('aria-expanded')) !== 'true')
    await toggle.click({ modifiers: ['Alt'] });
  const instanceToggle = rows
    .filter({ hasText: 'button_b \u00b7 Button' })
    .getByTestId('toggle-content');
  if ((await instanceToggle.getAttribute('aria-expanded')) !== 'true')
    await instanceToggle.click({ modifiers: ['Alt'] });
  await rows.filter({ hasText: 'ZUI text \u00b7 button_b' }).click();
  await page.getByTestId('viewport').press('Enter');
  const text = page.locator('[contenteditable="true"]').last();
  await text.waitFor({ timeout: 10000 });
  await page.keyboard.press('Control+A');
  await page.keyboard.type('Revert');
  await page.keyboard.press('Escape');
  await text.waitFor({ state: 'hidden', timeout: 10000 });
  const expected = cloneZuiDocument(original);
  expected.nodes!['button_b'].props!['text'] = 'Revert';
  assert.deepEqual(parseZuiDocument(await exportSource()).document, expected);
  return {
    contract: 'penpot-native-prefab-instances',
    nativeComponents: 2,
    nativeInstances: 3,
    distinctSizeConstraints: true,
    editedInstanceText: true,
    preservedSourceReferences: true,
  };
}

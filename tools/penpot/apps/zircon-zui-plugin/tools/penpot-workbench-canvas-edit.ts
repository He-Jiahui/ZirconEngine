import assert from 'node:assert/strict';
import type { Locator } from 'playwright';
import type { RenderedLayoutAudit } from '../src/penpot-render-layout';
import type { openPenpotSession } from './zui-layout-penpot-session';
import { requestPreviewLayoutAudit } from './penpot-preview-layout-audit';

type Runtime = Awaited<ReturnType<typeof openPenpotSession>>;
type MountedNode = NonNullable<RenderedLayoutAudit['semanticNodes']>[number];
const sourcePath =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';

async function mountedNodes(runtime: Runtime): Promise<MountedNode[]> {
  const { layoutAudit } = await requestPreviewLayoutAudit(runtime.pluginFrame);
  assert.equal(
    layoutAudit.refreshError,
    undefined,
    'Supported authoring reflow failed',
  );
  assert.deepEqual(
    layoutAudit.invalidGeometryNodeIds,
    [],
    'Reflow produced invalid geometry',
  );
  assert.deepEqual(
    layoutAudit.overflowNodeIds,
    [],
    'Reflow left visible overflow',
  );
  return layoutAudit.semanticNodes ?? [];
}

function owner(nodes: MountedNode[]): MountedNode {
  const candidates = nodes.filter(
    (node) =>
      node.sourcePath === sourcePath && node.sourceNodeId === 'window_content',
  );
  assert.equal(
    candidates.length,
    1,
    'Expected exactly one mounted source-owned window_content.',
  );
  return candidates[0]!;
}

function rowFor(runtime: Runtime, shapeId: string): Locator {
  assert.match(
    shapeId,
    /^[a-f\d-]{36}$/i,
    'Mounted Penpot shape ID must be a UUID.',
  );
  return runtime.page.locator(`[data-testid="layer-row"][id="${shapeId}"]`);
}

async function revealLayerRow(
  runtime: Runtime,
  shapeId: string,
): Promise<Locator> {
  const row = rowFor(runtime, shapeId);
  const scroll = runtime.page
    .getByTestId('layer-tree')
    .locator('[data-scroll-container="true"]');
  await scroll.hover();
  // Layers are lazily mounted in batches; invisible intrinsic probes can precede the asset root.
  await runtime.page.mouse.wheel(0, -100_000);
  await runtime.page.waitForTimeout(100);
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    if (await row.isVisible()) {
      await row.scrollIntoViewIfNeeded();
      return row;
    }
    await scroll.hover();
    await runtime.page.mouse.wheel(0, 900);
    await runtime.page.waitForTimeout(100);
  }
  throw new Error('Penpot layer did not mount after scrolling: ' + shapeId);
}

async function expand(runtime: Runtime, shapeId: string): Promise<void> {
  const row = await revealLayerRow(runtime, shapeId);
  const toggle = row.getByTestId('toggle-content');
  if ((await toggle.getAttribute('aria-expanded')) !== 'true')
    await toggle.click();
}

async function movePluginAside(runtime: Runtime): Promise<void> {
  const header = runtime.page.locator('plugin-modal .header');
  const bounds = await header.boundingBox();
  assert.ok(bounds, 'The actual Penpot plugin header must be mounted.');
  await runtime.page.mouse.move(
    bounds.x + bounds.width / 2,
    bounds.y + bounds.height / 2,
  );
  await runtime.page.mouse.down();
  try {
    await runtime.page.mouse.move(600, 100, { steps: 8 });
  } finally {
    await runtime.page.mouse.up();
  }
}

function gapInput(runtime: Runtime, axis: 'row' | 'column'): Locator {
  const label = axis === 'row' ? 'Row gap' : 'Column gap';
  return runtime.page
    .getByTestId('inspect-layout')
    .locator(
      `[aria-label="${label}"] input, [title="${label}"] input, input[aria-label="${label}"], input[data-type="${axis}-gap"]`,
    );
}

async function readGap(
  runtime: Runtime,
): Promise<{ rowGap: number; columnGap: number }> {
  const rowGap = Number(await gapInput(runtime, 'row').inputValue());
  const columnGap = Number(await gapInput(runtime, 'column').inputValue());
  assert.ok(
    Number.isFinite(rowGap) && Number.isFinite(columnGap),
    'Actual inspector gap must be numeric.',
  );
  return { rowGap, columnGap };
}

/** Edit the normal Penpot inspector; the plugin API remains inside its SES sandbox. */
export async function editWorkbenchSpacingInPenpot(runtime: Runtime) {
  const nodes = await mountedNodes(runtime);
  const target = owner(nodes);
  assert.ok(target.visible, 'WorkbenchWindow edit target must be visible.');
  const instancePath = JSON.parse(target.instancePath ?? '') as Array<{
    sourcePath: string;
    sourceNodeId: string;
  }>;
  assert.ok(
    Array.isArray(instancePath),
    'Mounted owner must retain its instance ancestry.',
  );
  assert.equal(
    JSON.stringify(instancePath),
    target.instancePath,
    'Instance ancestry must be compact JSON.',
  );
  for (const step of instancePath) {
    assert.ok(
      typeof step?.sourcePath === 'string' &&
        typeof step?.sourceNodeId === 'string',
    );
  }
  const byId = new Map(nodes.map((node) => [node.nodeId, node]));
  const ancestors: MountedNode[] = [];
  const visited = new Set<string>([target.nodeId]);
  let parentNodeId = target.parentNodeId;
  while (parentNodeId) {
    assert.ok(!visited.has(parentNodeId), 'Mounted hierarchy must be acyclic.');
    visited.add(parentNodeId);
    const node = byId.get(parentNodeId);
    assert.ok(node, `Mounted parent is missing: ${parentNodeId}`);
    ancestors.unshift(node);
    parentNodeId = node.parentNodeId;
  }
  const assetId = await runtime.statusPanel.getAttribute(
    'data-preview-board-id',
  );
  assert.ok(assetId, 'Imported asset must expose its actual board ID.');
  await movePluginAside(runtime);
  await runtime.page
    .getByRole('button', { name: 'Move (V)', exact: true })
    .click();
  await expand(runtime, assetId);
  for (const node of ancestors) await expand(runtime, node.shapeId);
  const targetRow = await revealLayerRow(runtime, target.shapeId);
  await targetRow.click();
  assert.equal(await targetRow.getAttribute('aria-checked'), 'true');
  if ((await gapInput(runtime, 'row').count()) === 0) {
    await runtime.page
      .getByTestId('inspect-layout')
      .getByRole('button', { name: 'Layout', exact: true })
      .click();
  }
  await gapInput(runtime, 'row').waitFor({ timeout: 30_000 });
  const before = await readGap(runtime);
  assert.equal(
    before.rowGap,
    before.columnGap,
    'The imported scalar gap must materialize equally on both axes.',
  );
  const editable = (await gapInput(runtime, 'row').isEnabled())
    ? 'row'
    : 'column';
  const inactive = editable === 'row' ? 'column' : 'row';
  assert.ok(
    !(await gapInput(runtime, inactive).isEnabled()),
    'The workbench regression target must use a single layout axis.',
  );
  const activeKey = editable === 'row' ? 'rowGap' : 'columnGap';
  const inactiveKey = editable === 'row' ? 'columnGap' : 'rowGap';
  const changedGap = before[activeKey] + 1;
  await gapInput(runtime, editable).fill(String(changedGap));
  await gapInput(runtime, editable).press('Enter');
  const deadline = Date.now() + 30_000;
  let after = before;
  let currentNodes = nodes;
  const children = (items: MountedNode[]) =>
    items
      .filter(
        (node) =>
          node.parentNodeId === target.nodeId && node.visible && !node.detached,
      )
      .map(({ nodeId, bounds }) => ({ nodeId, bounds }));
  const beforeChildren = children(nodes);
  assert.ok(
    beforeChildren.length > 1,
    'Spacing edit must have multiple visible child regions.',
  );
  while (Date.now() < deadline) {
    // The public audit serializes with shapechange reflow and returns only after
    // native layout settles. Read the normal inspector after that barrier.
    currentNodes = await mountedNodes(runtime);
    after = await readGap(runtime);
    if (
      after[activeKey] === changedGap &&
      after[inactiveKey] === before[inactiveKey] &&
      JSON.stringify(children(currentNodes)) !== JSON.stringify(beforeChildren)
    )
      break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.equal(
    after[activeKey],
    changedGap,
    'Penpot inspector did not apply the active gap.',
  );
  assert.equal(
    after[inactiveKey],
    before[inactiveKey],
    'Penpot inspector changed the disabled gap.',
  );
  assert.equal(
    owner(currentNodes).shapeId,
    target.shapeId,
    'The edited source owner changed identity.',
  );
  assert.notDeepEqual(
    children(currentNodes),
    beforeChildren,
    'The actual layout did not change.',
  );
  return {
    identity: {
      sourcePath,
      sourceNodeId: target.sourceNodeId!,
      controlId: target.controlId ?? null,
      instancePath,
    },
    shapeId: target.shapeId,
    method: 'Penpot layer selection and normal layout inspector',
    before: { ...before, children: beforeChildren },
    after: { ...after, children: children(currentNodes) },
  };
}

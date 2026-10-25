import assert from 'node:assert/strict';
import type { Locator } from 'playwright';
import type { RenderedLayoutAudit } from '../src/penpot-render-layout';
import type { openPenpotSession } from './zui-layout-penpot-session';
import { requestPreviewLayoutAudit } from './penpot-preview-layout-audit';
import type { SourceOwnerGapEditTarget } from './zui-layout-real-workbench-input';

type Runtime = Awaited<ReturnType<typeof openPenpotSession>>;
type MountedNode = NonNullable<RenderedLayoutAudit['semanticNodes']>[number];
const sourcePath =
  'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';

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
      node.sourcePath === sourcePath && node.sourceNodeId === 'editor_content',
  );
  assert.equal(
    candidates.length,
    1,
    'Expected exactly one mounted source-owned editor_content.',
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

/** Resolve the exact admitted raw source, never a similarly named business shape. */
export function sourceOwnerSpacingNode(
  nodes: readonly MountedNode[], expected: SourceOwnerGapEditTarget,
): MountedNode {
  assert.equal(expected.sourcePath, 'zircon_plugins/ai/editor/behavior_tree.zui');
  assert.equal(expected.sourceNodeId, 'behavior_tree_root');
  assert.equal(JSON.stringify(expected.instancePath), '[]');
  const candidates = nodes.filter(node => node.sourcePath === expected.sourcePath &&
    node.sourceNodeId === expected.sourceNodeId && node.controlId === expected.controlId &&
    node.instancePath === '[]' && node.visible && !node.detached);
  assert.equal(candidates.length, 1, 'Expected one visible, attached, exact source-owned spacing target');
  return candidates[0]!;
}

/** Keep the LevelEditor-only selector separate from explicitly admitted private source edits. */
export async function editWorkbenchSpacingInPenpot(runtime: Runtime, requestedGap?: number) {
  return editMountedSpacingInPenpot(runtime, owner, requestedGap);
}

export async function editSourceOwnerSpacingInPenpot(
  runtime: Runtime, expected: SourceOwnerGapEditTarget,
) {
  return editMountedSpacingInPenpot(runtime, nodes => sourceOwnerSpacingNode(nodes, expected),
    expected.afterGap, expected);
}

/** Edit the normal Penpot inspector; the plugin API remains inside its SES sandbox. */
async function editMountedSpacingInPenpot(
  runtime: Runtime, selectOwner: (nodes: MountedNode[]) => MountedNode,
  requestedGap?: number, sourceEdit?: SourceOwnerGapEditTarget,
) {
  const nodes = await mountedNodes(runtime);
  const target = selectOwner(nodes);
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
  if (sourceEdit) {
    // Actual top_toolbar.cljs exposes data-tool; account language stays unchanged.
    const move = runtime.page.locator('[data-tool="move"][aria-haspopup="true"]');
    assert.equal(await move.count(), 1, 'The actual Move tool must be uniquely mounted');
    await move.click();
  } else {
    await runtime.page.getByRole('button', { name: 'Move (V)', exact: true }).click();
  }
  await expand(runtime, assetId);
  for (const node of ancestors) await expand(runtime, node.shapeId);
  const targetRow = await revealLayerRow(runtime, target.shapeId);
  await targetRow.click();
  assert.equal(await targetRow.getAttribute('aria-checked'), 'true');
  if ((await gapInput(runtime, 'row').count()) === 0) {
    const panel = runtime.page.getByTestId('inspect-layout');
    if (sourceEdit) {
      // The source's section header has one text button; icon actions have no text.
      const title = panel.locator(':scope > div:first-child button').filter({ hasText: /\S/ });
      assert.equal(await title.count(), 1, 'The actual Layout section header must be unique');
      await title.click();
    } else {
      await panel.getByRole('button', { name: 'Layout', exact: true }).click();
    }
  }
  await gapInput(runtime, 'row').waitFor({ timeout: 30_000 });
  const before = await readGap(runtime);
  if (sourceEdit) {
    assert.equal(before.rowGap, sourceEdit.beforeGap, 'The Inspector scalar differs from the admitted original source');
    assert.equal(before.columnGap, sourceEdit.beforeGap);
  }
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
  const changedGap = requestedGap ?? before[activeKey] + 1;
  assert.ok(Number.isFinite(changedGap) && changedGap >= 0 && changedGap !== before[activeKey]);
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
    selectOwner(currentNodes).shapeId,
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
      sourcePath: target.sourcePath!,
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

/** Change the existing callsite constraint through the normal Penpot inspector. */
export async function editOutlinerMinimumHeightInPenpot(runtime: Runtime, requestedMinimum = 128) {
  assert.ok(Number.isFinite(requestedMinimum) && requestedMinimum >= 0);
  const nodes = await mountedNodes(runtime);
  const candidates = nodes.filter((node) =>
    node.sourcePath === 'zircon_editor/assets/ui/editor/unreal/components/outliner.zui' &&
    node.sourceNodeId === 'panel');
  assert.equal(candidates.length, 1, 'Expected one actual Outliner component root.');
  const target = candidates[0]!;
  assert.ok(target.visible && !target.detached, 'The actual Outliner must be mounted and visible.');
  const instancePath = JSON.parse(target.instancePath ?? '') as Array<{ sourcePath: string; sourceNodeId: string }>;
  assert.ok(Array.isArray(instancePath) && instancePath.length > 0);
  assert.equal(JSON.stringify(instancePath), target.instancePath, 'Retain the complete compact invocation identity.');
  for (const step of instancePath) {
    assert.deepEqual(Object.keys(step).sort(), ['sourceNodeId', 'sourcePath']);
    assert.ok(typeof step.sourcePath === 'string' && step.sourcePath.length > 0 &&
      typeof step.sourceNodeId === 'string' && step.sourceNodeId.length > 0);
  }
  assert.deepEqual(instancePath.at(-1), { sourcePath, sourceNodeId: 'outliner' },
    'The minimum height must belong to the actual main-window Outliner invocation.');
  const byId = new Map(nodes.map((node) => [node.nodeId, node]));
  assert.equal(byId.size, nodes.length, 'Mounted identities must be unique.');
  const ancestors: MountedNode[] = [];
  const visited = new Set([target.nodeId]);
  let parentNodeId = target.parentNodeId;
  while (parentNodeId) {
    assert.ok(!visited.has(parentNodeId), 'Mounted hierarchy must be acyclic.');
    visited.add(parentNodeId);
    const parent = byId.get(parentNodeId);
    assert.ok(parent, 'The actual Outliner parent must be present.');
    ancestors.unshift(parent);
    parentNodeId = parent.parentNodeId;
  }
  const boardId = await runtime.statusPanel.getAttribute('data-preview-board-id');
  assert.ok(boardId, 'The actual imported asset board must be selected.');
  await movePluginAside(runtime);
  await runtime.page.getByRole('button', { name: 'Move (V)', exact: true }).click();
  await expand(runtime, boardId);
  for (const ancestor of ancestors) await expand(runtime, ancestor.shapeId);
  const targetRow = await revealLayerRow(runtime, target.shapeId);
  await targetRow.click();
  assert.equal(await targetRow.getAttribute('aria-checked'), 'true');
  const fixed = runtime.page.getByTestId('behaviour-v-fix');
  const fill = runtime.page.getByTestId('behaviour-v-fill');
  await fixed.waitFor({ timeout: 30_000 });
  assert.ok(await fixed.locator('input').isChecked(), 'Weighted Outliner must start at its existing Fixed sizing.');
  let beforeMinimum: number | undefined;
  let afterMinimum: number | undefined;
  try {
    await fill.click();
    assert.ok(await fill.locator('input').isChecked(), 'The official Inspector must enter Fill sizing.');
    const minimumLabel = runtime.page.locator('section[aria-label="Layout item size constraints"]')
      .getByText('MIN H', { exact: true });
    await minimumLabel.waitFor({ timeout: 30_000 });
    assert.equal(await minimumLabel.count(), 1, 'Use the unique official minimum-height field label.');
    // The token input exposes its property through a tooltip; both official
    // field branches put the MIN H label and their sole input in one container.
    const minimum = minimumLabel.locator('..').locator('input');
    await minimum.waitFor({ timeout: 30_000 });
    assert.equal(await minimum.count(), 1, 'Use the official minimum-height inspector input.');
    beforeMinimum = Number(await minimum.inputValue());
    assert.ok(Number.isFinite(beforeMinimum) && beforeMinimum >= 0 && beforeMinimum !== requestedMinimum);
    await minimum.fill(String(requestedMinimum));
    await minimum.press('Enter');
    afterMinimum = Number(await minimum.inputValue());
    assert.equal(afterMinimum, requestedMinimum, 'The normal Inspector must commit the requested minimum.');
  } finally {
    await fixed.click();
  }
  assert.ok(await fixed.locator('input').isChecked(), 'Restore the original Fixed sizing before reconciliation.');
  const deadline = Date.now() + 30_000;
  let settled: MountedNode[] | undefined;
  let lastError: unknown;
  while (Date.now() < deadline) {
    try {
      const current = await mountedNodes(runtime);
      const actual = current.find((node) => node.nodeId === target.nodeId);
      assert.ok(actual && actual.shapeId === target.shapeId);
      assert.equal(actual.sourcePath, target.sourcePath);
      assert.equal(actual.sourceNodeId, target.sourceNodeId);
      assert.equal(actual.instancePath, target.instancePath);
      assert.equal(actual.visible, target.visible);
      assert.equal(actual.detached, target.detached);
      assert.deepEqual(current.map((node) => node.nodeId).sort(), nodes.map((node) => node.nodeId).sort());
      for (const node of current) {
        const original = byId.get(node.nodeId)!;
        for (const axis of ['x', 'y', 'width', 'height'] as const)
          assert.ok(Math.abs(node.bounds[axis] - original.bounds[axis]) <= 0.01,
            `Restoring Fixed must retain the baseline geometry: ${node.nodeId}.${axis}`);
      }
      settled = current;
      break;
    } catch (error) {
      lastError = error;
      await runtime.page.waitForTimeout(100);
    }
  }
  assert.ok(settled, 'The final Fixed layout did not settle without errors: ' + String(lastError));
  assert.equal(beforeMinimum, 96, 'This edit must start from the actual existing 96 DIP constraint.');
  assert.equal(afterMinimum, requestedMinimum);
  return {
    identity: { sourcePath: target.sourcePath!, sourceNodeId: target.sourceNodeId!,
      controlId: target.controlId ?? null, instancePath },
    shapeId: target.shapeId,
    method: 'Penpot layer selection and normal minimum-height inspector; restore Fixed',
    before: { heightMin: beforeMinimum, verticalSizing: 'fix', bounds: target.bounds },
    after: { heightMin: afterMinimum, verticalSizing: 'fix', bounds: settled.find((node) => node.nodeId === target.nodeId)!.bounds },
  };
}

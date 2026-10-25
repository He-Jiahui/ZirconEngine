import type { ZuiDocument } from './zui-document';
import type { LayoutReviewCaseMessage } from '../model';
import { reviewDocumentForCase } from './zui-review-case';
import { deriveTreeSelectionOwnerReview, nativeTreeOwnerFocusPublication, treeOwnerFocusedFromPublication, treeSelectionOwner } from './zui-tree-selection-owner';

const identity = { sourcePath: 'scene.zui', sourceNodeId: 'tree', instancePath: 'shell/tree', controlId: 'scene_tree' };
const origin = { ...identity, sourceNodeId: 'row_01', controlId: 'scene_row' };
const cursor = { structure: 40, geometry: 40, hitTest: 38, interaction: 9, viewport: 4, theme: 1, diagnostics: 1 };
interface TestEnvelope extends Record<string, unknown> {
  semanticNodes: Record<string, unknown>[];
  presentationCursor: Record<string, unknown>;
  presentedFrameCount: number;
  workbenchStatePublication: Record<string, unknown>;
}
const envelope = (focused: boolean): TestEnvelope => ({
  presentationCursor: { ...cursor }, presentedFrameCount: 14,
  workbenchStatePublication: { authority: 'normal-retained-workbench-projection', observed: true,
    presentationCursor: { ...cursor }, presentedFrameCount: 14 },
  // Deliberately conflicts with the painter input when focused=false.
  focusPublication: { authority: 'normal-native-pane-interaction', observed: true,
    focusedNativeControlId: 'HierarchyListPanel', templateFocusVisible: true,
    sourceOwner: identity, sourceOwnerMappingComplete: true },
  semanticNodes: [{ ...identity, nodeId: 'native/tree', treeOwnerFocused: focused,
    treeOwnerFocusPublication: { authority: 'normal-template-tree-row-painter-input',
      sourceAuthority: 'UiSurface.focus/tree_selection_owner', value: focused, observed: true,
      sourceOwner: { ...identity }, originComponent: { ...origin }, sourceOwnerMappingComplete: true,
      sourceFrame: { treeId: 'template.v2.shell.zui', generation: 16, ownerNodeId: 53, rowPrototypeNodeId: 54 },
      paintedNativeControlId: 'NativeHierarchyEntity:1', submittedOwnerCommandIndices: [101,102] } }],
});
const native = (value: TestEnvelope) => value.semanticNodes[0]!['treeOwnerFocusPublication'] as Record<string, unknown>;
const fixture = (): ZuiDocument => ({ asset: { kind: 'view', id: 'tree-owner-contract', version: 2 }, root: { node: 'root' }, nodes: {
  root: { component: 'Overlay', children: [{ node: 'tree' }, { node: 'outside' }] },
  tree: { component: 'VerticalGroup', control_id: identity.controlId,
    penpot_review_source_path: identity.sourcePath, penpot_review_source_node_id: identity.sourceNodeId,
    penpot_review_instance_path: identity.instancePath, props: { tree_selection_owner: true }, children: [{ node: 'row' }, { node: 'nested' }] },
  row: { component: 'TreeRow', props: { selected: true, hovered: true, focused: true, active: true, tree_owner_focused: true,
    selected_background_color: '#0070e0', selected_hover_background_color: '#0070e0', inactive_selected_background_color: '#40576f' } },
  nested: { component: 'VerticalGroup', props: { tree_selection_owner: true }, children: [{ node: 'nested_row' }] },
  nested_row: { component: 'TreeRow', props: { selected: true } }, outside: { component: 'TextField' },
} });

function workbenchFixture(focused = false) {
  const document = fixture(), packet = envelope(focused);
  const nativeWindow = 'res://ui/editor/unreal/level_editor_window.zui';
  const nativeOutliner = 'res://ui/editor/unreal/components/outliner.zui';
  const reviewWindow = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
  const reviewOutliner = 'zircon_editor/assets/ui/editor/unreal/components/outliner.zui';
  const prefix = [{ sourcePath: 'zircon_editor/assets/ui/editor/unreal/editor_shell.zui', sourceNodeId: 'pane_surface' }];
  const invocation = [{ sourcePath: nativeWindow, sourceNodeId: 'outliner' }];
  const reviewInvocation = [...prefix, { sourcePath: reviewWindow, sourceNodeId: 'outliner' }];
  const root = document.nodes!['root'];
  Object.assign(root, { control_id: 'WorkbenchWindowRoot', penpot_review_source_path: reviewWindow,
    penpot_review_source_node_id: 'root', penpot_review_instance_path: JSON.stringify(prefix) });
  for (const [id, sourceNodeId, controlId] of [['tree', 'tree', 'WorkbenchSceneTree'], ['row', 'row_01', 'WorkbenchSceneRootItem']])
    Object.assign(document.nodes![id!], { control_id: controlId, penpot_review_source_path: reviewOutliner,
      penpot_review_source_node_id: sourceNodeId, penpot_review_instance_path: JSON.stringify(reviewInvocation) });
  root.children = [{ node: 'editor_content' }, { node: 'outside' }];
  const ancestorFacts = [
    ['editor_content','UnrealLevelEditorContent',2,'VerticalGroup','root','main_band'],
    ['main_band','WorkbenchWindowMainBandRegion',27,'HorizontalGroup','editor_content','right_dock'],
    ['right_dock','UnrealRightDock',65,'VerticalGroup','main_band','panel'],
    ['panel','WorkbenchMainBandSceneTreePanel',66,'VerticalGroup','right_dock','content'],
    ['content','LeftDrawerContentRoot',69,'VerticalGroup','panel','tree'],
  ] as const;
  const nativeAncestor = (id: string, controlId: string) => ({ sourcePath: ['panel','content'].includes(id) ? nativeOutliner : nativeWindow,
    sourceNodeId: id, instancePath: ['panel','content'].includes(id) ? invocation : [], controlId, nodeId: `v2/${controlId}` });
  for (const [id, controlId, , component, , child] of ancestorFacts) document.nodes![id] = { component, control_id: controlId,
    penpot_review_source_path: ['panel','content'].includes(id) ? reviewOutliner : reviewWindow,
    penpot_review_source_node_id: id, penpot_review_instance_path: JSON.stringify(['panel','content'].includes(id) ? reviewInvocation : prefix),
    children: [{ node: child }] };
  const owner = { sourcePath: nativeOutliner, sourceNodeId: 'tree', instancePath: JSON.stringify(invocation), controlId: 'WorkbenchSceneTree' };
  const origin = { ...owner, sourceNodeId: 'row_01', controlId: 'WorkbenchSceneRootItem' };
  Object.assign(packet.semanticNodes[0]!, owner, { nodeId: 'v2/WorkbenchSceneTree' });
  Object.assign(native(packet), { sourceOwner: { ...owner }, originComponent: { ...origin },
    sourceFrame: { treeId: `template.v2.${nativeWindow}`, generation: 11, ownerNodeId: 77, rowPrototypeNodeId: 78 },
    submittedOwnerCommandIndices: [194,195] });
  packet.workbenchStatePublication['workbenchPresentation'] = { sourceFingerprint: { sourcePath: nativeWindow } };
  const rootIdentity = { sourcePath: nativeWindow, sourceNodeId: 'root', instancePath: [], controlId: 'WorkbenchWindowRoot' };
  const rows: Record<string, unknown>[] = [
    { ...rootIdentity, nodeId: 'v2/root', surfaceNodeId: 1, component: 'Overlay', parent: null },
    { ...owner, instancePath: invocation, nodeId: 'v2/WorkbenchSceneTree', surfaceNodeId: 77, component: 'ScrollableBox', parent: nativeAncestor('content','LeftDrawerContentRoot') },
    { ...origin, instancePath: invocation, nodeId: 'v2/WorkbenchSceneRootItem', surfaceNodeId: 78, component: 'TreeRow', parent: { ...owner, instancePath: invocation, nodeId: 'v2/WorkbenchSceneTree' } },
  ];
  for (const [id, controlId, surfaceNodeId, component, parent] of ancestorFacts) {
    const parentControl = ancestorFacts.find(([candidate]) => candidate === parent)?.[1];
    rows.push({ ...nativeAncestor(id,controlId), surfaceNodeId, component,
      parent: parent === 'root' ? { ...rootIdentity, nodeId: 'v2/root' } : nativeAncestor(parent,parentControl!) });
  }
  for (const row of rows) row['projected'] = { ...row,
    parentId: (row['parent'] as Record<string, unknown> | null)?.['nodeId'] ?? null };
  const surface = { schema: 'dev.zircon.editor.normal-surface-arranged-inventory', version: 1,
    scope: 'workbench-window', documentId: nativeWindow, coordinateSpace: 'surface-logical',
    frameAuthority: 'retained-host-projection', sourceFrameGeneration: 13, rootIds: [1], nodes: rows };
  packet['arrangedInventoryPublication'] = { schema: 'dev.zircon.editor.normal-arranged-inventory', version: 1,
    authority: 'normal-retained-runtime-surfaces', observed: true, complete: false, issues: ['unrelated prototype pending'],
    presentationCursor: { ...cursor }, presentedFrameCount: 14, surfaces: [surface] };
  packet['commands'] = [194,195].map((commandIndex, fragmentIndex) => ({ commandIndex, source: {
    ...owner, parentNodeId: 'v2/LeftDrawerContentRoot', parentSourcePath: nativeOutliner,
    parentSourceNodeId: 'content', parentInstancePath: JSON.stringify(invocation), sourceRef: { authority: 'surface-node-owner', primitiveIdentity: false,
      treeId: `template.v2.${nativeWindow}`, generation: 11, nodeId: 77, nodeCommandIndex: 0, fragmentIndex },
  } }));
  // The actual R53 full host includes these 13 source-bearing shell/Header
  // controls outside the main Workbench invocation prefix.
  for (const [id, component, controlId] of [
    ['host','UiHostWindow','UiHostWindowRoot'], ['document_host','DocumentHost','DocumentHostRoot'],
    ['pane_surface','PaneSurface','PaneSurfaceRoot'], ['dock_header_root','Overlay','DockHeaderRoot'],
    ['dock_header_bar','Container','DockHeaderBar'], ['dock_subtitle','Label','DockSubtitle'],
    ['dock_tab_strip','HorizontalGroup','DockTabStrip'], ['dock_tab_group_0','Overlay','DockTabGroup0'],
    ['dock_tab_0','Label','DockTab0'], ['dock_tab_group_1','Overlay','DockTabGroup1'], ['dock_tab_1','Label','DockTab1'],
    ['dock_tab_group_2','Overlay','DockTabGroup2'], ['dock_tab_2','Label','DockTab2'],
  ] as const) document.nodes![id] = { component, control_id: controlId,
    penpot_review_source_path: id.startsWith('dock_') ? 'zircon_editor/assets/ui/editor/unreal/components/dock_header.zui' : prefix[0]!.sourcePath,
    penpot_review_source_node_id: id, penpot_review_instance_path: '[]' };
  return { document, packet, surface, rows, reviewInvocation };
}

function currentInventoryFixture(focused = false) {
  const state = workbenchFixture(focused), { document, packet, rows } = state;
  const currentCursor = { structure: 6, geometry: 6, hitTest: 6, interaction: 1, viewport: 1, theme: 2, diagnostics: 1 };
  document.nodes!['tree'].component = 'ScrollableBox';
  for (const row of rows) {
    const attributes: Record<string, unknown> = {}, properties: Record<string, unknown> = {};
    if (row['controlId'] === 'WorkbenchSceneTree') {
      attributes['tree_selection_owner'] = true;
      properties['tree_selection_owner'] = true;
      properties['tree_owner_focused'] = focused;
    }
    if (row['controlId'] === 'WorkbenchSceneRootItem') properties['tree_owner_focused'] = focused;
    row['children'] = rows.filter((candidate) =>
      (candidate['parent'] as Record<string, unknown> | null)?.['nodeId'] === row['nodeId']).map((child) => child['surfaceNodeId']);
    row['rawMetadata'] = { component: row['component'], source_path: row['sourcePath'], source_node_id: row['sourceNodeId'],
      instance_path: row['instancePath'], control_id: row['controlId'], attributes };
    row['projected'] = { ...(row['projected'] as Record<string, unknown>), properties };
  }
  const surface = Object.assign(state.surface, { inventoryAudit: { complete: true, issues: [] },
    rawNodeCount: rows.length, arrangedNodeCount: rows.length, projectedNodeCount: rows.length,
    unarrangedNodes: [] as unknown[], unmatchedProjectionNodes: [] as unknown[] });
  const inventory = packet['arrangedInventoryPublication'] as Record<string, unknown>;
  delete inventory['issues'];
  Object.assign(inventory, { inventoryAudit: { complete: true, surfaceCount: 1 },
    pending: ['mounted-pane-source-style-overlay-closure'], presentationCursor: { ...currentCursor }, presentedFrameCount: 4 });
  packet.presentationCursor = { ...currentCursor };
  packet.presentedFrameCount = 4;
  Object.assign(packet.workbenchStatePublication, { presentationCursor: { ...currentCursor }, presentedFrameCount: 4 });
  // R85's owner 77 has no semantic or submitted command. The native entity
  // overlay command has source=null; origin 78 and focus live in the projection.
  const root = rows[0]!;
  packet.semanticNodes = [{ sourcePath: root['sourcePath'], sourceNodeId: root['sourceNodeId'],
    instancePath: JSON.stringify(root['instancePath']), controlId: root['controlId'], nodeId: root['nodeId'] }];
  packet['commands'] = [{ commandIndex: 242, source: null }];
  return { ...state, surface, inventory };
}

describe('same-present Native Tree owner painter input', () => {
  it.each([false, true])('uses current inventory projection focus %s for the unpainted R85 owner', (focused) => {
    const { document, packet, rows, inventory, surface } = currentInventoryFixture(focused);
    const before = structuredClone(packet), source = structuredClone(document);
    expect(surface.sourceFrameGeneration).toBe(13);
    expect(rows[1]!['surfaceNodeId']).toBe(77);
    expect(rows[2]!['surfaceNodeId']).toBe(78);
    expect(inventory['complete']).toBe(false);
    expect(((rows[1]!['rawMetadata'] as Record<string, unknown>)['attributes'] as Record<string, unknown>)['tree_owner_focused']).toBeUndefined();
    expect(nativeTreeOwnerFocusPublication(rows[1]!['projected'])).toBeUndefined();
    expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBe(focused);
    expect(document).toEqual(source);
    deriveTreeSelectionOwnerReview(document, packet);
    expect(document.nodes!['row'].props!['tree_owner_focused']).toBe(focused);
    expect(document.nodes!['row'].style!['self']).toMatchObject({ background_color: focused ? '#0070e0' : '#40576f' });
    expect(packet).toEqual(before);
    expect(packet.semanticNodes.every((node) => !Object.hasOwn(node, 'treeOwnerFocusPublication'))).toBe(true);
  });

  it('uses Runtime projected focus even when raw authored focus flags disagree', () => {
    const { document, packet, rows } = currentInventoryFixture(false);
    for (const row of [rows[1]!, rows[2]!])
      ((row['rawMetadata'] as Record<string, unknown>)['attributes'] as Record<string, unknown>)['tree_owner_focused'] = true;
    expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBe(false);
  });

  it.each(['missing inventory audit', 'incomplete surface audit', 'surface issues', 'unarranged node', 'unmatched projection',
    'inconsistent counts', 'zero generation', 'missing owner focus', 'missing origin focus', 'null owner focus',
    'string origin focus', 'disagreeing focus', 'author-only focus', 'missing raw declaration', 'missing projected declaration',
    'missing raw metadata', 'wrong raw tuple', 'wrong projected tuple', 'wrong origin component', 'wrong parent tuple',
    'wrong projected parent', 'missing child binding', 'nearer native owner', 'nearer review owner',
    'wrong native root', 'similar review prefix', 'unobserved inventory', 'another present'])(
    'keeps current inventory %s unknown', (kind) => {
      const { document, packet, rows, surface, inventory, reviewInvocation } = currentInventoryFixture();
      const owner = rows[1]!, origin = rows[2]!;
      const ownerProjected = owner['projected'] as Record<string, unknown>, originProjected = origin['projected'] as Record<string, unknown>;
      const ownerProps = ownerProjected['properties'] as Record<string, unknown>, originProps = originProjected['properties'] as Record<string, unknown>;
      const ownerRaw = owner['rawMetadata'] as Record<string, unknown>, originRaw = origin['rawMetadata'] as Record<string, unknown>;
      const ownerAttributes = ownerRaw['attributes'] as Record<string, unknown>, originAttributes = originRaw['attributes'] as Record<string, unknown>;
      if (kind === 'missing inventory audit') delete inventory['inventoryAudit'];
      if (kind === 'incomplete surface audit') surface.inventoryAudit.complete = false;
      if (kind === 'surface issues') (surface.inventoryAudit.issues as string[]).push('source mismatch');
      if (kind === 'unarranged node') surface.unarrangedNodes.push({ surfaceNodeId: 77 });
      if (kind === 'unmatched projection') surface.unmatchedProjectionNodes.push({ nodeId: origin['nodeId'] });
      if (kind === 'inconsistent counts') surface.projectedNodeCount++;
      if (kind === 'zero generation') surface.sourceFrameGeneration = 0;
      if (kind === 'missing owner focus') delete ownerProps['tree_owner_focused'];
      if (kind === 'missing origin focus') delete originProps['tree_owner_focused'];
      if (kind === 'null owner focus') ownerProps['tree_owner_focused'] = null;
      if (kind === 'string origin focus') originProps['tree_owner_focused'] = 'false';
      if (kind === 'disagreeing focus') originProps['tree_owner_focused'] = true;
      if (kind === 'author-only focus') {
        ownerAttributes['tree_owner_focused'] = false; originAttributes['tree_owner_focused'] = false;
        document.nodes!['row'].props!['tree_owner_focused'] = false;
        delete ownerProps['tree_owner_focused']; delete originProps['tree_owner_focused'];
      }
      if (kind === 'missing raw declaration') delete ownerAttributes['tree_selection_owner'];
      if (kind === 'missing projected declaration') delete ownerProps['tree_selection_owner'];
      if (kind === 'missing raw metadata') delete owner['rawMetadata'];
      if (kind === 'wrong raw tuple') ownerRaw['source_node_id'] = 'other_tree';
      if (kind === 'wrong projected tuple') originProjected['instancePath'] = [];
      if (kind === 'wrong origin component') { origin['component'] = 'Label'; originProjected['component'] = 'Label'; originRaw['component'] = 'Label'; }
      if (kind === 'wrong parent tuple') (origin['parent'] as Record<string, unknown>)['instancePath'] = [];
      if (kind === 'wrong projected parent') originProjected['parentId'] = 'v2/root';
      if (kind === 'missing child binding') owner['children'] = [];
      if (kind === 'nearer native owner') { originAttributes['tree_selection_owner'] = true; originProps['tree_selection_owner'] = true; }
      if (kind === 'nearer review owner') document.nodes!['row'].props!['tree_selection_owner'] = true;
      if (kind === 'wrong native root') rows[0]!['sourcePath'] = 'res://ui/editor/unreal/other.zui';
      if (kind === 'similar review prefix') document.nodes!['tree']['penpot_review_instance_path'] = JSON.stringify([
        { ...reviewInvocation[0]!, sourceNodeId: 'pane_surface_extra' }, reviewInvocation[1]!,
      ]);
      if (kind === 'unobserved inventory') inventory['observed'] = false;
      if (kind === 'another present') inventory['presentedFrameCount'] = 5;
      expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBeUndefined();
      if (kind === 'author-only focus') {
        deriveTreeSelectionOwnerReview(document, packet);
        expect(document.nodes!['row'].props!['tree_owner_focused']).toBeUndefined();
      }
    },
  );

  it.each(['owner', 'origin'])('rejects a duplicate full current %s tuple with distinct Runtime IDs', (kind) => {
    const { document, packet, rows, surface } = currentInventoryFixture();
    const duplicate = structuredClone(rows[kind === 'owner' ? 1 : 2]!);
    duplicate['nodeId'] = 'v2/duplicate'; duplicate['surfaceNodeId'] = 901;
    duplicate['sourcePath'] = 'zircon_editor/assets/ui/editor/unreal/components/outliner.zui';
    const projected = duplicate['projected'] as Record<string, unknown>;
    projected['nodeId'] = duplicate['nodeId']; projected['surfaceNodeId'] = duplicate['surfaceNodeId'];
    rows.push(duplicate);
    surface.rawNodeCount++; surface.arrangedNodeCount++; surface.projectedNodeCount++;
    const parentId = (duplicate['parent'] as Record<string, unknown>)['nodeId'];
    (rows.find((row) => row['nodeId'] === parentId)!['children'] as number[]).push(901);
    expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBeUndefined();
  });

  it('rejects an aliased duplicate review origin without guessing from its control', () => {
    const { document, packet } = currentInventoryFixture();
    document.nodes!['duplicate_row'] = structuredClone(document.nodes!['row']);
    document.nodes!['duplicate_row']['penpot_review_source_path'] = 'res://ui/editor/unreal/components/outliner.zui';
    document.nodes!['tree'].children!.push({ node: 'duplicate_row' });
    expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBeUndefined();
  });

  it.each(Object.keys(cursor))('requires the complete current inventory and business cursor %s', (key) => {
    const stale = currentInventoryFixture();
    (stale.inventory['presentationCursor'] as Record<string, unknown>)[key] = 999;
    expect(treeOwnerFocusedFromPublication(stale.document, 'row', stale.packet)).toBeUndefined();
    const missing = currentInventoryFixture();
    delete (missing.packet.workbenchStatePublication['presentationCursor'] as Record<string, unknown>)[key];
    expect(treeOwnerFocusedFromPublication(missing.document, 'row', missing.packet)).toBeUndefined();
  });

  it.each(['missing receipt', 'unobserved receipt', 'missing submitted commands', 'duplicate semantic', 'broken semantic tuple'])(
    'does not downgrade %s to current inventory authority', (kind) => {
      const { document, packet } = currentInventoryFixture();
      const semantic = workbenchFixture().packet.semanticNodes[0]!;
      if (kind === 'missing receipt') delete semantic['treeOwnerFocusPublication'];
      if (kind === 'unobserved receipt') (semantic['treeOwnerFocusPublication'] as Record<string, unknown>)['observed'] = false;
      if (kind === 'missing submitted commands') delete packet['commands'];
      if (kind === 'broken semantic tuple') delete semantic['instancePath'];
      packet.semanticNodes.push(semantic);
      if (kind === 'duplicate semantic') packet.semanticNodes.push(structuredClone(semantic));
      expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBeUndefined();
    },
  );

  it('joins cached source generation 11 with arranged generation 13 and all 13 actual shell/Header identities', () => {
    for (const focused of [false, true]) {
      const { document, packet } = workbenchFixture(focused), before = structuredClone(packet);
      expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBe(focused);
      deriveTreeSelectionOwnerReview(document, packet);
      expect(document.nodes!['row'].props!['tree_owner_focused']).toBe(focused);
      expect(document.nodes!['row'].style!['self']).toMatchObject({ background_color: focused ? '#0070e0' : '#40576f' });
      expect(packet).toEqual(before);
      expect((packet['arrangedInventoryPublication'] as Record<string, unknown>)['complete']).toBe(false);
    }
  });

  it.each(['wrong root', 'duplicate root', 'outside review owner', 'outside native owner', 'similar prefix',
    'missing inventory', 'stale inventory', 'duplicate semantic', 'duplicate source owner', 'duplicate owner outside prefix', 'arranged older than submitted frame',
    'wrong prototype owner', 'unknown publication'])(
    'keeps %s unknown instead of matching a control id or suffix', (kind) => {
      const { document, packet, rows, surface, reviewInvocation } = workbenchFixture();
      if (kind === 'wrong root') document.nodes!['root']['penpot_review_source_path'] = 'zircon_editor/assets/ui/editor/unreal/other.zui';
      if (kind === 'duplicate root') document.nodes!['duplicate_root'] = structuredClone(document.nodes!['root']);
      if (kind === 'outside review owner') document.nodes!['root'].children = [{ node: 'outside' }];
      if (kind === 'outside native owner') {
        rows[1]!['parent'] = null;
        (rows[1]!['projected'] as Record<string, unknown>)['parentId'] = null;
      }
      if (kind === 'similar prefix') document.nodes!['tree']['penpot_review_instance_path'] = JSON.stringify([
        { ...reviewInvocation[0]!, sourceNodeId: 'pane_surface_extra' }, reviewInvocation[1]!,
      ]);
      if (kind === 'missing inventory') delete packet['arrangedInventoryPublication'];
      if (kind === 'stale inventory') (packet['arrangedInventoryPublication'] as Record<string, unknown>)['presentedFrameCount'] = 15;
      if (kind === 'duplicate semantic') packet.semanticNodes.push(structuredClone(packet.semanticNodes[0]!));
      if (kind === 'duplicate source owner') {
        document.nodes!['duplicate_tree'] = structuredClone(document.nodes!['tree']);
        document.nodes!['duplicate_tree']['penpot_review_source_path'] = 'res://ui/editor/unreal/components/outliner.zui';
      }
      if (kind === 'duplicate owner outside prefix') {
        document.nodes!['duplicate_tree'] = structuredClone(document.nodes!['tree']);
        document.nodes!['duplicate_tree']['penpot_review_instance_path'] = '[]';
      }
      if (kind === 'arranged older than submitted frame') surface.sourceFrameGeneration = 10;
      if (kind === 'wrong prototype owner') {
        rows[2]!['parent'] = rows[1]!['parent'];
        (rows[2]!['projected'] as Record<string, unknown>)['parentId'] = 'v2/root';
      }
      if (kind === 'unknown publication') {
        native(packet)['observed'] = false; native(packet)['value'] = null;
        packet.semanticNodes[0]!['treeOwnerFocused'] = null;
      }
      expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBeUndefined();
    },
  );

  it.each(['missing commands','missing owner command','duplicate command index','publication generation rollback',
    'command generation rollback','wrong publication tree','foreign command tree','foreign matching tree',
    'wrong submitted node','wrong semantic runtime node','wrong source owner','wrong full parent','partial source reference'])(
    'keeps %s unknown without borrowing a newer arranged generation', (kind) => {
      const { document, packet } = workbenchFixture();
      const commands = packet['commands'] as Record<string, unknown>[];
      const source = commands[0]!['source'] as Record<string, unknown>;
      const reference = source['sourceRef'] as Record<string, unknown>;
      const frame = native(packet)['sourceFrame'] as Record<string, unknown>;
      if (kind === 'missing commands') delete packet['commands'];
      if (kind === 'missing owner command') commands.pop();
      if (kind === 'duplicate command index') commands.push(structuredClone(commands[0]!));
      if (kind === 'publication generation rollback') frame['generation'] = 10;
      if (kind === 'command generation rollback') reference['generation'] = 10;
      if (kind === 'wrong publication tree') frame['treeId'] = 'template.v2.other';
      if (kind === 'foreign command tree') reference['treeId'] = 'template.v2.other';
      if (kind === 'foreign matching tree') {
        frame['treeId'] = 'template.v2.other';
        for (const command of commands) ((command['source'] as Record<string, unknown>)['sourceRef'] as Record<string, unknown>)['treeId'] = frame['treeId'];
      }
      if (kind === 'wrong submitted node') reference['nodeId'] = 78;
      if (kind === 'wrong semantic runtime node') packet.semanticNodes[0]!['nodeId'] = 'v2/other';
      if (kind === 'wrong source owner') source['instancePath'] = '[]';
      if (kind === 'wrong full parent') source['parentInstancePath'] = JSON.stringify([{ sourcePath: 'res://ui/editor/unreal/level_editor_window.zui', sourceNodeId: 'other' }]);
      if (kind === 'partial source reference') delete reference['fragmentIndex'];
      expect(treeOwnerFocusedFromPublication(document, 'row', packet)).toBeUndefined();
    },
  );

  it('reads actual true/false and leaves an unobserved nearest owner unknown', () => {
    const document = fixture();
    expect(treeSelectionOwner(document, 'nested_row')).toBe('nested');
    expect(treeOwnerFocusedFromPublication(document, 'row', envelope(false))).toBe(false);
    expect(treeOwnerFocusedFromPublication(document, 'row', envelope(true))).toBe(true);
    expect(treeOwnerFocusedFromPublication(document, 'nested_row', envelope(true))).toBeUndefined();
    expect(treeOwnerFocusedFromPublication(document, 'row', undefined)).toBeUndefined();
  });

  it('keeps pointer focus, row active/hover/focus and source generation separate from owner focus', () => {
    const packet = envelope(false);
    expect(native(packet)['sourceFrame']).toMatchObject({ generation: 16 });
    expect(packet.presentedFrameCount).toBe(14);
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', packet)).toBe(false);
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', packet['focusPublication'])).toBeUndefined();
  });

  it.each(['authority', 'sourceAuthority', 'observed', 'sourceOwnerMappingComplete', 'value'])
  ('rejects malformed Native publication %s', (key) => {
    const packet = envelope(true);
    native(packet)[key] = 'invalid';
    expect(nativeTreeOwnerFocusPublication(native(packet))).toBeUndefined();
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', packet)).toBeUndefined();
  });

  it.each(['sourcePath', 'sourceNodeId', 'instancePath', 'controlId'])
  ('requires exact semantic and publication owner %s', (key) => {
    const packet = envelope(true);
    packet.semanticNodes[0]![key] = 'different';
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', packet)).toBeUndefined();
    const wrongPublication = envelope(true);
    native(wrongPublication)['sourceOwner'] = { ...identity, [key]: 'different' };
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', wrongPublication)).toBeUndefined();
  });

  it('keeps null, absent, disagreement and ambiguous owners unknown', () => {
    const unknown = envelope(true);
    unknown.semanticNodes[0]!['treeOwnerFocused'] = null;
    native(unknown)['value'] = null;
    native(unknown)['observed'] = false;
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', unknown)).toBeUndefined();
    const missing = envelope(true);
    delete missing.semanticNodes[0]!['treeOwnerFocusPublication'];
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', missing)).toBeUndefined();
    const disagreement = envelope(true);
    disagreement.semanticNodes[0]!['treeOwnerFocused'] = false;
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', disagreement)).toBeUndefined();
    const duplicatePacket = envelope(true);
    duplicatePacket.semanticNodes.push(structuredClone(duplicatePacket.semanticNodes[0]!));
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', duplicatePacket)).toBeUndefined();
    const duplicateDocument = fixture();
    duplicateDocument.nodes!['duplicate'] = { ...duplicateDocument.nodes!['tree'] };
    expect(treeOwnerFocusedFromPublication(duplicateDocument, 'row', envelope(true))).toBeUndefined();
  });

  it.each(['treeId', 'generation', 'ownerNodeId', 'rowPrototypeNodeId'])
  ('requires a complete valid source frame %s', (key) => {
    const packet = envelope(true);
    const frame = native(packet)['sourceFrame'] as Record<string, unknown>;
    delete frame[key];
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', packet)).toBeUndefined();
    const malformed = envelope(true);
    const badFrame = native(malformed)['sourceFrame'] as Record<string, unknown>;
    badFrame[key] = key === 'treeId' ? '' : -1;
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', malformed)).toBeUndefined();
  });

  it('rejects the same node as tree owner and row prototype or absent draw admission', () => {
    const packet = envelope(true);
    const frame = native(packet)['sourceFrame'] as Record<string, unknown>;
    frame['rowPrototypeNodeId'] = frame['ownerNodeId'];
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', packet)).toBeUndefined();
    const missingDraw = envelope(true);
    native(missingDraw)['paintedNativeControlId'] = 'HierarchyListPanel';
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', missingDraw)).toBeUndefined();
    const missingSource = envelope(true);
    native(missingSource)['submittedOwnerCommandIndices'] = [];
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', missingSource)).toBeUndefined();
  });

  it('rejects another normal present, missing counts and malformed business observation', () => {
    const stale = envelope(true);
    stale.presentedFrameCount++;
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', stale)).toBeUndefined();
    const missingCount = envelope(true);
    delete missingCount.workbenchStatePublication['presentedFrameCount'];
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', missingCount)).toBeUndefined();
    const unobserved = envelope(true);
    unobserved.workbenchStatePublication['observed'] = false;
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', unobserved)).toBeUndefined();
  });

  it.each(Object.keys(cursor))('requires exact same-present counter %s', (key) => {
    const stale = envelope(true);
    stale.presentationCursor[key] = Number(stale.presentationCursor[key]) + 1;
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', stale)).toBeUndefined();
    const missing = envelope(true);
    delete missing.presentationCursor[key];
    expect(treeOwnerFocusedFromPublication(fixture(), 'row', missing)).toBeUndefined();
  });

  it('projects known values through the real case entry without source, state or evidence mutation', () => {
    const source = fixture();
    const before = structuredClone(source);
    const packet = envelope(false);
    const seed = structuredClone(packet);
    const reviewCase: LayoutReviewCaseMessage = { id:'actual-tree',sourcePath:'scene.zui',host:'component',
      viewport:{width:360,height:520},dpi:1,locale:'en-US',state:'default',
      data:{normal_window_evidence:packet} };
    const reviewed = reviewDocumentForCase(source,reviewCase);
    expect(reviewed.nodes!['row'].props!['tree_owner_focused']).toBe(false);
    expect(reviewed.nodes!['row'].style!['self']).toMatchObject({background_color:'#40576f'});
    expect(source).toEqual(before);
    expect(packet).toEqual(seed);
    expect(reviewCase.state).toBe('default');
    const noEvidence = structuredClone(source);
    deriveTreeSelectionOwnerReview(noEvidence,undefined);
    expect(noEvidence.nodes!['row'].props!['tree_owner_focused']).toBeUndefined();
  });

  it.each(['disabled', 'loading'])('suppresses selected surface when %s', (key) => {
    const document = fixture();
    document.nodes!['row'].props![key] = true;
    deriveTreeSelectionOwnerReview(document,envelope(false));
    expect(document.nodes!['row'].style!['self']).toMatchObject({background_color:'transparent'});
  });
});

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

describe('same-present Native Tree owner painter input', () => {
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

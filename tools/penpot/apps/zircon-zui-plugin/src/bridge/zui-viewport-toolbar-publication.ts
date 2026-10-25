import { readViewportSnapState } from './zui-viewport-snap-state';

export interface ToolbarRect { x: number; y: number; width: number; height: number }
export interface ToolbarCursor {
  structure: number; geometry: number; interaction: number; viewport: number;
  hitTest: number; theme: number; diagnostics: number;
}
export interface ToolbarSourceNode {
  nodeId: string; parentNodeId: string | null;
  sourcePath: string; sourceNodeId: string; instancePath: string;
  parentSourcePath: string | null; parentSourceNodeId: string | null; parentInstancePath: string | null;
  controlId: string | null; component: string; visible: boolean;
  bounds: ToolbarRect; clip: ToolbarRect;
  state: {
    selected: boolean | null; checked: boolean | null; popupOpen: boolean | null;
    value: string | number | boolean | null; text: string | null; valueText: string | null; disabled: boolean;
  };
  scroll: { offset: number; viewportExtent: number; contentExtent: number } | null;
  /** Optional extra measurement is explicitly logical even in a physical packet. */
  desiredSize?: { width: number; height: number; coordinateSpace: 'logical-pixels' };
}
export interface ViewportToolbarPublication {
  schema: 'dev.zircon.editor.viewport-toolbar-publication'; version: 1;
  authority: 'normal-prepared-source-surface'; observed: true; coordinateSpace: 'physical-pixels';
  surfaceGeneration: number; surfaceTreeId: string;
  rootNodeId: string; rowNodeId: string; moreNodeId: string; popupNodeId: string;
  groupNodeIds: string[]; nodes: ToolbarSourceNode[];
  presentationCursor: ToolbarCursor; presentedFrameCount: number;
}
const FIELDS = ['structure','geometry','interaction','viewport','hitTest','theme','diagnostics'] as const;
const record = (value: unknown): value is Record<string, unknown> => value !== null && typeof value === 'object' && !Array.isArray(value);
const string = (value: unknown): value is string => typeof value === 'string' && value.trim().length > 0 && value.length <= 16384;
const uint = (value: unknown): value is number => typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
function requireValue(condition: unknown, message: string): asserts condition { if (!condition) throw new Error('Viewport toolbar publication: ' + message); }
function exact(value: Record<string, unknown>, keys: readonly string[], label: string): void {
  requireValue(Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value,key)),label+' requires its exact fields');
}
function cursor(value: unknown, label: string): asserts value is ToolbarCursor {
  requireValue(record(value),label+' is missing'); exact(value,FIELDS,label);
  requireValue(FIELDS.every(key => uint(value[key])),label+' has a missing/unsafe counter');
}
function sameCursor(a: ToolbarCursor, b: ToolbarCursor): boolean { return FIELDS.every(key => a[key] === b[key]); }
function instance(value: unknown, label: string): void {
  requireValue(string(value),label+' is missing');
  let decoded: unknown;
  try { decoded = JSON.parse(value); } catch { throw new Error('Viewport toolbar publication: '+label+' is not a compact JSON call-site list'); }
  requireValue(Array.isArray(decoded) && decoded.length <= 64 && decoded.every(step =>
    record(step) && Object.keys(step).length === 2 &&
    Object.hasOwn(step,'sourcePath') && Object.hasOwn(step,'sourceNodeId') &&
    string(step['sourcePath']) && string(step['sourceNodeId'])),label+' is not a call-site list');
  const canonical = decoded.map(step => ({sourcePath:step['sourcePath'],sourceNodeId:step['sourceNodeId']}));
  requireValue(JSON.stringify(canonical) === value,label+' is not the exact compact source identity');
}
function rect(value: unknown, label: string): void {
  requireValue(record(value),label+' is missing'); exact(value,['x','y','width','height'],label);
  requireValue(Object.values(value).every(v => typeof v === 'number' && Number.isFinite(v)) &&
    (value['width'] as number) >= 0 && (value['height'] as number) >= 0,label+' must be finite observed geometry');
}
export function toolbarSourceKey(value: Pick<ToolbarSourceNode,'sourcePath'|'sourceNodeId'|'instancePath'>): string {
  return JSON.stringify([value.sourcePath,value.sourceNodeId,value.instancePath]);
}
function freeze<T>(value: T): T {
  if (value !== null && typeof value === 'object') {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
}

/** Absent remains unknown. A present malformed or mismatched observation is rejected. */
export function readViewportToolbarPublication(evidence: unknown): ViewportToolbarPublication | undefined {
  if (!record(evidence) || !Object.hasOwn(evidence,'viewportToolbarPublication')) return undefined;
  const packet=evidence['viewportToolbarPublication'];
  requireValue(record(packet),'payload is not an object');
  exact(packet,['schema','version','authority','observed','coordinateSpace','surfaceGeneration','surfaceTreeId',
    'rootNodeId','rowNodeId','moreNodeId','popupNodeId','groupNodeIds','nodes','presentationCursor','presentedFrameCount'],'payload');
  requireValue(packet['schema']==='dev.zircon.editor.viewport-toolbar-publication' && packet['version']===1 &&
    packet['authority']==='normal-prepared-source-surface' && packet['observed']===true && packet['coordinateSpace']==='physical-pixels','unsupported source/observation authority');
  cursor(packet['presentationCursor'],'toolbar cursor'); cursor(evidence['presentationCursor'],'normal cursor');
  requireValue(sameCursor(packet['presentationCursor'],evidence['presentationCursor']),'toolbar and normal packet cursors differ');
  requireValue(uint(packet['presentedFrameCount']) && packet['presentedFrameCount']>0 &&
    packet['presentedFrameCount']===evidence['presentedFrameCount'],'toolbar belongs to a different successful frame');
  const business=evidence['workbenchStatePublication'];
  requireValue(record(business) && business['authority']==='normal-retained-workbench-projection' && business['observed']===true,'actual observed business publication is missing');
  cursor(business['presentationCursor'],'business cursor');
  requireValue(sameCursor(packet['presentationCursor'],business['presentationCursor']) && business['presentedFrameCount']===packet['presentedFrameCount'],'toolbar and business frame differ');
  requireValue(uint(packet['surfaceGeneration']) && string(packet['surfaceTreeId']),'prepared surface identity is missing');
  for (const key of ['rootNodeId','rowNodeId','moreNodeId','popupNodeId']) requireValue(string(packet[key]),key+' is missing');
  requireValue(new Set(['rootNodeId','rowNodeId','moreNodeId','popupNodeId'].map(key => packet[key])).size===4,'source owners are not distinct');
  requireValue(Array.isArray(packet['groupNodeIds']) && packet['groupNodeIds'].length>0 && packet['groupNodeIds'].length<=32 && packet['groupNodeIds'].every(string) &&
    new Set(packet['groupNodeIds']).size===packet['groupNodeIds'].length,'original group order is missing/duplicated');
  requireValue(Array.isArray(packet['nodes']) && packet['nodes'].length>0 && packet['nodes'].length<=512,'source subtree is missing/unbounded');
  const byId=new Map<string,ToolbarSourceNode>(), identities=new Set<string>();
  for (const raw of packet['nodes']) {
    requireValue(record(raw),'source node is not an object');
    const fields=['nodeId','parentNodeId','sourcePath','sourceNodeId','instancePath','parentSourcePath','parentSourceNodeId','parentInstancePath','controlId','component','visible','bounds','clip','state','scroll'];
    if (Object.hasOwn(raw,'desiredSize')) fields.push('desiredSize');
    exact(raw,fields,'source node');
    requireValue(string(raw['nodeId']) && (raw['parentNodeId']===null || string(raw['parentNodeId'])) && string(raw['sourcePath']) && string(raw['sourceNodeId']) && string(raw['component']) && typeof raw['visible']==='boolean' && (raw['controlId']===null || string(raw['controlId'])),'source node identity/state is incomplete');
    instance(raw['instancePath'],'instancePath');
    for (const key of ['parentSourcePath','parentSourceNodeId','parentInstancePath']) requireValue(raw[key]===null || string(raw[key]),key+' must explicitly preserve known/null identity');
    requireValue(['parentSourcePath','parentSourceNodeId','parentInstancePath'].every(key => raw[key]===null) ||
      ['parentSourcePath','parentSourceNodeId','parentInstancePath'].every(key => string(raw[key])),'partial parent source tuple');
    if (raw['parentInstancePath']!==null) instance(raw['parentInstancePath'],'parentInstancePath');
    rect(raw['bounds'],'bounds'); rect(raw['clip'],'clip');
    const state=raw['state']; requireValue(record(state),'source state is missing');
    exact(state,['selected','checked','popupOpen','value','text','valueText','disabled'],'source state');
    for (const key of ['selected','checked','popupOpen']) requireValue(state[key]===null || typeof state[key]==='boolean',key+' must remain explicit boolean/null');
    for (const key of ['text','valueText']) requireValue(state[key]===null || typeof state[key]==='string',key+' must remain string/null');
    requireValue(typeof state['disabled']==='boolean','disabled must be observed');
    requireValue(state['value']===null || typeof state['value']==='string' || typeof state['value']==='boolean' ||
      (typeof state['value']==='number' && Number.isFinite(state['value'])),'value must be an observed finite primitive/null');
    const scroll=raw['scroll'];
    if (scroll!==null) {
      requireValue(record(scroll),'scroll must explicitly preserve known/null state');
      exact(scroll,['offset','viewportExtent','contentExtent'],'scroll');
      requireValue(Object.values(scroll).every(v => typeof v==='number' && Number.isFinite(v) && v>=0),'scroll requires finite actual extents');
      requireValue((scroll['offset'] as number)<=Math.max(0,(scroll['contentExtent'] as number)-(scroll['viewportExtent'] as number))+0.01,'scroll offset is outside its actual content');
    }
    if (Object.hasOwn(raw,'desiredSize')) {
      const desired=raw['desiredSize']; requireValue(record(desired),'desired measurement is missing');
      exact(desired,['width','height','coordinateSpace'],'desired measurement');
      requireValue(desired['coordinateSpace']==='logical-pixels' && ['width','height'].every(key => typeof desired[key]==='number' && Number.isFinite(desired[key]) && (desired[key] as number)>=0),'desired measurement requires explicit logical units');
    }
    const node=raw as unknown as ToolbarSourceNode, identity=toolbarSourceKey(node);
    requireValue(!byId.has(node.nodeId) && !identities.has(identity),'duplicate native/source identity');
    byId.set(node.nodeId,node); identities.add(identity);
  }
  const owners=['rootNodeId','rowNodeId','moreNodeId','popupNodeId'].map(key => byId.get(packet[key] as string));
  requireValue(owners.every(Boolean),'missing original toolbar owner');
  const root=owners[0]!, row=owners[1]!, more=owners[2]!, popup=owners[3]!;
  requireValue(root.controlId==='UnrealViewportToolbar' && row.controlId==='UnrealViewportToolbarRow' && more.controlId==='UnrealViewportToolbarMore' && popup.controlId==='UnrealViewportToolbarOverflow','source controls do not own the viewport toolbar');
  requireValue(root.visible && row.visible,'normal toolbar root/row is not visible');
  requireValue(root.parentNodeId === null || !byId.has(root.parentNodeId),'root parent must be outside its prepared subtree');
  for (const node of byId.values()) {
    if (node.nodeId===root.nodeId) continue;
    const parent=byId.get(node.parentNodeId ?? ''); requireValue(parent,'source subtree has a missing parent');
    requireValue(!node.visible || parent.visible,'visible child has an invisible source ancestor');
    requireValue(node.parentSourcePath===parent.sourcePath && node.parentSourceNodeId===parent.sourceNodeId && node.parentInstancePath===parent.instancePath,'current parent tuple differs from its actual node');
    const seen=new Set<string>(); let current: ToolbarSourceNode | undefined=node;
    while (current && current.nodeId!==root.nodeId) {
      requireValue(!seen.has(current.nodeId),'cycle in source subtree'); seen.add(current.nodeId);
      current=byId.get(current.parentNodeId ?? '');
    }
    requireValue(current?.nodeId===root.nodeId,'node escapes the prepared source toolbar');
  }
  requireValue(row.parentNodeId===root.nodeId && more.parentNodeId===row.nodeId && popup.parentNodeId===root.nodeId,'toolbar owner topology differs');
  requireValue(typeof popup.state.popupOpen==='boolean','popup open authority is missing');
  // More is an IconButton, not a Dropdown. Its absent popupOpen remains unknown;
  // selected/checked represent its actual open-state highlight when known.
  for (const value of [more.state.selected,more.state.checked])
    if (value!==null) requireValue(value===popup.state.popupOpen,'More selection contradicts actual popup state');
  const groups=(packet['groupNodeIds'] as string[]).map(id => byId.get(id)); requireValue(groups.every(Boolean),'missing original group');
  for (const group of groups as ToolbarSourceNode[]) requireValue(group.parentNodeId===row.nodeId || group.parentNodeId===popup.nodeId,'group has an unrelated current parent');
  const split=(groups as ToolbarSourceNode[]).findIndex(group => group.parentNodeId===popup.nodeId), inline=split<0?groups.length:split;
  requireValue((groups as ToolbarSourceNode[]).every((group,index) => group.parentNodeId===(index<inline?row.nodeId:popup.nodeId)),'overflow is not the original ordered suffix');
  requireValue(more.visible===(inline<groups.length),'More visibility contradicts actual group ownership');
  requireValue(popup.visible===(popup.state.popupOpen && inline<groups.length),'popup visibility contradicts its actual source state');
  if (popup.visible) requireValue(popup.scroll!==null,'visible scrollable popup lacks actual scroll state');
  requireValue(record(business['workbenchPresentation']),'prepared business DTO is missing');
  const snap=readViewportSnapState(business['workbenchPresentation']['viewportSnap']);
  for (const [control,expected] of [['ToggleTranslateSnap',snap.translationEnabled],['ToggleRotateSnap',snap.rotationEnabled],['ToggleScaleSnap',snap.scaleEnabled]] as const) {
    const matches=[...byId.values()].filter(node => node.controlId===control); requireValue(matches.length===1,'snap source owner is missing/duplicated: '+control);
    const state=matches[0]!.state;
    requireValue(state.checked===expected && state.selected===expected && state.value===expected,'source snap differs from the actual independent business bool: '+control);
  }
  return freeze(JSON.parse(JSON.stringify(packet)) as ViewportToolbarPublication);
}

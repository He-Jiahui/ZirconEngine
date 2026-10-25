import { copyFontResources } from './penpot-font-selection';
import type { ZuiDocument, ZuiNode, ZuiChildMount, ZuiTable, ZuiValue } from './zui-document';
import type { ZuiAssetProjection, ProjectedZuiNode } from './penpot-projection-model';
import { readViewportToolbarPublication, toolbarSourceKey, type ToolbarSourceNode, type ViewportToolbarPublication } from './zui-viewport-toolbar-publication';
export interface ViewportToolbarContract {
  root: string; row: string; more: string; popup: string; groups: string[];
  originalRowChildren: ZuiChildMount[];
}
export interface ToolbarDesiredSize { width: number; height: number }
export interface ViewportToolbarPlan {
  contract: ViewportToolbarContract; inline: number; open: boolean;
  popupWidth: number; popupHeight: number;
  publication?: ViewportToolbarPublication;
  sourceByNativeId?: ReadonlyMap<string,string>; sourceOnlyNativeNodeIds?: ReadonlySet<string>; dpi?: number;
}
interface Review { packet: ViewportToolbarPublication; sourceByNativeId: Map<string,string>; sourceOnlyNativeNodeIds: Set<string>; dpi: number }
const reviews = new WeakMap<ZuiDocument,Review>();
const need = (condition: unknown, message: string): void => { if (!condition) throw new Error('Viewport toolbar: '+message); };
function parents(document: ZuiDocument): Map<string,string> {
  const result=new Map<string,string>();
  for(const [id,node] of Object.entries(document.nodes??{}))for(const child of node.children??[]){
    need(!result.has(child.node),'source has multiply owned nodes');result.set(child.node,id);
  }
  return result;
}
function ownControl(document: ZuiDocument, control: string): string {
  const ids=Object.entries(document.nodes??{}).filter(([,node])=>node.control_id===control).map(([id])=>id);
  need(ids.length===1,'source control is missing or duplicated: '+control);return ids[0]!;
}
/** Only this authored sibling row/popup contract permits runtime reparenting. */
export function viewportToolbarContract(document: ZuiDocument): ViewportToolbarContract | undefined {
  const roots=Object.entries(document.nodes??{}).filter(([,node])=>node.control_id==='UnrealViewportToolbar');
  if(!roots.length)return undefined;
  need(roots.length===1,'source toolbar is duplicated');const [root,source]=roots[0]!;
  const rowControl=source.props?.['toolbar_overflow_row_control'],groupControls=source.props?.['toolbar_overflow_group_controls'];
  need(typeof rowControl==='string'&&Array.isArray(groupControls)&&groupControls.length>0&&groupControls.length<=32,'source overflow contract is incomplete');
  const controls=groupControls as ZuiValue[];
  need(controls.every(value=>typeof value==='string')&&new Set(controls).size===controls.length,'source group order is invalid');
  const row=ownControl(document,rowControl as string),more=ownControl(document,'UnrealViewportToolbarMore'),popup=ownControl(document,'UnrealViewportToolbarOverflow');
  const groups=controls.map(control=>ownControl(document,control as string)),parent=parents(document);
  need(parent.get(row)===root&&parent.get(popup)===root&&parent.get(more)===row,'source More/popup ownership differs');
  need(document.nodes![popup]!.props?.['toolbar_overflow_role']==='toolbar-overflow','source popup role differs');
  need(groups.every(id=>parent.get(id)===row),'original group is outside its source row');
  const originalRowChildren=document.nodes![row]!.children??[];
  need(originalRowChildren.filter(mount=>groups.includes(mount.node)).map(mount=>mount.node).join('\0')===groups.join('\0'),'source row/group ordering differs');
  for(const id of groups)need(originalRowChildren.filter(mount=>mount.node===id&&mount.slot).length===1,'source group needs one original slot');
  return {root,row,more,popup,groups,originalRowChildren};
}
interface SourceStep { sourcePath: string; sourceNodeId: string }
const WORKBENCH_OWNER = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const CURSOR_FIELDS = ['structure','geometry','interaction','viewport','hitTest','theme','diagnostics'];
function identityNeed(condition: unknown, message: string): asserts condition { need(condition,message); }
function record(value: unknown): value is Record<string,unknown> {
  return value!==null && typeof value==='object' && !Array.isArray(value);
}
function canonicalSourcePath(value: string): string {
  const path=value.startsWith('res://ui/editor/') ? 'zircon_editor/assets/'+value.slice('res://'.length) : value;
  if(path.startsWith('zircon_editor/assets/ui/editor/'))
    need(!path.includes('\\') && path.split('/').every(part=>part && part!=='.' && part!=='..'),'invalid editor source alias');
  return path;
}
function invocationSteps(value: unknown): SourceStep[] {
  let decoded=value;
  if(typeof decoded==='string') {
    try { decoded=JSON.parse(decoded); } catch { throw new Error('Viewport toolbar: invalid source toolbar invocation identity'); }
  }
  identityNeed(Array.isArray(decoded) && decoded.length<=64,'invalid source toolbar invocation identity');
  return decoded.map(step=>{
    identityNeed(record(step) && Object.keys(step).length===2 && typeof step['sourcePath']==='string' &&
      step['sourcePath'].length>0 && typeof step['sourceNodeId']==='string' && step['sourceNodeId'].length>0,
      'invalid source toolbar invocation identity');
    return {sourcePath:canonicalSourcePath(step['sourcePath']),sourceNodeId:step['sourceNodeId']};
  });
}
function startsWithSteps(value: SourceStep[], prefix: SourceStep[]): boolean {
  return prefix.length<=value.length && prefix.every((step,index)=>
    step.sourcePath===value[index]!.sourcePath && step.sourceNodeId===value[index]!.sourceNodeId);
}
function sourceKey(identity: Pick<ToolbarSourceNode,'sourcePath'|'sourceNodeId'|'instancePath'>): string {
  return toolbarSourceKey({sourcePath:canonicalSourcePath(identity.sourcePath),sourceNodeId:identity.sourceNodeId,
    instancePath:JSON.stringify(invocationSteps(identity.instancePath))});
}
function nodeSourceKey(node: ZuiNode): string | undefined {
  const sourcePath=node['penpot_review_source_path'],sourceNodeId=node['penpot_review_source_node_id'],instancePath=node['penpot_review_instance_path'];
  return typeof sourcePath==='string'&&typeof sourceNodeId==='string'&&typeof instancePath==='string'
    ? sourceKey({sourcePath,sourceNodeId,instancePath}):undefined;
}
/** Match only complete owner prefixes established by the two actual roots. */
function ownerPrefixes(document: ZuiDocument,evidence: unknown,packet: ViewportToolbarPublication,
  toolbarRoot: string,parentById: Map<string,string>): {review: SourceStep[]; native: SourceStep[]} {
  let id: string|undefined=toolbarRoot;
  const seen=new Set<string>();
  while(id && document.nodes![id]!.control_id!=='WorkbenchWindowRoot') {
    need(!seen.has(id),'cyclic source toolbar owner ancestry');seen.add(id);id=parentById.get(id);
  }
  if(!id)return {review:[],native:[]};
  need(Object.values(document.nodes!).filter(node=>node.control_id==='WorkbenchWindowRoot').length===1,
    'review Workbench owner root is duplicated');
  const root=document.nodes![id]!;
  identityNeed(root.component==='Overlay' && root['penpot_review_source_node_id']==='root' &&
    typeof root['penpot_review_source_path']==='string' && canonicalSourcePath(root['penpot_review_source_path'])===WORKBENCH_OWNER,
    'review Workbench owner root differs');
  const review=invocationSteps(root['penpot_review_instance_path']);
  const inventory=record(evidence)?evidence['arrangedInventoryPublication']:undefined;
  // Existing unwrapped Source cases already share the exact invocation frame.
  if(inventory===undefined && review.length===0)return {review,native:[]};
  const business=record(evidence)?evidence['workbenchStatePublication']:undefined;
  const presentation=record(business)?business['workbenchPresentation']:undefined;
  const fingerprint=record(presentation)?presentation['sourceFingerprint']:undefined;
  identityNeed(record(fingerprint) && typeof fingerprint['sourcePath']==='string' &&
    canonicalSourcePath(fingerprint['sourcePath'])===WORKBENCH_OWNER,'actual business Workbench owner differs');
  identityNeed(record(inventory) && inventory['schema']==='dev.zircon.editor.normal-arranged-inventory' &&
    inventory['version']===1 && inventory['authority']==='normal-retained-runtime-surfaces' &&
    inventory['observed']===true && typeof inventory['complete']==='boolean',
    'actual Workbench owner publication is missing or unsupported');
  const cursor=inventory['presentationCursor'];
  identityNeed(record(cursor) && Object.keys(cursor).length===CURSOR_FIELDS.length &&
    CURSOR_FIELDS.every(key=>cursor[key]===packet.presentationCursor[key as keyof typeof packet.presentationCursor]) &&
    inventory['presentedFrameCount']===packet.presentedFrameCount,'Workbench owner belongs to a different actual frame');
  const surfaces=inventory['surfaces'];
  identityNeed(Array.isArray(surfaces) && surfaces.every(record),'Workbench owner surfaces are missing');
  const owners=surfaces.filter(surface=>surface['scope']==='workbench-window');
  identityNeed(owners.length===1,'actual Workbench owner surface is missing or duplicated');
  const surface=owners[0]!;
  identityNeed(surface['schema']==='dev.zircon.editor.normal-surface-arranged-inventory' && surface['version']===1 &&
    surface['documentId']==='res://ui/editor/unreal/level_editor_window.zui' &&
    surface['coordinateSpace']==='surface-logical' && surface['frameAuthority']==='retained-host-projection' &&
    typeof surface['sourceFrameGeneration']==='number' && Number.isSafeInteger(surface['sourceFrameGeneration']) &&
    surface['sourceFrameGeneration']>=0,'actual Workbench surface owner differs');
  const rows=surface['nodes'];
  identityNeed(Array.isArray(rows) && rows.every(record),'actual Workbench owner rows are missing');
  const roots=rows.filter(row=>row['controlId']==='WorkbenchWindowRoot');
  identityNeed(roots.length===1,'actual Workbench owner root is missing or duplicated');
  const nativeRoot=roots[0]!,rootIds=surface['rootIds'];
  identityNeed(nativeRoot['component']==='Overlay' && nativeRoot['parent']===null &&
    typeof nativeRoot['sourcePath']==='string' && canonicalSourcePath(nativeRoot['sourcePath'])===WORKBENCH_OWNER &&
    nativeRoot['sourceNodeId']==='root' && typeof nativeRoot['nodeId']==='string' &&
    typeof nativeRoot['surfaceNodeId']==='number' && Number.isSafeInteger(nativeRoot['surfaceNodeId']) &&
    nativeRoot['surfaceNodeId']>0 && Array.isArray(rootIds) && rootIds.length===1 && rootIds[0]===nativeRoot['surfaceNodeId'],
    'actual Workbench owner root differs');
  const native=invocationSteps(nativeRoot['instancePath']),projected=nativeRoot['projected'];
  identityNeed(record(projected) && projected['nodeId']===nativeRoot['nodeId'] &&
    projected['surfaceNodeId']===nativeRoot['surfaceNodeId'] && projected['parentId']===null &&
    projected['component']==='Overlay' && projected['controlId']==='WorkbenchWindowRoot' &&
    typeof projected['sourcePath']==='string' && canonicalSourcePath(projected['sourcePath'])===WORKBENCH_OWNER &&
    projected['sourceNodeId']==='root' && JSON.stringify(invocationSteps(projected['instancePath']))===JSON.stringify(native),
    'actual Workbench owner projection differs');
  return {review,native};
}
/** Validate the complete actual Source identities without changing the document or evidence. */
export function viewportToolbarSourceMapping(document: ZuiDocument,evidence: unknown,dpi: number): Review | undefined {
  const contract=viewportToolbarContract(document),packet=readViewportToolbarPublication(evidence);
  if(!packet){need(!contract||evidence===undefined,'actual normal evidence lacks its toolbar publication');return;}
  need(Number.isFinite(dpi)&&dpi>0,'case logical/physical scale is missing');
  need(contract,'observed toolbar has no source contract');
  const canonicalParents=parents(document),prefixes=ownerPrefixes(document,evidence,packet,contract!.root,canonicalParents);
  const nativeKey=(raw: Pick<ToolbarSourceNode,'sourcePath'|'sourceNodeId'|'instancePath'>): string=>{
    const steps=invocationSteps(raw.instancePath);
    need(startsWithSteps(steps,prefixes.native),'published source toolbar invocation is outside its actual owner prefix');
    return sourceKey({...raw,instancePath:JSON.stringify([...prefixes.review,...steps.slice(prefixes.native.length)])});
  };
  const index=new Map<string,string>(),duplicates=new Set<string>();
  for(const [id,node] of Object.entries(document.nodes??{})){
    const key=nodeSourceKey(node);if(!key)continue;if(index.has(key))duplicates.add(key);else index.set(key,id);
  }
  const expected=new Set<string>();
  const visit=(id: string):void=>{
    need(!expected.has(id),'cycle or duplicate in source toolbar');expected.add(id);
    const node=document.nodes![id];need(node,'source toolbar child is missing');
    need(startsWithSteps(invocationSteps(node!['penpot_review_instance_path']),prefixes.review),
      'source toolbar invocation is outside its complete owner prefix');
    for(const child of node!.children??[])visit(child.node);
  };visit(contract!.root);
  const sourceByNativeId=new Map<string,string>(),represented=new Set<string>(),sourceOnlyNativeNodeIds=new Set<string>();
  for(const raw of packet.nodes){
    const key=nativeKey(raw);need(!duplicates.has(key),'duplicate expanded source identity');const id=index.get(key);
    if(!id){
      // The other invocation frame may establish a known outside Source owner.
      // It can reject this packet entry, never supply an alternative match.
      const absolute=sourceKey(raw);
      need(!duplicates.has(absolute),'duplicate expanded source identity');
      need(!index.has(absolute),'published source node is outside its source toolbar');
      // No editable Source node owns this exact native tuple. These additional
      // prepared nodes remain explicitly source-only evidence, never a Board
      // fallback. Every canonical Source descendant is required below.
      sourceOnlyNativeNodeIds.add(raw.nodeId);continue;
    }
    need(expected.has(id),'published source node is outside its source toolbar');
    need(!represented.has(id),'published source node is duplicated');represented.add(id);
    need(document.nodes![id]!.component===raw.component,'source component owner differs');
    need(document.nodes![id]!.control_id===raw.controlId || (!document.nodes![id]!.control_id&&raw.controlId===null),'source control owner differs');
    sourceByNativeId.set(raw.nodeId,id);
  }
  for(const [native,id] of [[packet.rootNodeId,contract!.root],[packet.rowNodeId,contract!.row],[packet.moreNodeId,contract!.more],[packet.popupNodeId,contract!.popup]] as const)
    need(sourceByNativeId.get(native)===id,'published owner is not the source toolbar');
  need(packet.groupNodeIds.map(id=>sourceByNativeId.get(id)).join('\0')===contract!.groups.join('\0'),'published original groups differ from source');
  for(const id of expected)need(represented.has(id),'source semantic node lacks its exact published tuple: '+id);
  for(const raw of packet.nodes){
    const id=sourceByNativeId.get(raw.nodeId);if(!id)continue;
    if(contract!.groups.includes(id)){
      need([contract!.row,contract!.popup].includes(sourceByNativeId.get(raw.parentNodeId??'')??''),'original group has an unrelated published parent');continue;
    }
    const expectedParent=canonicalParents.get(id);
    if(id===contract!.root){
      if(expectedParent){
        const key=nodeSourceKey(document.nodes![expectedParent]!);
        need(key&&raw.parentNodeId!==null&&raw.parentSourcePath!==null&&raw.parentSourceNodeId!==null&&raw.parentInstancePath!==null&&
          nativeKey({sourcePath:raw.parentSourcePath,sourceNodeId:raw.parentSourceNodeId,instancePath:raw.parentInstancePath})===key,'source toolbar root has a different external parent tuple');
      }else need(raw.parentNodeId===null&&raw.parentSourcePath===null&&raw.parentSourceNodeId===null&&raw.parentInstancePath===null,'source toolbar root has an unrelated external parent');
    }else need(expectedParent&&sourceByNativeId.get(raw.parentNodeId??'')===expectedParent,'source semantic node has a different canonical parent: '+id);
  }
  return {packet,sourceByNativeId,sourceOnlyNativeNodeIds,dpi};
}
/** Apply represented known fields to the review clone; source graph/slots/events stay canonical. */
export function deriveViewportToolbarReview(document: ZuiDocument,evidence: unknown,dpi: number): void {
  reviews.delete(document);
  const review=viewportToolbarSourceMapping(document,evidence,dpi);if(!review)return;
  const {packet,sourceByNativeId}=review;
  for(const raw of packet.nodes){
    const id=sourceByNativeId.get(raw.nodeId);if(!id)continue;
    const node=document.nodes![id]!,props: ZuiTable={...node.props},state: ZuiTable={...node.state};
    const values: [string,ZuiValue|null][]=[['selected',raw.state.selected],['checked',raw.state.checked],['popup_open',raw.state.popupOpen],['value',raw.state.value],['text',raw.state.text],['value_text',raw.state.valueText],['disabled',raw.state.disabled]];
    for(const [key,value] of values)if(value!==null){props[key]=value;state[key]=value;}
    node.props=props;node.state=state;
  }
  reviews.set(document,review);
}
/** Allocate original More/popup painter children even when their runtime host is closed. */
export function prepareViewportToolbarProjection(document: ZuiDocument,shapes: ProjectedZuiNode[]): void {
  const contract=viewportToolbarContract(document);if(!contract)return;
  for(const shape of shapes)if(shape.nodeId===contract.more||shape.nodeId===contract.popup)shape.previewHidden=false;
}
/** Exact native prefix rule: one original stretching spacer plus, when needed, More. */
export function viewportToolbarInlinePrefix(widths: readonly number[],available: number,more: number,gap: number): number {
  need([available,more,gap,...widths].every(Number.isFinite)&&available>=0&&more>0&&gap>=0&&widths.every(value=>value>0),'intrinsic fit inputs are invalid');
  if(widths.reduce((sum,value)=>sum+value,0)+gap*widths.length<=available)return widths.length;
  let used=more+gap,count=0;for(const width of widths){const next=used+width+gap;if(next>available)break;used=next;count++;}return count;
}
export function planViewportToolbar(document: ZuiDocument,projection: ZuiAssetProjection,desired: ReadonlyMap<string,ToolbarDesiredSize>,rowWidth: number,heightBudget: number): ViewportToolbarPlan | undefined {
  const contract=viewportToolbarContract(document);if(!contract)return undefined;
  const byId=new Map(projection.shapes.map(shape=>[shape.nodeId,shape])),row=byId.get(contract.row),popup=byId.get(contract.popup);
  need(row?.container.kind==='flex'&&row.container.direction==='row'&&popup?.container.kind==='flex'&&popup.container.direction==='column','source toolbar container layout differs');
  const review=reviews.get(document),nativeNodes=review?new Map(review.packet.nodes.map(node=>[node.nodeId,node])):undefined;
  const nativeBySource=review?new Map([...review.sourceByNativeId].map(([native,id])=>[id,nativeNodes!.get(native)!])):undefined;
  const sizes=contract.groups.map(id=>{
    // Original Native group measurement excludes its original slot padding.
    const observed=nativeBySource?.get(id)?.desiredSize;
    if(review)need(observed?.coordinateSpace==='logical-pixels','observed original group lacks logical desired measurement');
    const size=observed??desired.get(id),padding=byId.get(id)?.slotPadding;
    need(size&&Number.isFinite(size.width)&&size.width>0&&Number.isFinite(size.height)&&size.height>0,'source group lacks actual intrinsic measurement: '+id);
    return {width:size!.width+(padding?.left??0)+(padding?.right??0),height:size!.height+(padding?.top??0)+(padding?.bottom??0)};
  });
  const available=Math.max(0,rowWidth-row!.container.padding.left-row!.container.padding.right),gap=row!.container.columnGap;
  let inline: number,open: boolean;
  if(review){
    const first=review.packet.groupNodeIds.findIndex(id=>nativeNodes!.get(id)!.parentNodeId===review.packet.popupNodeId);
    inline=first<0?contract.groups.length:first;open=nativeNodes!.get(review.packet.popupNodeId)!.state.popupOpen!;
  }else{
    const more=desired.get(contract.more);need(more&&more.width>0,'source More intrinsic measurement is absent');
    inline=viewportToolbarInlinePrefix(sizes.map(size=>size.width),available,more!.width,gap);
    open=inline<contract.groups.length&&(document.nodes![contract.popup]!.state?.['popup_open']??document.nodes![contract.popup]!.props?.['popup_open'])===true;
  }
  const suffix=sizes.slice(inline),padding=popup!.container.padding;
  let popupWidth=suffix.length?Math.min(available,Math.max(...suffix.map(size=>size.width))+padding.left+padding.right):0;
  let popupHeight=suffix.length?Math.min(Math.max(0,heightBudget),suffix.reduce((sum,size)=>sum+size.height,0)+popup!.container.rowGap*Math.max(0,suffix.length-1)+padding.top+padding.bottom):0;
  if(review){const actual=nativeNodes!.get(review.packet.popupNodeId)!;popupWidth=actual.bounds.width/review.dpi;popupHeight=actual.bounds.height/review.dpi;}
  return {contract,inline,open,popupWidth,popupHeight,...(review?{publication:review.packet,sourceByNativeId:review.sourceByNativeId,sourceOnlyNativeNodeIds:review.sourceOnlyNativeNodeIds,dpi:review.dpi}:{})};
}
/** A runtime layout clone only; exported source keeps original children and slot records. */
export function viewportToolbarRuntimeDocument(document: ZuiDocument,plan: ViewportToolbarPlan): ZuiDocument {
  const {contract,inline}=plan,nodes=document.nodes!,suffix=contract.groups.slice(inline),hidden=new Set(suffix);
  const rowChildren=contract.originalRowChildren.filter(mount=>!hidden.has(mount.node));
  const popupChildren=suffix.map(id=>{const original=contract.originalRowChildren.find(mount=>mount.node===id)!;return {...original,slot:{...original.slot,kind:'ScrollableSlot'}};});
  const originalPopup=nodes[contract.popup]!,layout={...originalPopup.layout};
  for(const [axis,extent] of [['width',plan.popupWidth],['height',plan.popupHeight]] as const){
    const old=layout[axis];layout[axis]={...(old&&typeof old==='object'&&!Array.isArray(old)?old as ZuiTable:{}),stretch:'Fixed',preferred:extent};
  }
  const result: ZuiDocument={...document,nodes:{...nodes,[contract.row]:{...nodes[contract.row]!,children:rowChildren},[contract.popup]:{...originalPopup,layout,children:popupChildren}}};
  copyFontResources(document,result);
  return result;
}
/** Export only restores these original direct children from their two allowed hosts. */
export function canonicalViewportToolbarParent(contract: ViewportToolbarContract|undefined,nodeId: string,parent: string|null): string|null {
  if(!contract||!contract.groups.includes(nodeId))return parent;
  need(parent===contract.row||parent===contract.popup,'original toolbar group moved outside its authored row/popup');return contract.row;
}

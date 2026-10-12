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
function sourceKey(node: ZuiNode): string | undefined {
  const sourcePath=node['penpot_review_source_path'],sourceNodeId=node['penpot_review_source_node_id'],instancePath=node['penpot_review_instance_path'];
  return typeof sourcePath==='string'&&typeof sourceNodeId==='string'&&typeof instancePath==='string'
    ? toolbarSourceKey({sourcePath,sourceNodeId,instancePath}):undefined;
}
/** Apply represented known fields to the review clone; source graph/slots/events stay canonical. */
export function deriveViewportToolbarReview(document: ZuiDocument,evidence: unknown,dpi: number): void {
  reviews.delete(document);const contract=viewportToolbarContract(document),packet=readViewportToolbarPublication(evidence);
  if(!packet){need(!contract||evidence===undefined,'actual normal evidence lacks its toolbar publication');return;}
  need(Number.isFinite(dpi)&&dpi>0,'case logical/physical scale is missing');
  need(contract,'observed toolbar has no source contract');
  const index=new Map<string,string>();
  for(const [id,node] of Object.entries(document.nodes??{})){
    const key=sourceKey(node);if(!key)continue;need(!index.has(key),'duplicate expanded source identity');index.set(key,id);
  }
  const expected=new Set<string>();
  const visit=(id: string):void=>{
    need(!expected.has(id),'cycle or duplicate in source toolbar');expected.add(id);
    const node=document.nodes![id];need(node,'source toolbar child is missing');
    for(const child of node!.children??[])visit(child.node);
  };visit(contract!.root);
  const sourceByNativeId=new Map<string,string>(),represented=new Set<string>(),sourceOnlyNativeNodeIds=new Set<string>();
  for(const raw of packet.nodes){
    const id=index.get(toolbarSourceKey(raw));
    if(!id){
      // No editable Source node owns this exact native tuple. These additional
      // prepared nodes remain explicitly source-only evidence, never a Board
      // fallback. Every canonical Source descendant is required below.
      sourceOnlyNativeNodeIds.add(raw.nodeId);continue;
    }
    need(expected.has(id),'published source node is outside its source toolbar');
    need(!represented.has(id),'published source node is duplicated');represented.add(id);
    need(document.nodes![id]!.control_id===raw.controlId || (!document.nodes![id]!.control_id&&raw.controlId===null),'source control owner differs');
    sourceByNativeId.set(raw.nodeId,id);
  }
  for(const [native,id] of [[packet.rootNodeId,contract!.root],[packet.rowNodeId,contract!.row],[packet.moreNodeId,contract!.more],[packet.popupNodeId,contract!.popup]] as const)
    need(sourceByNativeId.get(native)===id,'published owner is not the source toolbar');
  need(packet.groupNodeIds.map(id=>sourceByNativeId.get(id)).join('\0')===contract!.groups.join('\0'),'published original groups differ from source');
  for(const id of expected)need(represented.has(id),'source semantic node lacks its exact published tuple: '+id);
  const canonicalParents=parents(document);
  for(const raw of packet.nodes){
    const id=sourceByNativeId.get(raw.nodeId);if(!id)continue;
    if(contract!.groups.includes(id)){
      need([contract!.row,contract!.popup].includes(sourceByNativeId.get(raw.parentNodeId??'')??''),'original group has an unrelated published parent');continue;
    }
    const expectedParent=canonicalParents.get(id);
    if(id===contract!.root){
      if(expectedParent){
        const key=sourceKey(document.nodes![expectedParent]!);
        need(key&&raw.parentNodeId!==null&&raw.parentSourcePath!==null&&raw.parentSourceNodeId!==null&&raw.parentInstancePath!==null&&
          toolbarSourceKey({sourcePath:raw.parentSourcePath,sourceNodeId:raw.parentSourceNodeId,instancePath:raw.parentInstancePath})===key,'source toolbar root has a different external parent tuple');
      }else need(raw.parentNodeId===null&&raw.parentSourcePath===null&&raw.parentSourceNodeId===null&&raw.parentInstancePath===null,'source toolbar root has an unrelated external parent');
    }else need(expectedParent&&sourceByNativeId.get(raw.parentNodeId??'')===expectedParent,'source semantic node has a different canonical parent: '+id);
  }
  for(const raw of packet.nodes){
    const id=sourceByNativeId.get(raw.nodeId);if(!id)continue;
    const node=document.nodes![id]!,props: ZuiTable={...node.props},state: ZuiTable={...node.state};
    const values: [string,ZuiValue|null][]=[['selected',raw.state.selected],['checked',raw.state.checked],['popup_open',raw.state.popupOpen],['value',raw.state.value],['text',raw.state.text],['value_text',raw.state.valueText],['disabled',raw.state.disabled]];
    for(const [key,value] of values)if(value!==null){props[key]=value;state[key]=value;}
    node.props=props;node.state=state;
  }
  reviews.set(document,{packet,sourceByNativeId,sourceOnlyNativeNodeIds,dpi});
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

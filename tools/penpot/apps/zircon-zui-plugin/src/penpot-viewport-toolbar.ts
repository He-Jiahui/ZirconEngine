import { applySlotPadding,captureSlotPadding } from './penpot-slot-padding';
import type { Board } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import type { ZuiAssetProjection,ProjectionPadding } from './bridge/penpot-projection-model';
import type { ToolbarDesiredSize, ViewportToolbarPlan } from './bridge/zui-viewport-toolbar';
import { planViewportToolbar, viewportToolbarRuntimeDocument,viewportToolbarContract } from './bridge/zui-viewport-toolbar';
import { allocateNativeLayoutSize } from './penpot-layout-allocation';
import { resolveDesignNumber } from './bridge/zui-prefab-system';
import { ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID, ZUI_METADATA_ROLE, ZUI_ROLE_NODE } from './metadata';
const PADDING_ALLOCATION='viewport-toolbar-slot-padding-allocation';
const ZERO_PADDING:ProjectionPadding={left:0,right:0,top:0,bottom:0};
interface PaddingAllocation {
  version:1;group:string;row:string;popup:string;
  baseline:ProjectionPadding;authored:ProjectionPadding;runtime:ProjectionPadding;
}
function equalPadding(a:ProjectionPadding|undefined,b:ProjectionPadding|undefined):boolean {
  return Boolean(a&&b&&(['left','right','top','bottom'] as const).every(side=>a[side]===b[side]));
}
function paddingAllocation(current:Board):PaddingAllocation|undefined {
  const encoded=current.getSharedPluginData(ZUI_METADATA_NAMESPACE,PADDING_ALLOCATION);if(!encoded)return undefined;
  const raw=JSON.parse(encoded) as PaddingAllocation;
  if(!raw||Object.keys(raw).sort().join(',')!=='authored,baseline,group,popup,row,runtime,version'||raw.version!==1||
    [raw.group,raw.row,raw.popup].some(value=>typeof value!=='string'||!value)||
    [raw.baseline,raw.authored,raw.runtime].some(value=>!value||Object.keys(value).sort().join(',')!=='bottom,left,right,top'||Object.values(value).some(axis=>typeof axis!=='number'||!Number.isFinite(axis)||axis<0))||!equalPadding(raw.runtime,ZERO_PADDING))
    throw new Error('Viewport toolbar: invalid Source slot padding allocation');
  return raw;
}
/** Runtime ScrollableBox ignores slot padding; keep the editable Source values separately. */
export function captureViewportToolbarSlotPadding(document:ZuiDocument,current:Board,captured:ProjectionPadding|undefined):ProjectionPadding|undefined {
  const allocated=paddingAllocation(current);
  if(!allocated){
    const id=current.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_NODE_ID);
    const parent=current.parent?.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_NODE_ID);
    if(parent&&document.nodes?.[parent]?.control_id==='UnrealViewportToolbarOverflow'&&viewportToolbarContract(document)?.groups.includes(id))
      throw new Error('Viewport toolbar: original Source padding allocation is missing');
    return captured;
  }
  const contract=viewportToolbarContract(document),id=current.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_NODE_ID);
  if(!contract||!contract.groups.includes(id)||allocated.group!==id||allocated.row!==contract.row||allocated.popup!==contract.popup||
    ![contract.row,contract.popup].includes(current.parent?.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_NODE_ID)??''))
    throw new Error('Viewport toolbar: Source padding is outside its authored group hosts');
  if(!captured)throw new Error('Viewport toolbar: original slot padding capture is missing');
  return Object.fromEntries((['left','right','top','bottom'] as const).map(side=>[side,captured[side]===allocated.runtime[side]?allocated.authored[side]:captured[side]])) as unknown as ProjectionPadding;
}
function preserveGroupPadding(current:Board,id:string,row:string,popup:string,toPopup:boolean,baseline:ProjectionPadding):void {
  const previous=paddingAllocation(current);
  if(!previous&&current.parent?.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_NODE_ID)===popup)
    throw new Error('Viewport toolbar: original Source padding allocation is missing');
  const captured=captureSlotPadding(current,baseline);
  if(!captured)throw new Error('Viewport toolbar: Source group padding baseline is missing');
  if(previous&&(previous.group!==id||previous.row!==row||previous.popup!==popup))throw new Error('Viewport toolbar: Source padding allocation owner changed');
  const authored=previous&&!equalPadding(previous.baseline,baseline)?baseline:
    previous?Object.fromEntries((['left','right','top','bottom'] as const).map(side=>[side,captured[side]===previous.runtime[side]?previous.authored[side]:captured[side]])) as unknown as ProjectionPadding:captured;
  if(toPopup){
    const record:PaddingAllocation={version:1,group:id,row,popup,baseline:{...baseline},authored:{...authored},runtime:{...ZERO_PADDING}};
    current.setSharedPluginData(ZUI_METADATA_NAMESPACE,PADDING_ALLOCATION,JSON.stringify(record));
  }else if(previous){
    applySlotPadding(current,authored);current.setSharedPluginData(ZUI_METADATA_NAMESPACE,PADDING_ALLOCATION,'');
  }
}
export interface PreparedToolbarLayout {
  plan: ViewportToolbarPlan; document: ZuiDocument; boards: Map<string,Board>;
  sourceNodes: Map<string,NonNullable<ViewportToolbarPlan['publication']>['nodes'][number]>;
}
function semanticBoards(asset: Board): Map<string,Board> {
  const result=new Map<string,Board>();
  const visit=(board: Board):void=>{
    if(board.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_ROLE)===ZUI_ROLE_NODE){
      const id=board.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_NODE_ID);
      if(!id||result.has(id))throw new Error('Viewport toolbar: missing/duplicated semantic board');result.set(id,board);
    }
    for(const child of board.children)if(child.type==='board')visit(child);
  };visit(asset);return result;
}
function board(boards: Map<string,Board>,id: string): Board {
  const result=boards.get(id);if(!result)throw new Error('Viewport toolbar: original semantic board missing '+id);return result;
}
/** Preserve the semantic sibling order, including the original stretching spacer. */
function restoreSemanticOrder(host: Board,ordered: readonly Board[]): void {
  const semantic=()=>host.children.filter(child=>child.type==='board'&&child.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_ROLE)===ZUI_ROLE_NODE);
  const same=()=>{const actual=semantic();return actual.length===ordered.length&&actual.every((child,index)=>child.id===ordered[index]?.id);};
  if(same())return;
  // appendChild moves the existing object. Nonsemantic painter children retain
  // their order, and all Source metadata, slots and event routes stay attached.
  for(const child of ordered)host.appendChild(child);
  if(!same())throw new Error('Viewport toolbar: original semantic board order differs');
}
/** Move existing contract children. Painter descendants and source metadata are never cloned. */
export function prepareViewportToolbarLayout(asset: Board,document: ZuiDocument,projection: ZuiAssetProjection,desired: ReadonlyMap<string,ToolbarDesiredSize>): PreparedToolbarLayout | undefined {
  const boards=semanticBoards(asset);
  const rowShape=projection.shapes.find(shape=>shape.controlId==='UnrealViewportToolbarRow');if(!rowShape)return undefined;
  const row=board(boards,rowShape.nodeId),rootShape=projection.shapes.find(shape=>shape.controlId==='UnrealViewportToolbar');
  if(!rootShape)throw new Error('Viewport toolbar: row has no authored owner');
  const root=board(boards,rootShape.nodeId);
  const viewportShape=projection.shapes.find(shape=>shape.controlId==='UnrealSceneViewViewport');
  if(!viewportShape)throw new Error('Viewport toolbar: source viewport boundary is missing');
  const viewport=board(boards,viewportShape.nodeId);
  const popupShape=projection.shapes.find(shape=>shape.controlId==='UnrealViewportToolbarOverflow');
  const position=document.nodes?.[popupShape?.nodeId??'']?.layout?.['position'];
  const offset=position&&typeof position==='object'&&!Array.isArray(position)
    ? resolveDesignNumber(document,(position as Record<string,unknown>)['y']):undefined;
  if(typeof offset!=='number'||!Number.isFinite(offset))throw new Error('Viewport toolbar: source popup position is missing');
  const budget=Math.max(0,viewport.y+viewport.height-(root.y+root.height)-offset);
  const plan=planViewportToolbar(document,projection,desired,row.width,budget);if(!plan)return undefined;
  if(!plan.publication)for(const item of boards.values()){
    const original=item.getSharedPluginData(ZUI_METADATA_NAMESPACE,'viewport-toolbar-original-absolute');
    if(original&&item.layoutChild){
      if(original!=='true'&&original!=='false')throw new Error('Viewport toolbar: invalid original child placement');
      item.layoutChild.absolute=original==='true';
      item.setSharedPluginData(ZUI_METADATA_NAMESPACE,'viewport-toolbar-original-absolute','');
    }
  }
  const popup=board(boards,plan.contract.popup),more=board(boards,plan.contract.more);
  const targetFor=new Map(plan.contract.groups.map((id,index)=>[id,index<plan.inline?row:popup]));
  for(const [id,target] of targetFor){
    const child=board(boards,id);
    if(child.parent?.id!==row.id&&child.parent?.id!==popup.id)throw new Error('Viewport toolbar: original board escaped authored hosts '+id);
    preserveGroupPadding(child,id,plan.contract.row,plan.contract.popup,target===popup,projection.shapes.find(shape=>shape.nodeId===id)?.slotPadding??ZERO_PADDING);
    if(child.parent?.id!==target.id)target.appendChild(child);
    child.hidden=false;
    if(child.layoutChild)child.layoutChild.absolute=false;
  }
  const suffix=new Set(plan.contract.groups.slice(plan.inline));
  restoreSemanticOrder(row,plan.contract.originalRowChildren.filter(mount=>!suffix.has(mount.node)).map(mount=>board(boards,mount.node)));
  restoreSemanticOrder(popup,plan.contract.groups.slice(plan.inline).map(id=>board(boards,id)));
  more.hidden=plan.inline===plan.contract.groups.length;popup.hidden=!plan.open;
  allocateNativeLayoutSize(popup,plan.popupWidth,plan.popupHeight,'anchored');
  const sourceNodes=new Map<string,NonNullable<ViewportToolbarPlan['publication']>['nodes'][number]>();
  if(plan.publication)for(const raw of plan.publication.nodes){const id=plan.sourceByNativeId!.get(raw.nodeId);if(id)sourceNodes.set(id,raw);}
  return {plan,boards,sourceNodes,document:viewportToolbarRuntimeDocument(document,plan)};
}
/** Called before this board's own paint/text layout; evidence is already immutable and same-present. */
export function applyPublishedViewportToolbarBoard(asset: Board,current: Board,layout: PreparedToolbarLayout|undefined): void {
  if(!layout?.plan.publication)return;
  const id=current.getSharedPluginData(ZUI_METADATA_NAMESPACE,ZUI_METADATA_NODE_ID),raw=layout.sourceNodes.get(id);if(!raw)return;
  current.hidden=!raw.visible;
  if(!raw.visible)return;
  const dpi=layout.plan.dpi!,bounds=raw.bounds;
  const parent=current.parent,flow=parent?.type==='board'&&Boolean(parent.flex??parent.grid);
  allocateNativeLayoutSize(current,bounds.width/dpi,bounds.height/dpi,flow?'flow':'anchored');
  current.x=asset.x+bounds.x/dpi;current.y=asset.y+bounds.y/dpi;
  // A normal review observes this immutable native allocation. Designer edits
  // remain recorded separately by the existing native-layout-allocation owner.
  if(current.layoutChild){
    if(!current.getSharedPluginData(ZUI_METADATA_NAMESPACE,'viewport-toolbar-original-absolute'))
      current.setSharedPluginData(ZUI_METADATA_NAMESPACE,'viewport-toolbar-original-absolute',String(current.layoutChild.absolute === true));
    current.layoutChild.absolute=true;
  }
  if(id===layout.plan.contract.popup&&raw.scroll){
    current.setSharedPluginData(ZUI_METADATA_NAMESPACE,'viewport-toolbar-observed-scroll',JSON.stringify(raw.scroll));
  }
}

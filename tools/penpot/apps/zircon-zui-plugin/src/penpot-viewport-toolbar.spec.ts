import { encodeZuiMetadata,decodeZuiMetadata } from './bridge/zui-document';
import { describe,it,expect } from 'vitest';
import type { Board } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import type { ZuiAssetProjection,ProjectedZuiNode,ProjectionContainer } from './bridge/penpot-projection-model';
import { prepareViewportToolbarLayout,applyPublishedViewportToolbarBoard,captureViewportToolbarSlotPadding } from './penpot-viewport-toolbar';
import { applyNativeScrollContentLayout } from './penpot-scroll-content-layout';
import { applySlotPadding,captureSlotPadding } from './penpot-slot-padding';
import type { ToolbarSourceNode,ViewportToolbarPublication } from './bridge/zui-viewport-toolbar-publication';
import { ZUI_METADATA_NAMESPACE as NS,ZUI_METADATA_ROLE as ROLE,ZUI_METADATA_NODE_ID as NODE,ZUI_ROLE_NODE } from './metadata';
interface Fake {
  id:string;type:'board';children:Fake[];parent:Fake|null;width:number;height:number;x:number;y:number;hidden:boolean;
  layoutChild:{absolute:boolean;horizontalSizing:'fix'|'fill';verticalSizing:'fix'|'fill';topMargin:number;rightMargin:number;bottomMargin:number;leftMargin:number};flex?:object;grid?:object;data:Map<string,string>;
  getSharedPluginData(ns:string,key:string):string;setSharedPluginData(ns:string,key:string,value:string):void;
  appendChild(child:Fake):void;resize(width:number,height:number):void;
}
function create(id:string,width=200,height=28): Fake {
  const item: Fake={id,type:'board',children:[],parent:null,width,height,x:0,y:0,hidden:false,layoutChild:{absolute:false,horizontalSizing:'fill',verticalSizing:'fill',topMargin:0,rightMargin:0,bottomMargin:0,leftMargin:0},data:new Map(),
    getSharedPluginData(ns,key){return this.data.get(ns+':'+key)??'';},setSharedPluginData(ns,key,value){this.data.set(ns+':'+key,value);},
    appendChild(child){if(child.parent)child.parent.children=child.parent.children.filter(old=>old!==child);this.children.push(child);child.parent=this;},
    resize(w,h){if(!Number.isFinite(w)||!Number.isFinite(h)||w<=0||h<=0)throw new Error('resize requires finite positive dimensions');this.width=w;this.height=h;}};
  item.setSharedPluginData(NS,ROLE,ZUI_ROLE_NODE);item.setSharedPluginData(NS,NODE,id);return item;
}
function setup(){
  const nodes: NonNullable<ZuiDocument['nodes']>={
    main:{component:'Overlay',children:[{node:'toolbar'},{node:'viewport'}]},
    toolbar:{component:'Overlay',control_id:'UnrealViewportToolbar',props:{toolbar_overflow_row_control:'UnrealViewportToolbarRow',toolbar_overflow_group_controls:['G1','G2','G3']},children:[{node:'row'},{node:'popup'}]},
    row:{component:'HorizontalGroup',control_id:'UnrealViewportToolbarRow',children:['g1','fill','g2','g3','more'].map(node=>({node,slot:{kind:'LinearSlot',linear_size:{rule:node==='fill'?'Stretch':'Auto'}}}))},
    popup:{component:'ScrollableBox',control_id:'UnrealViewportToolbarOverflow',props:{toolbar_overflow_role:'toolbar-overflow',popup_open:false},layout:{position:{x:0,y:4}},children:[]},
    more:{component:'IconButton',control_id:'UnrealViewportToolbarMore',events:[{id:'original-more',event:'Click',route:'original-more'}]},
    fill:{component:'Space'},viewport:{component:'Media',control_id:'UnrealSceneViewViewport'},
    g1:{component:'HorizontalGroup',control_id:'G1'},g2:{component:'HorizontalGroup',control_id:'G2'},g3:{component:'HorizontalGroup',control_id:'G3'},
  };
  const document: ZuiDocument={asset:{kind:'view',id:'ue',version:2},root:{node:'main'},nodes};
  const boards=new Map(Object.keys(nodes).map(id=>[id,create(id)]));
  for(const [id,node] of Object.entries(nodes))for(const child of node.children??[])boards.get(id)!.appendChild(boards.get(child.node)!);
  const asset=create('asset',200,200);asset.setSharedPluginData(NS,ROLE,'asset');asset.appendChild(boards.get('main')!);
  boards.get('row')!.width=85;boards.get('viewport')!.height=200;
  const container: ProjectionContainer={kind:'flex',direction:'row',wrap:false,gap:4,rowGap:4,columnGap:4,columns:1,rows:1,padding:{top:0,right:0,bottom:0,left:0},alignItems:'stretch',justifyContent:'start',clip:true};
  const shapes=Object.entries(nodes).map(([nodeId,node])=>({nodeId,controlId:node.control_id,previewHidden:false,container:{...container,direction:nodeId==='popup'?'column':'row',...(nodeId==='popup'?{padding:{top:4,right:4,bottom:4,left:4}}:{})}})) as ProjectedZuiNode[];
  const projection={assetId:'ue',displayName:'ue',rootNodes:[],detachedNodes:[],shapes,diagnostics:[]} as ZuiAssetProjection;
  const desired=new Map(['g1','g2','g3'].map(id=>[id,{width:50,height:28}]));desired.set('more',{width:28,height:28});
  return {asset,boards,document,projection,desired};
}
describe('production original Board toolbar reflow',()=>{
  it('keeps zero extent closed popups hidden through wide narrow wide reflow without resizing or cloning Source boards',()=>{
    const fixture=setup(),row=fixture.boards.get('row')!,popup=fixture.boards.get('popup')!;
    fixture.document.nodes!.popup!.props!['popup_open']=true;
    const before=JSON.stringify(fixture.document),originalBoards=new Map(fixture.boards);
    const originalChildren=new Map([...originalBoards].map(([id,current])=>[id,[...current.children]] as const));
    const initialSize=[popup.width,popup.height];
    row.width=200;
    const closed=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    expect([closed.plan.inline,closed.plan.open,closed.plan.popupWidth,closed.plan.popupHeight]).toEqual([3,false,0,0]);
    expect(popup.hidden).toBe(true);expect([popup.width,popup.height]).toEqual(initialSize);
    expect(JSON.stringify(fixture.document)).toBe(before);
    row.width=85;
    const narrow=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    expect([narrow.plan.inline,narrow.plan.open,narrow.plan.popupWidth,narrow.plan.popupHeight]).toEqual([0,true,58,100]);
    expect(popup.hidden).toBe(false);expect([popup.width,popup.height]).toEqual([58,100]);
    expect(popup.children).toHaveLength(3);
    ['g1','g2','g3'].forEach((id,index)=>expect(popup.children[index]).toBe(originalBoards.get(id)));
    expect(JSON.stringify(fixture.document)).toBe(before);
    row.width=200;
    const wide=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    expect([wide.plan.inline,wide.plan.open,wide.plan.popupWidth,wide.plan.popupHeight]).toEqual([3,false,0,0]);
    expect(popup.hidden).toBe(true);expect([popup.width,popup.height]).toEqual([58,100]);
    expect(fixture.boards.get('more')!.hidden).toBe(true);
    for(const layout of [closed,narrow,wide])for(const [id,current] of originalBoards)expect(layout.boards.get(id)).toBe(current);
    for(const [id,children] of originalChildren){
      const current=originalBoards.get(id)!;expect(current.children).toHaveLength(children.length);
      children.forEach((child,index)=>expect(current.children[index]).toBe(child));
    }
    expect(JSON.stringify(fixture.document)).toBe(before);
  });

  it('rejects an open popup with zero height before moving original Source boards',()=>{
    const fixture=setup(),row=fixture.boards.get('row')!,children=[...row.children];
    fixture.document.nodes!.popup!.props!['popup_open']=true;
    fixture.boards.get('viewport')!.height=28;
    const before=JSON.stringify(fixture.document);
    expect(()=>prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)).toThrow(/open popup requires finite positive dimensions \(width=58, height=0\)/);
    expect(row.children).toHaveLength(children.length);
    children.forEach((child,index)=>expect(row.children[index]).toBe(child));
    expect(fixture.boards.get('popup')!.children).toEqual([]);
    expect(JSON.stringify(fixture.document)).toBe(before);
  });

  it('moves original boards on measured resize and restores their canonical source parents without cloning',()=>{
    const fixture=setup(),before=JSON.stringify(fixture.document),identities=['g1','g2','g3'].map(id=>fixture.boards.get(id)!);
    const closed=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    expect(closed.plan.inline).toBe(0);expect(fixture.boards.get('popup')!.hidden).toBe(true);
    expect(identities.every(item=>item.parent===fixture.boards.get('popup'))).toBe(true);
    expect(fixture.boards.get('row')!.children.map(item=>item.id)).toEqual(['fill','more']);
    expect(fixture.boards.get('popup')!.children.map(item=>item.id)).toEqual(['g1','g2','g3']);
    fixture.boards.get('row')!.width=200;
    const wide=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    expect(wide.plan.inline).toBe(3);expect(fixture.boards.get('more')!.hidden).toBe(true);
    expect(identities.every(item=>item.parent===fixture.boards.get('row'))).toBe(true);
    expect(fixture.boards.get('row')!.children.map(item=>item.id)).toEqual(['g1','fill','g2','g3','more']);
    expect(fixture.boards.get('popup')!.children).toEqual([]);
    expect(JSON.stringify(fixture.document)).toBe(before);
    const walk=(item:Fake):number=>1+item.children.reduce((sum,child)=>sum+walk(child),0);
    expect(walk(fixture.asset)).toBe(fixture.boards.size+1);
  });
  it('keeps open popup operations as original editable boards with intrinsic main extent',()=>{
    const fixture=setup();fixture.document.nodes!.popup!.props!['popup_open']=true;
    const actual=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    expect(fixture.boards.get('popup')!.hidden).toBe(false);
    expect(actual.document.nodes!.popup!.children!.map(mount=>mount.node)).toEqual(['g1','g2','g3']);
    expect(actual.plan.popupWidth).toBe(58);expect(actual.plan.popupHeight).toBe(100);
    expect(fixture.boards.get('more')!.data.get(NS+':'+NODE)).toBe('more');
  });
  it('applies physical allocation once using real flex ownership and restores source placement after release',()=>{
    const fixture=setup();fixture.asset.x=100;fixture.asset.y=200;fixture.boards.get('row')!.flex={};
    const layout=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    const raw:ToolbarSourceNode={nodeId:'more',parentNodeId:'row',sourcePath:'res://viewport.zui',sourceNodeId:'more',instancePath:'[]',parentSourcePath:'res://viewport.zui',parentSourceNodeId:'row',parentInstancePath:'[]',controlId:'UnrealViewportToolbarMore',component:'IconButton',visible:true,bounds:{x:30,y:45,width:42,height:30},clip:{x:0,y:0,width:300,height:300},state:{selected:false,checked:false,popupOpen:null,value:null,text:null,valueText:null,disabled:false},scroll:null};
    const cursor={structure:1,geometry:1,interaction:1,viewport:1,hitTest:1,theme:1,diagnostics:1};
    // This unit exercises the allocation consumer. Reader admission and the
    // source-owner graph have their separate complete-packet contract specs.
    const publication:ViewportToolbarPublication={schema:'dev.zircon.editor.viewport-toolbar-publication',version:1,authority:'normal-prepared-source-surface',observed:true,coordinateSpace:'physical-pixels',surfaceGeneration:1,surfaceTreeId:'allocation-fixture',rootNodeId:'toolbar',rowNodeId:'row',moreNodeId:'more',popupNodeId:'popup',groupNodeIds:['g1','g2','g3'],nodes:[raw],presentationCursor:cursor,presentedFrameCount:1};
    const observed={...layout,plan:{...layout.plan,publication,dpi:1.5},sourceNodes:new Map([['more',raw]])},more=fixture.boards.get('more')!;
    applyPublishedViewportToolbarBoard(fixture.asset as unknown as Board,more as unknown as Board,observed);
    expect([more.x,more.y,more.width,more.height]).toEqual([120,230,28,20]);
    expect(JSON.parse(more.getSharedPluginData(NS,'native-layout-allocation'))).toEqual({width:28,height:20,parent:'row'});
    expect(more.layoutChild.absolute).toBe(true);
    expect(more.getSharedPluginData(NS,'viewport-toolbar-original-absolute')).toBe('false');
    prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired);
    expect(more.layoutChild.absolute).toBe(false);
    expect(more.getSharedPluginData(NS,'viewport-toolbar-original-absolute')).toBe('');
  });
  it('keeps real ScrollableBox zero margins separate from asymmetric Source padding through capture and return to the row',()=>{
    const fixture=setup(),padding={left:2,right:6,top:3,bottom:5},group=fixture.boards.get('g2')!;
    fixture.document.nodes!.g2!.layout={width:{stretch:'Fixed'},height:{stretch:'Fixed'}};
    const mount=fixture.document.nodes!.row!.children!.find(mount=>mount.node==='g2')!;
    mount.slot!['layout']={padding:{...padding}};
    fixture.projection.shapes.find(shape=>shape.nodeId==='g2')!.slotPadding=padding;
    applySlotPadding(group as unknown as Board,padding);
    const sourceBefore=JSON.stringify(fixture.document);
    const layout=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    const popup=fixture.boards.get('popup')!,container=fixture.projection.shapes.find(shape=>shape.nodeId==='popup')!.container;
    popup.setSharedPluginData(NS,'container-kind','scroll');
    applyNativeScrollContentLayout(popup as unknown as Board,layout.document,container,fixture.desired);
    const actual=captureSlotPadding(group as unknown as Board,padding);
    expect(actual).toEqual({left:0,right:0,top:0,bottom:0});
    expect(captureViewportToolbarSlotPadding(fixture.document,group as unknown as Board,actual)).toEqual(padding);
    expect(group.height).toBe(28);
    // A real edit to one runtime margin preserves the untouched Source sides.
    group.layoutChild.rightMargin=9;
    const edited=captureViewportToolbarSlotPadding(fixture.document,group as unknown as Board,captureSlotPadding(group as unknown as Board,padding))!;
    expect(edited).toEqual({...padding,right:9});
    group.layoutChild.rightMargin=0;
    fixture.boards.get('row')!.width=200;
    const wide=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    expect(wide.plan.inline).toBe(3);
    expect(captureSlotPadding(group as unknown as Board,padding)).toEqual(padding);
    expect(fixture.boards.get('row')!.children.map(item=>item.id)).toEqual(['g1','fill','g2','g3','more']);
    expect(JSON.stringify(fixture.document)).toBe(sourceBefore);
  });
  it('consumes actual scrolled physical child bounds once and retains the exact same-frame scroll offset',()=>{
    const fixture=setup();fixture.asset.x=100;fixture.asset.y=200;
    const layout=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    const popup=fixture.boards.get('popup')!,child=fixture.boards.get('g2')!;
    const raw:ToolbarSourceNode={nodeId:'popup',parentNodeId:'toolbar',sourcePath:'res://viewport.zui',sourceNodeId:'popup',instancePath:'[]',parentSourcePath:'res://viewport.zui',parentSourceNodeId:'toolbar',parentInstancePath:'[]',controlId:'UnrealViewportToolbarOverflow',component:'ScrollableBox',visible:true,bounds:{x:0,y:60,width:87,height:48},clip:{x:0,y:60,width:87,height:48},state:{selected:true,checked:null,popupOpen:true,value:null,text:null,valueText:null,disabled:false},scroll:{offset:4.5,viewportExtent:48,contentExtent:162}};
    const group:ToolbarSourceNode={...structuredClone(raw),nodeId:'g2',parentNodeId:'popup',sourceNodeId:'g2',parentSourceNodeId:'popup',controlId:'G2',component:'HorizontalGroup',bounds:{x:6,y:103.5,width:75,height:42},state:{selected:null,checked:null,popupOpen:null,value:null,text:null,valueText:null,disabled:false},scroll:null};
    const cursor={structure:1,geometry:1,interaction:1,viewport:1,hitTest:1,theme:1,diagnostics:1};
    const publication:ViewportToolbarPublication={schema:'dev.zircon.editor.viewport-toolbar-publication',version:1,authority:'normal-prepared-source-surface',observed:true,coordinateSpace:'physical-pixels',surfaceGeneration:1,surfaceTreeId:'scroll-allocation-fixture',rootNodeId:'toolbar',rowNodeId:'row',moreNodeId:'more',popupNodeId:'popup',groupNodeIds:['g1','g2','g3'],nodes:[raw,group],presentationCursor:cursor,presentedFrameCount:1};
    const observed={...layout,plan:{...layout.plan,publication,dpi:1.5},sourceNodes:new Map([['popup',raw],['g2',group]])};
    applyPublishedViewportToolbarBoard(fixture.asset as unknown as Board,popup as unknown as Board,observed);
    applyPublishedViewportToolbarBoard(fixture.asset as unknown as Board,child as unknown as Board,observed);
    expect(JSON.parse(popup.getSharedPluginData(NS,'viewport-toolbar-observed-scroll'))).toEqual(raw.scroll);
    expect([child.x,child.y,child.width,child.height]).toEqual([104,269,50,28]);
    applyPublishedViewportToolbarBoard(fixture.asset as unknown as Board,child as unknown as Board,observed);
    expect(child.y).toBe(269); // The actual 3 logical-pixel offset is already in the published frame.
  });
  it('restores original asymmetric Source padding from fresh persisted group metadata and rejects a lost allocation',()=>{
    const fixture=setup(),padding={left:2,right:6,top:3,bottom:5},group=fixture.boards.get('g2')!;
    fixture.document.nodes!.row!.children!.find(mount=>mount.node==='g2')!.slot!['layout']={padding:{...padding}};
    fixture.document.nodes!.g2!.layout={width:{stretch:'Fixed'},height:{stretch:'Fixed'}};
    fixture.projection.shapes.find(shape=>shape.nodeId==='g2')!.slotPadding=padding;
    applySlotPadding(group as unknown as Board,padding);
    const narrow=prepareViewportToolbarLayout(fixture.asset as unknown as Board,fixture.document,fixture.projection,fixture.desired)!;
    const popup=fixture.boards.get('popup')!;
    popup.setSharedPluginData(NS,'container-kind','scroll');
    applyNativeScrollContentLayout(popup as unknown as Board,narrow.document,fixture.projection.shapes.find(shape=>shape.nodeId==='popup')!.container,fixture.desired);
    const records=JSON.parse(JSON.stringify([...fixture.boards].map(([id,item])=>({id,data:[...item.data],children:item.children.map(child=>child.id),layoutChild:{...item.layoutChild},width:item.width,height:item.height,x:item.x,y:item.y,hidden:item.hidden})))) as Array<{id:string;data:[string,string][];children:string[];layoutChild:Fake['layoutChild'];width:number;height:number;x:number;y:number;hidden:boolean}>;
    const fresh=setup();
    fresh.document=decodeZuiMetadata(encodeZuiMetadata(fixture.document));
    fresh.projection=structuredClone(fixture.projection);
    // Every Board, metadata map and document below is a fresh object, as on
    // reload. The capture does not consult any earlier projection context.
    for(const record of records){const item=fresh.boards.get(record.id)!;item.data=new Map(record.data);item.children=[];item.parent=null;Object.assign(item,{layoutChild:{...record.layoutChild},width:record.width,height:record.height,x:record.x,y:record.y,hidden:record.hidden});}
    for(const record of records)for(const id of record.children)fresh.boards.get(record.id)!.appendChild(fresh.boards.get(id)!);
    fresh.asset.children=[];fresh.asset.appendChild(fresh.boards.get('main')!);
    const restored=fresh.boards.get('g2')!,allocation=restored.getSharedPluginData(NS,'viewport-toolbar-slot-padding-allocation');
    expect(restored.parent!.id).toBe('popup');
    expect(captureViewportToolbarSlotPadding(fresh.document,restored as unknown as Board,captureSlotPadding(restored as unknown as Board,padding))).toEqual(padding);
    restored.setSharedPluginData(NS,'viewport-toolbar-slot-padding-allocation','');
    expect(()=>captureViewportToolbarSlotPadding(fresh.document,restored as unknown as Board,captureSlotPadding(restored as unknown as Board,padding))).toThrow(/Source padding allocation is missing/);
    expect(()=>prepareViewportToolbarLayout(fresh.asset as unknown as Board,fresh.document,fresh.projection,fresh.desired)).toThrow(/Source padding allocation is missing/);
    restored.setSharedPluginData(NS,'viewport-toolbar-slot-padding-allocation',allocation);
    fresh.boards.get('row')!.width=200;
    prepareViewportToolbarLayout(fresh.asset as unknown as Board,fresh.document,fresh.projection,fresh.desired);
    expect(captureSlotPadding(restored as unknown as Board,padding)).toEqual(padding);
    expect(restored.getSharedPluginData(NS,'viewport-toolbar-slot-padding-allocation')).toBe('');
    expect(fresh.boards.get('row')!.children.map(item=>item.id)).toEqual(['g1','fill','g2','g3','more']);
  });
  it('rejects escaped original boards and missing measurements rather than allocating guessed widths',()=>{
    const escaped=setup();escaped.asset.appendChild(escaped.boards.get('g2')!);
    expect(()=>prepareViewportToolbarLayout(escaped.asset as unknown as Board,escaped.document,escaped.projection,escaped.desired)).toThrow(/escaped/);
    const missing=setup();missing.desired.delete('g2');
    expect(()=>prepareViewportToolbarLayout(missing.asset as unknown as Board,missing.document,missing.projection,missing.desired)).toThrow(/intrinsic/);
  });
});

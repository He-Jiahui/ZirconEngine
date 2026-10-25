import { describe, expect, it } from 'vitest';
import { readViewportToolbarPublication, type ToolbarSourceNode } from './zui-viewport-toolbar-publication';
const cursor={structure:3,geometry:4,interaction:5,viewport:6,hitTest:7,theme:8,diagnostics:9};
function fixture(open=false, mask=2) {
  const nodes: ToolbarSourceNode[]=[];
  function node(id:string,control:string,parent:string|null,visible=true): ToolbarSourceNode {
    const result: ToolbarSourceNode={nodeId:id,parentNodeId:parent,sourcePath:'res://ui/editor/unreal/components/viewport.zui',sourceNodeId:id,instancePath:'[]',
      parentSourcePath:parent?'res://ui/editor/unreal/components/viewport.zui':null,parentSourceNodeId:parent,parentInstancePath:parent?'[]':null,
      controlId:control,component:'HorizontalGroup',visible,bounds:{x:0,y:0,width:28,height:28},clip:{x:0,y:0,width:800,height:600},
      state:{selected:null,checked:null,popupOpen:null,value:null,text:null,valueText:null,disabled:false},scroll:null};
    nodes.push(result); return result;
  }
  node('root','UnrealViewportToolbar',null);
  node('row','UnrealViewportToolbarRow','root');
  const more=node('more','UnrealViewportToolbarMore','row'); more.component='IconButton'; more.state.selected=open;more.state.checked=open;
  const popup=node('popup','UnrealViewportToolbarOverflow','root',open);popup.component='ScrollableBox';popup.state.popupOpen=open;
  if(open)popup.scroll={offset:3,viewportExtent:40,contentExtent:110};
  const groups=['translation-group','rotation-group','scale-group'];
  const controls=['ToggleTranslateSnap','ToggleRotateSnap','ToggleScaleSnap'];
  for(let index=0;index<3;index++){
    const parent=index===0?'row':'popup',visible=index===0||open;
    node(groups[index]!,groups[index]!,parent,visible);
    const snap=node(controls[index]!,controls[index]!,groups[index]!,visible);snap.component='IconButton';
    const enabled=Boolean(mask&(1<<index));snap.state.checked=enabled;snap.state.selected=enabled;snap.state.value=enabled;
  }
  const toolbar={schema:'dev.zircon.editor.viewport-toolbar-publication',version:1,authority:'normal-prepared-source-surface',observed:true,coordinateSpace:'physical-pixels',
    surfaceGeneration:11,surfaceTreeId:'actual-fixture-source',rootNodeId:'root',rowNodeId:'row',moreNodeId:'more',popupNodeId:'popup',groupNodeIds:groups,nodes,
    presentationCursor:{...cursor},presentedFrameCount:12};
  return {presentationCursor:{...cursor},presentedFrameCount:12,viewportToolbarPublication:toolbar,
    workbenchStatePublication:{authority:'normal-retained-workbench-projection',observed:true,presentationCursor:{...cursor},presentedFrameCount:12,
      workbenchPresentation:{viewportSnap:{translationEnabled:Boolean(mask&1),rotationEnabled:Boolean(mask&2),scaleEnabled:Boolean(mask&4)}}}};
}
const clone=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T;
describe('same-present prepared viewport toolbar evidence',()=>{
  it('preserves the actual authored component invocation path from the normal GPU source',()=>{
    const input=fixture();
    const path=JSON.stringify([{sourcePath:'res://ui/editor/unreal/level_editor_window.zui',sourceNodeId:'viewport'}]);
    for(const node of input.viewportToolbarPublication.nodes){
      node.instancePath=path;
      if(node.parentNodeId!==null)node.parentInstancePath=path;
    }
    const actual=readViewportToolbarPublication(input)!;
    expect(actual.nodes.every(node=>node.instancePath===path)).toBe(true);
    expect(actual.nodes.filter(node=>node.parentNodeId!==null).every(node=>node.parentInstancePath===path)).toBe(true);
  });
  it('rejects string aliases, malformed invocation steps and noncanonical source paths',()=>{
    const valid={sourcePath:'res://ui/editor/unreal/level_editor_window.zui',sourceNodeId:'viewport'};
    for(const path of [
      JSON.stringify(['viewport']),
      JSON.stringify([{source_path:valid.sourcePath,source_node_id:valid.sourceNodeId}]),
      JSON.stringify([{...valid,slot:0}]),
      JSON.stringify([{sourcePath:valid.sourcePath}]),
      JSON.stringify([{...valid,sourceNodeId:' '}]),
      JSON.stringify([{sourceNodeId:valid.sourceNodeId,sourcePath:valid.sourcePath}]),
      JSON.stringify([valid],null,2),
      JSON.stringify(Array.from({length:65},()=>valid)),
    ]){
      const input=fixture();input.viewportToolbarPublication.nodes[0]!.instancePath=path;
      expect(()=>readViewportToolbarPublication(input)).toThrow(/call-site list|compact source identity/);
    }
  });
  it('keeps absence unknown and returns an immutable detached observed subtree with nulls',()=>{
    expect(readViewportToolbarPublication({})).toBeUndefined();
    const input=fixture(),actual=readViewportToolbarPublication(input)!;
    expect(actual.nodes.find(node=>node.controlId==='UnrealViewportToolbarMore')!.state.popupOpen).toBeNull();
    input.viewportToolbarPublication.nodes[0]!.bounds.x=123;
    expect(actual.nodes[0]!.bounds.x).toBe(0);
    expect(Object.isFrozen(actual.nodes[0]!.state)).toBe(true);
  });
  it('preserves hidden original suffix, actual parent identity and observed scroll',()=>{
    const closed=readViewportToolbarPublication(fixture())!,open=readViewportToolbarPublication(fixture(true))!;
    expect(closed.groupNodeIds).toEqual(open.groupNodeIds);
    expect(closed.nodes.find(node=>node.nodeId==='scale-group')!.visible).toBe(false);
    expect(open.nodes.find(node=>node.nodeId==='scale-group')!.parentNodeId).toBe('popup');
    expect(open.nodes.find(node=>node.nodeId==='popup')!.scroll!.offset).toBe(3);
  });
  it('requires all seven counters and the exact successful normal/business frame',()=>{
    for(const field of Object.keys(cursor)){
      const wrong=clone(fixture());
      (wrong.viewportToolbarPublication.presentationCursor as Record<string,number>)[field]!+=1;
      expect(()=>readViewportToolbarPublication(wrong)).toThrow(/cursors differ/);
    }
    const count=fixture();count.viewportToolbarPublication.presentedFrameCount+=1;
    expect(()=>readViewportToolbarPublication(count)).toThrow(/different successful frame/);
    const stale=fixture();stale.workbenchStatePublication.observed=false;
    expect(()=>readViewportToolbarPublication(stale)).toThrow(/business publication/);
  });
  it('rejects duplicated/missing ownership, partial parent tuples and unordered reparenting',()=>{
    for(const mutate of [
      (p:ReturnType<typeof fixture>)=>p.viewportToolbarPublication.nodes.push(clone(p.viewportToolbarPublication.nodes[0]!)),
      (p:ReturnType<typeof fixture>)=>p.viewportToolbarPublication.nodes.splice(0,1),
      (p:ReturnType<typeof fixture>)=>{p.viewportToolbarPublication.nodes[1]!.parentSourcePath=null;},
      (p:ReturnType<typeof fixture>)=>{const first=p.viewportToolbarPublication.nodes.find(node=>node.nodeId==='translation-group')!;first.parentNodeId='popup';first.parentSourceNodeId='popup';const second=p.viewportToolbarPublication.nodes.find(node=>node.nodeId==='rotation-group')!;second.parentNodeId='row';second.parentSourceNodeId='row';},
    ]){const wrong=fixture();mutate(wrong);expect(()=>readViewportToolbarPublication(wrong)).toThrow();}
  });
  it('requires actual finite scroll geometry and explicit logical desired measurement units',()=>{
    const missing=fixture(true);missing.viewportToolbarPublication.nodes.find(node=>node.nodeId==='popup')!.scroll=null;
    expect(()=>readViewportToolbarPublication(missing)).toThrow(/scroll state/);
    const outside=fixture(true);outside.viewportToolbarPublication.nodes.find(node=>node.nodeId==='popup')!.scroll!.offset=999;
    expect(()=>readViewportToolbarPublication(outside)).toThrow(/outside/);
    const measured=fixture();measured.viewportToolbarPublication.nodes[0]!.desiredSize={width:34,height:28,coordinateSpace:'logical-pixels'};
    expect(readViewportToolbarPublication(measured)!.nodes[0]!.desiredSize!.width).toBe(34);
    (measured.viewportToolbarPublication.nodes[0]!.desiredSize as unknown as Record<string,unknown>)['coordinateSpace']='physical-pixels';
    expect(()=>readViewportToolbarPublication(measured)).toThrow(/logical units/);
  });
  it('preserves all independent snap combinations and rejects source/DTO disagreement',()=>{
    for(let mask=0;mask<8;mask++)expect(readViewportToolbarPublication(fixture(false,mask))).toBeDefined();
    const wrong=fixture();wrong.viewportToolbarPublication.nodes.find(node=>node.controlId==='ToggleScaleSnap')!.state.checked=true;
    expect(()=>readViewportToolbarPublication(wrong)).toThrow(/independent business bool/);
  });
  it('does not accept a static or false authority with apparently correct bounds',()=>{
    for(const field of ['authority','observed','coordinateSpace']){
      const wrong=fixture();const packet=wrong.viewportToolbarPublication as unknown as Record<string,unknown>;packet[field]=field==='observed'?false:'static';
      expect(()=>readViewportToolbarPublication(wrong)).toThrow(/authority/);
    }
  });
});

describe('prepared subtree root and inherited visibility admission',()=>{
  it('accepts an actual external source parent and rejects self or descendant root parents',()=>{
    const external=fixture();const root=external.viewportToolbarPublication.nodes[0]!;
    root.parentNodeId='window';root.parentSourcePath='res://ui/editor/unreal/level_editor_window.zui';
    root.parentSourceNodeId='window';root.parentInstancePath='[]';
    expect(readViewportToolbarPublication(external)).toBeDefined();
    for(const parent of ['root','row','scale-group']){
      const invalid=fixture();invalid.viewportToolbarPublication.nodes[0]!.parentNodeId=parent;
      expect(()=>readViewportToolbarPublication(invalid)).toThrow(/root parent/);
    }
  });
  it('rejects visible descendants below a hidden popup or original source group',()=>{
    const popupChild=fixture();popupChild.viewportToolbarPublication.nodes.find(node=>node.nodeId==='rotation-group')!.visible=true;
    expect(()=>readViewportToolbarPublication(popupChild)).toThrow(/invisible source ancestor/);
    const nestedChild=fixture();nestedChild.viewportToolbarPublication.nodes.find(node=>node.controlId==='ToggleScaleSnap')!.visible=true;
    expect(()=>readViewportToolbarPublication(nestedChild)).toThrow(/invisible source ancestor/);
  });
});

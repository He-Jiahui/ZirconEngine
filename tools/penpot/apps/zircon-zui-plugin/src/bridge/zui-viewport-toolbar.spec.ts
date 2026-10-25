import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { importedActualNormalReviewCase, reviewCaseFromSavedMetadata, reviewDocumentForCase } from './zui-review-case';
import { encodeZuiMetadata,decodeZuiMetadata,parseZuiDocument,serializeZuiDocument } from './zui-document';
import { reviewHost } from './zui-review-host';
import type { LayoutReviewCaseMessage } from '../model';
import { bindFontResources,fontResourcesForDocument,parseFontResourceManifest } from './penpot-font-selection';
import { deriveViewportToolbarReview,planViewportToolbar,viewportToolbarSourceMapping } from './zui-viewport-toolbar';
import type { ZuiAssetProjection } from './penpot-projection-model';
import type { ToolbarSourceNode } from './zui-viewport-toolbar-publication';
import { describe,it,expect } from 'vitest';
import type { ZuiDocument, ZuiChildMount } from './zui-document';
import { viewportToolbarContract,viewportToolbarInlinePrefix,viewportToolbarRuntimeDocument,canonicalViewportToolbarParent,prepareViewportToolbarProjection } from './zui-viewport-toolbar';
import type { ProjectedZuiNode } from './penpot-projection-model';
function source(): ZuiDocument {
  const mounts: ZuiChildMount[]=['g1','fill','g2','g3','more'].map(node=>({node,slot:{kind:'LinearSlot',linear_size:{rule:node==='fill'?'Stretch':'Auto'},padding:{left:2,right:2}}}));
  return {asset:{kind:'view',id:'ue',version:2},root:{node:'root'},nodes:{
    root:{component:'Overlay',control_id:'UnrealViewportToolbar',props:{toolbar_overflow_row_control:'UnrealViewportToolbarRow',toolbar_overflow_group_controls:['G1','G2','G3']},children:[{node:'row'},{node:'popup'}]},
    row:{component:'HorizontalGroup',control_id:'UnrealViewportToolbarRow',children:mounts},
    popup:{component:'ScrollableBox',control_id:'UnrealViewportToolbarOverflow',props:{toolbar_overflow_role:'toolbar-overflow',popup_open:false},children:[]},
    more:{component:'IconButton',control_id:'UnrealViewportToolbarMore',events:[{id:'workbench.unreal.viewport.overflow.open',event:'Click',route:'workbench.unreal.viewport.overflow.open'}]},
    fill:{component:'Space',control_id:'UnrealViewportToolbarFill'},
    g1:{component:'HorizontalGroup',control_id:'G1'},g2:{component:'HorizontalGroup',control_id:'G2',events:[{id:'original',event:'Click',route:'original'}]},g3:{component:'HorizontalGroup',control_id:'G3'},
  }};
}
describe('original Unreal toolbar ownership and measured reflow',()=>{
  it('reserves original spacer gaps and measured More without hiding an operation',()=>{
    expect(viewportToolbarInlinePrefix([50,50,50],162,28,4)).toBe(3);
    expect(viewportToolbarInlinePrefix([50,50,50],161,28,4)).toBe(2);
    expect(viewportToolbarInlinePrefix([50,50,50],85,28,4)).toBe(0);
    expect(viewportToolbarInlinePrefix([50,50,50],86,28,4)).toBe(1);
    expect(()=>viewportToolbarInlinePrefix([0,50],200,28,4)).toThrow(/intrinsic/);
  });
  it('rejects unrelated popup ownership, multiply owned groups and reordered source groups',()=>{
    const missing=source();missing.nodes!.root!.children=[{node:'row'}];expect(()=>viewportToolbarContract(missing)).toThrow(/ownership/);
    const duplicate=source();duplicate.nodes!.popup!.children=[{node:'g2'}];expect(()=>viewportToolbarContract(duplicate)).toThrow(/multiply/);
    const order=source();order.nodes!.root!.props!['toolbar_overflow_group_controls']=['G2','G1','G3'];expect(()=>viewportToolbarContract(order)).toThrow(/ordering/);
  });
  it('keeps exact TTC resource context on the runtime clone without adding it to exported Source',()=>{
    const original=source(),before=JSON.stringify(original),hash='a'.repeat(64);
    const resources=parseFontResourceManifest({schema:'zircon.penpot.font-resources.v1',selections:[{source:{family:'SourceFace',weight:'400',style:'normal'},engine:{assetPath:'res://fonts/source.ttc',assetSha256:hash,faceIndex:2,sfntSha256:hash},penpot:{fontId:'registered-family',variantId:'normal-400',sfntSha256:hash}}]});
    bindFontResources(original,resources);
    const runtime=viewportToolbarRuntimeDocument(original,{contract:viewportToolbarContract(original)!,inline:1,open:true,popupWidth:100,popupHeight:80});
    expect(fontResourcesForDocument(runtime)).toBe(resources);
    expect(fontResourcesForDocument(original)).toBe(resources);
    expect(JSON.stringify(original)).toBe(before);
    expect(JSON.stringify(runtime)).not.toContain('registered-family');
  });
  it('requires the dedicated Source overflow marker without taking the registered widget role',()=>{
    const original=source();original.nodes!.popup!.props!['component_role']='registered-scroll-role';
    expect(viewportToolbarContract(original)!.popup).toBe('popup');
    delete original.nodes!.popup!.props!['toolbar_overflow_role'];original.nodes!.popup!.props!['component_role']='toolbar-overflow';
    expect(()=>viewportToolbarContract(original)).toThrow(/popup role/);
  });
  it('moves the original suffix in a runtime clone and keeps source ids/events/slots/order intact',()=>{
    const original=source(),before=JSON.stringify(original),contract=viewportToolbarContract(original)!;
    const runtime=viewportToolbarRuntimeDocument(original,{contract,inline:1,open:true,popupWidth:100,popupHeight:80});
    expect(runtime.nodes!.row!.children!.map(mount=>mount.node)).toEqual(['g1','fill','more']);
    expect(runtime.nodes!.popup!.children!.map(mount=>mount.node)).toEqual(['g2','g3']);
    expect(runtime.nodes!.popup!.children![0]!.slot!['kind']).toBe('ScrollableSlot');
    expect(runtime.nodes!.g2).toBe(original.nodes!.g2);
    expect(JSON.stringify(original)).toBe(before);
    expect(canonicalViewportToolbarParent(contract,'g2','popup')).toBe('row');
    expect(canonicalViewportToolbarParent(contract,'g2','row')).toBe('row');
    expect(canonicalViewportToolbarParent(contract,'fill','row')).toBe('row');
    expect(()=>canonicalViewportToolbarParent(contract,'g2','outside')).toThrow(/outside/);
  });
  it('allocates original closed More and popup painter content while leaving other hidden source widgets alone',()=>{
    const document=source(),shapes=[{nodeId:'more',previewHidden:true},{nodeId:'popup',previewHidden:true},{nodeId:'outside',previewHidden:true}] as ProjectedZuiNode[];
    prepareViewportToolbarProjection(document,shapes);
    expect(shapes.map(shape=>shape.previewHidden)).toEqual([false,false,true]);
    expect(document.nodes!.popup!.props!['popup_open']).toBe(false);
  });
});

describe('same-present source tuple mapping and logical measurements',()=>{
  function observed(){
    const document=source(),path='res://ui/editor/unreal/components/viewport.zui';
    document.nodes!['translate']={component:'IconButton',control_id:'ToggleTranslateSnap'};
    document.nodes!['rotate']={component:'IconButton',control_id:'ToggleRotateSnap'};
    document.nodes!['scale']={component:'IconButton',control_id:'ToggleScaleSnap'};
    for(const [group,control] of [['g1','translate'],['g2','rotate'],['g3','scale']])document.nodes![group!]!.children=[{node:control!}];
    const parent=new Map<string,string>();for(const [id,node] of Object.entries(document.nodes!))for(const child of node.children??[])parent.set(child.node,id);
    const nodes:ToolbarSourceNode[]=[];
    for(const [id,node] of Object.entries(document.nodes!)){
      node['penpot_review_source_path']=path;node['penpot_review_source_node_id']=id;node['penpot_review_instance_path']='[]';
      const actualParent=['g2','g3'].includes(id)?'popup':parent.get(id)??null,hidden=['popup','g2','g3','rotate','scale'].includes(id);
      const raw:ToolbarSourceNode={nodeId:id,parentNodeId:actualParent,sourcePath:path,sourceNodeId:id,instancePath:'[]',parentSourcePath:actualParent?path:null,parentSourceNodeId:actualParent,parentInstancePath:actualParent?'[]':null,controlId:node.control_id??null,component:node.component,visible:!hidden,bounds:{x:30,y:45,width:42,height:42},clip:{x:0,y:0,width:900,height:620},state:{selected:null,checked:null,popupOpen:null,value:null,text:null,valueText:null,disabled:false},scroll:null};
      if(['g1','g2','g3'].includes(id))raw.desiredSize={width:50,height:28,coordinateSpace:'logical-pixels'};
      if(id==='more'){raw.state.selected=false;raw.state.checked=false;}
      if(id==='popup'){raw.state.popupOpen=false;raw.bounds.width=0;raw.bounds.height=0;}
      if(['translate','rotate','scale'].includes(id)){const enabled=id==='rotate';raw.state.selected=enabled;raw.state.checked=enabled;raw.state.value=enabled;}
      nodes.push(raw);
    }
    const cursor={structure:1,geometry:2,interaction:3,viewport:4,hitTest:5,theme:6,diagnostics:7};
    const evidence={presentationCursor:cursor,presentedFrameCount:8,workbenchStatePublication:{authority:'normal-retained-workbench-projection',observed:true,presentationCursor:cursor,presentedFrameCount:8,workbenchPresentation:{viewportSnap:{translationEnabled:false,rotationEnabled:true,scaleEnabled:false}}},viewportToolbarPublication:{schema:'dev.zircon.editor.viewport-toolbar-publication',version:1,authority:'normal-prepared-source-surface',observed:true,coordinateSpace:'physical-pixels',surfaceGeneration:9,surfaceTreeId:'fixture',rootNodeId:'root',rowNodeId:'row',moreNodeId:'more',popupNodeId:'popup',groupNodeIds:['g1','g2','g3'],nodes,presentationCursor:cursor,presentedFrameCount:8}};
    return {document,evidence};
  }
  it('maps actual independent bools by exact source tuple without changing original source parents/events',()=>{
    const {document,evidence}=observed(),parents=JSON.stringify(Object.fromEntries(Object.entries(document.nodes!).map(([id,node])=>[id,node.children])));
    deriveViewportToolbarReview(document,evidence,1.5);
    expect(document.nodes!.translate!.props!['checked']).toBe(false);expect(document.nodes!.rotate!.props!['checked']).toBe(true);expect(document.nodes!.scale!.props!['checked']).toBe(false);
    expect(JSON.stringify(Object.fromEntries(Object.entries(document.nodes!).map(([id,node])=>[id,node.children])))).toBe(parents);
    expect(document.nodes!.more!.events![0]!['route']).toBe('workbench.unreal.viewport.overflow.open');
    const padding={left:0,right:0,top:0,bottom:0};
    const shapes=Object.keys(document.nodes!).map(nodeId=>({nodeId,slotPadding:padding,container:{kind:'flex',direction:nodeId==='popup'?'column':'row',padding,gap:4,columnGap:4,rowGap:4}})) as ProjectedZuiNode[];
    const projection={shapes} as ZuiAssetProjection;
    const plan=planViewportToolbar(document,projection,new Map(),900,600)!;
    expect(plan.inline).toBe(1);expect(plan.dpi).toBe(1.5);expect(plan.popupWidth).toBe(0);
    // A complete native measurement is logical even when arranged group width is42 physical pixels.
    expect(plan.publication!.nodes.find(node=>node.nodeId==='g1')!.desiredSize!.width).toBe(50);
  });
  it('rejects an unmapped original leaf tuple rather than using its source default state',()=>{
    const wrong=observed();wrong.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='translate')!.instancePath='[{"sourcePath":"res://wrong-owner.zui","sourceNodeId":"wrong-call-site"}]';
    expect(()=>deriveViewportToolbarReview(wrong.document,wrong.evidence,1)).toThrow(/semantic node.*translate/);
    const missing=observed();delete missing.document.nodes!.rotate!['penpot_review_source_node_id'];
    expect(()=>deriveViewportToolbarReview(missing.document,missing.evidence,1)).toThrow(/semantic node.*rotate/);
  });
  it('classifies extra native-only tuples as source-only after requiring every canonical Source descendant',()=>{
    const fixture=observed(),native=structuredClone(fixture.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='fill')!);
    native.nodeId='native-painter';native.sourceNodeId='runtime-painter';native.controlId=null;native.component='NativePainter';
    fixture.evidence.viewportToolbarPublication.nodes.push(native);
    deriveViewportToolbarReview(fixture.document,fixture.evidence,1);
    const padding={left:0,right:0,top:0,bottom:0};
    const shapes=Object.keys(fixture.document.nodes!).map(nodeId=>({nodeId,container:{kind:'flex',direction:nodeId==='popup'?'column':'row',padding,gap:4,columnGap:4,rowGap:4}})) as ProjectedZuiNode[];
    const plan=planViewportToolbar(fixture.document,{shapes} as ZuiAssetProjection,new Map(),900,600)!;
    expect([...plan.sourceOnlyNativeNodeIds!]).toEqual(['native-painter']);
    expect(plan.sourceByNativeId!.has('native-painter')).toBe(false);
  });
  it('requires a packet on the actual normal review entry while true Source-only authoring remains editable',()=>{
    expect(()=>deriveViewportToolbarReview(source(),undefined,1)).not.toThrow();
    const missing=observed(),evidence:Record<string,unknown>={...missing.evidence};delete evidence['viewportToolbarPublication'];
    expect(()=>deriveViewportToolbarReview(missing.document,evidence,1)).toThrow(/normal evidence lacks/);
    const malformed=observed();
    expect(()=>deriveViewportToolbarReview(malformed.document,{...malformed.evidence,viewportToolbarPublication:null},1)).toThrow(/payload/);
  });
  it('rejects leaf reparenting to another group or the row while retaining the actual ordered group suffix',()=>{
    const toRow=observed(),leaf=toRow.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='translate')!;
    leaf.parentNodeId='row';leaf.parentSourceNodeId='row';
    expect(()=>deriveViewportToolbarReview(toRow.document,toRow.evidence,1)).toThrow(/canonical parent.*translate/);
    const toOtherGroup=observed(),rotate=toOtherGroup.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='rotate')!;
    rotate.parentNodeId='g3';rotate.parentSourceNodeId='g3';
    expect(()=>deriveViewportToolbarReview(toOtherGroup.document,toOtherGroup.evidence,1)).toThrow(/canonical parent.*rotate/);
  });
  it('matches an external toolbar-root parent by its complete Source tuple and rejects an unrelated external parent',()=>{
    const fixture=observed(),path='res://outer.zui';
    fixture.document.nodes!['outer']={component:'Overlay',children:[{node:'root'}],penpot_review_source_path:path,penpot_review_source_node_id:'outer',penpot_review_instance_path:'[]'};
    fixture.document.root={node:'outer'};
    const root=fixture.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='root')!;
    root.parentNodeId='native-outer';root.parentSourcePath=path;root.parentSourceNodeId='outer';root.parentInstancePath='[]';
    expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1)).not.toThrow();
    root.parentSourceNodeId='other-outer';
    expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1)).toThrow(/external parent tuple/);
  });
  it('rejects wrong source identity and missing actual group measurements without inferring from bounds',()=>{
    const wrong=observed();wrong.document.nodes!.more!['penpot_review_instance_path']='["different-instance"]';
    expect(()=>deriveViewportToolbarReview(wrong.document,wrong.evidence,1)).toThrow(/source toolbar/);
    const missing=observed();delete missing.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='g1')!.desiredSize;
    deriveViewportToolbarReview(missing.document,missing.evidence,1);
    const padding={left:0,right:0,top:0,bottom:0};
    const shapes=Object.keys(missing.document.nodes!).map(nodeId=>({nodeId,container:{kind:'flex',direction:nodeId==='popup'?'column':'row',padding,gap:4,columnGap:4,rowGap:4}})) as ProjectedZuiNode[];
    expect(()=>planViewportToolbar(missing.document,{shapes} as ZuiAssetProjection,new Map([['g1',{width:99,height:28}]]),900,600)).toThrow(/logical desired/);
  });
  function importedFixture() {
    const fixture=observed();
    const reviewCase: LayoutReviewCaseMessage={id:'actual-import',sourcePath:'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui',host:'editor',viewport:{width:900,height:620},dpi:1.5,locale:'en-US',state:'default',data:{normal_window_evidence:fixture.evidence}};
    fixture.document['penpot_actual_normal_review_case']=reviewCase;
    const wrapper: ZuiDocument={...source(),penpot_review_host:fixture.document};
    // This is the production .zui serializer/parser, including its typed JSON
    // review host. TOML cannot represent the publication's null values itself.
    const imported=reviewHost(parseZuiDocument(serializeZuiDocument(wrapper)).document)!;
    const padding={left:0,right:0,top:0,bottom:0};
    const shapes=Object.keys(imported.nodes!).map(nodeId=>({nodeId,container:{kind:'flex',direction:nodeId==='popup'?'column':'row',padding,gap:4,columnGap:4,rowGap:4}})) as ProjectedZuiNode[];
    return {imported,reviewCase,projection:{shapes} as ZuiAssetProjection};
  }
  it('uses the actual imported .zui host case and retains null states, independent bools and original Source graph',()=>{
    const fixture=importedFixture(),before=encodeZuiMetadata(fixture.imported);
    const actual=importedActualNormalReviewCase(fixture.imported)!;
    expect(actual).toEqual(fixture.reviewCase);
    const document=reviewDocumentForCase(fixture.imported,actual);
    const plan=planViewportToolbar(document,fixture.projection,new Map(),900,600)!;
    expect(plan.inline).toBe(1);expect(plan.dpi).toBe(1.5);
    expect(plan.publication!.nodes.find(node=>node.nodeId==='fill')!.state.checked).toBeNull();
    expect(plan.publication!.nodes.find(node=>node.nodeId==='popup')!.bounds.width).toBe(0);
    expect(document.nodes!.translate!.props!['checked']).toBe(false);
    expect(document.nodes!.rotate!.props!['checked']).toBe(true);
    expect(document.nodes!.scale!.props!['checked']).toBe(false);
    expect(document.nodes!.row!.children).toEqual(fixture.imported.nodes!.row!.children);
    expect(document.nodes!.more!.events).toEqual(fixture.imported.nodes!.more!.events);
    expect(encodeZuiMetadata(fixture.imported)).toBe(before);
  });
  it('rebuilds strict toolbar context from fresh saved document and Board case metadata',()=>{
    const fixture=importedFixture(),sourceMetadata=encodeZuiMetadata(fixture.imported),caseMetadata=JSON.stringify(importedActualNormalReviewCase(fixture.imported));
    const fresh=decodeZuiMetadata(sourceMetadata);
    const saved=reviewCaseFromSavedMetadata(fresh,caseMetadata)!;
    const restored=reviewDocumentForCase(fresh,saved);
    const plan=planViewportToolbar(restored,fixture.projection,new Map(),1,0)!;
    // One inline group is published. The current Source fit budget cannot
    // manufacture that parent suffix or the successful frame's measured sizes.
    expect(plan.inline).toBe(1);
    expect(plan.publication).toEqual((fixture.reviewCase.data['normal_window_evidence'] as ReturnType<typeof observed>['evidence']).viewportToolbarPublication);
    expect(plan.publication!.nodes.find(node=>node.nodeId==='g2')!.desiredSize).toEqual({width:50,height:28,coordinateSpace:'logical-pixels'});
    expect(encodeZuiMetadata(fresh)).toBe(sourceMetadata);
  });
  it('rejects missing or malformed imported normal packets while unannotated Source authoring remains valid',()=>{
    expect(importedActualNormalReviewCase(source())).toBeUndefined();
    for(const bad of [undefined,null]) {
      const fixture=importedFixture(),value=fixture.imported['penpot_actual_normal_review_case'] as LayoutReviewCaseMessage;
      const evidence=value.data['normal_window_evidence'] as Record<string,unknown>;
      if(bad===undefined)delete evidence['viewportToolbarPublication'];else evidence['viewportToolbarPublication']=bad;
      expect(()=>importedActualNormalReviewCase(fixture.imported)).toThrow(/toolbar publication|payload/);
    }
    const invalid=source();invalid['penpot_actual_normal_review_case']=null;
    expect(()=>importedActualNormalReviewCase(invalid)).toThrow(/complete Editor case/);
  });
  it('rejects lost saved case metadata, Source-only replacement, malformed packet and unrelated saved Source owner',()=>{
    const fixture=importedFixture();
    expect(()=>reviewCaseFromSavedMetadata(fixture.imported,'')).toThrow(/saved case metadata/);
    expect(()=>reviewCaseFromSavedMetadata(fixture.imported,JSON.stringify({...fixture.reviewCase,data:{}}))).toThrow(/complete Editor case/);
    const malformed=structuredClone(fixture.reviewCase);
    (malformed.data['normal_window_evidence'] as Record<string,unknown>)['viewportToolbarPublication']=null;
    expect(()=>reviewCaseFromSavedMetadata(fixture.imported,JSON.stringify(malformed))).toThrow(/payload/);
    expect(()=>reviewCaseFromSavedMetadata(fixture.imported,JSON.stringify({...fixture.reviewCase,sourcePath:'different.zui'}))).toThrow(/different Source owner/);
    expect(reviewCaseFromSavedMetadata(source(),'')).toBeNull();
  });

  describe('actual full-workbench toolbar identity',()=>{
    const owner='zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
    const viewport='zircon_editor/assets/ui/editor/unreal/components/viewport.zui';
    const shell='zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
    const alias=(path:string):string=>path.replace(/^zircon_editor\/assets\//,'res://');
    const ownerCall={sourcePath:owner,sourceNodeId:'viewport'};
    const reviewPrefix={sourcePath:shell,sourceNodeId:'pane_surface'};
    const reviewCalls=JSON.stringify([reviewPrefix,ownerCall]);
    const nativeCalls=JSON.stringify([{...ownerCall,sourcePath:alias(owner)}]);
    function fullWorkbench(){
      const fixture=observed(),document=fixture.document,packet=fixture.evidence.viewportToolbarPublication;
      for(const node of Object.values(document.nodes!)){
        node['penpot_review_source_path']=viewport;node['penpot_review_instance_path']=reviewCalls;
      }
      for(const node of packet.nodes){node.instancePath=nativeCalls;node.sourcePath=alias(viewport);}
      document.nodes!['projection']={component:'Dropdown',control_id:'SetProjectionMode',
        penpot_review_source_path:'zircon_editor/assets/ui/editor/workbench/primitives/inputs/dropdown.zui',
        penpot_review_source_node_id:'root',penpot_review_instance_path:JSON.stringify([reviewPrefix,ownerCall,{sourcePath:viewport,sourceNodeId:'projection'}])};
      document.nodes!.g1!.children!.push({node:'projection'});
      packet.nodes.push({...structuredClone(packet.nodes.find(node=>node.nodeId==='translate')!),
        nodeId:'projection',sourcePath:'res://ui/editor/workbench/primitives/inputs/dropdown.zui',sourceNodeId:'root',
        instancePath:JSON.stringify([{...ownerCall,sourcePath:alias(owner)},{sourcePath:alias(viewport),sourceNodeId:'projection'}]),
        controlId:'SetProjectionMode',component:'Dropdown'});
      for(const node of packet.nodes){
        const parent=packet.nodes.find(candidate=>candidate.nodeId===node.parentNodeId);
        if(parent){node.parentSourcePath=parent.sourcePath;node.parentSourceNodeId=parent.sourceNodeId;node.parentInstancePath=parent.instancePath;}
      }
      document.nodes!['layer']={component:'Overlay',control_id:'UnrealViewportToolbarLayer',children:[{node:'root'}],
        penpot_review_source_path:viewport,penpot_review_source_node_id:'toolbar_layer',penpot_review_instance_path:reviewCalls};
      document.nodes!['window']={component:'Overlay',control_id:'WorkbenchWindowRoot',children:[{node:'layer'}],
        penpot_review_source_path:owner,penpot_review_source_node_id:'root',penpot_review_instance_path:JSON.stringify([reviewPrefix])};
      document.root={node:'window'};
      const toolbar=packet.nodes.find(node=>node.nodeId==='root')!;
      toolbar.parentNodeId='native-layer';toolbar.parentSourcePath=alias(viewport);toolbar.parentSourceNodeId='toolbar_layer';toolbar.parentInstancePath=nativeCalls;
      const prototypeCalls=JSON.stringify([reviewPrefix,{sourcePath:owner,sourceNodeId:'details'},
        {sourcePath:'zircon_editor/assets/ui/editor/unreal/components/details.zui',sourceNodeId:'prototype'}]);
      const itemKeys=['157694992433500694610706526866931910355','57620256968940470099537706358547345741','17313027826898521308317255830018820214'];
      for(const [index,id] of ['prototype','fov','far','near'].entries()){
        document.nodes![id]={component:'HorizontalGroup',penpot_review_source_path:'zircon_editor/assets/ui/editor/workbench/data/component_property_row.zui',
          penpot_review_source_node_id:'root',penpot_review_instance_path:prototypeCalls,
          ...(index>0?{penpot_review_generated_product_row:{kind:'inspectorProperty',itemKey:itemKeys[index-1]!}}:{})};
        document.nodes!.window!.children!.push({node:id});
      }
      for(const [id,runtimeSurfaceKey] of [['header-document','document:3c9d8d65-2ba1-461c-9fa7-5513554212c2'],['header-bottom','bottom']]){
        document.nodes![id!]={component:'Overlay',penpot_review_source_path:'zircon_editor/assets/ui/editor/unreal/components/dock_header.zui',
          penpot_review_source_node_id:'dock_header_root',penpot_review_instance_path:JSON.stringify([reviewPrefix]),runtimeSurfaceKey:runtimeSurfaceKey!};
        document.nodes!.window!.children!.push({node:id!});
      }
      const projected={nodeId:'v2/root',surfaceNodeId:1,parentId:null,sourcePath:alias(owner),sourceNodeId:'root',
        instancePath:[],controlId:'WorkbenchWindowRoot',component:'Overlay'};
      const evidence={...fixture.evidence,workbenchStatePublication:{...fixture.evidence.workbenchStatePublication,
        workbenchPresentation:{...fixture.evidence.workbenchStatePublication.workbenchPresentation,sourceFingerprint:{sourcePath:owner}}},
        arrangedInventoryPublication:{schema:'dev.zircon.editor.normal-arranged-inventory',version:1,
        authority:'normal-retained-runtime-surfaces',observed:true,complete:false,pending:[{sourceNodeId:'Slot04Row'}],
        presentationCursor:structuredClone(fixture.evidence.presentationCursor),presentedFrameCount:8,surfaces:[{
          schema:'dev.zircon.editor.normal-surface-arranged-inventory',version:1,scope:'workbench-window',
          documentId:alias(owner),coordinateSpace:'surface-logical',frameAuthority:'retained-host-projection',sourceFrameGeneration:13,rootIds:[1],
          nodes:[{...projected,parent:null,projected}],
        }]}};
      return {document,evidence};
    }
    function identityGraph(document:ZuiDocument){
      return Object.fromEntries(Object.entries(document.nodes!).map(([id,node])=>[id,{
        component:node.component,controlId:node.control_id,sourcePath:node['penpot_review_source_path'],
        sourceNodeId:node['penpot_review_source_node_id'],instancePath:node['penpot_review_instance_path'],children:node.children,events:node.events,
      }]));
    }
    function projection(document:ZuiDocument):ZuiAssetProjection{
      const popup=viewportToolbarContract(document)!.popup,padding={left:0,right:0,top:0,bottom:0};
      return {shapes:Object.keys(document.nodes!).map(nodeId=>({nodeId,container:{kind:'flex',direction:nodeId===popup?'column':'row',padding,gap:4,columnGap:4,rowGap:4}})) as ProjectedZuiNode[]} as ZuiAssetProjection;
    }
    it('joins official aliases and complete real owner-relative invocations while retaining unrelated Inspector/Header duplicates',()=>{
      const fixture=fullWorkbench(),before=JSON.stringify(identityGraph(fixture.document)),raw=JSON.stringify(fixture.evidence);
      deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25);
      const plan=planViewportToolbar(fixture.document,projection(fixture.document),new Map(),900,620)!;
      expect(plan.sourceByNativeId!.size).toBe(fixture.evidence.viewportToolbarPublication.nodes.length);
      expect(plan.sourceByNativeId!.get('projection')).toBe('projection');
      expect(plan.sourceOnlyNativeNodeIds!.size).toBe(0);
      expect(plan.inline).toBe(1);expect(plan.publication).toEqual(fixture.evidence.viewportToolbarPublication);
      expect(JSON.stringify(identityGraph(fixture.document))).toBe(before);expect(JSON.stringify(fixture.evidence)).toBe(raw);
      expect(fixture.evidence.arrangedInventoryPublication.complete).toBe(false);
      expect(fixture.evidence.arrangedInventoryPublication.pending).toEqual([{sourceNodeId:'Slot04Row'}]);
    });
    it('resolves persisted original group identities without applying their observed state to Source',()=>{
      const fixture=fullWorkbench(),before=encodeZuiMetadata(fixture.document),raw=JSON.stringify(fixture.evidence);
      const mapping=viewportToolbarSourceMapping(fixture.document,fixture.evidence,1.25)!;
      const contract=viewportToolbarContract(fixture.document)!;
      expect(mapping.packet.groupNodeIds.map(id=>mapping.sourceByNativeId.get(id))).toEqual(contract.groups);
      expect(mapping.sourceByNativeId.size).toBe(mapping.packet.nodes.length);
      expect(encodeZuiMetadata(fixture.document)).toBe(before);
      expect(JSON.stringify(fixture.evidence)).toBe(raw);
      expect(()=>planViewportToolbar(fixture.document,projection(fixture.document),new Map(),900,620))
        .toThrow(/source group lacks actual intrinsic measurement: g1/);
    });
    it.each(['inside','outside'])('rejects a relevant tuple duplicated %s the toolbar',placement=>{
      const fixture=fullWorkbench();fixture.document.nodes!['duplicate']={...structuredClone(fixture.document.nodes!.translate!),control_id:'DuplicatePhysicalNode'};
      fixture.document.nodes![placement==='inside'?'g1':'window']!.children!.push({node:'duplicate'});
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/duplicate expanded source identity/);
    });
    it('rejects a publication naming a unique known outside node instead of treating it as source-only',()=>{
      const fixture=fullWorkbench(),calls=[{sourcePath:owner,sourceNodeId:'details'}];
      fixture.document.nodes!['outside']={component:'Text',control_id:'KnownOutside',penpot_review_source_path:viewport,
        penpot_review_source_node_id:'outside',penpot_review_instance_path:JSON.stringify([reviewPrefix,...calls])};
      fixture.document.nodes!.window!.children!.push({node:'outside'});
      fixture.evidence.viewportToolbarPublication.nodes.push({...structuredClone(fixture.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='fill')!),
        nodeId:'outside',sourceNodeId:'outside',instancePath:JSON.stringify(calls.map(step=>({...step,sourcePath:alias(step.sourcePath)}))),controlId:'KnownOutside',component:'Text'});
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/outside its source toolbar/);
    });
    it('rejects a packet key shared by unrelated outside rows when that key is actually queried',()=>{
      const fixture=fullWorkbench(),prototype=fixture.document.nodes!.prototype!;
      fixture.evidence.viewportToolbarPublication.nodes.push({...structuredClone(fixture.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='fill')!),
        nodeId:'claimed-inspector',sourcePath:alias(prototype['penpot_review_source_path'] as string),sourceNodeId:'root',controlId:null,component:'HorizontalGroup',
        instancePath:JSON.stringify(JSON.parse(prototype['penpot_review_instance_path'] as string).slice(1).map((step:{sourcePath:string;sourceNodeId:string})=>({...step,sourcePath:alias(step.sourcePath)})))});
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/duplicate expanded source identity/);
    });
    it.each(['res://assets/ui/editor/unreal/components/viewport.zui','other/zircon_editor/assets/ui/editor/unreal/components/viewport.zui',
      'res://ui/editor/unreal/components/../components/viewport.zui','zircon_editor\\assets\\ui\\editor\\unreal\\components\\viewport.zui'])('rejects an unofficial or noncanonical source alias: %s',path=>{
      const fixture=fullWorkbench();fixture.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='translate')!.sourcePath=path;
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/source alias|semantic node.*translate/);
    });
    it.each(['wrong-prefix','missing-callee','wrong-callee','wrong-source','wrong-source-node','wrong-component','wrong-control'])('rejects a dropdown %s without suffix or control fallback',change=>{
      const fixture=fullWorkbench(),dropdown=fixture.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='projection')!;
      const steps=JSON.parse(dropdown.instancePath) as {sourcePath:string;sourceNodeId:string}[];
      if(change==='wrong-prefix')steps.unshift({sourcePath:alias(shell),sourceNodeId:'other_mount'});
      if(change==='missing-callee')steps.pop();
      if(change==='wrong-callee')steps[1]!.sourceNodeId='another_projection';
      if(change==='wrong-source')dropdown.sourcePath=alias(viewport);
      if(change==='wrong-source-node')dropdown.sourceNodeId='another_root';
      if(change==='wrong-component')dropdown.component='Text';
      if(change==='wrong-control')dropdown.controlId='SetOtherMode';
      dropdown.instancePath=JSON.stringify(steps);
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/semantic node.*projection|component owner|control owner/);
    });
    it.each(['review-owner','business-owner','native-owner','projected-owner','cursor','frame','root-id'])('rejects a different actual owner or successful frame: %s',change=>{
      const fixture=fullWorkbench(),inventory=fixture.evidence.arrangedInventoryPublication,surface=inventory.surfaces[0]!,root=surface.nodes[0]!;
      if(change==='review-owner')fixture.document.nodes!.window!['penpot_review_source_path']=viewport;
      if(change==='business-owner')fixture.evidence.workbenchStatePublication.workbenchPresentation.sourceFingerprint.sourcePath=viewport;
      if(change==='native-owner')root.sourcePath=alias(viewport);
      if(change==='projected-owner')root.projected.sourceNodeId='other_root';
      if(change==='cursor')inventory.presentationCursor.diagnostics++;
      if(change==='frame')inventory.presentedFrameCount++;
      if(change==='root-id')surface.rootIds=[99];
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/owner|frame|projection/);
    });
    it('requires the full external parent tuple after owner-relative mapping',()=>{
      const fixture=fullWorkbench();fixture.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='root')!.parentInstancePath=JSON.stringify([{sourcePath:alias(owner),sourceNodeId:'another_viewport'}]);
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/external parent tuple/);
    });
    it('keeps canonical-source completeness after irrelevant duplicate keys are deferred',()=>{
      const fixture=fullWorkbench();fixture.evidence.viewportToolbarPublication.nodes=fixture.evidence.viewportToolbarPublication.nodes.filter(node=>node.nodeId!=='projection');
      expect(()=>deriveViewportToolbarReview(fixture.document,fixture.evidence,1.25)).toThrow(/semantic node.*projection/);
    });

    const replayNames=['ZUI_LAYOUT_ACTUAL_PREPARED_INPUT','ZUI_LAYOUT_ACTUAL_PREPARED_INPUT_SHA256',
      'ZUI_LAYOUT_ACTUAL_REVIEW_CASE','ZUI_LAYOUT_ACTUAL_REVIEW_CASE_SHA256'] as const;
    const replayValues=replayNames.map(name=>process.env[name]);
    const provided=replayValues.filter(value=>value!==undefined).length;
    if(provided!==0 && (provided!==replayNames.length || replayValues.some(value=>!value?.trim())))
      throw new Error('actual full-workbench toolbar identity replay requires all four path/hash environment pins');
    it.skipIf(provided===0)('replays pinned actual prepared host and original case with all 33 Source nodes and unchanged inputs',()=>{
      const [preparedPath,preparedSha,casePath,caseSha]=replayValues as [string,string,string,string];
      const hash=(bytes:Uint8Array):string=>createHash('sha256').update(bytes).digest('hex');
      expect(preparedSha).toMatch(/^[0-9a-f]{64}$/);expect(caseSha).toMatch(/^[0-9a-f]{64}$/);
      const prepared=readFileSync(preparedPath),caseBytes=readFileSync(casePath);
      expect(hash(prepared)).toBe(preparedSha);expect(hash(caseBytes)).toBe(caseSha);
      const host=reviewHost(parseZuiDocument(prepared.toString('utf8')).document)!;
      expect(host).toBeDefined();
      const before=encodeZuiMetadata(host),graph=identityGraph(host),actual=importedActualNormalReviewCase(host)!;
      expect(actual).toEqual(JSON.parse(caseBytes.toString('utf8')));
      const evidence=actual.data['normal_window_evidence'] as {viewportToolbarPublication:{nodes:ToolbarSourceNode[]}};
      const raw=JSON.stringify(evidence),mapping=viewportToolbarSourceMapping(host,evidence,actual.dpi)!;
      expect(mapping.packet.groupNodeIds.map(id=>mapping.sourceByNativeId.get(id))).toEqual(viewportToolbarContract(host)!.groups);
      expect(mapping.sourceByNativeId.size).toBe(33);expect(encodeZuiMetadata(host)).toBe(before);
      const document=reviewDocumentForCase(host,actual);
      const plan=planViewportToolbar(document,projection(document),new Map(),actual.viewport.width,actual.viewport.height)!;
      expect(evidence.viewportToolbarPublication.nodes).toHaveLength(33);
      expect(plan.sourceByNativeId!.size).toBe(33);expect(plan.sourceOnlyNativeNodeIds!.size).toBe(0);
      expect(plan.publication).toEqual(evidence.viewportToolbarPublication);
      expect(identityGraph(document)).toEqual(graph);expect(encodeZuiMetadata(host)).toBe(before);
      const fresh=decodeZuiMetadata(before),saved=reviewCaseFromSavedMetadata(fresh,JSON.stringify(actual))!;
      const reopened=reviewDocumentForCase(fresh,saved),reopenedPlan=planViewportToolbar(reopened,projection(reopened),new Map(),actual.viewport.width,actual.viewport.height)!;
      expect(reopenedPlan.sourceByNativeId!.size).toBe(33);expect(reopenedPlan.sourceOnlyNativeNodeIds!.size).toBe(0);
      expect(saved).toEqual(actual);expect(identityGraph(reopened)).toEqual(graph);
      expect(JSON.stringify(evidence)).toBe(raw);expect(encodeZuiMetadata(fresh)).toBe(before);
      expect(hash(readFileSync(preparedPath))).toBe(preparedSha);expect(hash(readFileSync(casePath))).toBe(caseSha);
    });
  });
});

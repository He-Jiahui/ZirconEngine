import { importedActualNormalReviewCase, reviewCaseFromSavedMetadata, reviewDocumentForCase } from './zui-review-case';
import { encodeZuiMetadata,decodeZuiMetadata,parseZuiDocument,serializeZuiDocument } from './zui-document';
import { reviewHost } from './zui-review-host';
import type { LayoutReviewCaseMessage } from '../model';
import { bindFontResources,fontResourcesForDocument,parseFontResourceManifest } from './penpot-font-selection';
import { deriveViewportToolbarReview,planViewportToolbar } from './zui-viewport-toolbar';
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
    const wrong=observed();wrong.evidence.viewportToolbarPublication.nodes.find(node=>node.nodeId==='translate')!.instancePath='["wrong-call-site"]';
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
});

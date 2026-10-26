import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { deflateSync } from 'node:zlib';
import { CASE_MAX_BYTES, decodeScenePng, observedSubmittedCommands, runtimeScenePacket, sceneRgba, validateSceneCommand } from './normal-workbench-case-producer.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const encoded = value => Buffer.from(JSON.stringify(value));
function chunk(kind, body) {
  const type = Buffer.from(kind), output = Buffer.alloc(body.length + 12); output.writeUInt32BE(body.length); type.copy(output,4); body.copy(output,8);
  let crc = 0xffffffff;
  for (const byte of Buffer.concat([type,body])) { crc ^= byte; for (let bit=0;bit<8;bit++)crc=crc&1?(crc>>>1)^0xedb88320:crc>>>1; }
  output.writeUInt32BE((crc^0xffffffff)>>>0,body.length+8);return output;
}
function png(rgba, size=[1,1], filter=0) {
  const header=Buffer.alloc(13);header.writeUInt32BE(size[0]);header.writeUInt32BE(size[1],4);header[8]=8;header[9]=6;
  const rows=Buffer.alloc((size[0]*4+1)*size[1]);
  for(let y=0;y<size[1];y++){rows[y*(size[0]*4+1)]=filter;for(let x=0;x<size[0]*4;x++){
    const left=x>=4?rgba[y*size[0]*4+x-4]:0,up=y?rgba[(y-1)*size[0]*4+x]:0,diagonal=y&&x>=4?rgba[(y-1)*size[0]*4+x-4]:0;
    const p=left+up-diagonal,d=[Math.abs(p-left),Math.abs(p-up),Math.abs(p-diagonal)];
    const predictor=[0,left,up,Math.floor((left+up)/2),[left,up,diagonal][d.indexOf(Math.min(...d))]][filter];
    rows[y*(size[0]*4+1)+x+1]=(rgba[y*size[0]*4+x]-predictor)&255;
  }}
  const compressed=deflateSync(rows), half=Math.floor(compressed.length/2);
  return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('tEXt',Buffer.from('Unit\0PNG row filters')),
    chunk('IDAT',compressed.subarray(0,half)),chunk('IDAT',compressed.subarray(half)),chunk('IEND',Buffer.alloc(0))]);
}
function fixture() {
  // Small unit packets retain real authority/identity structure; they are not GPU acceptance evidence.
  const ownerPath='res://ui/editor/unreal/level_editor_window.zui', viewportPath='res://ui/editor/unreal/components/viewport.zui';
  const invocation=JSON.stringify([{sourcePath:ownerPath,sourceNodeId:'viewport'}]);
  const owner={sourcePath:viewportPath,sourceNodeId:'scene_media',instancePath:invocation},viewportRoot={...owner,sourceNodeId:'root'};
  const sourceRef={authority:'surface-node-owner',primitiveIdentity:false,treeId:`template.v2.${ownerPath}`,generation:11,nodeId:3,nodeCommandIndex:0,fragmentIndex:0};
  const bounds={x:0,y:50,width:1,height:1},source={...owner,controlId:'WorkbenchViewportSurface',sourceRef,parentNodeId:'viewport',parentSourcePath:viewportPath,parentSourceNodeId:'viewport',parentInstancePath:invocation};
  const command={commandIndex:4,source,bounds,clip:bounds,paint:{kind:'image',resourceKey:'viewport:unit',generation:1,width:1,height:1,residentAfterSuccessfulPresent:true,hasUploadBytes:false}};
  // R57's unrelated brush Source/indices; unit pixels are not GPU acceptance evidence.
  const dockImages=[[78,6,0,0],[81,9,0,0],[84,12,0,0],[94,6,1,1],[98,9,1,1],[102,12,1,1],[104,6,2,0],[106,9,2,0],[108,12,2,0]]
    .map(([commandIndex,nodeId,nodeCommandIndex,fragmentIndex],index)=>({...command,commandIndex,
      source:{sourcePath:null,sourceNodeId:null,instancePath:null,controlId:`DockTab${index%3}`,
        sourceRef:{authority:'surface-node-owner',primitiveIdentity:false,treeId:'host.bottom.dock.header',generation:3,nodeId,nodeCommandIndex,fragmentIndex}}}));
  const sourceNullImages=[284,286,288].map(commandIndex=>({...command,commandIndex,source:null}));
  const cursor=Object.fromEntries(['structure','geometry','interaction','viewport','hitTest','theme','diagnostics'].map(key=>[key,1]));
  const presentation={stateFingerprint:'d'.repeat(64)},managed={projectRoot:'E:/unit',sources:[]};
  const publication={authority:'normal-retained-workbench-projection',observed:true,presentationCursor:cursor,presentedFrameCount:4,workbenchPresentation:presentation,managedSceneFingerprint:managed};
  const root={sourcePath:ownerPath,sourceNodeId:'root',instancePath:[],nodeId:'root',surfaceNodeId:1,component:'Overlay',controlId:'WorkbenchWindowRoot',parent:null};
  const authoredRoot={...viewportRoot,nodeId:'viewport-root',surfaceNodeId:4,component:'VerticalGroup',controlId:'WorkbenchMainBandViewportPanel',parent:root};
  const viewport={...owner,sourceNodeId:'viewport',nodeId:'viewport',surfaceNodeId:2,component:'Overlay',controlId:'UnrealSceneViewViewport',parent:authoredRoot};
  const image={...owner,nodeId:'scene',surfaceNodeId:3,component:'SceneViewViewport',controlId:'WorkbenchViewportSurface',parent:viewport};
  for(const row of [root,authoredRoot,viewport,image]){const {parent,...fields}=row;row.projected={...fields,parentId:parent?.nodeId??null};}
  const hold={schema:'dev.zircon.editor.normal-capture-hold',version:1,authority:'normal-successful-gpu-present',status:'held',pixelFrameVerified:false,
    nonce:'unit-nonce1',pid:7,caseSha256:'a'.repeat(64),physicalSize:[1,1],presentationCursor:'actual-unit-cursor',presentedFrameCount:4,acquiredAtUnixMs:10,expiresAtUnixMs:100};
  const normal={captureSequence:5,presentationCursor:cursor,presentedFrameCount:4,commandStreamFullRebuild:true,commands:[command,...dockImages,...sourceNullImages],captureHold:hold,
    workbenchStatePublication:publication,arrangedInventoryPublication:{schema:'dev.zircon.editor.normal-arranged-inventory',version:1,
      authority:'normal-retained-runtime-surfaces',observed:true,complete:false,presentationCursor:cursor,presentedFrameCount:4,
      surfaces:[{schema:'dev.zircon.editor.normal-surface-arranged-inventory',version:1,coordinateSpace:'surface-logical',frameAuthority:'retained-host-projection',scope:'workbench-window',documentId:ownerPath,sourceFrameGeneration:13,rootIds:[1],nodes:[root,authoredRoot,viewport,image]}]}};
  const profile={presenter_backend:'gpu',capture_sequence:5,winit_scale_factor:1,window_client_size:{width:1,height:1},window_client_logical_size:{width:1,height:1},normal_window_evidence:normal,
    submitted_gpu_text:{presentedFrameCount:4,commandStreamFullRebuild:true,coverage:'submitted-command-stream'}};
  const raw=Buffer.from([20,128,230,255]),scenePng=png(raw),ticket={device_id:1,generation:2,queue_class:'Graphics',sequence:3};
  const receipt=request=>({request,kind:'Texture',terminal:'Succeeded',byte_len:256,frame_key:{device_id:1,generation:2,submission:ticket}});
  const scene={schema:'dev.zircon.editor.normal-scene-image-readback',version:1,authority:'normal-actual-submitted-runtime-image-readback',owner,viewportRoot,
    surfaceKey:'actual-unit-surface',paneKind:'Scene',commandIndex:4,sourceRef,commandBounds:bounds,commandClip:bounds,atlasUv:null,overlayPresent:true,
    resourceKey:'viewport:unit',resourceGeneration:1,physicalSize:[1,1],sourceFormat:'Rgba8UnormSrgb',sourceByteLayout:'rgba8',colorEncoding:'srgb',alphaMode:'opaque',opaquePixels:true,
    readbackCompleted:true,failure:null,rawRowBytes:4,rawByteLength:4,rawSha256:sha(raw),rgba8Sha256:sha(raw),pngByteLength:scenePng.length,pngSha256:sha(scenePng),pngColorEncoding:'srgb',
    conversion:{algorithm:'identity-rgba8-srgb',inputSha256:sha(raw),outputSha256:sha(raw)},receipt:receipt(9)};
  const readback={...hold,schema:'dev.zircon.editor.normal-frame-readback',authority:'normal-actual-gpu-packet-and-readback-receipt',hwnd:9,gpuReadbackCompleted:true,failure:null,
    colorEncoding:'srgb',opaquePixels:true,sourceFormat:'Rgba8UnormSrgb',sourceByteLayout:'rgba8',rawByteLength:4,rawSha256:sha(raw),submission:ticket,receipt:receipt(8),sceneImage:scene};
  const bootstrapCase={host:'editor',sourcePath:'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui',dpi:1,viewport:{width:1,height:1},
    data:{workbenchPresentation:presentation,managedSceneFingerprint:managed,normal_window_evidence:{presentationCursor:cursor}}};
  const bootstrapBytes=encoded(bootstrapCase),proof={schema:'dev.zircon.editor.normal-frame-pixel-association',version:1,pixelFrameVerified:true,wholeUiAccepted:false,
    captureMethod:'win32-desktop-bitblt-gdiflush-own-client',
    nonce:hold.nonce,pid:7,hwnd:9,caseSha256:hold.caseSha256,caseJsonSha256:sha(bootstrapBytes),holdBefore:hold,holdAfter:hold,
    physicalSize:[1,1],presentationCursor:hold.presentationCursor,presentedFrameCount:4,capturedAtUnixMs:20,normalReadbackReceipt:readback,
    actualWinitScaleFactor:1,actualWindowDpi:96,rawGpuSha256:sha(raw),decodedRgba8Sha256:sha(raw),clientPngSha256:sha(scenePng)};
  return {bootstrapCase,bootstrapBytes,bootstrapCaseSha256:hold.caseSha256,proof,proofBytes:encoded(proof),profile,sceneRaw:raw,scenePng,windowRaw:raw,clientPng:scenePng,
    bootstrapPath:'E:/unit/bootstrap.json',proofPath:'E:/unit/association.json'};
}

test('preserves actual cached generation 11 in arranged generation 13, raw case SHA and original nonce receipt',()=>{
  const input=fixture(),before=structuredClone(input.proof),packet=runtimeScenePacket(input);
  assert.equal(packet.bootstrapCase.caseJsonSha256,sha(input.bootstrapBytes));assert.equal(packet.bootstrapCase.caseSha256,'a'.repeat(64));
  assert.equal(packet.image.pngSha256,sha(input.scenePng));assert.deepEqual(packet.bootstrapPixelAssociation.observed,before);
  assert.equal(packet.image.base64,input.scenePng.toString('base64'));assert.equal(packet.sceneVerified,undefined);
});
test('actual scene root identity joins the three authored layers through official aliases without rewriting the packet',()=>{
  const input=fixture(),scene=input.proof.normalReadbackReceipt.sceneImage,packetBefore=encoded(scene);
  const canonical=path=>path.startsWith('res://ui/editor/')?'zircon_editor/assets/'+path.slice(6):path;
  const alias=value=>{value.sourcePath=canonical(value.sourcePath);const steps=typeof value.instancePath==='string'?JSON.parse(value.instancePath):value.instancePath;
    value.instancePath=JSON.stringify(steps.map(step=>({...step,sourcePath:canonical(step.sourcePath)})));};
  const rows=input.profile.normal_window_evidence.arrangedInventoryPublication.surfaces[0].nodes;
  for(const row of rows){alias(row);alias(row.projected);}
  const command=validateSceneCommand(scene,input.profile);
  assert.equal(command.commandIndex,4);assert.equal(scene.viewportRoot.sourceNodeId,'root');
  assert.equal(rows.find(row=>row.nodeId==='viewport').parent.nodeId,'viewport-root');
  assert.deepEqual(encoded(scene),packetBefore);
});
test('actual scene root identity rejects root/viewport interchange, wrong controls, kinds, aliases and full parent bindings',()=>{
  for(const kind of ['root is inner viewport','root invocation','root control','root kind','viewport control','viewport kind',
    'media control','media kind','command control','viewport skips root','media skips viewport','duplicate root','projected control','source suffix','invocation alias']){
    const input=fixture(),scene=input.proof.normalReadbackReceipt.sceneImage;
    const rows=input.profile.normal_window_evidence.arrangedInventoryPublication.surfaces[0].nodes;
    const root=rows.find(row=>row.nodeId==='viewport-root'),viewport=rows.find(row=>row.nodeId==='viewport'),media=rows.find(row=>row.nodeId==='scene');
    if(kind==='root is inner viewport')scene.viewportRoot={...scene.viewportRoot,sourceNodeId:'viewport'};
    if(kind==='root invocation')scene.viewportRoot={...scene.viewportRoot,instancePath:'[]'};
    if(kind==='root control')root.controlId='UnrealSceneViewViewport';
    if(kind==='root kind')root.component='Overlay';
    if(kind==='viewport control')viewport.controlId='WorkbenchMainBandViewportPanel';
    if(kind==='viewport kind')viewport.component='VerticalGroup';
    if(kind==='media control')media.controlId='another';
    if(kind==='media kind')media.component='Overlay';
    if(kind==='command control')input.profile.normal_window_evidence.commands[0].source.controlId='another';
    if(kind==='viewport skips root'){viewport.parent=rows[0];viewport.projected.parentId=rows[0].nodeId;}
    if(kind==='media skips viewport'){media.parent=root;media.projected.parentId=root.nodeId;}
    if(kind==='duplicate root')rows.push({...structuredClone(root),nodeId:'another-root',surfaceNodeId:5});
    if(kind==='projected control')viewport.projected.controlId='another';
    if(kind==='source suffix')scene.viewportRoot={...scene.viewportRoot,sourcePath:'foreign/ui/editor/unreal/components/viewport.zui'};
    if(kind==='invocation alias')scene.viewportRoot={...scene.viewportRoot,instancePath:scene.viewportRoot.instancePath.replace('res://ui/editor/','res://other/editor/')};
    assert.throws(()=>validateSceneCommand(scene,input.profile),undefined,kind);
  }
});
test('actual scene command selection preserves unrelated R57 DockTab and source-null image brushes',()=>{
  const input=fixture(),scene=input.proof.normalReadbackReceipt.sceneImage,commands=input.profile.normal_window_evidence.commands;
  const before=structuredClone(commands);
  assert.deepEqual(commands.filter(command=>command.source?.sourceRef?.treeId==='host.bottom.dock.header').map(command=>command.commandIndex),
    [78,81,84,94,98,102,104,106,108]);
  assert.deepEqual(commands.filter(command=>command.source===null).map(command=>command.commandIndex),[284,286,288]);
  assert.equal(validateSceneCommand(scene,input.profile),commands[0]);
  assert.equal(runtimeScenePacket(input).image.pngSha256,scene.pngSha256);
  assert.deepEqual(commands,before);
});
test('actual scene command selection rejects target identity gaps and duplicate owners among unrelated images',()=>{
  for(const kind of ['missing invocation','null invocation','malformed invocation','partial invocation step','wrong alias','wrong node','wrong invocation',
    'partial candidate beside target','duplicate owner']){
    const input=fixture(),scene=input.proof.normalReadbackReceipt.sceneImage,commands=input.profile.normal_window_evidence.commands;
    const source=commands[0].source;
    if(kind==='missing invocation')delete source.instancePath;
    if(kind==='null invocation')source.instancePath=null;
    if(kind==='malformed invocation')source.instancePath='[';
    if(kind==='partial invocation step')source.instancePath=JSON.stringify([{sourcePath:'res://ui/editor/unreal/level_editor_window.zui'}]);
    if(kind==='wrong alias')source.sourcePath='res://other/editor/unreal/components/viewport.zui';
    if(kind==='wrong node')source.sourceNodeId='toolbar';
    if(kind==='wrong invocation')source.instancePath='[]';
    if(kind==='partial candidate beside target'||kind==='duplicate owner'){
      const extra=structuredClone(commands[0]);extra.commandIndex=5;extra.source.sourcePath='zircon_editor/assets/ui/editor/unreal/components/viewport.zui';
      if(kind==='partial candidate beside target')delete extra.source.instancePath;
      commands.push(extra);
    }
    assert.throws(()=>validateSceneCommand(scene,input.profile),undefined,kind);
  }
});
test('forwards the same actual command array unchanged and rejects partial or duplicate indices',()=>{
  const input=fixture(),normal=input.profile.normal_window_evidence,before=structuredClone(normal.commands);
  assert.equal(observedSubmittedCommands(normal),normal.commands);
  assert.deepEqual(normal.commands,before);
  for(const commands of [undefined,null,[null],[{commandIndex:true}],[{commandIndex:-1}],[{commandIndex:2**53}],
    [normal.commands[0],structuredClone(normal.commands[0])]]){
    assert.throws(()=>observedSubmittedCommands({...normal,commands}));
  }
});
test('decodes all official row filters, multiple contiguous IDAT and bounded ancillary chunks',()=>{
  const bytes=Buffer.from([1,20,30,255,50,60,200,255,5,6,7,255,80,100,220,255]);
  for(let filter=0;filter<=4;filter++)assert.deepEqual(decodeScenePng(png(bytes,[2,2],filter),[2,2]),bytes);
  const valid=png(bytes,[2,2],4);assert.throws(()=>decodeScenePng(Buffer.concat([valid,Buffer.from('x')]),[2,2]));
  const bad=Buffer.from(valid);bad[40]^=1;assert.throws(()=>decodeScenePng(bad,[2,2]));
});
test('rejects partial, stale, duplicate, cross-owner and altered bootstrap evidence',()=>{
  for(const kind of ['window only','case raw hash','whole ui','nonce','all7','duplicate command','wrong owner','wrong generation','bad PNG','scene submission','partial alpha','oversize','unsupported desktop','partial source reference','invalid command rectangle','duplicate actual id','wrong root invocation','receipt generation rollback','command generation rollback','foreign matching tree','future source frame','wrong physical owner','wrong physical parent']){
    const input=fixture(),scene=input.proof.normalReadbackReceipt.sceneImage;
    if(kind==='window only')delete input.proof.normalReadbackReceipt.sceneImage;
    if(kind==='case raw hash')input.bootstrapBytes=Buffer.from('{}');
    if(kind==='whole ui')input.proof.wholeUiAccepted=true;
    if(kind==='nonce')input.proof.nonce='another';
    if(kind==='all7')input.bootstrapCase.data.normal_window_evidence.presentationCursor={theme:99};
    if(kind==='duplicate command')input.profile.normal_window_evidence.commands.push(structuredClone(input.profile.normal_window_evidence.commands[0]));
    if(kind==='wrong owner')scene.owner={...scene.owner,instancePath:'[]'};
    if(kind==='wrong generation')scene.resourceGeneration=2;
    if(kind==='bad PNG'){input.scenePng=Buffer.from(input.scenePng);input.scenePng[20]^=1;}
    if(kind==='scene submission')scene.receipt={...scene.receipt,frame_key:{...scene.receipt.frame_key,submission:{device_id:2,generation:2,queue_class:'Graphics',sequence:3}}};
    if(kind==='partial alpha')scene.alphaMode='premultiplied';
    if(kind==='oversize')input.bootstrapBytes=Buffer.alloc(CASE_MAX_BYTES+1);
    if(kind==='unsupported desktop')input.proof.captureMethod='disk-image';
    if(kind==='partial source reference')delete scene.sourceRef.treeId;
    if(kind==='invalid command rectangle')scene.commandBounds.width=0;
    if(kind==='duplicate actual id')input.profile.normal_window_evidence.arrangedInventoryPublication.surfaces[0].nodes[1].surfaceNodeId=3;
    if(kind==='wrong root invocation')input.profile.normal_window_evidence.arrangedInventoryPublication.surfaces[0].nodes[0].instancePath=[{sourcePath:'res://ui/editor/unreal/level_editor_window.zui',sourceNodeId:'another'}];
    if(kind==='receipt generation rollback')scene.sourceRef={...scene.sourceRef,generation:10};
    if(kind==='command generation rollback')input.profile.normal_window_evidence.commands[0].source.sourceRef={...scene.sourceRef,generation:10};
    if(kind==='foreign matching tree')scene.sourceRef.treeId='template.v2.other';
    if(kind==='future source frame')scene.sourceRef.generation=14;
    if(kind==='wrong physical owner')scene.sourceRef.nodeId=999;
    if(kind==='wrong physical parent')input.profile.normal_window_evidence.commands[0].source.parentNodeId='another';
    input.proofBytes=encoded(input.proof);
    assert.throws(()=>runtimeScenePacket(input),undefined,kind);
  }
});
test('linear GPU sampling values require the explicit conversion and cannot be relabelled sRGB',()=>{
  const input=fixture(),raw=Buffer.from([0,128,255,255]),rgba=Buffer.from([0,188,255,255]);
  const scene={...input.proof.normalReadbackReceipt.sceneImage,sourceFormat:'Rgba8Unorm',sourceByteLayout:'rgba8',colorEncoding:'linear',rawSha256:sha(raw),rgba8Sha256:sha(rgba),
    conversion:{algorithm:'linear8-to-rgba8-srgb-iec61966-2-1',inputSha256:sha(raw),outputSha256:sha(rgba)}};
  assert.deepEqual(sceneRgba(scene,raw),rgba);assert.throws(()=>sceneRgba({...scene,colorEncoding:'srgb'},raw));
  assert.throws(()=>sceneRgba({...scene,sourceFormat:'constructor'},raw));
});

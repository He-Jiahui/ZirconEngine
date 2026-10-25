import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { lstat, readFile, readdir, realpath, writeFile } from 'node:fs/promises';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { inflateSync } from 'node:zlib';

const sourcePath = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const canonicalRepo = 'E:/Git/ZirconEngine';
const cursorFields = ['structure', 'geometry', 'interaction', 'viewport', 'hitTest', 'theme', 'diagnostics'];
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const record = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
const uint = (value) => Number.isSafeInteger(value) && value >= 0;
const jsonBytes = (value) => Buffer.from(JSON.stringify(value, null, 2) + '\n');
const physicalKey = (path) => path.replaceAll('/', '\\').replace(/^\\\\\?\\/, '').replace(/[\\]+$/, '').toLowerCase();
const requireValue = (condition, reason) => { if (!condition) throw new Error(reason); };
export const CASE_MAX_BYTES = 8 * 1024 * 1024;
const MEDIA_MAX_BYTES = 24 * 1024 * 1024;
const sceneSource = 'zircon_editor/assets/ui/editor/unreal/components/viewport.zui';

/** Preserve the actual seed packet, including its immutable cached source refs. */
export function observedSubmittedCommands(evidence) {
  requireValue(record(evidence) && Array.isArray(evidence.commands), 'Actual submitted normal commands are absent');
  const indices = new Set();
  for (const command of evidence.commands) {
    requireValue(record(command) && uint(command.commandIndex) && !indices.has(command.commandIndex), 'Actual normal command indices are incomplete or duplicated');
    indices.add(command.commandIndex);
  }
  return evidence.commands;
}

function sceneSourceAlias(value) {
  return value.startsWith('res://ui/editor/') ? 'zircon_editor/assets/' + value.slice(6) : value;
}
function sceneSourcePath(value) {
  requireValue(typeof value === 'string' && value.length > 0, 'Scene source path is incomplete');
  const path = sceneSourceAlias(value);
  requireValue(!path.includes('\\') && !path.split('/').some(part => part === '.' || part === '..'), 'Invalid Scene source alias');
  return path;
}
function sourceTuple(value) {
  requireValue(record(value) && typeof value.sourcePath === 'string' && value.sourcePath.length > 0 &&
    typeof value.sourceNodeId === 'string' && value.sourceNodeId.length > 0, 'Scene owner is incomplete');
  const steps = typeof value.instancePath === 'string' ? JSON.parse(value.instancePath) : value.instancePath;
  requireValue(Array.isArray(steps) && steps.length <= 64 && steps.every(step => record(step) &&
    Object.keys(step).length === 2 && typeof step.sourcePath === 'string' && step.sourcePath &&
    typeof step.sourceNodeId === 'string' && step.sourceNodeId), 'Scene invocation is incomplete');
  return JSON.stringify([sceneSourcePath(value.sourcePath), value.sourceNodeId,
    steps.map(step => ({sourcePath:sceneSourcePath(step.sourcePath), sourceNodeId:step.sourceNodeId}))]);
}
function crc32(bytes) {
  let result = 0xffffffff;
  for (const byte of bytes) {
    result ^= byte;
    for (let bit = 0; bit < 8; bit++) result = result & 1 ? (result >>> 1) ^ 0xedb88320 : result >>> 1;
  }
  return (result ^ 0xffffffff) >>> 0;
}
export function decodeScenePng(bytes, size, inputLimit = CASE_MAX_BYTES) {
  requireValue(Buffer.isBuffer(bytes) && bytes.length <= inputLimit && Array.isArray(size) && size.length === 2 &&
    size.every(value => uint(value) && value > 0 && value <= 8192) && size[0] * size[1] * 4 <= MEDIA_MAX_BYTES &&
    bytes.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10])), 'Scene PNG signature or byte extent differs');
  const kinds = [], payload = []; let offset = 8, ended = false;
  while (offset < bytes.length) {
    requireValue(kinds.length < 1024 && offset + 12 <= bytes.length, 'Scene PNG chunk extent differs');
    const length = bytes.readUInt32BE(offset), end = offset + length + 12;
    requireValue(end <= bytes.length, 'Scene PNG chunk is truncated');
    const kindBytes = bytes.subarray(offset + 4, offset + 8), kind = kindBytes.toString('ascii'), body = bytes.subarray(offset + 8, end - 4);
    requireValue(/^[A-Za-z]{4}$/.test(kind) && !(kindBytes[2] & 32) &&
      crc32(bytes.subarray(offset + 4, end - 4)) === bytes.readUInt32BE(end - 4), 'Scene PNG chunk or CRC differs');
    if (!kinds.length) {
      requireValue(kind === 'IHDR' && length === 13 && body.readUInt32BE(0) === size[0] && body.readUInt32BE(4) === size[1] &&
        body.subarray(8).equals(Buffer.from([8,6,0,0,0])), 'Scene PNG format differs');
    } else if (kind === 'IHDR') throw new Error('Duplicate scene IHDR');
    else if (kind === 'IDAT') { requireValue(!ended, 'Scene IDAT chunks are not contiguous'); payload.push(body); }
    else if (kind === 'IEND') requireValue(length === 0 && end === bytes.length && kinds.includes('IDAT'), 'Scene PNG end differs');
    else {
      requireValue((kindBytes[0] & 32) && !['iCCP','cICP','mDCV','cLLI','tRNS','acTL','fcTL','fdAT'].includes(kind), 'Unsupported scene PNG critical/color chunk');
      if (kinds.includes('IDAT')) ended = true;
      if (kind === 'gAMA') requireValue(length === 4 && body.readUInt32BE(0) === 45455 && !kinds.includes('IDAT') && !kinds.includes(kind), 'Scene PNG transfer differs');
      if (kind === 'sRGB') requireValue(length === 1 && body[0] <= 3 && !kinds.includes('IDAT') && !kinds.includes(kind), 'Scene PNG transfer differs');
      if (kind === 'cHRM') requireValue(length === 32 && [31270,32900,64000,33000,30000,60000,15000,6000].every((value,index)=>body.readUInt32BE(index*4)===value) && !kinds.includes('IDAT') && !kinds.includes(kind),'Scene PNG primaries differ');
    }
    kinds.push(kind); offset = end;
  }
  requireValue(kinds.at(-1) === 'IEND', 'Scene PNG is incomplete');
  const stride = size[0] * 4, expected = (stride + 1) * size[1], compressed = Buffer.concat(payload);
  const decoded = inflateSync(compressed, {maxOutputLength:expected + 1, info:true});
  requireValue(decoded.buffer.length === expected && decoded.engine.bytesWritten === compressed.length, 'Scene PNG inflated extent or trailing stream differs');
  const output = Buffer.alloc(stride * size[1]);
  for (let row = 0; row < size[1]; row++) {
    const filter = decoded.buffer[row * (stride + 1)]; requireValue(filter <= 4, 'Scene PNG filter differs');
    for (let x = 0; x < stride; x++) {
      const left = x >= 4 ? output[row * stride + x - 4] : 0, up = row ? output[(row - 1) * stride + x] : 0,
        diagonal = row && x >= 4 ? output[(row - 1) * stride + x - 4] : 0;
      let predictor = [0,left,up,Math.floor((left+up)/2)][filter];
      if (filter === 4) { const p=left+up-diagonal, distances=[Math.abs(p-left),Math.abs(p-up),Math.abs(p-diagonal)]; predictor=[left,up,diagonal][distances.indexOf(Math.min(...distances))]; }
      output[row * stride + x] = (decoded.buffer[row * (stride + 1) + x + 1] + predictor) & 255;
    }
  }
  return output;
}
export function sceneRgba(scene, raw) {
  const size = scene.physicalSize, formats = {Rgba8UnormSrgb:['rgba8','srgb'],Bgra8UnormSrgb:['bgra8','srgb'],Rgba8Unorm:['rgba8','linear'],Bgra8Unorm:['bgra8','linear']};
  requireValue(Array.isArray(size) && size.length === 2 && size.every(value=>uint(value)&&value>0&&value<=8192) &&
    raw.length === size[0] * size[1] * 4 && raw.length <= MEDIA_MAX_BYTES && raw.length === scene.rawByteLength &&
    scene.rawRowBytes === size[0] * 4 && sha(raw) === scene.rawSha256 && scene.alphaMode === 'opaque' && scene.opaquePixels === true &&
    scene.pngColorEncoding === 'srgb' && scene.readbackCompleted === true && scene.failure === null, 'Scene raw extent, alpha or completion differs');
  const format = Object.hasOwn(formats,scene.sourceFormat) ? formats[scene.sourceFormat] : undefined;
  requireValue(format && format[0] === scene.sourceByteLayout && format[1] === scene.colorEncoding, 'Scene actual GPU view format differs');
  const output = Buffer.from(raw);
  let algorithm = format[0] === 'rgba8' ? 'identity-rgba8-srgb' : 'bgra8-srgb-to-rgba8-srgb';
  const lut = Array.from({length:256}, (_,value)=> {const linear=value/255;return Math.floor(255*(linear<=.0031308?12.92*linear:1.055*linear**(1/2.4)-.055)+.5);});
  for(let index=0; index<raw.length; index+=4) {
    requireValue(raw[index+3] === 255, 'Scene contains nonopaque actual pixels');
    if(format[0] === 'bgra8') {output[index]=raw[index+2];output[index+2]=raw[index];}
    if(format[1] === 'linear') for(let channel=0;channel<3;channel++)output[index+channel]=lut[output[index+channel]];
  }
  if(format[1] === 'linear')algorithm='linear8-to-rgba8-srgb-iec61966-2-1';
  assert.deepEqual(scene.conversion,{algorithm,inputSha256:sha(raw),outputSha256:sha(output)},'Scene conversion evidence differs');
  requireValue(scene.rgba8Sha256 === sha(output), 'Scene RGBA SHA differs'); return output;
}
export function validateSceneCommand(scene, profile) {
  requireValue(record(scene) && scene.schema === 'dev.zircon.editor.normal-scene-image-readback' && scene.version === 1 &&
    scene.authority === 'normal-actual-submitted-runtime-image-readback' && scene.paneKind === 'Scene' &&
    typeof scene.surfaceKey === 'string' && scene.surfaceKey && scene.atlasUv === null && typeof scene.overlayPresent === 'boolean', 'No exact runtime scene image');
  requireValue(record(scene.sourceRef) && Object.keys(scene.sourceRef).length === 7 &&
    scene.sourceRef.authority === 'surface-node-owner' && scene.sourceRef.primitiveIdentity === false &&
    typeof scene.sourceRef.treeId === 'string' && scene.sourceRef.treeId.length > 0 &&
    ['generation','nodeId','nodeCommandIndex','fragmentIndex'].every(key=>uint(scene.sourceRef[key])) &&
    [scene.commandBounds,scene.commandClip].every(bounds=>record(bounds) && Object.keys(bounds).length === 4 &&
      ['x','y','width','height'].every(key=>Number.isFinite(bounds[key])) && bounds.width > 0 && bounds.height > 0),
    'Scene source reference or actual command rectangles are incomplete');
  requireValue(record(scene.owner) && record(scene.viewportRoot) && Object.keys(scene.owner).length===3 && Object.keys(scene.viewportRoot).length===3 &&
    typeof scene.owner.instancePath==='string' && typeof scene.viewportRoot.instancePath==='string', 'Scene owner must preserve the actual native invocation string');
  const owner = sourceTuple(scene.owner), root = sourceTuple(scene.viewportRoot);
  const viewport = sourceTuple({...scene.owner,sourceNodeId:'viewport'});
  requireValue(sceneSourcePath(scene.owner.sourcePath) === sceneSource && scene.owner.sourceNodeId === 'scene_media' &&
    sceneSourcePath(scene.viewportRoot.sourcePath) === sceneSource && scene.viewportRoot.sourceNodeId === 'root' &&
    sourceTuple({...scene.owner,sourceNodeId:'root'}) === root, 'Scene authored owner differs');
  const commands = observedSubmittedCommands(profile.normal_window_evidence);
  // Other normal image brushes can legitimately have no authored Source.
  // Once path/node match, a partial invocation is an error, even beside a valid target.
  const matches = commands.filter(command=>command.paint?.kind==='image' && record(command.source) &&
    command.source.sourceNodeId===scene.owner.sourceNodeId && typeof command.source.sourcePath==='string' &&
    sceneSourceAlias(command.source.sourcePath)===sceneSource && sourceTuple(command.source)===owner);
  requireValue(matches.length === 1, 'Scene needs one exact actual image command');
  const command = matches[0], paint = command.paint;
  requireValue(uint(scene.commandIndex) && command.commandIndex === scene.commandIndex &&
    paint.residentAfterSuccessfulPresent === true && paint.hasUploadBytes === false && paint.resourceKey === scene.resourceKey &&
    uint(scene.resourceGeneration) && paint.generation === scene.resourceGeneration &&
    scene.physicalSize?.[0] === paint.width && scene.physicalSize?.[1] === paint.height &&
    command.source.controlId === 'WorkbenchViewportSurface' &&
    sourceTuple({sourcePath:command.source.parentSourcePath,sourceNodeId:command.source.parentSourceNodeId,instancePath:command.source.parentInstancePath}) === viewport,
    'Scene command resource, size or viewport parent differs');
  for(const [actual,expected] of [[scene.sourceRef,command.source.sourceRef],[scene.commandBounds,command.bounds],[scene.commandClip,command.clip]])
    assert.deepEqual(actual,expected,'Scene frame-local command differs');
  const normal=profile.normal_window_evidence, inventory=normal.arrangedInventoryPublication;
  requireValue(record(inventory) && inventory.schema==='dev.zircon.editor.normal-arranged-inventory' && inventory.version===1 &&
    inventory.authority==='normal-retained-runtime-surfaces' && inventory.observed===true && typeof inventory.complete==='boolean' &&
    uint(inventory.presentedFrameCount) && inventory.presentedFrameCount===normal.presentedFrameCount,'Scene inventory belongs to another frame');
  assert.deepEqual(inventory.presentationCursor,normal.presentationCursor,'Scene inventory all7 differs');
  const surfaces=inventory.surfaces?.filter(surface=>surface.scope==='workbench-window');
  requireValue(surfaces?.length===1 && surfaces[0].schema==='dev.zircon.editor.normal-surface-arranged-inventory' && surfaces[0].version===1 &&
    surfaces[0].coordinateSpace==='surface-logical' && surfaces[0].frameAuthority==='retained-host-projection' &&
    uint(surfaces[0].sourceFrameGeneration) && surfaces[0].documentId==='res://ui/editor/unreal/level_editor_window.zui','Scene Workbench surface is ambiguous');
  const surface=surfaces[0], byId=new Map(), surfaceIds=new Set();
  for(const row of surface.nodes??[]) {
    requireValue(record(row)&&typeof row.nodeId==='string'&&row.nodeId.length>0&&!byId.has(row.nodeId),'Scene inventory has duplicate runtime IDs');
    if(row.surfaceNodeId!==undefined&&row.surfaceNodeId!==null){requireValue(uint(row.surfaceNodeId)&&row.surfaceNodeId>0&&!surfaceIds.has(row.surfaceNodeId),'Scene inventory has invalid or duplicate actual source IDs');surfaceIds.add(row.surfaceNodeId);}
    byId.set(row.nodeId,row);
  }
  const roots=[...byId.values()].filter(row=>row.controlId==='WorkbenchWindowRoot');
  const sceneRows=key=>[...byId.values()].filter(row=>row.sourceNodeId===JSON.parse(key)[1] && typeof row.sourcePath==='string' &&
    sceneSourcePath(row.sourcePath)===sceneSource && sourceTuple(row)===key);
  const owners=sceneRows(owner), authoredRoots=sceneRows(root), viewports=sceneRows(viewport);
  requireValue(roots.length===1 && owners.length===1 && authoredRoots.length===1 && viewports.length===1 &&
    owners[0].component==='SceneViewViewport' && owners[0].controlId==='WorkbenchViewportSurface' &&
    authoredRoots[0].component==='VerticalGroup' && authoredRoots[0].controlId==='WorkbenchMainBandViewportPanel' &&
    viewports[0].component==='Overlay' && viewports[0].controlId==='UnrealSceneViewViewport' &&
    roots[0].parent===null && roots[0].component==='Overlay' &&
    sceneSourcePath(roots[0].sourcePath)===sourcePath && roots[0].sourceNodeId==='root' &&
    uint(roots[0].surfaceNodeId)&&roots[0].surfaceNodeId>0&&scene.sourceRef.authority==='surface-node-owner'&&scene.sourceRef.primitiveIdentity===false&&
    scene.sourceRef.nodeId===owners[0].surfaceNodeId && scene.sourceRef.treeId===`template.v2.${surface.documentId}` &&
    scene.sourceRef.generation>0 && scene.sourceRef.generation<=surface.sourceFrameGeneration &&
    command.source.parentNodeId===owners[0].parent?.nodeId,
    'Scene lacks its actual submitted source frame or surface owner');
  assert.deepEqual(surface.rootIds,[roots[0].surfaceNodeId]);
  let current=owners[0];const seen=new Set(), rootSteps=JSON.parse(sourceTuple(roots[0]))[2];
  assert.deepEqual(JSON.parse(owner)[2],[...rootSteps,{sourcePath,sourceNodeId:'viewport'}],'Scene invocation is not the actual Workbench viewport mount');
  requireValue(sourceTuple(owners[0].parent)===viewport && byId.get(owners[0].parent?.nodeId)===viewports[0] &&
    sourceTuple(viewports[0].parent)===root && byId.get(viewports[0].parent?.nodeId)===authoredRoots[0],
    'Actual Scene root, viewport and media ancestry differs');
  for(;;){
    requireValue(!seen.has(current.nodeId)&&uint(current.surfaceNodeId)&&current.surfaceNodeId>0,'Actual Scene ancestry is incomplete');seen.add(current.nodeId);
    assert.deepEqual(JSON.parse(sourceTuple(current))[2].slice(0,rootSteps.length),rootSteps,'Scene ancestry belongs to another full owner invocation');
    const projected=current.projected;
    requireValue(record(projected)&&projected.nodeId===current.nodeId&&projected.surfaceNodeId===current.surfaceNodeId&&
      projected.component===current.component&&projected.controlId===current.controlId&&sourceTuple(projected)===sourceTuple(current),'Scene actual source projection differs');
    if(current===roots[0]){requireValue(projected.parentId===null,'Scene root was rebound');break;}
    const parent=byId.get(current.parent?.nodeId);
    requireValue(parent&&projected.parentId===parent.nodeId&&sourceTuple(current.parent)===sourceTuple(parent),'Actual Scene owner parent differs');current=parent;
  }
  return command;
}

export function runtimeScenePacket({bootstrapCase, bootstrapBytes, bootstrapCaseSha256, proof, proofBytes, profile, sceneRaw, scenePng, windowRaw, clientPng, bootstrapPath, proofPath}) {
  requireValue(record(bootstrapCase) && !Object.hasOwn(bootstrapCase.data?.normal_window_evidence ?? {},'runtimeSceneMedia') &&
    bootstrapBytes.length <= CASE_MAX_BYTES && proofBytes.length <= CASE_MAX_BYTES &&
    proof.schema === 'dev.zircon.editor.normal-frame-pixel-association' && proof.version === 1 && proof.pixelFrameVerified === true &&
    ['win32-desktop-bitblt-gdiflush-own-client','win32-printwindow-client-renderfullcontent-gdiflush'].includes(proof.captureMethod) &&
    proof.wholeUiAccepted === false && proof.caseJsonSha256 === sha(bootstrapBytes) && proof.caseSha256 === bootstrapCaseSha256 &&
    /^[0-9a-f]{64}$/.test(bootstrapCaseSha256), 'Bootstrap case/association identity differs');
  assert.deepEqual(JSON.parse(proofBytes),proof,'Original bootstrap association differs');
  const readback=proof.normalReadbackReceipt, hold=proof.holdBefore;
  requireValue(record(readback) && record(hold) && readback.schema==='dev.zircon.editor.normal-frame-readback' && readback.version===1 &&
    readback.authority==='normal-actual-gpu-packet-and-readback-receipt' && readback.gpuReadbackCompleted===true && readback.failure===null &&
    readback.pixelFrameVerified===false && typeof proof.nonce==='string' && /^[A-Za-z0-9_-]{1,128}$/.test(proof.nonce) && uint(proof.pid) && proof.pid>0 && uint(proof.hwnd) && proof.hwnd>0 &&
    readback.hwnd===proof.hwnd && hold.schema==='dev.zircon.editor.normal-capture-hold' && hold.version===1 &&
    hold.authority==='normal-successful-gpu-present' && hold.status==='held' && hold.pixelFrameVerified===false &&
    uint(proof.capturedAtUnixMs) && uint(hold.acquiredAtUnixMs) && uint(hold.expiresAtUnixMs) &&
    hold.acquiredAtUnixMs<=proof.capturedAtUnixMs && proof.capturedAtUnixMs<hold.expiresAtUnixMs, 'Bootstrap has no successful actual held readback');
  assert.deepEqual(proof.holdAfter,hold,'Bootstrap hold changed');
  for(const key of ['nonce','pid','caseSha256','physicalSize','presentationCursor','presentedFrameCount','expiresAtUnixMs'])assert.deepEqual(readback[key],hold[key],'Bootstrap hold/readback differs');
  for(const key of ['nonce','pid','caseSha256','physicalSize','presentationCursor','presentedFrameCount'])assert.deepEqual(proof[key],hold[key],'Bootstrap association/hold differs');
  const {evidence,publication,cursor}=actualNormalSeed(profile);
  requireValue(bootstrapCase.host==='editor' && bootstrapCase.sourcePath===sourcePath && bootstrapCase.dpi===profile.winit_scale_factor,
    'Bootstrap is not the same actual Editor source or scale');
  assert.deepEqual(bootstrapCase.viewport,profile.window_client_logical_size,'Bootstrap logical viewport differs');
  assert.deepEqual(evidence.captureHold,hold,'Bootstrap profile belongs to another actual nonce');
  assert.deepEqual(profile.window_client_size,{width:proof.physicalSize[0],height:proof.physicalSize[1]});
  requireValue(profile.winit_scale_factor===proof.actualWinitScaleFactor && proof.actualWindowDpi===Math.round(profile.winit_scale_factor*96), 'Bootstrap DPI differs');
  for(const key of ['workbenchPresentation','managedSceneFingerprint'])assert.deepEqual(bootstrapCase.data[key],publication[key],'Bootstrap business/source differs');
  assert.deepEqual(bootstrapCase.data.normal_window_evidence.presentationCursor,cursor,'Bootstrap all7 differs');
  requireValue(readback.colorEncoding==='srgb' && readback.opaquePixels===true && sha(windowRaw)===readback.rawSha256 && sha(windowRaw)===proof.rawGpuSha256 &&
    windowRaw.length===readback.rawByteLength && windowRaw.length===proof.physicalSize[0]*proof.physicalSize[1]*4 &&
    sha(clientPng)===proof.clientPngSha256, 'Bootstrap whole-client byte identity differs');
  let windowRgba=Buffer.from(windowRaw);
  requireValue(['Rgba8UnormSrgb','Bgra8UnormSrgb'].includes(readback.sourceFormat) &&
    readback.sourceByteLayout===(readback.sourceFormat.startsWith('Bgra')?'bgra8':'rgba8'), 'Bootstrap window format differs');
  for(let index=0;index<windowRaw.length;index+=4){requireValue(windowRaw[index+3]===255,'Bootstrap window alpha differs');if(readback.sourceByteLayout==='bgra8'){windowRgba[index]=windowRaw[index+2];windowRgba[index+2]=windowRaw[index];}}
  assert.deepEqual(decodeScenePng(clientPng,proof.physicalSize,MEDIA_MAX_BYTES+65536),windowRgba,'Bootstrap desktop does not match actual GPU bytes');
  requireValue(sha(windowRgba)===proof.decodedRgba8Sha256,'Bootstrap decoded GPU hash differs');
  const scene=readback.sceneImage; validateSceneCommand(scene,profile);
  const rgba=sceneRgba(scene,sceneRaw);
  requireValue(scenePng.length===scene.pngByteLength && sha(scenePng)===scene.pngSha256,'Official scene PNG identity differs');
  assert.deepEqual(decodeScenePng(scenePng,scene.physicalSize),rgba,'Official scene PNG does not decode to actual GPU image');
  const ticket=readback.submission;
  requireValue(record(ticket) && ['device_id','generation','sequence'].every(key=>uint(ticket[key])) && ['Graphics','Compute','Copy'].includes(ticket.queue_class),'Actual submission ticket is absent');
  for(const [receipt,size] of [[readback.receipt,proof.physicalSize],[scene.receipt,scene.physicalSize]]){
    const padded=Math.ceil(size[0]*4/256)*256*size[1];
    requireValue(record(receipt) && receipt.kind==='Texture' && receipt.terminal==='Succeeded' && uint(receipt.request) &&
      uint(receipt.byte_len) && padded<=receipt.byte_len && receipt.byte_len<padded+256 && receipt.frame_key?.device_id===ticket.device_id &&
      receipt.frame_key?.generation===ticket.generation,'Actual successful texture readback receipt differs');
    assert.deepEqual(receipt.frame_key.submission,ticket,'Scene/window receipts use different submissions');
  }
  requireValue(scene.receipt.request!==readback.receipt.request && scene.receipt.byte_len+readback.receipt.byte_len<=MEDIA_MAX_BYTES,'Scene/window request identity or joint consumer budget differs');
  return {schema:'dev.zircon.editor.runtime-scene-media',version:1,
    bootstrapCase:{path:bootstrapPath,caseJsonSha256:sha(bootstrapBytes),caseSha256:bootstrapCaseSha256},
    bootstrapPixelAssociation:{path:proofPath,sha256:sha(proofBytes),observed:proof},
    image:{mime:'image/png',byteLength:scenePng.length,pngSha256:sha(scenePng),base64:scenePng.toString('base64')}};
}

async function unaliased(path) {
  const absolute = resolve(path);
  let ancestor = absolute;
  for (;;) {
    requireValue(!(await lstat(ancestor)).isSymbolicLink(), `Path contains a reparse alias: ${path}`);
    const parent = dirname(ancestor);
    if (parent === ancestor) break;
    ancestor = parent;
  }
  const physical = await realpath(absolute);
  requireValue(physicalKey(physical) === physicalKey(absolute), `Path is not its physical location: ${path}`);
  return absolute;
}

function actualCursor(value, label) {
  requireValue(record(value) && Object.keys(value).length === cursorFields.length &&
    cursorFields.every((field) => Object.hasOwn(value, field) && uint(value[field])),
  `${label} requires the exact seven actual safe counters`);
  return value;
}

function actualNormalSeed(profile) {
  const evidence = profile.normal_window_evidence;
  const publication = evidence?.workbenchStatePublication;
  requireValue(profile.presenter_backend === 'gpu' && uint(profile.capture_sequence) && profile.capture_sequence > 0 &&
    record(evidence) && evidence.captureSequence === profile.capture_sequence && record(publication) &&
    publication.authority === 'normal-retained-workbench-projection' && publication.observed === true,
  'Pending: no observed actual successful normal GPU business publication; no case was synthesized');
  const cursor = actualCursor(publication.presentationCursor, 'Business publication');
  actualCursor(evidence.presentationCursor, 'Normal packet');
  assert.deepEqual(evidence.presentationCursor, cursor, 'Actual normal/business packet cursors differ');
  requireValue(uint(publication.presentedFrameCount) && publication.presentedFrameCount > 0 &&
    publication.presentedFrameCount === evidence.presentedFrameCount &&
    publication.presentedFrameCount === profile.submitted_gpu_text?.presentedFrameCount &&
    evidence.commandStreamFullRebuild === true && profile.submitted_gpu_text?.commandStreamFullRebuild === true &&
    profile.submitted_gpu_text?.coverage === 'submitted-command-stream',
  'Pending: actual business DTO does not belong to a complete successfully submitted normal GPU frame');
  return { evidence, publication, cursor };
}

async function currentContractInputs(modulePath, repo) {
  const auditPath = resolve(dirname(modulePath), 'normal-workbench-contract-inputs.json');
  await unaliased(modulePath);
  await unaliased(auditPath);
  const bytes = await readFile(auditPath);
  const audit = JSON.parse(bytes);
  requireValue(audit.schema === 'dev.zircon.editor.normal-workbench-contract-inputs' && audit.version === 1 &&
    physicalKey(audit.repoRoot) === physicalKey(repo) && Array.isArray(audit.inputs) && audit.inputs.length > 0 &&
    physicalKey(audit.canonicalAppRoot ?? '') === physicalKey(resolve(repo, 'tools/penpot/apps/zircon-zui-plugin')) &&
    audit.sourceCopiesCreated === false &&
    /^[0-9a-f]{64}$/.test(audit.outputSha256), 'Canonical contract input audit is missing or unsupported');
  requireValue(sha(await readFile(modulePath)) === audit.outputSha256, 'Compiled canonical contract changed');
  const paths = new Set();
  const canonicalPrefix = resolve(repo, 'tools/penpot/apps/zircon-zui-plugin') + sep;
  for (const input of audit.inputs) {
    requireValue(record(input) && typeof input.path === 'string' && /^[0-9a-f]{64}$/.test(input.sha256),
      'Canonical contract input identity is malformed');
    const path = await unaliased(input.path);
    requireValue(!physicalKey(path).startsWith(physicalKey(resolve(repo, 'dev/penpot/plugins')) + '\\'),
      'Reference-tree module cannot own a current contract');
    requireValue(!paths.has(physicalKey(path)), 'Canonical contract input audit contains duplicates');
    paths.add(physicalKey(path));
    requireValue(sha(await readFile(path)) === input.sha256, `Canonical contract input changed: ${path}`);
  }
  for (const file of ['tools/zui-layout-review-contract.ts', 'tools/zui-layout-workbench-presentation.ts',
    'tools/zui-layout-workbench-managed-inputs.ts', 'tools/zui-layout-component-hosts.ts']) {
    requireValue(paths.has(physicalKey(canonicalPrefix + file)), `Canonical contract owner missing: ${file}`);
  }
  return { audit, auditPath, auditSha256: sha(bytes), moduleSha256: audit.outputSha256 };
}

async function sourceInputs(repo) {
  const paths = [];
  async function scan(directory) {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      requireValue(!entry.isSymbolicLink(), 'Source closure contains a reparse alias');
      const path = join(directory, entry.name);
      if (entry.isDirectory()) await scan(path);
      else if (entry.isFile() && entry.name.endsWith('.zui')) paths.push(relative(repo, path).replaceAll('\\', '/'));
    }
  }
  await unaliased(resolve(repo, 'zircon_editor/assets/ui'));
  await scan(resolve(repo, 'zircon_editor/assets/ui'));
  for (const dependency of ['zircon_editor/assets/i18n/en.toml',
    'zircon_editor/assets/i18n/zh-CN.toml', 'zircon_runtime/assets/fonts/editor-ui.ttc']) {
    await unaliased(resolve(repo, dependency));
    paths.push(dependency);
  }
  paths.sort();
  const inputs = [];
  for (const path of paths) inputs.push({ path, sha256: sha(await readFile(resolve(repo, path))) });
  return inputs;
}

async function bootstrapRuntimeScene(options, contracts, profilePath, profileBytes, profile, contractPin) {
  const pending = new Map();
  const read = async (value, label, limit = CASE_MAX_BYTES, expected) => {
    const path = await unaliased(contracts.requireApprovedPenpotArtifactFile(value, label));
    requireValue((await lstat(path)).size <= limit, `${label} exceeds bounded input size`);
    const bytes = await readFile(path);
    requireValue(bytes.length <= limit && (expected === undefined || sha(bytes) === expected), `${label} changed`);
    pending.set(path, sha(bytes)); return {path, bytes};
  };
  const bootstrapInput = await read(options.get('--bootstrap-case-json'), 'bootstrap case');
  const proofInput = await read(options.get('--bootstrap-pixel-association'), 'bootstrap pixel association', CASE_MAX_BYTES,
    options.get('--bootstrap-pixel-association-sha256'));
  const bootstrap = contracts.validateLayoutReviewCase(JSON.parse(bootstrapInput.bytes));
  const proof = JSON.parse(proofInput.bytes);
  requireValue(proof.officialCaseHashModuleSha256 === contractPin.moduleSha256 && proof.officialCaseHashInputAuditSha256 === contractPin.auditSha256 &&
    proof.caseContract?.moduleSha256 === contractPin.moduleSha256 && proof.caseContract?.inputAuditSha256 === contractPin.auditSha256,
    'Bootstrap case hash was not computed by the current official contract');
  const actualProfile = await unaliased(contracts.requireApprovedPenpotArtifactFile(proof.normalSourceProfile, 'bootstrap held profile'));
  requireValue(physicalKey(actualProfile) === physicalKey(profilePath) && sha(profileBytes) === proof.normalSourceProfileSha256,
    'Final case must use the actual nonce1 held source profile');
  const readbackInput = await read(proof.normalReadbackReceiptPath, 'bootstrap normal readback', CASE_MAX_BYTES, proof.normalReadbackReceiptSha256);
  requireValue(readbackInput.path === resolve(dirname(readbackInput.path), `normal-gpu-${proof.nonce}.json`), 'Bootstrap readback name differs');
  const readback = JSON.parse(readbackInput.bytes);
  assert.deepEqual(readback,proof.normalReadbackReceipt,'Bootstrap original readback differs');
  requireValue(readback.rawFile === `normal-gpu-${proof.nonce}.color8` && readback.sceneImage?.rawFile === `normal-scene-${proof.nonce}.color8` &&
    readback.sceneImage?.pngFile === `normal-scene-${proof.nonce}.png`, 'Bootstrap image file names differ');
  const root = dirname(readbackInput.path);
  const windowRaw = await read(resolve(root,readback.rawFile),'bootstrap actual window raw',MEDIA_MAX_BYTES,readback.rawSha256);
  const client = await read(proof.clientPng,'bootstrap actual desktop PNG',MEDIA_MAX_BYTES+65536,proof.clientPngSha256);
  const raw = await read(resolve(root,readback.sceneImage.rawFile),'bootstrap scene raw',MEDIA_MAX_BYTES,readback.sceneImage.rawSha256);
  const png = await read(resolve(root,readback.sceneImage.pngFile),'bootstrap official scene PNG',CASE_MAX_BYTES,readback.sceneImage.pngSha256);
  const image = await unaliased(contracts.requireApprovedPenpotArtifactFile(proof.processImage?.path,'bootstrap product executable'));
  requireValue(sha(await readFile(image)) === proof.processImage?.sha256, 'Bootstrap product executable changed');
  pending.set(image,proof.processImage.sha256);
  const packet = runtimeScenePacket({bootstrapCase:bootstrap,bootstrapBytes:bootstrapInput.bytes,bootstrapCaseSha256:contracts.caseSha256(bootstrap),
    proof,proofBytes:proofInput.bytes,profile,sceneRaw:raw.bytes,scenePng:png.bytes,windowRaw:windowRaw.bytes,clientPng:client.bytes,
    bootstrapPath:bootstrapInput.path,proofPath:proofInput.path});
  return {packet, verify:async()=>{for(const [path,hash] of pending)requireValue(sha(await readFile(path))===hash,`Runtime scene input changed during preparation: ${path}`);}};
}

async function main() {
  const usage = 'normal-workbench-case-producer.mjs --repo-root <actual repo> --profile <immutable actual normal GPU profile> --case-id <capture CaseId> --output-dir <existing approved artifact directory> --contract-module <current producer-sibling contracts.mjs> [--bootstrap-case-json <original nonce1 case> --bootstrap-pixel-association <actual nonce1 proof> --bootstrap-pixel-association-sha256 <sha256>]';
  const keys = ['--repo-root', '--profile', '--case-id', '--output-dir', '--contract-module'];
  const optional = ['--bootstrap-case-json', '--bootstrap-pixel-association', '--bootstrap-pixel-association-sha256'];
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === '--help') { console.log(usage); return; }
  const options = new Map();
  requireValue(args.length % 2 === 0, usage);
  for (let index = 0; index < args.length; index += 2) {
    requireValue([...keys,...optional].includes(args[index]) && args[index + 1] && !options.has(args[index]), usage);
    options.set(args[index], args[index + 1]);
  }
  requireValue(keys.every(key=>options.has(key)) && [0,optional.length].includes(optional.filter(key=>options.has(key)).length),usage);
  const repo = await unaliased(options.get('--repo-root'));
  requireValue(physicalKey(repo) === physicalKey(resolve(canonicalRepo)), 'Use the actual reviewed repository root');
  const modulePath = resolve(options.get('--contract-module'));
  requireValue(modulePath === resolve(dirname(fileURLToPath(import.meta.url)), 'normal-workbench-contracts.mjs'),
    'Contract module must be the reviewed producer sibling');
  const contractPin = await currentContractInputs(modulePath, repo);
  const contracts = await import(pathToFileURL(modulePath));
  const output = await unaliased(contracts.requireApprovedPenpotArtifactDirectory(options.get('--output-dir'), 'normal case output'));
  const profilePath = await unaliased(contracts.requireApprovedPenpotArtifactFile(options.get('--profile'), 'actual normal profile'));
  const profileBytes = await readFile(profilePath);
  requireValue(profileBytes.length <= 8 * 1024 * 1024, 'Actual normal profile exceeds bounded metadata size');
  const profile = JSON.parse(profileBytes);
  const { evidence, publication, cursor } = actualNormalSeed(profile);
  const sceneMedia = options.has('--bootstrap-case-json')
    ? await bootstrapRuntimeScene(options,contracts,profilePath,profileBytes,profile,contractPin) : undefined;
  requireValue(contracts.readViewportToolbarPublication(evidence) !== undefined,
    'Pending: actual same-frame viewport toolbar publication is absent; no state/overflow was synthesized');
  const presentation = publication.workbenchPresentation;
  // The canonical validator returns void. Preserve the original observed DTO after validation.
  contracts.validateWorkbenchPresentation(presentation);
  const managed = await contracts.verifyWorkbenchManagedSceneFingerprint(publication.managedSceneFingerprint);
  contracts.assertWorkbenchManagedProjectPath(managed, presentation.status.projectPath);
  const sourceBytes = await readFile(resolve(repo, sourcePath));
  requireValue(presentation.sourceFingerprint.sourcePath === sourcePath &&
    presentation.sourceFingerprint.sha256 === sha(sourceBytes),
  'Current Unreal source owner differs from the actual normal business publication');
  const logical = profile.window_client_logical_size;
  const physical = profile.window_client_size;
  const dpi = profile.winit_scale_factor;
  requireValue(record(logical) && record(physical) && Number.isFinite(dpi) && dpi > 0 &&
    [logical.width, logical.height].every((value) => Number.isInteger(value) && value >= 1 && value <= 8192) &&
    [physical.width, physical.height].every((value) => uint(value) && value > 0) &&
    Math.abs(logical.width * dpi - physical.width) <= 0.51 &&
    Math.abs(logical.height * dpi - physical.height) <= 0.51,
  'Pending: actual viewport/DPI is not faithfully representable by LayoutReviewCase');
  const stableBusiness = { authority: publication.authority, observed: true, presentationCursor: cursor,
    presentedFrameCount: publication.presentedFrameCount,
    workbenchPresentation: presentation, managedSceneFingerprint: managed };
  const stableEvidence = { workbenchStatePublication: stableBusiness,
    presentationCursor: evidence.presentationCursor, presentedFrameCount: evidence.presentedFrameCount };
  if (Object.hasOwn(evidence, 'commands')) stableEvidence.commands = observedSubmittedCommands(evidence);
  // Forward the same successful immutable present; the reader admits actual owner focus or keeps unknown.
  if (Object.hasOwn(evidence, 'semanticNodes')) stableEvidence.semanticNodes = evidence.semanticNodes;
  // Preserve the exact observed source subtree, all7 cursor and successful frame.
  stableEvidence.viewportToolbarPublication = evidence.viewportToolbarPublication;
  // Missing focus remains unknown. Do not turn absent observations into known unfocused state.
  if (Object.hasOwn(evidence, 'focusPublication')) stableEvidence.focusPublication = evidence.focusPublication;
  // Keep the exact observations from this successful present in the hashed case.
  if (record(evidence.paintEvidence) && Object.hasOwn(evidence.paintEvidence, 'designTokenInput')) {
    stableEvidence.designTokenInput = evidence.paintEvidence.designTokenInput;
  }
  if (Object.hasOwn(evidence, 'arrangedInventoryPublication')) {
    stableEvidence.arrangedInventoryPublication = evidence.arrangedInventoryPublication;
  }
  if (sceneMedia) stableEvidence.runtimeSceneMedia = sceneMedia.packet;
  const inputSources = await sourceInputs(repo);
  const dependencies = new contracts.LayoutDependencies();
  await dependencies.load(repo, inputSources.filter((item) => item.path.endsWith('.zui')).map((item) => item.path));
  const source = sourceBytes.toString('utf8');
  const document = contracts.parseZuiLayoutSource(source, sourcePath).document;
  const rootId = document.root?.node;
  const rootControl = document.nodes?.[rootId]?.control_id;
  requireValue(typeof rootId === 'string' && typeof rootControl === 'string' && rootControl.length > 0,
    'Actual Unreal source has no addressable root selector');
  const host = await contracts.prepareComponentReviewHost(repo, sourcePath, document, dependencies, undefined, presentation);
  requireValue(host && typeof host.source === 'string' && record(host.projection),
    'Current authored Unreal source has no supported complete host preparation');
  assert.deepEqual(host.projection.penpot_review_workbench_presentation, presentation,
    'Prepared host must carry the exact observed business DTO');
  const hostPath = resolve(output, 'normal-workbench-review-host.zui');
  const casePath = resolve(output, 'normal-workbench-case.json');
  const receiptPath = resolve(output, 'normal-workbench-case-receipt.json');
  const bindingsPath = resolve(output, 'normal-workbench-case-bindings.json');
  const reviewCase = contracts.validateLayoutReviewCase({ id: options.get('--case-id'), sourcePath, host: 'editor',
    viewport: { width: logical.width, height: logical.height }, dpi,
    locale: presentation.activeLocale === 'en' ? 'en-US' : presentation.activeLocale, state: 'default',
    data: { workbenchPresentation: presentation, workbenchState: { sourcePath, sourceNodeId: rootId, controlId: rootControl },
      managedSceneFingerprint: managed, normal_window_evidence: stableEvidence },
    reviewHost: { path: hostPath, sha256: sha(Buffer.from(host.source)) } });
  const caseHash = contracts.caseSha256(reviewCase);
  const caseBytes = jsonBytes(reviewCase);
  requireValue(caseBytes.length <= CASE_MAX_BYTES, 'Complete actual case including runtime PNG exceeds 8 MiB');
  for (const input of inputSources) requireValue(sha(await readFile(resolve(repo, input.path))) === input.sha256,
    `Source changed during actual host preparation: ${input.path}`);
  requireValue(sha(await readFile(profilePath)) === sha(profileBytes), 'Immutable seed profile changed during preparation');
  await contracts.verifyWorkbenchManagedSceneFingerprint(managed);
  const afterPin = await currentContractInputs(modulePath, repo);
  assert.deepEqual(afterPin, contractPin, 'Contract input audit changed during preparation');
  if (sceneMedia) await sceneMedia.verify();
  const receipt = { schema: 'dev.zircon.editor.normal-workbench-case-preparation', version: 1, status: 'pending',
    reason: 'Actual normal business generation prepared. A fresh nonce frame must independently match the seven counters, business/source/managed/focus identity, full submitted frame and actual GPU/native pixels.',
    pixelFrameVerified: false, sceneVerified: false, wholeUiAccepted: false, sourceCopiesCreated: false,
    caseJsonPath: casePath, caseJsonSha256: sha(caseBytes), caseSha256: caseHash,
    seedProfilePath: profilePath, seedProfileSha256: sha(profileBytes), seedCaptureSequence: profile.capture_sequence,
    seedPresentedFrameCount: publication.presentedFrameCount, presentationCursor: cursor,
    profilePath, profileSha256: sha(profileBytes), profileRole: 'seed-normal-observation-not-final-capture-proof',
    reviewHost: reviewCase.reviewHost, bindingsPath, sourceInputs: inputSources, consumers: host.consumers,
    contract: { path: modulePath, sha256: contractPin.moduleSha256, inputAuditPath: contractPin.auditPath,
      inputAuditSha256: contractPin.auditSha256, inputCount: contractPin.audit.inputs.length } };
  // Create once: interrupted or failed evidence remains on disk for diagnosis and is never overwritten.
  await writeFile(hostPath, host.source, { flag: 'wx', flush: true });
  await writeFile(casePath, caseBytes, { flag: 'wx', flush: true });
  await writeFile(bindingsPath, jsonBytes({ schema: 'dev.zircon.editor.normal-frame-case-bindings', version: 1,
    cases: { [reviewCase.id]: { caseJsonPath: casePath, caseJsonSha256: receipt.caseJsonSha256 } } }), { flag: 'wx', flush: true });
  await writeFile(receiptPath, jsonBytes(receipt), { flag: 'wx', flush: true });
  console.log(JSON.stringify({ status: receipt.status, receiptPath, caseJsonPath: casePath,
    caseJsonSha256: receipt.caseJsonSha256, caseSha256: caseHash, bindingsPath, profilePath,
    profileSha256: receipt.profileSha256, profileRole: receipt.profileRole, presentationCursor: cursor,
    pixelFrameVerified: false, wholeUiAccepted: false }));
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url)
  main().catch((error) => { console.error(error.stack ?? String(error)); process.exitCode = 1; });

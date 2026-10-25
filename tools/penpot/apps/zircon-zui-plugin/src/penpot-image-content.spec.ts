import { createHash } from 'node:crypto';
import { deflateSync } from 'node:zlib';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Board, ImageData } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import type { RuntimeSceneEmbeddedImage } from './bridge/zui-review-case';
import { ZUI_METADATA_NODE_ID } from './metadata';
import { auditRuntimeSceneMedia, loadAssetImages, runtimeSceneBytesSha256, runtimeSceneImageBytes, verifyUploadedRuntimeSceneImage } from './penpot-image-content';

// Source/owner/case validation has real fixtures in zui-review-case.spec. Here
// the boundary is the official upload/original-data API and saved image shape.
const contract = vi.hoisted(() => ({ actual: undefined as unknown, target: undefined as unknown, packet: undefined as unknown }));
vi.mock('./bridge/zui-review-case', () => ({
  importedActualNormalReviewCase: () => contract.actual,
  reviewCaseFromSavedMetadata: () => contract.actual,
  runtimeSceneMediaForCase: () => contract.packet,
  runtimeSceneMediaReviewTarget: () => contract.target,
}));
const sha = (bytes: Uint8Array) => createHash('sha256').update(bytes).digest('hex');
function png(): Buffer {
  const chunk = (kind: string, body: Buffer): Buffer => {
    const data = Buffer.concat([Buffer.from(kind), body]); let crc = 0xffffffff;
    for (const value of data) { crc ^= value; for (let bit = 0; bit < 8; bit++) crc = crc & 1 ? (crc >>> 1) ^ 0xedb88320 : crc >>> 1; }
    const result = Buffer.alloc(body.length + 12); result.writeUInt32BE(body.length); data.copy(result,4); result.writeUInt32BE((crc ^ 0xffffffff) >>> 0,result.length-4); return result;
  };
  return Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',Buffer.from([0,0,0,1,0,0,0,1,8,6,0,0,0])),
    chunk('IDAT',deflateSync(Buffer.from([0,20,128,230,255]))),chunk('IEND',Buffer.alloc(0))]);
}
function fixture() {
  const bytes = png(), hash = sha(bytes);
  const image: RuntimeSceneEmbeddedImage = { uri:'runtime-scene:'+hash,name:'normal-scene-'+hash.slice(0,16)+'.png',mime:'image/png',base64:bytes.toString('base64'),
    runtimeScene:{pngSha256:hash,byteLength:bytes.length,physicalSize:[1,1]} };
  const media = { id:'official-media-id',width:1,height:1,mtype:'image/png',data:vi.fn(async()=>new Uint8Array(bytes)) } as ImageData;
  const content = { id:'image-shape',type:'rect',fills:[] as unknown[], getSharedPluginData:(_: string,key: string)=>key==='image-content'?'true':'' };
  const owner = { id:'scene-board',type:'board',children:[content],getSharedPluginData:(_: string,key: string)=>key===ZUI_METADATA_NODE_ID?'scene_media':'' };
  const asset = { id:'asset-board',type:'board',children:[owner],getSharedPluginData:()=>'' } as unknown as Board;
  const document: ZuiDocument = {asset:{kind:'view',id:'unit-scene-upload',version:2},nodes:{scene_media:{component:'SceneViewViewport',penpot_image:image}}};
  contract.actual={id:'actual-unit-case'};contract.target={nodeId:'scene_media',image};
  contract.packet={image:{pngSha256:hash,byteLength:bytes.length},bootstrapCase:{caseJsonSha256:'1'.repeat(64),caseSha256:'2'.repeat(64)},bootstrapPixelAssociation:{sha256:'3'.repeat(64)}};
  return {bytes,image,media,content,owner,asset,document};
}

describe('actual portable scene bytes and official media boundary',()=>{
  beforeEach(()=>{
    contract.actual=undefined;contract.target=undefined;contract.packet=undefined;
    vi.stubGlobal('TextEncoder',undefined);vi.stubGlobal('crypto',undefined);
  });
  afterEach(()=>vi.unstubAllGlobals());

  it.each([
    ['empty', '', 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'],
    ['abc', 'abc', 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'],
    ['two-block padding', 'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq',
      '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1'],
    ['million a', 'a'.repeat(1_000_000), 'cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0'],
  ])('matches the SHA-256 %s vector without Web Crypto or TextEncoder',(_name,text,expected)=>{
    expect(globalThis.crypto).toBeUndefined();expect(globalThis.TextEncoder).toBeUndefined();
    expect(runtimeSceneBytesSha256(new Uint8Array(Buffer.from(text,'utf8')))).toBe(expected);
  });

  it('hashes only the selected byte view across padding and block boundaries without modifying its backing bytes',()=>{
    const backing=Uint8Array.from({length:150},(_,index)=>(index*131+17)&255),before=new Uint8Array(backing);
    for(const length of [0,1,55,56,63,64,65,119,120,127,128,129]) {
      const view=backing.subarray(5,5+length);
      expect(runtimeSceneBytesSha256(view)).toBe(sha(view));
    }
    expect(backing).toEqual(before);
  });

  it('accepts the real RGBA PNG bytes with an independently pinned digest and no sandbox crypto',async()=>{
    const {bytes,image}=fixture(),before=Buffer.from(bytes);
    expect(runtimeSceneBytesSha256(bytes)).toBe(sha(bytes));
    expect(Buffer.from(await runtimeSceneImageBytes(image))).toEqual(bytes);
    expect(bytes).toEqual(before);
  });

  it('rejects a same-length PNG payload alteration and changed official original bytes',async()=>{
    const {bytes,image,media}=fixture(),changed=Buffer.from(bytes);
    const payload=changed.indexOf(Buffer.from('IDAT'))+4;
    expect(payload).toBeGreaterThan(4);changed[payload]=changed[payload]!^1;
    expect(changed.length).toBe(bytes.length);expect(sha(changed)).not.toBe(image.runtimeScene.pngSha256);
    await expect(runtimeSceneImageBytes({...image,base64:changed.toString('base64')})).rejects.toThrow(/SHA differ/);
    await expect(verifyUploadedRuntimeSceneImage({...media,data:async()=>new Uint8Array(changed)},image))
      .rejects.toThrow(/original media bytes differ/);
  });

  it('uploads only exact runtime PNG bytes and reads the original before assigning its fill',async()=>{
    const value=fixture();let filled=false;
    const upload=vi.fn(async(name: string,bytes: Uint8Array,mime: string)=>{
      expect(name).toBe(value.image.name);expect(Buffer.from(bytes)).toEqual(value.bytes);expect(mime).toBe('image/png');
      expect(value.content.fills).toEqual([]);return value.media;
    });
    vi.stubGlobal('penpot',{uploadMediaData:upload});
    const original=value.media.data;value.media.data=vi.fn(async()=>{expect(filled).toBe(false);return original();});
    await loadAssetImages(value.asset,value.document);filled=true;
    expect(upload).toHaveBeenCalledTimes(1);expect(value.media.data).toHaveBeenCalledTimes(1);
    expect(value.content.fills).toEqual([{fillImage:value.media,fillOpacity:1}]);
  });

  it('rejects encoded byte/hash/size/type alterations before official upload',async()=>{
    const {image}=fixture();
    for(const changed of [
      {...image,base64:image.base64.slice(0,-4)+'AAAA'},
      {...image,runtimeScene:{...image.runtimeScene,pngSha256:'a'.repeat(64)}},
      {...image,runtimeScene:{...image.runtimeScene,physicalSize:[2,1] as [number,number]}},
      {...image,mime:'image/jpeg'},
    ]) await expect(runtimeSceneImageBytes(changed as RuntimeSceneEmbeddedImage)).rejects.toThrow();
  });

  it('rejects official original media with another byte stream, extent or MIME',async()=>{
    const {media,image}=fixture();
    for(const changed of [
      {...media,data:async()=>new Uint8Array([0])}, {...media,width:2}, {...media,mtype:'image/jpeg'}, {...media,keepAspectRatio:true},
    ]) await expect(verifyUploadedRuntimeSceneImage(changed,image)).rejects.toThrow();
  });

  it('does not assign a fill when official upload data differs from the receipt',async()=>{
    const value=fixture(),upload=vi.fn(async()=>({...value.media,data:async()=>new Uint8Array([0])}));
    vi.stubGlobal('penpot',{uploadMediaData:upload});
    await expect(loadAssetImages(value.asset,value.document)).rejects.toThrow(/original media bytes/);
    expect(value.content.fills).toEqual([]);
  });

  it('audits freshly read saved original media without uploading or replacing the fill',async()=>{
    const value=fixture(),upload=vi.fn();value.content.fills=[{fillImage:value.media,fillOpacity:1}];
    vi.stubGlobal('penpot',{uploadMediaData:upload});const before=value.content.fills;
    const result=await auditRuntimeSceneMedia(value.asset,value.document);
    expect(result).toMatchObject({verified:true,caseId:'actual-unit-case',nodeId:'scene_media',ownerShapeId:'scene-board',shapeId:'image-shape',mediaId:'official-media-id',pngSha256:sha(value.bytes)});
    expect(value.media.data).toHaveBeenCalledTimes(1);expect(upload).not.toHaveBeenCalled();expect(value.content.fills).toBe(before);
    value.content.fills=[{fillImage:value.media,fillOpacity:.5}];
    await expect(auditRuntimeSceneMedia(value.asset,value.document)).rejects.toThrow(/fill was replaced/);
    value.content.fills=['#000000'];
    await expect(auditRuntimeSceneMedia(value.asset,value.document)).rejects.toThrow(/fill was replaced/);
  });

  it('rejects missing/duplicate actual scene content and preserves static image behavior',async()=>{
    const value=fixture();vi.stubGlobal('penpot',{uploadMediaData:vi.fn(async()=>value.media)});
    value.owner.children=[];
    await expect(loadAssetImages(value.asset,value.document)).rejects.toThrow(/missing or duplicated/);
    contract.actual=undefined;contract.target=undefined;contract.packet=undefined;
    value.owner.children=[value.content];
    value.document.nodes!['scene_media']['penpot_image']={uri:'res://unit.png',name:'unit.png',mime:'image/png',base64:value.bytes.toString('base64')};
    await loadAssetImages(value.asset,value.document);
    expect(await auditRuntimeSceneMedia(value.asset,value.document)).toBeUndefined();
  });
});

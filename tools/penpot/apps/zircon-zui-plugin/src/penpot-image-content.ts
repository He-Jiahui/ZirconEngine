import type { Board, ImageData } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { importedActualNormalReviewCase, reviewCaseFromSavedMetadata, runtimeSceneMediaForCase,
  runtimeSceneMediaReviewTarget, type RuntimeSceneEmbeddedImage } from './bridge/zui-review-case';
import { reviewHost } from './bridge/zui-review-host';
import type { RuntimeSceneMediaAudit } from './penpot-render-layout';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_METADATA_NODE_ID,
} from './metadata';

interface EmbeddedImage {
  uri: string;
  name: string;
  mime: string;
  base64: string;
  runtimeScene?: RuntimeSceneEmbeddedImage['runtimeScene'];
}

const need = (condition: unknown, message: string): void => {
  if (!condition) throw new Error('Runtime scene media: ' + message);
};
/** SHA-256 over the exact byte view; the official SES sandbox does not endow Web Crypto. */
export function runtimeSceneBytesSha256(bytes: Uint8Array): string {
  const round = new Uint32Array([
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
  ]);
  const hash = new Uint32Array([0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]);
  const padded = new Uint8Array(Math.ceil((bytes.length + 9) / 64) * 64);
  padded.set(bytes);
  padded[bytes.length] = 0x80;
  const view = new DataView(padded.buffer);
  view.setUint32(padded.length - 8, Math.floor(bytes.length / 0x20000000));
  view.setUint32(padded.length - 4, (bytes.length * 8) >>> 0);
  const words = new Uint32Array(64);
  const rotate = (value: number, amount: number): number => (value >>> amount) | (value << (32 - amount));
  for (let offset = 0; offset < padded.length; offset += 64) {
    for (let index = 0; index < 16; index++) words[index] = view.getUint32(offset + index * 4);
    for (let index = 16; index < 64; index++) {
      const previous = words[index - 15]!, recent = words[index - 2]!;
      const low = rotate(previous, 7) ^ rotate(previous, 18) ^ (previous >>> 3);
      const high = rotate(recent, 17) ^ rotate(recent, 19) ^ (recent >>> 10);
      words[index] = (words[index - 16]! + low + words[index - 7]! + high) >>> 0;
    }
    let a = hash[0]!, b = hash[1]!, c = hash[2]!, d = hash[3]!;
    let e = hash[4]!, f = hash[5]!, g = hash[6]!, h = hash[7]!;
    for (let index = 0; index < 64; index++) {
      const sum = rotate(e, 6) ^ rotate(e, 11) ^ rotate(e, 25);
      const choose = (e & f) ^ (~e & g);
      const first = (h + sum + choose + round[index]! + words[index]!) >>> 0;
      const majority = (a & b) ^ (a & c) ^ (b & c);
      const second = ((rotate(a, 2) ^ rotate(a, 13) ^ rotate(a, 22)) + majority) >>> 0;
      h = g; g = f; f = e; e = (d + first) >>> 0;
      d = c; c = b; b = a; a = (first + second) >>> 0;
    }
    const compressed = [a, b, c, d, e, f, g, h];
    for (let index = 0; index < hash.length; index++) hash[index] = (hash[index]! + compressed[index]!) >>> 0;
  }
  return Array.from(hash, (value) => value.toString(16).padStart(8, '0')).join('');
}

/** This checks encoded integrity; the physical producer independently decodes all PNG rows. */
export async function runtimeSceneImageBytes(image: RuntimeSceneEmbeddedImage): Promise<Uint8Array> {
  const facts = image.runtimeScene;
  need(image.mime === 'image/png' && /^[0-9a-f]{64}$/.test(facts.pngSha256) &&
    image.uri === 'runtime-scene:' + facts.pngSha256 && Number.isSafeInteger(facts.byteLength) &&
    facts.byteLength > 0 && facts.byteLength <= 8 * 1024 * 1024 && typeof image.base64 === 'string' &&
    image.base64.length === 4 * Math.ceil(facts.byteLength / 3), 'invalid content-addressed PNG metadata');
  const binary = atob(image.base64);
  need(btoa(binary) === image.base64 && binary.length === facts.byteLength, 'noncanonical PNG encoding');
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  const header = [137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82];
  need(bytes.length >= 33 && header.every((value, index) => bytes[index] === value), 'invalid PNG signature or IHDR');
  const data = new DataView(bytes.buffer);
  need(facts.physicalSize.length === 2 && facts.physicalSize.every((value) => Number.isSafeInteger(value) && value > 0 && value <= 8192) &&
    data.getUint32(16) === facts.physicalSize[0] && data.getUint32(20) === facts.physicalSize[1] &&
    bytes[24] === 8 && bytes[25] === 6 && bytes[26] === 0 && bytes[27] === 0 && bytes[28] === 0 &&
    runtimeSceneBytesSha256(bytes) === facts.pngSha256, 'PNG bytes, size, format or SHA differ from the runtime receipt');
  return bytes;
}

export async function verifyUploadedRuntimeSceneImage(image: ImageData, expected: RuntimeSceneEmbeddedImage): Promise<void> {
  need(typeof image.id === 'string' && image.id.length > 0 && image.width === expected.runtimeScene.physicalSize[0] &&
    image.height === expected.runtimeScene.physicalSize[1] && image.mtype === 'image/png' && image.keepAspectRatio !== true &&
    typeof image.data === 'function', 'official uploaded media metadata differs');
  const bytes = await image.data();
  need(bytes instanceof Uint8Array && bytes.byteLength === expected.runtimeScene.byteLength &&
    runtimeSceneBytesSha256(bytes) === expected.runtimeScene.pngSha256, 'official original media bytes differ from the runtime PNG');
}

export function createImageContent(board: Board, node: ZuiNode): void {
  if (!node['penpot_image'] && !node['penpot_image_svg']) return;
  const image =
    typeof node['penpot_image_svg'] === 'string'
      ? penpot.createShapeFromSvg(node['penpot_image_svg'])
      : penpot.createRectangle();
  if (!image) throw new Error('Unable to create image content');
  image.name = 'Image';
  if (!node['penpot_image_svg']) image.fills = [];
  image.resize(board.width, board.height);
  board.appendChild(image);
  image.x = board.x;
  image.y = board.y;
  if (image.layoutChild) image.layoutChild.absolute = true;
  image.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  image.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'image-content', 'true');
}

export async function loadAssetImages(
  asset: Board,
  document: ZuiDocument,
): Promise<void> {
  const cache = new Map<string, ImageData>();
  const source = reviewHost(document) ?? document;
  const actual = importedActualNormalReviewCase(source);
  const expectedRuntime = actual ? runtimeSceneMediaReviewTarget(source, actual) : undefined;
  let runtimeCount = 0;
  const visit = async (board: Board): Promise<void> => {
    const id = board.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_NODE_ID,
    );
    const embedded = document.nodes?.[id]?.['penpot_image'] as unknown as
      EmbeddedImage | undefined;
    for (const child of board.children) {
      if (
        embedded &&
        child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'image-content')
      ) {
        let image = cache.get(embedded.uri);
        let runtime: RuntimeSceneEmbeddedImage | undefined;
        let bytes: Uint8Array | undefined;
        if (embedded.runtimeScene || embedded.uri.startsWith('runtime-scene:')) {
          need(actual, 'runtime PNG requires its original actual case');
          const target = expectedRuntime;
          need(target && target.nodeId === id && JSON.stringify(target.image) === JSON.stringify(embedded),
            'embedded image is outside its unique actual Scene owner');
          runtime = target!.image;
          runtimeCount++;
          bytes = await runtimeSceneImageBytes(runtime);
        }
        if (!image) {
          const binary = bytes ? undefined : atob(embedded.base64);
          image = await penpot.uploadMediaData(
            embedded.name,
            bytes ?? Uint8Array.from(binary!, (character) => character.charCodeAt(0)),
            embedded.mime,
          );
          if (runtime) await verifyUploadedRuntimeSceneImage(image, runtime);
          cache.set(embedded.uri, image);
        } else if (runtime) await verifyUploadedRuntimeSceneImage(image, runtime);
        child.fills = [{ fillImage: image, fillOpacity: 1 }];
      }
      if (child.type === 'board') await visit(child);
    }
  };
  await visit(asset);
  need(runtimeCount === (expectedRuntime ? 1 : 0), 'actual Scene image shape is missing or duplicated');
}

/** Read the saved image through the official original-media API; never upload or repair it. */
export async function auditRuntimeSceneMedia(asset: Board, input: ZuiDocument): Promise<RuntimeSceneMediaAudit | undefined> {
  const source = reviewHost(input) ?? input;
  const imported = importedActualNormalReviewCase(source);
  if (!imported || !runtimeSceneMediaForCase(imported)) return;
  const saved = reviewCaseFromSavedMetadata(source, asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'review-case'));
  need(saved, 'saved actual scene case metadata is missing');
  const packet = runtimeSceneMediaForCase(saved!)!, target = runtimeSceneMediaReviewTarget(source, saved!)!;
  await runtimeSceneImageBytes(target.image);
  const boards: Board[] = [];
  const visit = (board: Board): void => {
    if (board.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID) === target.nodeId) boards.push(board);
    for (const child of board.children) if (child.type === 'board') visit(child);
  }; visit(asset);
  need(boards.length === 1, 'saved Scene board is missing or duplicated');
  const images = boards[0]!.children.filter((shape) => shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'image-content') === 'true');
  need(images.length === 1, 'saved Scene image shape is missing or duplicated');
  const shape = images[0]!, fills = shape.fills;
  const fill = fills.length === 1 && typeof fills[0] !== 'string' ? fills[0] : undefined;
  need(fill && fill.fillOpacity === 1 && fill.fillImage &&
    !fill.fillColor && !fill.fillColorGradient, 'saved Scene fill was replaced or changed');
  const image = fill!.fillImage!;
  await verifyUploadedRuntimeSceneImage(image, target.image);
  return { schema: 'dev.zircon.penpot.runtime-scene-media-audit', version: 1,
    authority: 'official-original-media-data-after-layout-refresh', verified: true,
    caseId: saved!.id, nodeId: target.nodeId, ownerShapeId: boards[0]!.id, shapeId: shape.id, mediaId: image.id,
    pngSha256: packet.image.pngSha256, byteLength: packet.image.byteLength, physicalSize: target.image.runtimeScene.physicalSize,
    bootstrapCaseJsonSha256: packet.bootstrapCase.caseJsonSha256, bootstrapCaseSha256: packet.bootstrapCase.caseSha256,
    bootstrapPixelAssociationSha256: packet.bootstrapPixelAssociation.sha256 };
}

export function refreshImageContent(board: Board): void {
  for (const child of board.children) {
    if (!child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'image-content'))
      continue;
    child.resize(board.width, board.height);
    child.x = board.x;
    child.y = board.y;
  }
}

import assert from 'node:assert/strict';
import { inflate } from 'node:zlib';
import { promisify } from 'node:util';

const inflateAsync = promisify(inflate);

export interface ScreenshotPixels {
  width: number;
  height: number;
  pixels: Buffer;
}

/**
 * Compare two captures for frame stability while allowing Chromium's one-byte
 * text antialiasing variation. Layout and semantic audits remain strict; this
 * helper only prevents a handful of rasterizer bytes from keeping a settled
 * frame pending forever.
 */
export function rasterFramesEquivalent(
  first: ScreenshotPixels,
  second: ScreenshotPixels,
): boolean {
  if (
    first.width !== second.width ||
    first.height !== second.height ||
    first.pixels.length !== second.pixels.length
  )
    return false;
  const pixelCount = first.width * first.height;
  if (pixelCount <= 0 || first.pixels.length % pixelCount !== 0) return false;
  const channels = first.pixels.length / pixelCount;
  if (channels !== 3 && channels !== 4) return false;
  // Keep the tolerance tiny and bounded. A broad repaint, color shift, or
  // geometry change must still require a fresh stable frame. The percentage
  // covers small captures while the hard cap covers the observed one-pixel
  // antialiasing cluster on the 640x520 native gizmo edge.
  const maxChangedPixels = Math.max(
    1,
    Math.min(64, Math.floor(pixelCount * 0.0002)),
  );
  let changedPixels = 0;
  for (let offset = 0; offset < first.pixels.length; offset += channels) {
    let changed = false;
    for (let channel = 0; channel < channels; channel += 1) {
      if (Math.abs(first.pixels[offset + channel] - second.pixels[offset + channel]) > 1)
        return false;
      if (first.pixels[offset + channel] !== second.pixels[offset + channel])
        changed = true;
    }
    if (changed) {
      changedPixels += 1;
      if (changedPixels > maxChangedPixels) return false;
    }
  }
  return true;
}

export async function readPng(buffer: Buffer): Promise<ScreenshotPixels> {
  const signature = buffer.subarray(0, 8).toString('hex');
  assert.equal(signature, '89504e470d0a1a0a', 'invalid PNG signature');
  let offset = 8;
  let width = 0;
  let height = 0;
  let bitDepth = 0;
  let colorType = 0;
  const idat: Buffer[] = [];
  while (offset + 12 <= buffer.length) {
    const length = buffer.readUInt32BE(offset);
    const type = buffer.subarray(offset + 4, offset + 8).toString('ascii');
    const data = buffer.subarray(offset + 8, offset + 8 + length);
    if (type === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      bitDepth = data[8];
      colorType = data[9];
    } else if (type === 'IDAT') {
      idat.push(data);
    } else if (type === 'IEND') {
      break;
    }
    offset += 12 + length;
  }
  assert.equal(bitDepth, 8, 'visual validator expects 8-bit PNG screenshots');
  assert.ok(
    colorType === 2 || colorType === 6,
    `unsupported PNG color type ${colorType}`,
  );
  const channels = colorType === 6 ? 4 : 3;
  const stride = width * channels;
  const inflated = await inflateAsync(Buffer.concat(idat));
  const pixels = Buffer.alloc(width * height * channels);
  let sourceOffset = 0;
  for (let y = 0; y < height; y += 1) {
    const filter = inflated[sourceOffset];
    sourceOffset += 1;
    const rowOffset = y * stride;
    for (let x = 0; x < stride; x += 1) {
      const raw = inflated[sourceOffset + x];
      const left = x >= channels ? pixels[rowOffset + x - channels] : 0;
      const up = y > 0 ? pixels[rowOffset - stride + x] : 0;
      const upLeft =
        y > 0 && x >= channels ? pixels[rowOffset - stride + x - channels] : 0;
      pixels[rowOffset + x] = unfilter(filter, raw, left, up, upLeft);
    }
    sourceOffset += stride;
  }
  return { width, height, pixels };
}

function unfilter(
  filter: number,
  raw: number,
  left: number,
  up: number,
  upLeft: number,
): number {
  switch (filter) {
    case 0:
      return raw;
    case 1:
      return (raw + left) & 0xff;
    case 2:
      return (raw + up) & 0xff;
    case 3:
      return (raw + Math.floor((left + up) / 2)) & 0xff;
    case 4:
      return (raw + paeth(left, up, upLeft)) & 0xff;
    default:
      throw new Error(`unsupported PNG filter ${filter}`);
  }
}

function paeth(left: number, up: number, upLeft: number): number {
  const prediction = left + up - upLeft;
  const leftDistance = Math.abs(prediction - left);
  const upDistance = Math.abs(prediction - up);
  const upLeftDistance = Math.abs(prediction - upLeft);
  if (leftDistance <= upDistance && leftDistance <= upLeftDistance) return left;
  return upDistance <= upLeftDistance ? up : upLeft;
}

export function nonEmptyRatio(image: ScreenshotPixels): number {
  const channels = image.pixels.length / (image.width * image.height);
  let nonEmpty = 0;
  for (let offset = 0; offset < image.pixels.length; offset += channels) {
    const red = image.pixels[offset];
    const green = image.pixels[offset + 1];
    const blue = image.pixels[offset + 2];
    if (
      Math.max(red, green, blue) - Math.min(red, green, blue) > 4 ||
      Math.max(red, green, blue) > 40
    )
      nonEmpty += 1;
  }
  return nonEmpty / (image.width * image.height);
}

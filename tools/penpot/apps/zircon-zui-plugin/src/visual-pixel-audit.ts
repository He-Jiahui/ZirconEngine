export interface VisualPixelAudit {
  colorBucketCount: number;
  detailPixelRatio: number;
}

/** A divider occupies little of its host; include the adjacent background. */
export function hasVisibleThinControl(
  image: { width: number; height: number; pixels: Uint8Array },
  bounds: { x: number; y: number; width: number; height: number }[],
  dpi: number,
): boolean {
  if (bounds.length !== 1 || !Number.isFinite(dpi) || dpi <= 0) return false;
  const bound = bounds[0];
  if (
    ![bound.x, bound.y, bound.width, bound.height].every(Number.isFinite) ||
    Math.min(bound.width, bound.height) <= 0 ||
    Math.min(bound.width, bound.height) > 8 ||
    Math.max(bound.width, bound.height) < 16
  )
    return false;
  const left = Math.max(0, Math.floor((bound.x - 2) * dpi));
  const top = Math.max(0, Math.floor((bound.y - 2) * dpi));
  const right = Math.min(
    image.width,
    Math.ceil((bound.x + bound.width + 2) * dpi),
  );
  const bottom = Math.min(
    image.height,
    Math.ceil((bound.y + bound.height + 2) * dpi),
  );
  if (right <= left || bottom <= top) return false;
  const channels = image.pixels.length / (image.width * image.height);
  if (channels !== 3 && channels !== 4) return false;
  const width = right - left;
  const height = bottom - top;
  const pixels = new Uint8Array(width * height * channels);
  for (let row = 0; row < height; row++) {
    const start = ((top + row) * image.width + left) * channels;
    pixels.set(
      image.pixels.subarray(start, start + width * channels),
      row * width * channels,
    );
  }
  const audit = auditVisualPixels(width, height, pixels);
  return audit.colorBucketCount > 1 && audit.detailPixelRatio >= 0.05;
}

/** A compact control is inspected in its semantic bounds, not in the empty host. */
export function hasVisibleSmallControl(
  image: { width: number; height: number; pixels: Uint8Array },
  bounds: { x: number; y: number; width: number; height: number }[],
  dpi: number,
): boolean {
  if (bounds.length !== 1 || !Number.isFinite(dpi) || dpi <= 0) return false;
  const bound = bounds[0];
  if (
    bound.width <= 0 ||
    bound.height <= 0 ||
    bound.width > 64 ||
    bound.height > 64
  )
    return false;
  const left = Math.max(0, Math.floor(bound.x * dpi));
  const top = Math.max(0, Math.floor(bound.y * dpi));
  const right = Math.min(image.width, Math.ceil((bound.x + bound.width) * dpi));
  const bottom = Math.min(
    image.height,
    Math.ceil((bound.y + bound.height) * dpi),
  );
  if (right <= left || bottom <= top) return false;
  const channels = image.pixels.length / (image.width * image.height);
  if (channels !== 3 && channels !== 4) return false;
  const width = right - left;
  const height = bottom - top;
  const pixels = new Uint8Array(width * height * channels);
  for (let row = 0; row < height; row++) {
    const start = ((top + row) * image.width + left) * channels;
    pixels.set(
      image.pixels.subarray(start, start + width * channels),
      row * width * channels,
    );
  }
  const audit = auditVisualPixels(width, height, pixels);
  return audit.colorBucketCount > 1 && audit.detailPixelRatio >= 0.01;
}

const CHANNELS_RGB = 3;
const CHANNELS_RGBA = 4;
const COLOR_QUANTUM = 16;

export function auditVisualPixels(
  width: number,
  height: number,
  pixels: Uint8Array,
): VisualPixelAudit {
  const pixelCount = width * height;
  const channels =
    pixels.length === pixelCount * CHANNELS_RGBA ? CHANNELS_RGBA : CHANNELS_RGB;
  if (
    !Number.isInteger(width) ||
    !Number.isInteger(height) ||
    width <= 0 ||
    height <= 0 ||
    pixels.length !== pixelCount * channels
  ) {
    throw new Error('Visual pixel dimensions do not match the pixel buffer.');
  }

  const buckets = new Map<string, number>();
  for (let offset = 0; offset < pixels.length; offset += channels) {
    const alpha = channels === CHANNELS_RGBA ? pixels[offset + 3] : 255;
    const key =
      alpha < 16
        ? 'transparent'
        : colorBucket(pixels[offset], pixels[offset + 1], pixels[offset + 2]);
    buckets.set(key, (buckets.get(key) ?? 0) + 1);
  }

  if (buckets.size === 0) {
    return { colorBucketCount: 0, detailPixelRatio: 0 };
  }
  const dominantCount = Math.max(...buckets.values());
  return {
    colorBucketCount: buckets.size,
    detailPixelRatio: (pixelCount - dominantCount) / pixelCount,
  };
}

function colorBucket(red: number, green: number, blue: number): string {
  return [red, green, blue]
    .map((channel) => Math.floor(channel / COLOR_QUANTUM))
    .join(':');
}

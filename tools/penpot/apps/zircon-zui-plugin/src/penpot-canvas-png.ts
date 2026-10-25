/**
 * Validate the public SDK result at the worker boundary. This retains the
 * existing byte/header/dimension guard; it does not decode or certify a PNG.
 */
export function inspectCanvasPng(value: unknown): {
  bytes: Uint8Array; width: number; height: number;
} {
  const isBytes = value instanceof Uint8Array;
  const diagnostic = () => JSON.stringify({
    stage: 'shape-export-return',
    objectTag: Object.prototype.toString.call(value).slice(0, 80),
    arrayBufferView: ArrayBuffer.isView(value),
    byteLength: isBytes ? value.byteLength : null,
    byteOffset: isBytes ? value.byteOffset : null,
    headerHex: isBytes
      ? Array.from(value.subarray(0, 24), byte => byte.toString(16).padStart(2, '0')).join('')
      : null,
  });
  const signature = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (!isBytes || value.byteLength < 24 ||
    !signature.every((byte, index) => value[index] === byte) ||
    value[12] !== 0x49 || value[13] !== 0x48 ||
    value[14] !== 0x44 || value[15] !== 0x52) {
    throw new Error('Penpot shape export did not return a valid PNG byte stream; SDK return diagnostic: '+diagnostic());
  }
  const header = new DataView(value.buffer, value.byteOffset, value.byteLength);
  const width = header.getUint32(16, false), height = header.getUint32(20, false);
  if (width === 0 || height === 0)
    throw new Error('Penpot shape export returned an empty PNG canvas; SDK return diagnostic: '+diagnostic());
  return { bytes: value, width, height };
}

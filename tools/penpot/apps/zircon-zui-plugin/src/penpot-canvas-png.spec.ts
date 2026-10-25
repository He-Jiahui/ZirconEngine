import { describe, expect, it } from 'vitest';
import { inspectCanvasPng } from './penpot-canvas-png';

// Header fixtures exercise the existing boundary checks only. These are not
// decoded or official exported media and are never counted as PNG acceptance.
function header(width = 2, height = 3, offset = 0) {
  const bytes = new Uint8Array(new ArrayBuffer(24 + offset), offset, 24);
  bytes.set([0x89,0x50,0x4e,0x47,0x0d,0x0a,0x1a,0x0a], 0);
  bytes.set([0x49,0x48,0x44,0x52], 12);
  const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  data.setUint32(16, width, false); data.setUint32(20, height, false);
  return bytes;
}
describe('Canvas PNG SDK return diagnostics', () => {
  it('retains valid Uint8Array bytes and their dimensions', () => {
    const bytes = header();
    expect(inspectCanvasPng(bytes)).toEqual({bytes,width:2,height:3});
    expect(inspectCanvasPng(bytes).bytes).toBe(bytes);
  });
  it('uses a view offset rather than reading an unrelated buffer prefix', () => {
    const bytes = header(9, 7, 11);
    expect(inspectCanvasPng(bytes)).toEqual({bytes,width:9,height:7});
  });
  it('rejects ArrayBuffer without coercing it to accepted bytes', () => {
    expect(() => inspectCanvasPng(header().buffer)).toThrow(/valid PNG byte stream.*shape-export-return.*ArrayBuffer/);
  });
  it('rejects HTML or SVG bytes and reports only the bounded header', () => {
    const bytes = new TextEncoder().encode('<svg xmlns="http://www.w3.org/2000/svg">secret-tail</svg>');
    let message = '';
    try { inspectCanvasPng(bytes); } catch (error) { message = (error as Error).message; }
    expect(message).toContain('shape-export-return');
    const diagnostic = JSON.parse(message.split('SDK return diagnostic: ')[1]);
    expect(diagnostic.byteLength).toBe(bytes.length);
    expect(diagnostic.headerHex).toHaveLength(48);
    expect(message).not.toContain('secret-tail');
  });
  it('rejects a truncated header', () => {
    expect(() => inspectCanvasPng(header().subarray(0, 23))).toThrow(/valid PNG byte stream/);
  });
  it('retains rejection of a zero-size canvas', () => {
    expect(() => inspectCanvasPng(header(0, 3))).toThrow(/empty PNG canvas/);
  });
  it('retains rejection of an incorrect IHDR marker', () => {
    const bytes = header(); bytes[12] = 0;
    expect(() => inspectCanvasPng(bytes)).toThrow(/valid PNG byte stream/);
  });
});

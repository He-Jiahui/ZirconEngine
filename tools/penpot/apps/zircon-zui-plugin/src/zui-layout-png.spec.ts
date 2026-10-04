import { describe, expect, it } from 'vitest';
import {
  rasterFramesEquivalent,
  type ScreenshotPixels,
} from '../tools/zui-layout-png';

function pixels(width: number, height: number, values: number[]): ScreenshotPixels {
  return { width, height, pixels: Buffer.from(values) };
}

describe('rasterFramesEquivalent', () => {
  it('accepts exact frames and one-channel antialiasing drift', () => {
    const first = pixels(2, 1, [10, 20, 30, 40, 50, 60]);
    const second = pixels(2, 1, [10, 20, 30, 40, 51, 60]);

    expect(rasterFramesEquivalent(first, second)).toBe(true);
  });

  it('rejects geometry changes and material pixel changes', () => {
    const first = pixels(2, 1, [10, 20, 30, 40, 50, 60]);

    expect(rasterFramesEquivalent(first, pixels(1, 2, [10, 20, 30, 40, 50, 60]))).toBe(
      false,
    );
    expect(rasterFramesEquivalent(first, pixels(2, 1, [10, 20, 30, 40, 52, 60]))).toBe(
      false,
    );
  });

  it('rejects too many small pixel changes', () => {
    const first = pixels(20, 1, new Array(60).fill(20));
    const second = Buffer.from(first.pixels);
    for (let offset = 0; offset < second.length; offset += 3)
      second[offset] = 21;

    expect(
      rasterFramesEquivalent(first, {
        width: first.width,
        height: first.height,
        pixels: second,
      }),
    ).toBe(false);
  });

  it('allows a small bounded antialiasing cluster on a large capture', () => {
    const width = 640;
    const height = 520;
    const first = pixels(width, height, new Array(width * height * 3).fill(20));
    const second = Buffer.from(first.pixels);
    // Chromium can repaint a narrow vector edge by one gray level between
    // consecutive captures. Keep this allowance bounded well below a visible
    // frame change while covering the observed 640x520 gizmo edge.
    for (let index = 0; index < 44; index += 1) second[index * 3] = 21;

    expect(
      rasterFramesEquivalent(first, {
        width,
        height,
        pixels: second,
      }),
    ).toBe(true);

    const broad = Buffer.from(first.pixels);
    for (let index = 0; index < 65; index += 1) broad[index * 3] = 21;
    expect(
      rasterFramesEquivalent(first, {
        width,
        height,
        pixels: broad,
      }),
    ).toBe(false);
  });
});

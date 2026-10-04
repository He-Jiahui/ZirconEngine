import {
  auditVisualPixels,
  hasVisibleSmallControl,
  hasVisibleThinControl,
} from './visual-pixel-audit';

describe('Penpot preview pixel audit', () => {
  it('checks a real 1px divider locally and rejects a blank or displaced line', () => {
    const image = {
      width: 360,
      height: 520,
      pixels: new Uint8Array(360 * 520 * 3).fill(24),
    };
    const bounds = [{ x: 0, y: 0, width: 360, height: 4 }];
    expect(hasVisibleThinControl(image, bounds, 1)).toBe(false);
    image.pixels.fill(51, 360 * 3, 720 * 3);
    expect(hasVisibleThinControl(image, bounds, 1)).toBe(true);
    expect(hasVisibleThinControl(image, [{ ...bounds[0], y: 24 }], 1)).toBe(
      false,
    );
    expect(
      hasVisibleThinControl(image, [{ ...bounds[0], height: 520 }], 1),
    ).toBe(false);
  });
  it('accepts a small icon with fewer than 100 colored pixels inside its control', () => {
    const image = {
      width: 360,
      height: 520,
      pixels: new Uint8Array(360 * 520 * 3).fill(24),
    };
    for (let y = 8; y < 16; y++)
      for (let x = 8; x < 16; x++)
        image.pixels.fill(200, (y * 360 + x) * 3, (y * 360 + x + 1) * 3);
    expect(
      hasVisibleSmallControl(image, [{ x: 0, y: 0, width: 32, height: 32 }], 1),
    ).toBe(true);
    expect(
      hasVisibleSmallControl(
        image,
        [{ x: 100, y: 100, width: 32, height: 32 }],
        1,
      ),
    ).toBe(false);
  });

  it('still rejects a uniform small control and refuses the exception for page-size roots', () => {
    const image = {
      width: 360,
      height: 520,
      pixels: new Uint8Array(360 * 520 * 3).fill(24),
    };
    expect(
      hasVisibleSmallControl(image, [{ x: 0, y: 0, width: 32, height: 32 }], 1),
    ).toBe(false);
    expect(
      hasVisibleSmallControl(
        image,
        [{ x: 0, y: 0, width: 360, height: 520 }],
        1,
      ),
    ).toBe(false);
  });
  it('rejects a flat screenshot with no visible detail', () => {
    const pixels = new Uint8Array(10 * 10 * 3).fill(255);

    expect(auditVisualPixels(10, 10, pixels)).toEqual({
      colorBucketCount: 1,
      detailPixelRatio: 0,
    });
  });

  it('measures controls against the dominant RGB canvas color', () => {
    const pixels = new Uint8Array(10 * 10 * 3).fill(255);
    for (let pixel = 90; pixel < 100; pixel += 1) {
      const offset = pixel * 3;
      pixels[offset] = 24;
      pixels[offset + 1] = 32;
      pixels[offset + 2] = 38;
    }

    expect(auditVisualPixels(10, 10, pixels)).toEqual({
      colorBucketCount: 2,
      detailPixelRatio: 0.1,
    });
  });

  it('counts opaque component content on a transparent RGBA background', () => {
    const pixels = new Uint8Array(10 * 10 * 4);
    for (let pixel = 95; pixel < 100; pixel += 1) {
      const offset = pixel * 4;
      pixels[offset] = 45;
      pixels[offset + 1] = 212;
      pixels[offset + 2] = 191;
      pixels[offset + 3] = 255;
    }

    expect(auditVisualPixels(10, 10, pixels)).toEqual({
      colorBucketCount: 2,
      detailPixelRatio: 0.05,
    });
  });
});

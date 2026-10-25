import type { Board, Text } from '@penpot/plugin-types';
import { describe, expect, it } from 'vitest';
import {
  currentContentMeasurementScale,
  LINEAR_CONTENT_MEASUREMENT_SCALE,
} from './penpot-content-measurement-guard';
import {
  assertExportShapeGuard,
  writeExportShapeGuard,
} from './penpot-export-guard';
function fixture(legacy = false) {
  const data = new Map<string, string>();
  const probeData = new Map<string, string>();
  const root = {
    id: 'asset',
    type: 'board',
    name: 'Asset',
    x: 0,
    y: 0,
    width: 960,
    height: 640,
    children: [] as Text[],
    getSharedPluginData: (_: string, k: string) =>
      k === 'role' ? 'asset' : (data.get(k) ?? ''),
    setSharedPluginData: (_: string, k: string, v: string) => {
      data.set(k, v);
    },
  } as unknown as Board;
  const probe = {
    id: 'probe',
    type: 'text',
    name: 'Content font measurement',
    x: 0,
    y: 0,
    width: 100,
    height: 18,
    parent: root,
    opacity: 0,
    growType: 'auto-width',
    layoutChild: { absolute: true },
    fontFamily: 'Fira Sans',
    fontSize: String(14 * (legacy ? 1 : LINEAR_CONTENT_MEASUREMENT_SCALE)),
    fontWeight: '400',
    lineHeight: '1.2',
    letterSpacing: '0',
    characters: 'Before',
    getSharedPluginData: (_: string, k: string) =>
      k === 'role' ? 'auxiliary' : (probeData.get(k) ?? ''),
  } as unknown as Text;
  root.children.push(probe);
  probeData.set('linear-content-measurement', 'title');
  const update = (text: string) => {
    probe.characters = text;
    probeData.set(
      'linear-content-measurement-signature',
      JSON.stringify({
        text,
        ...(legacy ? {} : { measurementScale: LINEAR_CONTENT_MEASUREMENT_SCALE }),
        style: {
          family: 'Fira Sans',
          size: 14,
          weight: '400',
          lineHeight: 1.2,
        },
        runs: [],
      }),
    );
  };
  update('Before');
  writeExportShapeGuard(root);
  return { root, probe, update, data, probeData };
}
it('accepts a regenerated invisible intrinsic probe without rewriting its immutable guard', () => {
  const f = fixture();
  const original = f.data.get('export-shapes-guard');
  f.update('After supported text edit');
  f.probe.resize = () => {};
  Object.assign(f.probe, { width: 220, height: 22 });
  expect(() => assertExportShapeGuard(f.root)).not.toThrow();
  expect(f.data.get('export-shapes-guard')).toBe(original);
});
it('rejects a probe made visible or changed outside its source signature', () => {
  const f = fixture();
  f.probe.opacity = 1;
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
  f.probe.opacity = 0;
  f.probe.characters = 'Unmapped probe edit';
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
});
it('continues guarding measurements with an unsupported font or grow policy', () => {
  const f = fixture();
  f.probe.fontFamily = 'Other Font';
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
  f.probe.fontFamily = 'Fira Sans';
  f.probe.growType = 'fixed';
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
});

describe('signed intrinsic measurement scale', () => {
  it('accepts only matching scaled typography without changing the immutable shape guard', () => {
    const f = fixture();
    const original = f.data.get('export-shapes-guard');
    expect(currentContentMeasurementScale(f.probe)).toBe(LINEAR_CONTENT_MEASUREMENT_SCALE);
    f.update('A supported new value');
    Object.assign(f.probe, { width: 1457, height: 269 });
    expect(() => assertExportShapeGuard(f.root)).not.toThrow();
    expect(f.data.get('export-shapes-guard')).toBe(original);
  });
  it('retains validated legacy 1x probes until a source refresh upgrades them', () => {
    const f = fixture(true);
    expect(currentContentMeasurementScale(f.probe)).toBe(1);
    f.update('Still a signed legacy value');
    expect(() => assertExportShapeGuard(f.root)).not.toThrow();
    const signature = JSON.parse(f.probeData.get('linear-content-measurement-signature')!);
    signature.measurementScale = LINEAR_CONTENT_MEASUREMENT_SCALE;
    f.probeData.set('linear-content-measurement-signature', JSON.stringify(signature));
    f.probe.fontSize = String(14 * LINEAR_CONTENT_MEASUREMENT_SCALE);
    expect(currentContentMeasurementScale(f.probe)).toBe(LINEAR_CONTENT_MEASUREMENT_SCALE);
    expect(() => assertExportShapeGuard(f.root)).not.toThrow();
  });
  it.each([0, 1, 2, -16, '16', null, {}, Number.POSITIVE_INFINITY])(
    'rejects unknown explicit scale %s rather than interpreting it as a legacy probe', (scale) => {
      const f = fixture();
      const signature = JSON.parse(f.probeData.get('linear-content-measurement-signature')!);
      signature.measurementScale = scale;
      f.probeData.set('linear-content-measurement-signature', JSON.stringify(signature));
      // Even a matching 1x font cannot make an explicit unsupported scale valid.
      if (scale === 1) f.probe.fontSize = '14';
      expect(currentContentMeasurementScale(f.probe)).toBeUndefined();
      expect(() => assertExportShapeGuard(f.root)).toThrow(/without a ZUI source mapping/);
    },
  );
  it.each([
    ['fontSize', '14'],
    ['lineHeight', String(1.2 * LINEAR_CONTENT_MEASUREMENT_SCALE)],
    ['letterSpacing', '1'],
    ['letterSpacing', 'mixed'],
    ['letterSpacing', ''],
    ['letterSpacing', null],
    ['fontWeight', '700'],
  ])('rejects unscaled or unmapped %s=%s', (property, value) => {
    const f = fixture();
    Object.assign(f.probe, { [String(property)]: value });
    expect(currentContentMeasurementScale(f.probe)).toBeUndefined();
    expect(() => assertExportShapeGuard(f.root)).toThrow(/without a ZUI source mapping/);
  });
  it('does not reinterpret a scaled font as legacy when its scale metadata is removed', () => {
    const f = fixture();
    const signature = JSON.parse(f.probeData.get('linear-content-measurement-signature')!);
    delete signature.measurementScale;
    f.probeData.set('linear-content-measurement-signature', JSON.stringify(signature));
    expect(currentContentMeasurementScale(f.probe)).toBeUndefined();
    expect(() => assertExportShapeGuard(f.root)).toThrow(/without a ZUI source mapping/);
  });
});

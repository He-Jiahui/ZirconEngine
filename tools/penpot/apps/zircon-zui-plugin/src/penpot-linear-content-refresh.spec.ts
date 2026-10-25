import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Board, Text } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { projectZuiDocument } from './bridge/penpot-projection';
import {
  synchronizeLinearContentMeasurements,
  measuredLinearDesiredSizes,
  linearContentMeasurementSnapshot,
  linearContentMeasurementsReady,
} from './penpot-linear-content';
import { LINEAR_CONTENT_MEASUREMENT_SCALE } from './penpot-content-measurement-guard';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { layoutAuditStabilitySnapshot } from './penpot-layout-settlement';
import { auditLayoutBounds } from './penpot-render-layout';

// Emulate the SDK's ceil(longest_line) / ceil(paragraph height), using
// arbitrary scalable glyph advances rather than any observed menu geometry.
function intrinsicWidth(characters: string, size: number, weight = '400'): number {
  const ems = [...characters].reduce(
    (sum, character) => sum +
      (character === ' ' ? 0.253 : character.codePointAt(0)! > 127 ? 1 : 0.5173),
    0,
  );
  return ems * size * (weight === '700' ? 1.08 : 1);
}

function fixture() {
  const texts: Text[] = [];
  let fontApplications = 0;
  let rangeApplications = 0;
  const font = {
    fontFamily: 'Fira Sans',
    variants: [
      { fontWeight: '400', fontStyle: 'normal' },
      { fontWeight: '700', fontStyle: 'normal' },
    ],
    applyToText(text: Text, variant: { fontWeight: string }) {
      fontApplications++;
      text.fontFamily = 'Fira Sans';
      text.fontWeight = variant.fontWeight;
    },
    applyToRange(range: { text: Text }) {
      rangeApplications++;
      range.text.fontWeight = 'mixed';
    },
  };
  const asset = {
    x: 0,
    y: 0,
    children: texts,
    appendChild(text: Text) {
      texts.push(text);
      Object.assign(text, { parent: asset });
    },
    getSharedPluginData: (_namespace: string, key: string) =>
      key === 'role' ? 'asset' : '',
  } as unknown as Board;
  vi.stubGlobal('penpot', {
    fonts: { all: [font] },
    createText(characters: string) {
      const metadata = new Map<string, string>();
      let bounds: { width: number; height: number } | undefined;
      let fontSize = '14';
      const text = {
        type: 'text',
        characters,
        get fontSize(): string { return fontSize; },
        set fontSize(value: string) {
          fontSize = value;
          bounds = undefined;
        },
        lineHeight: '1.4',
        fontWeight: '400',
        get width(): number {
          return bounds?.width ?? Math.ceil(intrinsicWidth(
            text.characters, Number(text.fontSize), text.fontWeight,
          ));
        },
        get height(): number {
          return bounds?.height ?? Math.ceil(
            Number(text.fontSize) * Number(text.lineHeight),
          );
        },
        growType: 'fixed',
        layoutChild: { absolute: false },
        getSharedPluginData(namespace: string, key: string) {
          return metadata.get(namespace + ':' + key) ?? '';
        },
        setSharedPluginData(namespace: string, key: string, value: string) {
          metadata.set(namespace + ':' + key, value);
        },
        resize(width: number, height: number) {
          bounds = { width, height };
        },
        getRange() {
          return { text: text as unknown as Text };
        },
        remove() {
          texts.splice(texts.indexOf(text as unknown as Text), 1);
        },
      };
      return text;
    },
  });
  const document: ZuiDocument = {
    asset: { kind: 'view', id: 'test', version: 2 },
    root: { node: 'root' },
    nodes: {
      root: {
        component: 'Label',
        props: { text: 'First', font_family: 'Fira Sans', font_size: 14 },
        layout: { width: { min: 32, max: 32, stretch: 'Fixed' } },
      },
    },
  };
  return {
    asset, texts, document,
    fontApplications: () => fontApplications,
    rangeApplications: () => rangeApplications,
  };
}
afterEach(() => vi.unstubAllGlobals());
describe('reused intrinsic font probes', () => {
  it('updates text and font without duplicating or repeatedly reshaping unchanged probes', () => {
    const f = fixture();
    synchronizeLinearContentMeasurements(
      f.asset,
      f.document,
      projectZuiDocument(f.document),
    );
    const first = f.texts[0];
    expect(first.growType).toBe('auto-width');
    const before = linearContentMeasurementSnapshot(f.asset);
    synchronizeLinearContentMeasurements(
      f.asset,
      f.document,
      projectZuiDocument(f.document),
    );
    expect(f.fontApplications()).toBe(1);
    expect(f.texts).toHaveLength(1);
    f.document.nodes!['root'].props!['text'] = 'Long Chinese label 中文';
    f.document.nodes!['root'].props!['font_size'] = 18;
    synchronizeLinearContentMeasurements(
      f.asset,
      f.document,
      projectZuiDocument(f.document),
    );
    expect(f.texts[0]).toBe(first);
    expect(first.characters).toBe('Long Chinese label 中文');
    expect(first.fontSize).toBe(String(18 * LINEAR_CONTENT_MEASUREMENT_SCALE));
    expect(f.fontApplications()).toBe(2);
    const after = linearContentMeasurementSnapshot(f.asset);
    expect(after[0].styleSignature).not.toBe(before[0].styleSignature);
    const audit = {
      ...auditLayoutBounds([]),
      contentMeasurements: before,
    };
    expect(layoutAuditStabilitySnapshot(audit)).not.toEqual(
      layoutAuditStabilitySnapshot({ ...audit, contentMeasurements: after }),
    );
  });
  it('keeps fixed-width intrinsic text unwrapped and removes probes no longer in the subtree', () => {
    const f = fixture();
    synchronizeLinearContentMeasurements(
      f.asset,
      f.document,
      projectZuiDocument(f.document),
    );
    f.texts[0].resize(
      280 * LINEAR_CONTENT_MEASUREMENT_SCALE,
      20 * LINEAR_CONTENT_MEASUREMENT_SCALE,
    );
    expect(
      measuredLinearDesiredSizes(
        f.asset,
        f.document,
        projectZuiDocument(f.document),
      ).get('root'),
    ).toEqual({ width: 32, height: 20 });
    f.document.nodes!['root'].props!['text'] = '';
    synchronizeLinearContentMeasurements(
      f.asset,
      f.document,
      projectZuiDocument(f.document),
    );
    expect(f.texts).toHaveLength(0);
  });
  it('normalizes SDK integer dimensions once and keeps visible typography and source unchanged', () => {
    const f = fixture();
    f.document.nodes!['root'].layout = { width: { stretch: 'Fixed' } };
    Object.assign(f.document.nodes!['root'].props!, {
      font_size: 13.375, font_weight: 700, line_height: 17.25,
    });
    const before = JSON.stringify(f.document);
    const projection = projectZuiDocument(f.document);
    const visibleText = structuredClone(projection.shapes[0].text);
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    const probe = f.texts[0];
    expect(probe.fontSize).toBe(String(13.375 * LINEAR_CONTENT_MEASUREMENT_SCALE));
    expect(probe.lineHeight).toBe(String(17.25 / 13.375));
    expect(probe.letterSpacing).toBe('0');
    expect(probe.fontWeight).toBe('700');
    const logicalWidth = intrinsicWidth('First', 13.375, '700');
    const expected = {
      width: Math.ceil(logicalWidth * LINEAR_CONTENT_MEASUREMENT_SCALE) /
        LINEAR_CONTENT_MEASUREMENT_SCALE,
      height: 17.25,
    };
    const snapshot = linearContentMeasurementSnapshot(f.asset)[0];
    expect(snapshot.width).toBe(expected.width);
    expect(snapshot.height).toBe(expected.height);
    expect(snapshot.width - logicalWidth).toBeGreaterThanOrEqual(0);
    expect(snapshot.width - logicalWidth).toBeLessThan(1 / LINEAR_CONTENT_MEASUREMENT_SCALE);
    expect(JSON.parse(snapshot.styleSignature)).toMatchObject({
      measurementScale: LINEAR_CONTENT_MEASUREMENT_SCALE,
      style: { size: 13.375, lineHeight: 17.25 / 13.375 },
    });
    expect(measuredLinearDesiredSizes(f.asset, f.document, projection).get('root'))
      .toEqual(expected);
    expect(projection.shapes[0].text).toEqual(visibleText);
    expect(projection.shapes[0].text?.fontSize).toBe(13.375);
    expect(JSON.stringify(f.document)).toBe(before);
  });
  it('bounds cumulative rounding across different sizes and scripts before row allocation', () => {
    const f = fixture();
    const labels = [
      { text: 'Alpha', size: 13.375 },
      { text: '属性', size: 16.25 },
      { text: 'Long label', size: 11.125 },
    ];
    f.document.nodes = {
      root: {
        component: 'HorizontalBox',
        layout: {
          container: { kind: 'HorizontalBox', gap: 2.25 },
          padding: { left: 3.5, right: 4.25 },
        },
        children: labels.map((_label, index) => ({ node: 'label' + index })),
      },
      ...Object.fromEntries(labels.map((label, index) => ['label' + index, {
        component: 'Label',
        props: { text: label.text, font_family: 'Fira Sans', font_size: label.size },
      }])),
    };
    const before = JSON.stringify(f.document);
    const projection = projectZuiDocument(f.document);
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    const logicalAdvances = labels.map((label) => intrinsicWidth(label.text, label.size));
    const logicalSum = logicalAdvances.reduce((sum, value) => sum + value, 0) +
      2 * 2.25 + 3.5 + 4.25;
    const width = measuredLinearDesiredSizes(f.asset, f.document, projection).get('root')!.width;
    expect(width - logicalSum).toBeGreaterThanOrEqual(0);
    expect(width - logicalSum).toBeLessThan(labels.length / LINEAR_CONTENT_MEASUREMENT_SCALE);
    expect(linearContentMeasurementSnapshot(f.asset).map((probe) => probe.width))
      .toEqual(logicalAdvances.map((advance) => Math.ceil(
        advance * LINEAR_CONTENT_MEASUREMENT_SCALE,
      ) / LINEAR_CONTENT_MEASUREMENT_SCALE));
    expect(JSON.stringify(f.document)).toBe(before);
  });
  it('upgrades a valid legacy probe once without replacing it', () => {
    const f = fixture();
    const projection = projectZuiDocument(f.document);
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    const probe = f.texts[0];
    const signature = JSON.parse(probe.getSharedPluginData(
      ZUI_METADATA_NAMESPACE, 'linear-content-measurement-signature',
    ));
    delete signature.measurementScale;
    probe.setSharedPluginData(
      ZUI_METADATA_NAMESPACE, 'linear-content-measurement-signature', JSON.stringify(signature),
    );
    probe.fontSize = String(signature.style.size);
    probe.resize(90, 20);
    expect(linearContentMeasurementSnapshot(f.asset)[0].width).toBe(90);
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    expect(f.texts).toEqual([probe]);
    expect(probe.fontSize).toBe(String(14 * LINEAR_CONTENT_MEASUREMENT_SCALE));
    expect(f.fontApplications()).toBe(2);
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    expect(f.fontApplications()).toBe(2);
  });
  it('keeps inline weight ranges at the same probe scale', () => {
    const f = fixture();
    const projection = projectZuiDocument(f.document);
    projection.shapes[0].text!.richTextRuns = [{ start: 0, end: 2, fontWeight: '700' }];
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    expect(f.rangeApplications()).toBe(1);
    expect(f.texts[0].fontWeight).toBe('mixed');
    expect(f.texts[0].fontSize).toBe(String(14 * LINEAR_CONTENT_MEASUREMENT_SCALE));
    expect(() => linearContentMeasurementSnapshot(f.asset)).not.toThrow();
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    expect(f.rangeApplications()).toBe(1);
  });
  it.each(['fontSize', 'lineHeight', 'letterSpacing', 'scale', 'legacyScaleMismatch', 'signature'])(
    'rejects a changed %s before refreshing a probe', (field) => {
      const f = fixture();
      const projection = projectZuiDocument(f.document);
      synchronizeLinearContentMeasurements(f.asset, f.document, projection);
      const probe = f.texts[0];
      if (field === 'fontSize') probe.fontSize = '14';
      else if (field === 'lineHeight') probe.lineHeight = String(Number(probe.lineHeight) * 16);
      else if (field === 'letterSpacing') probe.letterSpacing = '1';
      else {
        const signature = JSON.parse(probe.getSharedPluginData(
          ZUI_METADATA_NAMESPACE, 'linear-content-measurement-signature',
        ));
        if (field === 'scale') signature.measurementScale = 2;
        else delete signature.measurementScale;
        probe.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'linear-content-measurement-signature',
          field === 'signature' ? '{invalid' : JSON.stringify(signature));
      }
      expect(() => synchronizeLinearContentMeasurements(f.asset, f.document, projection))
        .toThrow(/Invalid native content font measurement/);
      expect(() => measuredLinearDesiredSizes(f.asset, f.document, projection))
        .toThrow(/Invalid native content font measurement/);
      expect(() => linearContentMeasurementSnapshot(f.asset))
        .toThrow(/Invalid native content font measurement/);
      expect(f.fontApplications()).toBe(1);
    },
  );
  it('checks readiness in logical units and rejects nonfinite measured dimensions', () => {
    const f = fixture();
    const projection = projectZuiDocument(f.document);
    synchronizeLinearContentMeasurements(f.asset, f.document, projection);
    expect(linearContentMeasurementsReady(f.asset)).toBe(true);
    f.texts[0].resize(16, 32);
    expect(linearContentMeasurementsReady(f.asset)).toBe(false);
    f.texts[0].resize(32, 16);
    expect(linearContentMeasurementsReady(f.asset)).toBe(false);
    f.texts[0].resize(32, 32);
    expect(linearContentMeasurementsReady(f.asset)).toBe(true);
    f.texts[0].resize(Infinity, 32);
    expect(linearContentMeasurementsReady(f.asset)).toBe(false);
    expect(() => measuredLinearDesiredSizes(f.asset, f.document, projection))
      .toThrow(/Invalid native content font measurement dimensions/);
  });
  it('does not read allocated preview geometry as painter content', () => {
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'test', version: 2 },
      root: { node: 'root' },
      nodes: {
        root: {
          component: 'Table',
          layout: { width: { stretch: 'Fixed' }, height: { stretch: 'Fixed' } },
        },
      },
    };
    const projection = projectZuiDocument(doc),
      root = projection.shapes[0];
    root.table = {} as NonNullable<typeof root.table>;
    const asset = { children: [] } as unknown as Board;
    const before = measuredLinearDesiredSizes(asset, doc, projection).get(
      'root',
    );
    root.geometry.width = 900;
    root.geometry.height = 900;
    expect(
      measuredLinearDesiredSizes(asset, doc, projection).get('root'),
    ).toEqual(before);
    expect(before).toEqual({ width: 0, height: 0 });
  });
});

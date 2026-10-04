import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Board, Text } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { projectZuiDocument } from './bridge/penpot-projection';
import {
  synchronizeLinearContentMeasurements,
  measuredLinearDesiredSizes,
  linearContentMeasurementSnapshot,
} from './penpot-linear-content';
import { layoutAuditStabilitySnapshot } from './penpot-layout-settlement';
import { auditLayoutBounds } from './penpot-render-layout';

function fixture() {
  const texts: Text[] = [];
  let fontApplications = 0;
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
    applyToRange() {},
  };
  const asset = {
    x: 0,
    y: 0,
    children: texts,
    appendChild(text: Text) {
      texts.push(text);
    },
  } as unknown as Board;
  vi.stubGlobal('penpot', {
    fonts: { all: [font] },
    createText(characters: string) {
      const metadata = new Map<string, string>();
      const text = {
        type: 'text',
        characters,
        width: 90,
        height: 20,
        growType: 'fixed',
        layoutChild: { absolute: false },
        getSharedPluginData(namespace: string, key: string) {
          return metadata.get(namespace + ':' + key) ?? '';
        },
        setSharedPluginData(namespace: string, key: string, value: string) {
          metadata.set(namespace + ':' + key, value);
        },
        resize(width: number, height: number) {
          text.width = width;
          text.height = height;
        },
        getRange() {
          return {};
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
  return { asset, texts, document, fontApplications: () => fontApplications };
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
    expect(first.fontSize).toBe('18');
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
    f.texts[0].resize(280, 20);
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

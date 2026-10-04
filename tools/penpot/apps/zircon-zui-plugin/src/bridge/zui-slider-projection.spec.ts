import type { ZuiDocument } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import { sliderGeometry } from './zui-slider-geometry';
import {
  createPenpotBridgeAsset,
  parsePenpotBridgeAsset,
  serializePenpotBridgeAsset,
  reconcilePenpotBridgeAsset,
} from './penpot-asset';

function fixture(range = true): ZuiDocument {
  return {
    asset: { id: 'res://slider.zui', kind: 'component', version: 2 },
    components: { Sample: { root: 'root' } },
    tokens: { 'label.color': '#b3b3b3', 'thumb.size': 8 },
    nodes: {
      root: {
        component: range ? 'RangeSlider' : 'RangeField',
        props: {
          value: 80,
          min: 0,
          max: 100,
          font_size: 14,
          label_color: '$label.color',
          thumb_size: '$thumb.size',
          value_text_inset: 8,
          ...(range
            ? {
                label_text: 'Range',
                range_min_percent: 0.2,
                value_percent: 0.8,
                value_text: '0.80',
              }
            : {}),
        },
        layout: {
          width: { preferred: 360, stretch: 'Stretch' },
          height: { preferred: 32 },
        },
        events: [
          {
            id: 'Slider/Changed',
            event: 'Change',
            route: 'tests.slider.update_value',
          },
        ],
        custom_owner: { retain: ['source', 'contract'] },
      },
    },
  };
}

describe('native slider projection', () => {
  it('keeps the exact source while projecting independent label and value text', () => {
    const document = fixture();
    const projection = projectZuiDocument(document);
    const node = projection.shapes[0];
    expect(node.text).toMatchObject({
      characters: '0.80',
      property: 'value_text',
      fontSize: 14,
    });
    expect(node.textFragments?.['label']).toMatchObject({
      characters: 'Range',
      property: 'label_text',
      color: '#b3b3b3',
    });
    expect(node.textFragments?.['range-min']).toMatchObject({
      characters: '0.20',
      property: null,
    });
    expect(
      reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(document);
  });

  it('uses native label, track and value lanes at each component width', () => {
    const slider = projectZuiDocument(fixture()).shapes[0].slider!;
    for (const width of [240, 360, 480]) {
      const geometry = sliderGeometry(slider, width, 32);
      expect(geometry.parts['track']).toMatchObject({
        x: 70,
        y: 14,
        width: width - 132,
        height: 4,
      });
      expect(geometry.parts['primary']).toMatchObject({
        x: width - 52,
        y: 4,
        width: 44,
        height: 24,
      });
      expect(geometry.texts['label']).toMatchObject({
        x: 8,
        width: 50,
        height: 19.6,
      });
      expect(geometry.texts['primary']).toMatchObject({
        x: width - 44,
        width: 28,
      });
      expect(geometry.parts['thumb'].x).toBeCloseTo(
        70 + (width - 132) * 0.8 - 4,
      );
      expect(geometry.parts['range-thumb'].x).toBeCloseTo(
        70 + (width - 132) * 0.2 - 4,
      );
      expect(geometry.texts['range-min']).toBeUndefined();
    }
    expect(sliderGeometry(slider, 360, 64).texts['range-min']).toMatchObject({
      x: 78,
      width: 28,
    });
    expect(sliderGeometry(slider, 60, 32).parts).toEqual({});
  });

  it('normalizes values and preserves explicit formatting without changing numeric data', () => {
    const document = fixture(false);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    expect(snapshot.shapes[0].current.text?.characters).toBe('0.80');
    snapshot.shapes[0].current.text!.characters = '80%';
    expect(
      reconcileZuiDocument(document, snapshot).document.nodes!['root'].props,
    ).toMatchObject({ value: 80, value_text: '80%' });
  });

  it('writes label content and color to their own authored properties', () => {
    const document = fixture();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.textFragments!['label'].characters = 'Level';
    snapshot.shapes[0].current.textFragments!['label'].color = '#cccccc';
    const result = reconcileZuiDocument(document, snapshot).document;
    expect(result.nodes!['root'].props).toMatchObject({
      label_text: 'Level',
      label_color: '#cccccc',
      value_text: '0.80',
    });
    expect(result.tokens).toEqual(document.tokens);
    expect(result.nodes!['root'].events).toEqual(
      document.nodes!['root'].events,
    );
    expect(result.nodes!['root']['custom_owner']).toEqual(
      document.nodes!['root']['custom_owner'],
    );
  });

  it('serializes fragment baselines and detects tampered source-property mappings', () => {
    const document = fixture();
    const asset = parsePenpotBridgeAsset(
      serializePenpotBridgeAsset(
        createPenpotBridgeAsset(document, 'slider.zui'),
      ),
    );
    expect(reconcilePenpotBridgeAsset(asset).document).toEqual(document);
    asset.snapshot.shapes[0].current.textFragments!['label'].property =
      'value_text';
    expect(() => reconcilePenpotBridgeAsset(asset)).toThrow(
      /Text property metadata/,
    );
  });

  it('retains explicit owner paint and rejects unresolved native metrics', () => {
    const document = fixture();
    document.nodes!['root'].props!['background_color'] = '#242424';
    expect(projectZuiDocument(document).shapes[0].paint.fillColor).toBe(
      '#242424',
    );
    document.nodes!['root'].props!['thumb_size'] = '$missing.size';
    expect(() => projectZuiDocument(document)).toThrow(
      /Unresolved slider token/,
    );
  });

  it('rejects lost fragments, derived-value edits and styles unsupported by the native painter', () => {
    const document = fixture();
    for (const edit of [
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        delete s.shapes[0].current.textFragments!['label'];
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.textFragments!['range-min'].characters = '0.30';
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.textFragments!['label'].fontSize = 16;
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.text!.fontWeight = '600';
      },
    ]) {
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      edit(snapshot);
      expect(() => reconcileZuiDocument(document, snapshot)).toThrow();
    }
  });

  it('bounds ticks and applies native validation, unavailable and halo states', () => {
    const document = fixture();
    document.nodes!['root'].props!['tick_count'] = 10000;
    document.nodes!['root'].state = {
      validation_level: 'error',
      pressed: true,
    };
    let node = projectZuiDocument(document).shapes[0];
    let geometry = sliderGeometry(node.slider!, 240, 32);
    expect(geometry.parts['fill'].fill).toBe('#eb605c');
    expect(geometry.parts['primary'].stroke).toBe('#eb605c');
    expect(geometry.parts['thumb-halo']).toBeDefined();
    expect(
      Object.keys(geometry.parts).filter((key) => key.startsWith('tick-')),
    ).toHaveLength(108);
    document.nodes!['root'].state = { disabled: true, pressed: true };
    node = projectZuiDocument(document).shapes[0];
    geometry = sliderGeometry(node.slider!, 240, 32);
    expect(node.text?.color).toBe('#737373');
    expect(geometry.parts['fill'].fill).toBe('#737373');
    expect(geometry.parts['thumb-halo']).toBeUndefined();
  });
});

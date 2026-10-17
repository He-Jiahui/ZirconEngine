import { describe, expect, it } from 'vitest';
import { measureNativeLeafContent } from './penpot-content-material';
import { projectZuiDocument } from './bridge/penpot-projection';
import { measureLinearDesiredSizes } from './penpot-linear-content';
import type { ZuiDocument } from './bridge/zui-document';

function document(): ZuiDocument {
  return {
    asset: { kind: 'view', id: 'res://tests/desired.zui', version: 2 },
    root: { node: 'drawer' },
    nodes: {
      drawer: {
        component: 'VerticalBox',
        layout: {
          container: { kind: 'VerticalBox', gap: 10 },
          padding: { left: 8, right: 8, top: 6, bottom: 6 },
          height: { stretch: 'Stretch' },
        },
        children: [
          { node: 'heading' },
          {
            node: 'stack',
            slot: { layout: { padding: { top: 3, bottom: 5 } } },
          },
        ],
      },
      heading: {
        component: 'Label',
        layout: {
          height: { min: 24, preferred: 24, max: 24, stretch: 'Fixed' },
        },
      },
      stack: {
        component: 'VerticalBox',
        layout: {
          container: { kind: 'VerticalBox', gap: 4 },
          height: { stretch: 'Stretch' },
        },
        children: [{ node: 'one' }, { node: 'two' }, { node: 'hidden' }],
      },
      one: {
        component: 'Button',
        layout: {
          height: { min: 32, preferred: 32, max: 32, stretch: 'Fixed' },
        },
      },
      two: {
        component: 'Button',
        layout: {
          height: { min: 32, preferred: 32, max: 32, stretch: 'Fixed' },
        },
      },
      hidden: {
        component: 'Button',
        props: { visibility: 'collapsed' },
        layout: { height: { preferred: 500, stretch: 'Fixed' } },
      },
    },
  };
}
describe('native desired sizes before weighted allocation', () => {
  it('uses bottom-up child content including gaps and both padding owners', () => {
    const doc = document(),
      projection = projectZuiDocument(doc);
    const measured = measureLinearDesiredSizes(doc, projection, () => ({
      width: 40,
      height: 18,
    }));
    expect(measured.get('stack')).toEqual({ width: 40, height: 68 });
    expect(measured.get('drawer')).toEqual({ width: 56, height: 122 });
    expect(measured.has('hidden')).toBe(false);
  });
  it('does not take a projection fill height as intrinsic content', () => {
    const doc = document(),
      projection = projectZuiDocument(doc);
    projection.shapes.forEach((shape) => {
      if (shape.nodeId === 'drawer' || shape.nodeId === 'stack')
        shape.geometry.height = 900;
    });
    expect(
      measureLinearDesiredSizes(doc, projection, () => ({
        width: 40,
        height: 18,
      })).get('drawer')?.height,
    ).toBe(122);
  });
  it('respects Fixed and ParentDirected boundaries and min/max constraints', () => {
    const doc = document();
    doc.nodes!['drawer'].layout!['boundary'] = 'ParentDirected';
    doc.nodes!['drawer'].layout!['height'] = {
      min: 40,
      preferred: 60,
      max: 70,
      stretch: 'Stretch',
    };
    expect(
      measureLinearDesiredSizes(doc, projectZuiDocument(doc), () => ({
        width: 40,
        height: 18,
      })).get('drawer')?.height,
    ).toBe(60);
    doc.nodes!['drawer'].layout!['boundary'] = 'ContentDriven';
    expect(
      measureLinearDesiredSizes(doc, projectZuiDocument(doc), () => ({
        width: 40,
        height: 18,
      })).get('drawer')?.height,
    ).toBe(70);
  });
  it('keeps the own main-axis contract while accepting a slot cross-axis table', () => {
    const doc = document();
    doc.nodes!['drawer'].children![1].slot = {
      layout: {
        height: { preferred: 400, stretch: 'Fixed' },
        width: { preferred: 96, stretch: 'Fixed' },
      },
    };
    const size = measureLinearDesiredSizes(
      doc,
      projectZuiDocument(doc),
      () => ({ width: 40, height: 18 }),
    ).get('stack');
    expect(size).toEqual({ width: 96, height: 68 });
  });
  it('measures rows before StretchContent allocation from real leaf measurements', () => {
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'res://tests/toolbar.zui', version: 2 },
      root: { node: 'row' },
      nodes: {
        row: {
          component: 'HorizontalBox',
          layout: {
            container: { kind: 'HorizontalBox', gap: 8 },
            padding: { left: 4, right: 4 },
          },
          children: [{ node: 'a' }, { node: 'b' }],
        },
        a: { component: 'Label', props: { text: '甲' } },
        b: { component: 'Label', props: { text: 'Long label' } },
      },
    };
    const size = measureLinearDesiredSizes(
      doc,
      projectZuiDocument(doc),
      (_node, projected) =>
        projected.nodeId === 'a'
          ? { width: 14, height: 20 }
          : { width: 88, height: 20 },
    ).get('row');
    expect(size).toEqual({ width: 118, height: 20 });
  });
});

describe('dropdown desired size uses the same effective style as painting', () => {
  it('resolves ancestor caret styles and inline overrides without editing the document', () => {
    const doc: ZuiDocument = {
      asset: { kind: 'view', id: 'res://tests/dropdown-content.zui', version: 2 },
      root: { node: 'row' },
      stylesheets: [{ id: 'dropdown-style', rules: [{
        selector: '.toolbar > .choice', set: { self: { caret_gap: 10, caret_size: 20 } },
      }] }],
      nodes: {
        row: { component: 'HorizontalBox', classes: ['toolbar'],
          layout: { container: { kind: 'HorizontalBox', gap: 0 } },
          children: [{ node: 'choice', slot: { layout: { linear_size: { rule: 'Auto', shrink_value: 0 } } } }] },
        choice: { component: 'Dropdown', classes: ['choice'],
          props: { value_text: 'Perspective', caret_size: 12, caret_gap: 4 },
          style: { self: { caret_size: 24 } }, layout: { width: { stretch: 'Fixed' } } },
      },
    };
    const before = JSON.stringify(doc);
    const sizes = measureLinearDesiredSizes(doc, projectZuiDocument(doc),
      (node) => measureNativeLeafContent(doc, node, { width: 40, height: 16 }));
    expect(sizes.get('choice')).toEqual({ width: 94, height: 16 });
    expect(sizes.get('row')).toEqual({ width: 94, height: 16 });
    expect(JSON.stringify(doc)).toBe(before);
    doc.nodes!['choice'].layout!['width'] = { min: 50, max: 80, stretch: 'Fixed' };
    expect(measureLinearDesiredSizes(doc, projectZuiDocument(doc),
      (node) => measureNativeLeafContent(doc, node, { width: 40, height: 16 })).get('choice')?.width).toBe(80);
  });
});

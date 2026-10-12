import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import { parseZuiDocument } from './zui-document';
import { projectZuiDocument, cloneProjectionSnapshot, reconcileZuiDocument } from './penpot-projection';
import { fieldGeometry } from './zui-field-geometry';
import { FIELD_ICON_PATHS } from './zui-field-projection';

const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
const folder = 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs';
const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const dependencies = new LayoutDependencies();
beforeAll(async () => dependencies.load(repo, [theme, ...['field', 'number_field', 'search_input'].map((name) => `${folder}/workbench_${name}.zui`),
  'zircon_editor/assets/ui/editor/workbench/data/component_property_row.zui']));
async function load(name: string) {
  const sourcePath = `${folder}/workbench_${name}.zui`;
  const document = parseZuiDocument(await readFile(resolve(repo, sourcePath), 'utf8')).document;
  dependencies.embed(document, sourcePath, theme);
  return document;
}

describe('native Editor text fields', () => {

  it('keeps a source-owned property label separate from the editable value and surface', async () => {
    const sourcePath = 'zircon_editor/assets/ui/editor/workbench/data/component_property_row.zui';
    const document = parseZuiDocument(await readFile(resolve(repo, sourcePath), 'utf8')).document;
    dependencies.embed(document, sourcePath, theme);
    Object.assign(document.nodes!['root'].props!, { text: 'Far', value_text: '200' });
    const projection = projectZuiDocument(document);
    const row = projection.shapes[0];
    expect(row.text).toMatchObject({ characters: '200', property: 'value_text', fontSize: 13.333333 });
    expect(row.textFragments!['field-label']).toMatchObject({ characters: 'Far', property: 'text' });
    expect(row.paint.fillColor).toBeNull();
    const geometry = fieldGeometry(row.field!, 240, 28);
    expect(geometry.parts['surface']).toMatchObject({ x: 92, y: 0, width: 148, height: 28 });
    expect(geometry.texts['primary'].x).toBe(100);
    expect(geometry.texts['primary'].width).toBe(132);
    expect(geometry.texts['field-label']).toMatchObject({ x: 8, width: 76 });
    expect(geometry.texts['field-label'].y).toBeCloseTo(6, 5);
    expect(geometry.texts['primary'].height).toBeCloseTo(16, 5);
    expect(reconcileZuiDocument(document, cloneProjectionSnapshot(projection)).changes).toEqual([]);
    const snapshot = cloneProjectionSnapshot(projection);
    snapshot.shapes[0].current.text!.characters = '300';
    const edited = reconcileZuiDocument(document, snapshot).document;
    expect(edited.nodes!['root'].props!['value_text']).toBe('300');
    expect(edited.nodes!['root'].props!['text']).toBe('Far');
    document.nodes!['root'].props!['value_text'] = '';
    expect(projectZuiDocument(document).shapes[0].text?.characters).toBe('');
    const collapsed = fieldGeometry(row.field!, 80, 28);
    expect(collapsed.texts['primary']).toBeUndefined();
    expect(collapsed.parts['surface']).toBeUndefined();
    expect(collapsed.texts['field-label']).toMatchObject({ x: 8, width: 64 });
  });

  it('reserves the search and clear actions around the actual query', async () => {
    const document = await load('search_input');
    document.nodes!['root'].props!['query'] = 'material';
    const node = projectZuiDocument(document).shapes[0];
    expect(node.text).toMatchObject({ characters: 'material', property: 'query', fontSize: 13.333333 });
    expect(document['penpot_dependency_sources']).toEqual(expect.arrayContaining(Object.values(FIELD_ICON_PATHS)));
    for (const width of [240, 360, 480]) {
      const result = fieldGeometry(node.field!, width, 32);
      expect(result.icons['search']).toMatchObject({ x: 8, width: 16 });
      expect(result.icons['clear']).toMatchObject({ x: width - 24, width: 16 });
      expect(result.texts['primary']).toMatchObject({ x: 28, width: width - 60 });
    }
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.text!.characters = 'shader';
    expect(reconcileZuiDocument(document, snapshot).document.nodes!['root'].props!['query']).toBe('shader');
    const authoredRight = fieldGeometry({ ...node.field!, padRight: 12 }, 360, 32);
    expect(authoredRight.icons['clear']).toMatchObject({ x: 336, width: 16 });
    expect(authoredRight.texts['primary']).toMatchObject({ x: 28, width: 296 });
    const tall = fieldGeometry(node.field!, 360, 64);
    expect(tall.parts['surface']).toMatchObject({ y: 16, height: 32 });
    expect(tall.icons['search'].y).toBe(24);
  });

  it('keeps native search and stepper hit geometry when the authored text size changes', async () => {
    const search = await load('search_input');
    Object.assign(search.nodes!['root'].props!, { query: 'material', font_size: 20 });
    const shape = projectZuiDocument(search).shapes[0];
    expect(shape.text).toMatchObject({ fontSize: 20, lineHeight: 1.2 });
    const geometry = fieldGeometry(shape.field!, 240, 32);
    expect(geometry.icons['search']).toMatchObject({ x: 8, y: 8, width: 16, height: 16 });
    expect(geometry.icons['clear']).toMatchObject({ x: 216, y: 8, width: 16, height: 16 });
    expect(geometry.texts['primary']).toMatchObject({ x: 28, y: 4, width: 180, height: 24 });

    const number = await load('number_field');
    number.nodes!['root'].props!['font_size'] = 20;
    const stepper = fieldGeometry(projectZuiDocument(number).shapes[0].field!, 360, 32);
    expect(stepper.icons['stepper']).toMatchObject({ x: 346, y: 8, width: 10, height: 16 });
    expect(stepper.parts['stepper-divider']).toMatchObject({ x: 342, y: 4, width: 1, height: 24 });

    search.nodes!['root'].props!['font_size'] = 25;
    expect(fieldGeometry(projectZuiDocument(search).shapes[0].field!, 240, 32).texts['primary'])
      .toBeUndefined();
  });

  it('keeps the authored empty-value label before the placeholder and maps case-sensitive edits', async () => {
    const document = await load('field');
    document.nodes!['root'].props!['text'] = 'Name';
    document.nodes!['root'].props!['placeholder'] = 'Search';
    expect(projectZuiDocument(document).shapes[0].text).toMatchObject({ characters: 'Name', property: 'text' });
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.text!.characters = 'NAME';
    expect(reconcileZuiDocument(document, snapshot).document.nodes!['root'].props!['text']).toBe('NAME');
  });

  it('preserves native stepper geometry and reserves the trailing value inset', async () => {
    const document = await load('number_field');
    const node = projectZuiDocument(document).shapes[0];
    expect(node.text?.characters).toBe('42');
    const geometry = fieldGeometry(node.field!, 360, 32);
    expect(geometry.parts['stepper-divider']).toMatchObject({ x: 342, y: 4, width: 1, height: 24 });
    expect(geometry.icons['stepper']).toMatchObject({ x: 346, y: 8, width: 10, height: 16 });
    expect(geometry.texts['primary'].width).toBe(326);
    expect(reconcileZuiDocument(document, cloneProjectionSnapshot(projectZuiDocument(document))).changes).toEqual([]);
  });

  it('uses native validation colors and exports an authored font edit', async () => {
    const document = await load('field');
    document.nodes!['root'].state = { disabled: true };
    expect(projectZuiDocument(document).shapes[0].paint.fillColor).toBe('#2f2f2f');
    document.nodes!['root'].state = {};
    document.nodes!['root'].props!['validation_level'] = 'error';
    expect(projectZuiDocument(document).shapes[0].paint.strokeColor).toBe('#EF3535');
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.text!.fontSize = 25;
    const edited = reconcileZuiDocument(document, snapshot).document;
    expect(edited.nodes!['root'].props!['font_size']).toBe(25);
    expect(projectZuiDocument(edited).shapes[0].text!.fontSize).toBe(25);
    expect(document.nodes!['root'].props!['font_size']).toBe('$editor.typography.body.size');
  });

  it('preserves alpha and element opacity through a no-edit field roundtrip', async () => {
    const document = await load('field');
    Object.assign(document.nodes!['root'].props!, { background_color: '#11223380', border_color: '#44556640', opacity: 0.25 });
    const projection = projectZuiDocument(document);
    expect(projection.shapes[0].paint).toMatchObject({ fillColor: '#112233', fillOpacity: 128 / 255,
      strokeColor: '#445566', strokeOpacity: 64 / 255, opacity: 0.25 });
    expect(reconcileZuiDocument(document, cloneProjectionSnapshot(projection)).changes).toEqual([]);
  });

  it('rejects a numeric model edit until its source number type can be preserved', async () => {
    const document = await load('number_field');
    delete document.nodes!['root'].props!['value_text'];
    document.nodes!['root'].props!['value'] = 42;
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    expect(snapshot.shapes[0].current.text!.property).toBeNull();
    snapshot.shapes[0].current.text!.characters = '43';
    expect(() => reconcileZuiDocument(document, snapshot)).toThrow('no mapped .zui property');
    expect(document.nodes!['root'].props!['value']).toBe(42);
  });

  it('rejects multiline edits before exporting a single-line field', async () => {
    const document = await load('search_input');
    const projection = projectZuiDocument(document);
    for (const separator of ['\r', '\n', '\u2028', '\u2029']) {
      const snapshot = cloneProjectionSnapshot(projection);
      snapshot.shapes[0].current.text!.characters = `foo${separator}bar`;
      expect(() => reconcileZuiDocument(document, snapshot)).toThrow('single-line value');
    }
  });
});

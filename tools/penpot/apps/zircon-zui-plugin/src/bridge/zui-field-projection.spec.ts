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
beforeAll(async () => dependencies.load(repo, [theme, ...['field', 'number_field', 'search_input'].map((name) => `${folder}/workbench_${name}.zui`)]));
async function load(name: string) {
  const sourcePath = `${folder}/workbench_${name}.zui`;
  const document = parseZuiDocument(await readFile(resolve(repo, sourcePath), 'utf8')).document;
  dependencies.embed(document, sourcePath, theme);
  return document;
}

describe('native Editor text fields', () => {
  it('reserves the search and clear actions around the actual query', async () => {
    const document = await load('search_input');
    document.nodes!['root'].props!['query'] = 'material';
    const node = projectZuiDocument(document).shapes[0];
    expect(node.text).toMatchObject({ characters: 'material', property: 'query', fontSize: 14 });
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
    const tall = fieldGeometry(node.field!, 360, 64);
    expect(tall.parts['surface']).toMatchObject({ y: 16, height: 32 });
    expect(tall.icons['search'].y).toBe(24);
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

  it('uses the native disabled and validation colors and rejects unmapped font edits', async () => {
    const document = await load('field');
    document.nodes!['root'].state = { disabled: true };
    expect(projectZuiDocument(document).shapes[0].paint.fillColor).toBe('#2f2f2f');
    document.nodes!['root'].state = {};
    document.nodes!['root'].props!['validation_level'] = 'error';
    expect(projectZuiDocument(document).shapes[0].paint.strokeColor).toBe('#eb605c');
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.text!.fontSize = 25;
    expect(() => reconcileZuiDocument(document, snapshot)).toThrow('cannot represent text.fontSize');
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

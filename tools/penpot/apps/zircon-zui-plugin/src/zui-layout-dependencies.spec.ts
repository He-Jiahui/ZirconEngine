import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import { parseZuiDocument } from './bridge/zui-document';
import { styledPreviewNode } from './bridge/zui-style-projection';
import { projectZuiDocument } from './bridge/penpot-projection';

describe('layout catalog dependencies', () => {
  it('projects the actual shared section heading and group-spacing tokens', async () => {
    const root = resolve(
      dirname(fileURLToPath(import.meta.url)),
      '../../../../..',
    );
    const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
    const component =
      'zircon_editor/assets/ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui';
    const dependencies = new LayoutDependencies();
    await dependencies.load(root, [theme, component]);
    const document = parseZuiDocument(
      await readFile(resolve(root, component), 'utf8'),
    ).document;
    dependencies.embed(document, component);
    const heading = projectZuiDocument(document).shapes.find(
      (shape) => shape.nodeId === 'root',
    );
    expect(heading?.text?.fontSize).toBe(16);
    expect(heading?.text?.fontWeight).toBe('600');
    expect(document.tokens?.['editor.density.gap.group']).toBe(24);
  });
  it('rejects missing dependencies without clearing the authored imports', () => {
    const document = parseZuiDocument(
      '[asset]\nkind="style"\nid="res://test"\nversion=2\n[imports]\nstyles=["res://missing.zui"]\n',
    ).document;
    expect(() =>
      new LayoutDependencies().embed(document, 'app/assets/view.zui'),
    ).toThrow('Missing dependency');
    expect(document.imports?.['styles']).toEqual(['res://missing.zui']);
  });

  it('uses the specificity of the matching selector and all authored boolean states', () => {
    const document = parseZuiDocument(
      '[asset]\nkind="view"\nid="res://test"\nversion=2\n[root]\nnode="root"\n[nodes.root]\ncomponent="Button"\nclasses=["action"]\nprops={loading=true,focus_visible=true,selected=false}\nstate={selected=true}\n[[stylesheets]]\nid="test"\n[[stylesheets.rules]]\nselector="#unrelated, Button"\nset={self={font_size=24}}\n[[stylesheets.rules]]\nselector=".action:loading:focus:selected"\nset={self={font_size=13}}\n',
    ).document;
    expect(
      styledPreviewNode(document, document.nodes!['root']).props?.['font_size'],
    ).toBe(13);
  });

  it('rejects unknown and mistyped component parameters', () => {
    const source =
      '[asset]\nkind="view"\nid="res://test"\nversion=2\n[root]\nnode="instance"\n[components.Action]\nroot="template"\nparams={count={type="int",default=1}}\n[nodes.template]\ncomponent="Button"\n[nodes.instance]\ncomponent="Action"\n';
    for (const params of ['params={unknown=1}', 'params={count="one"}']) {
      const document = parseZuiDocument(source + params).document;
      expect(() =>
        new LayoutDependencies().embed(document, 'app/assets/view.zui'),
      ).toThrow(/param/i);
    }
  });
  it('instantiates local params and replaces named slots without an extra sibling', () => {
    const document = parseZuiDocument(`[asset]
kind="view"
id="res://fixture"
version=2
[root]
node="view"
[components.Row]
root="template"
slots={value={required=true,multiple=false,accepts=["Label"]}}
params={title={type="string",default="Default"}}
[nodes.view]
component="VerticalBox"
children=[{node="row"}]
[nodes.row]
component="Row"
params={title="Name"}
children=[{node="value",slot={name="value"}}]
[nodes.value]
component="Label"
props={text="Scene"}
[nodes.template]
component="HorizontalBox"
props={text="$param.title"}
children=[{node="slot"}]
[nodes.slot]
component="Slot"
props={name="value"}
layout={width={stretch="Stretch"}}
`).document;
    new LayoutDependencies().embed(document, 'fixture.zui');
    expect(document.nodes!['row'].component).toBe('HorizontalBox');
    expect(document.nodes!['row'].props?.['text']).toBe('Name');
    expect(document.nodes!['row'].children?.map((child) => child.node)).toEqual(
      ['value'],
    );
    expect(document.nodes!['row__slot']).toBeUndefined();
    expect(document.nodes!['value'].layout?.['width']).toEqual({
      stretch: 'Stretch',
    });
    expect(document.nodes!['template'].props?.['text']).toBe('Default');
  });
  it('resolves declared token names and creates independent reusable component instances', async () => {
    const root = await mkdtemp(join(tmpdir(), 'zui-dependencies-'));
    const paths = [
      'app/assets/tokens.zui',
      'app/assets/button.zui',
      'app/assets/view.zui',
    ];
    const sources = [
      '[asset]\nkind="theme_tokens"\nid="res://tokens.zui"\nversion=2\n[typography]\nbody_size=14\n[names.typography]\nbody_size="editor.typography.body.size"\n',
      '[asset]\nkind="component"\nid="res://button.zui"\nversion=2\n[imports]\nstyles=["res://tokens.zui"]\n[components.Action]\nroot="root"\n[nodes.root]\ncomponent="HorizontalBox"\nchildren=[{node="label"}]\n[nodes.label]\ncomponent="Label"\nprops={text="Shared",font_size="$editor.typography.body.size"}\n',
      '[asset]\nkind="view"\nid="res://view.zui"\nversion=2\n[imports]\nwidgets=["res://button.zui#Action"]\n[root]\nnode="root"\n[nodes.root]\ncomponent="VerticalBox"\nchildren=[{node="first"},{node="second"}]\n[nodes.first]\ncomponent="Action"\nevents=[{event="Click",route="save"}]\n[nodes.second]\ncomponent="Action"\nprops={selected=true}\n',
    ];
    try {
      await mkdir(join(root, 'app/assets'), { recursive: true });
      for (let index = 0; index < paths.length; index++)
        await writeFile(join(root, paths[index]), sources[index]);
      const dependencies = new LayoutDependencies();
      await dependencies.load(root, paths);
      const document = parseZuiDocument(
        sources[2].replace('event="Click"', 'id="Save",event="Click"'),
      ).document;
      dependencies.embed(document, paths[2]);
      expect(document.tokens?.['editor.typography.body.size']).toBe(14);
      expect(document.nodes?.['first__label']?.props?.['text']).toBe('Shared');
      expect(document.nodes?.['second__label']).not.toBe(
        document.nodes?.['first__label'],
      );
      expect(document.nodes?.['first']?.events?.[0]?.['route']).toBe('save');
      expect(document.nodes?.['second']?.props?.['selected']).toBe(true);
      expect(document.nodes?.['first']?.['penpot_prefab_source']).toBe(
        'app/assets/button.zui#Action',
      );
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it('applies matching static styles while keeping authored props and inactive states', () => {
    const document = parseZuiDocument(
      '[asset]\nkind="view"\nid="style"\nversion=2\n[root]\nnode="root"\n[nodes.root]\ncomponent="Label"\nclasses=["caption"]\nprops={text="Asset",font_size=13}\n[[stylesheets]]\nid="shared"\n[[stylesheets.rules]]\nselector=".caption"\nset={self={font_size=11,foreground={color="#aabbcc"}}}\n[[stylesheets.rules]]\nselector=".caption:hover"\nset={self={foreground={color="#ff0000"}}}\n',
    ).document;
    const node = styledPreviewNode(document, document.nodes!['root']);
    expect(node.props?.['font_size']).toBe(11);
    expect(node.props?.['foreground_color']).toBe('#aabbcc');
    expect(document.nodes!['root'].props?.['foreground_color']).toBeUndefined();
  });
});

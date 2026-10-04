import { refineMaterialSamples } from '../tools/zui-material-sample-refinement';
import { parseZuiDocument } from './bridge/zui-document';

describe('Material source specimen reflow', () => {
  it('retains state, event, binding and unknown fields when stacking every specimen', () => {
    const source = `
[asset]
kind="component"
id="res://specimen"
version=2
[components.Specimen]
root="root"
[nodes.root]
component="VerticalBox"
layout={height={preferred=180}}
children=[{node="title"},{node="sample"}]
[nodes.title]
component="WorkbenchLabel"
props={text="Checkboxes"}
[nodes.sample]
component="HorizontalBox"
props={text="Checkboxes", checked=true}
layout={height={preferred=34},container={kind="HorizontalBox",gap=4}}
events=[{id="Toggle",event="Toggle",route="material_lab.checkboxes.toggle"}]
children=[{node="first"},{node="second"}]
[nodes.first]
component="Checkbox"
props={text="Checked",checked=true,corner_radius=10}
layout={width={preferred=86},height={preferred=32}}
bindings=[{property="value",path="selection"}]
unknown_contract={route="retained",enabled=true}
[nodes.second]
component="Checkbox"
props={text="Disabled",disabled=true}
layout={width={preferred=86},height={preferred=32}}
state={checked=true}
`;
    const original = parseZuiDocument(source).document;
    const result = refineMaterialSamples(source);
    const next = parseZuiDocument(result.source).document;
    expect(next.nodes!['sample'].children).toEqual(
      original.nodes!['sample'].children,
    );
    for (const [id, node] of Object.entries(original.nodes!)) {
      expect(next.nodes![id].events).toEqual(node.events);
      expect(next.nodes![id]['bindings']).toEqual(node['bindings']);
      expect(next.nodes![id]['unknown_contract']).toEqual(
        node['unknown_contract'],
      );
      expect(next.nodes![id].state).toEqual(node.state);
    }
    expect(next.nodes!['sample'].layout?.['container']).toMatchObject({
      kind: 'GridBox',
      columns: 1,
      rows: 2,
    });
    expect(next.nodes!['sample'].props?.['text']).toBe('');
    expect(next.nodes!['title'].props?.['text']).toBe('Checkboxes');
    expect(refineMaterialSamples(result.source).changes).toEqual([]);
  });
});

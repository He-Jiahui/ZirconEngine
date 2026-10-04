import { cloneProjectionSnapshot, projectZuiDocument, reconcileZuiDocument } from './bridge/penpot-projection';
import { parseZuiDocument, normalizeZuiDocument } from './bridge/zui-document';
import { restoreVirtualNodes } from './penpot-virtual-nodes';

it('preserves hidden subtree events, child order and text during export', () => {
  const source = parseZuiDocument(`[asset]
kind="view"
id="virtual"
version=2
[root]
node="root"
[nodes.root]
component="VerticalBox"
children=[{node="panel"}]
[nodes.panel]
component="VerticalBox"
props={visibility="collapsed"}
children=[{node="first"},{node="second"}]
[nodes.first]
component="Label"
props={text="Hidden A"}
[nodes.second]
component="Button"
props={text="Hidden B"}
events=[{id="commit",event="Click",route="save"}]
`).document;
  const snapshot = cloneProjectionSnapshot(projectZuiDocument(source));
  const virtual = snapshot.shapes.filter((shape) => shape.parentNodeId === 'panel');
  const rendered = snapshot.shapes.filter((shape) => shape.parentNodeId !== 'panel');
  rendered.find((shape) => shape.nodeId === 'panel')!.childNodeIds = [];
  snapshot.shapes = restoreVirtualNodes(rendered, virtual);
  expect(normalizeZuiDocument(reconcileZuiDocument(source, snapshot).document)).toEqual(normalizeZuiDocument(source));
});

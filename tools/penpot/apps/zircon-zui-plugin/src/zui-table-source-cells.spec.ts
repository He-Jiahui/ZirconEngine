import { explicitTableCells } from '../tools/zui-table-source-cells';
import { parseZuiDocument } from './bridge/zui-document';

const source = `[asset]
kind = "component"
version = 2
id = "res://review.zui"
[components.Review]
root = "row"
[nodes.row]
component = "WorkbenchTableRow"
props = { text = "Renderer.Smoke", value_text = "Running   62%   Worker_03", custom = "preserve" }
events = [{ id = "Run", event = "Click", route = "automation.run" }]
`;

describe('authored table cell migration', () => {
  it('adds explicit cells while retaining source formatting and all other properties', () => {
    const migrated = explicitTableCells(source.replaceAll('\n', '\r\n'));
    expect(migrated.changes).toEqual([{ nodeId: 'row', options: ['Renderer.Smoke', 'Running', '62%', 'Worker_03'] }]);
    expect(migrated.source).toContain('custom = "preserve", options = ');
    expect(migrated.source).not.toMatch(/(?<!\r)\n/);
    const before = parseZuiDocument(source).document;
    const after = parseZuiDocument(migrated.source).document;
    delete after.nodes!['row'].props!['options'];
    expect(after).toEqual(before);
    expect(explicitTableCells(migrated.source).changes).toEqual([]);
  });

  it('refuses to silently discard a fifth column', () => {
    const wider = source.replace('Running   62%   Worker_03', 'Game   128   Valid   Source.ini');
    const result = explicitTableCells(wider);
    expect(result.source).toBe(wider);
    expect(result.changes).toEqual([]);
    expect(result.unresolved[0].reason).toContain('5 columns');
  });
});

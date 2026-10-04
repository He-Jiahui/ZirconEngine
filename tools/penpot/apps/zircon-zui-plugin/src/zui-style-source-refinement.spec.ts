import { describe, expect, it } from 'vitest';

import { parseZuiDocument } from '../src/bridge/zui-document';
import { refineZuiStyleSource } from '../tools/zui-style-source-refinement';

const source = `[asset]
kind = "view"
id = "style-refinement"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalBox"
children = [{ node = "command" }, { node = "glow" }]

[nodes.command]
component = "WorkbenchButton"
props = { text = "Run", font_size = 11, gap = 6, corner_radius = 10, unknown_marker = { retain = "yes" } }
layout = { height = { min = 30, preferred = 34, max = 34, stretch = "Fixed" }, padding = { top = 6, right = 6, bottom = 6, left = 6 }, container = { gap = 10 } }
events = [{ id = "Run", event = "Click", route = "command.run" }]
bindings = { enabled = "model.enabled" }

[nodes.glow]
component = "Space"
props = { background_color = "#8ca0b633", border_width = 0, corner_radius = 72 }
layout = { height = { preferred = 20 } }
`;

describe('zui style source refinement', () => {
  it('normalizes ordinary component values without losing runtime fields', () => {
    const result = refineZuiStyleSource(
      source,
      'zircon_editor/assets/ui/editor/style-refinement.zui',
    );
    expect(result.changed).toBe(true);
    expect(result.changes).toEqual([
      'nodes.command.props.font_size: 11 -> 12',
      'nodes.command.props.gap: 6 -> 8',
      'nodes.command.props.corner_radius: 10 -> 4',
      'nodes.command.layout.height: 30/34/34 -> 32',
      'nodes.command.layout.container.gap: 10 -> 12',
      'nodes.command.layout.padding.top: 6 -> 8',
      'nodes.command.layout.padding.right: 6 -> 8',
      'nodes.command.layout.padding.bottom: 6 -> 8',
      'nodes.command.layout.padding.left: 6 -> 8',
    ]);
    const document = parseZuiDocument(result.source).document;
    const command = document.nodes!['command'];
    expect(command.props).toMatchObject({
      font_size: 12,
      gap: 8,
      corner_radius: 4,
      unknown_marker: { retain: 'yes' },
    });
    expect(command.layout).toMatchObject({
      height: { min: 32, preferred: 32, max: 32, stretch: 'Fixed' },
      padding: { top: 8, right: 8, bottom: 8, left: 8 },
      container: { gap: 12 },
    });
    expect(command.events).toEqual([
      { id: 'Run', event: 'Click', route: 'command.run' },
    ]);
    expect(command['bindings']).toEqual({ enabled: 'model.enabled' });
    expect(document.nodes!['glow'].props!['corner_radius']).toBe(72);
  });
});

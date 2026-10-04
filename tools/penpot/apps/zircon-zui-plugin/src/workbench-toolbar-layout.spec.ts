import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseZuiDocument } from './bridge/zui-document';

const repoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(process.cwd(), '../../../../'));
const sourcePath = resolve(
  repoRoot,
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui',
);

describe('Workbench top toolbar responsive layout', () => {
  const { document } = parseZuiDocument(readFileSync(sourcePath, 'utf8'));
  const nodes = document.nodes!;

  it('keeps each horizontal group at least as wide as its visible controls', () => {
    expect(nodes['toolbar_command_row'].layout?.container).toMatchObject({
      kind: 'ScrollableBox',
      axis: 'Horizontal',
    });

    for (const id of [
      'toolbar_file_group',
      'toolbar_module_commands',
      'toolbar_tool_group',
      'toolbar_run_group',
      'toolbar_layout_group',
    ]) {
      const group = nodes[id];
      expect(group.layout?.container).toMatchObject({
        kind: 'HorizontalBox',
        gap: '$editor.density.gap.small',
      });
      const visibleChildren = (group.children ?? [])
        .map(({ node }) => nodes[node])
        .filter((node) => node.props?.['visibility'] !== 'collapsed');
      const minimum = visibleChildren.reduce(
        (sum, node) => {
          const width = node.layout?.['width'];
          expect(width).toMatchObject({ stretch: 'Fixed' });
          expect(typeof width?.['min']).toBe('number');
          return sum + (width?.['min'] as number);
        },
        4 * Math.max(visibleChildren.length - 1, 0),
      );
      const width = group.layout?.['width'];
      for (const key of ['min', 'preferred', 'max']) {
        expect(
          width?.[key],
          `${id}.${key} must hold ${minimum}px`,
        ).toBeGreaterThanOrEqual(minimum);
      }
    }
  });

  it('keeps global menu, play, and the compact context drawer before scrollable module tools', () => {
    const row = nodes['toolbar_command_row'];
    const order = row.children?.map(({ node }) => node) ?? [];
    expect(order.indexOf('toolbar_file_group')).toBeLessThan(
      order.indexOf('toolbar_run_group'),
    );
    expect(order.indexOf('toolbar_run_group')).toBeLessThan(
      order.indexOf('toolbar_layout_group'),
    );
    expect(order.indexOf('toolbar_layout_group')).toBeLessThan(
      order.indexOf('toolbar_module_commands'),
    );
    expect(nodes['toolbar_menu'].events?.[0]).toMatchObject({
      route: 'workbench.menu.main.open',
    });
    expect(nodes['run_play'].events?.[0]).toMatchObject({
      action: { action: 'runtime.play_mode.enter' },
    });
    expect(nodes['module_details_drawer_toggle'].events?.[0]).toMatchObject({
      route: 'workbench.module.details_drawer.toggle',
    });
  });
});

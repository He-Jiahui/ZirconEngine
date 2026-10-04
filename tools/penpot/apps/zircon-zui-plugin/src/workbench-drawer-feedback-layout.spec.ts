import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseZuiDocument } from './bridge/zui-document';

const repoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(process.cwd(), '../../../../'));
const source = readFileSync(
  resolve(
    repoRoot,
    'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_component_drawer.zui',
  ),
  'utf8',
);

describe('Workbench drawer feedback sample', () => {
  const { document } = parseZuiDocument(source);
  const nodes = document.nodes!;

  function minimumHeight(node: string): number {
    const height = nodes[node].layout?.['height'] as
      | { min?: number; preferred?: number; max?: number; stretch?: string }
      | undefined;
    expect(height?.stretch).toBe('Fixed');
    return height?.min ?? 0;
  }

  function fixedHeight(node: string): number {
    const height = nodes[node].layout?.['height'] as
      | { min?: number; preferred?: number; max?: number; stretch?: string }
      | undefined;
    minimumHeight(node);
    expect(height?.min).toBe(height?.preferred);
    expect(height?.max).toBe(height?.preferred);
    return height?.preferred ?? 0;
  }

  it('allocates all four alerts without spilling into the tooltip', () => {
    expect(
      nodes['feedback_alerts'].children?.map((child) => child.node),
    ).toEqual(['info_alert', 'success_alert', 'warning_alert', 'error_alert']);
    const alertHeight =
      ['info_alert', 'success_alert', 'warning_alert', 'error_alert']
        .map(minimumHeight)
        .reduce((sum, height) => sum + height, 0) +
      3 * 4; // Three editor.density.gap.small intervals.
    expect(fixedHeight('feedback_alerts')).toBeGreaterThanOrEqual(alertHeight);
  });

  it('fits the progress, skeleton, spacer, and toast within their card', () => {
    expect(
      nodes['feedback_toast_column'].children?.map((child) => child.node),
    ).toEqual([
      'feedback_progress',
      'feedback_skeleton',
      'feedback_toast_spacer',
      'feedback_toast',
    ]);
    const columnHeight =
      [
        'feedback_progress',
        'feedback_skeleton',
        'feedback_toast_spacer',
        'feedback_toast',
      ]
        .map(minimumHeight)
        .reduce((sum, height) => sum + height, 0) +
      3 * 8; // Three editor.density.gap.regular intervals.
    expect(fixedHeight('feedback_toast_column')).toBeGreaterThanOrEqual(
      columnHeight,
    );
  });

  it('reserves room for three padded samples and both inter-sample gaps', () => {
    const feedbackHeight =
      fixedHeight('feedback_alerts') +
      fixedHeight('feedback_tooltip') +
      fixedHeight('feedback_toast_column') +
      3 * (4 + 4) + // Each slot receives top/bottom gap.small padding.
      2 * 8; // Two gap.medium intervals.
    expect(fixedHeight('component_feedback')).toBeGreaterThanOrEqual(
      feedbackHeight,
    );
  });

  it('keeps the full clipped table above feedback in the scrollable lower row', () => {
    const tableHeight =
      fixedHeight('table_title') +
      minimumHeight('table_group') +
      4 + // Table title slot top padding.
      8 + // Gap between title and group.
      4; // Table group slot bottom padding.
    expect(fixedHeight('component_table')).toBeGreaterThanOrEqual(tableHeight);
    expect(fixedHeight('component_lower_row')).toBeGreaterThanOrEqual(
      fixedHeight('component_table') + fixedHeight('component_feedback') + 8,
    );
  });
});

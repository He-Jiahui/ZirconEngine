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

describe('Workbench drawer segmented samples', () => {
  const { document } = parseZuiDocument(source);
  const nodes = document.nodes!;

  it('allocates both the authored label and 28px options inside the labeled control', () => {
    const segmented = nodes['input_segmented'];
    expect(segmented.props?.['label_text']).toBe('Segmented Control');
    expect(segmented.props?.['options']).toEqual(['left', 'center', 'right']);
    // 14px label + 4px gap + 5px inset + 20px line + 5px inset = 48px.
    expect(segmented.layout?.['height']).toMatchObject({
      min: 48,
      preferred: 48,
      max: 48,
      stretch: 'Fixed',
    });
  });

  it('keeps the full Columns label inside its option and the padded sample card', () => {
    const segmented = nodes['icon_toggle_segment'];
    const card = nodes['component_icon_buttons'];
    expect(segmented.props?.['options']).toEqual(['grid', 'list', 'columns']);
    const segmentWidth = segmented.layout?.['width'] as
      { min?: number; preferred?: number; max?: number } | undefined;
    // Measured Columns ink exceeds 54px; each option needs 8px on both sides.
    expect(segmentWidth?.min).toBeGreaterThanOrEqual(216);
    expect(segmentWidth?.preferred).toBeGreaterThanOrEqual(216);
    expect(segmentWidth?.max).toBeGreaterThanOrEqual(216);
    const cardWidth = card.layout?.['width'] as
      { min?: number; preferred?: number; max?: number } | undefined;
    for (const key of ['min', 'preferred', 'max'] as const)
      expect(cardWidth?.[key]).toBeGreaterThanOrEqual(
        (segmentWidth?.[key] ?? 0) + 16,
      );
  });
});

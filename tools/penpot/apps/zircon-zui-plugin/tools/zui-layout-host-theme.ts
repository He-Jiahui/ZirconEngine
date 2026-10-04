import type { ZuiDocument, ZuiTable } from '../src/bridge/zui-document';

/** Mirrors the retained host palette projection and its material role resolver. */
export function editorHostColors(theme: ZuiDocument): ZuiTable {
  const palette = theme['palette'] as ZuiTable | undefined;
  if (theme.asset.kind !== 'theme_tokens' || !palette)
    throw new Error('Editor review host requires a theme_tokens palette asset');
  const surfaces = palette['surface'];
  if (!Array.isArray(surfaces) || surfaces.length !== 4)
    throw new Error('Editor review host requires four theme surfaces');
  const colors: ZuiTable = {};
  const add = (value: unknown, names: string[]) => {
    if (typeof value !== 'string')
      throw new Error(`Missing host theme color: ${names[0]}`);
    for (const name of names) colors[name] = value;
  };
  add(palette['accent'], [
    'primary',
    'accent',
    'material.primary',
    'material_color_primary',
  ]);
  add(surfaces[0], [
    'on_primary',
    'material.on_primary',
    'material_color_on_primary',
  ]);
  for (const [value, aliases] of [
    [surfaces[2], ['surface', 'material.surface']],
    [palette['surface_recessed'], ['surface_inset', 'material.surface_inset']],
    [palette['surface_hover'], ['surface_hover', 'material.surface_hover']],
    [surfaces[3], ['surface_pressed', 'material.surface_pressed']],
    [
      palette['surface_selected'],
      ['surface_selected', 'material.surface_selected'],
    ],
    [palette['surface_disabled'], ['disabled', 'material.disabled']],
    [palette['border'], ['border', 'outline', 'material.outline']],
    [palette['focus_ring'], ['focus', 'focus_ring', 'material.focus_ring']],
    [
      palette['text_primary'],
      ['text', 'on_surface', 'material.text', 'material.on_surface'],
    ],
    [palette['text_secondary'], ['text_muted', 'muted', 'material.text_muted']],
    [palette['text_disabled'], ['text_disabled', 'material.text_disabled']],
    [palette['warning'], ['warning', 'material.warning']],
    [palette['error'], ['error', 'danger', 'material.error']],
    [palette['success'], ['success', 'material.success']],
    [palette['info'], ['info', 'material.info']],
  ] as const)
    add(value, [...aliases]);
  return colors;
}

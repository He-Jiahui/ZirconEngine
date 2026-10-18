// Synthetic engineering fixture. This is not a Penpot-authored operator design.
import { createBundle } from './bundle.mjs';
import { stableJson } from './contract.mjs';
import { sha256 } from './compiler.mjs';

export const tokens = { editor: { surface: { base: { $type: 'color', $value: '#151515' } } } };
const fixed = value => ({ mode: 'fixed', value });
const fill = { mode: 'fill' };
export function node(id, component, binding, layout) {
  return { id, name: id, component, binding, layout, opacity: 1, rotation: 0, visible: true,
    radius: 0, fills: [], strokes: [], shadows: [], blur: null, children: [] };
}
export function screenFixture() {
  const root = node('root', 'VerticalGroup', { role: 'project.browser.root' }, {
    kind: 'flex', direction: 'column', gap: 4, align: 'stretch', justify: 'start', wrap: false,
    padding: [4, 4, 4, 4], width: fixed(1280), height: fixed(720),
  });
  const projects = node('projects', 'Slot', { role: 'project.browser.projects', slot: 'recent.projects' }, { kind: 'leaf', width: fill, height: fill });
  const open = node('open', 'Button', { role: 'project.browser.open', command: 'file.project.open', text: 'Open' }, { kind: 'leaf', width: fixed(80), height: fixed(28) });
  open.component_ref = 'zop/Button/md';
  root.children.push(projects, open);
  return { screen: 'project', asset_path: 'shell/project_browser.zui', root };
}

export async function fixtureBundle(screen = screenFixture()) {
  const encode = object => new TextEncoder().encode(stableJson(object) + '\n');
  const result = await createBundle({ 'screens/project.json': encode(screen), 'tokens.dtcg.json': encode(tokens) },
    [{ id: 'synthetic-fixture', revision: 1 }], async bytes => sha256(bytes));
  return result;
}

import { runCurrentSourceRefinement, type SourceRefinementContexts } from './zui-source-refinement-current';
import assert from 'node:assert/strict';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { parseZuiDocument, type ZuiTable } from '../src/bridge/zui-document';
import { inline } from './zui-material-source-refinement';

const dense = '$editor.control.height.dense';
const fixed = (value: string | number) => ({
  min: value,
  preferred: value,
  max: value,
  stretch: 'Fixed',
});
const table = (value: unknown) => value as ZuiTable;
const targets = [
  'components/showcase/showcase_collections_section.zui',
  'components/showcase/showcase_input_section.zui',
  'components/showcase/showcase_selection_section.zui',
  'components/workbench/composites/animation/workbench_transport_controls.zui',
  'components/workbench/composites/animation/workbench_sample_weights.zui',
  'components/workbench/floating/workbench_command_palette.zui',
];

export function refineWorkbenchSource(source: string) {
  const original = parseZuiDocument(source).document;
  const document = structuredClone(original);
  const nodes = document.nodes!;
  const rootId = Object.values(document.components!)[0].root;
  const root = nodes[rootId];
  const changes: string[] = [];
  if (document.components?.['WorkbenchTransportControls']) {
    for (const [id, node] of Object.entries(nodes)) {
      if (node.component !== 'WorkbenchIconButton') continue;
      node.layout!['height'] = { min: dense, max: dense, stretch: 'Stretch' };
      node.props!['layout_min_height'] = dense;
      node.props!['layout_min_width'] = dense;
      changes.push(
        `${id}: override inherited 32px minimum with the 28px dense lane bounds; preserve the 20px icon, 2px padding and click route`,
      );
    }
  } else if (document.components?.['WorkbenchSampleWeights']) {
    for (const [id, node] of Object.entries(nodes)) {
      if (id.endsWith('_row') && node.component === 'HorizontalGroup') {
        node.layout!['height'] = fixed(dense);
        changes.push(`${id}: align weight labels and track on a 28px row`);
      }
      if (node.component === 'WorkbenchProgressBar') {
        node.layout!['height'] = { min: dense, max: dense, stretch: 'Stretch' };
        node.props!['background_color'] = 'transparent';
        changes.push(
          `${id}: constrain the inherited progress control to its row and keep only the track surface`,
        );
      }
    }
  } else if (rootId === 'palette') {
    root.layout!['width'] = {
      ...table(root.layout!['width']),
      min: 0,
      stretch: 'Stretch',
    };
    const result = nodes['first_result'];
    assert.equal(result.props?.['label'], 'Open command');
    result.props!['text'] = result.props!['label'];
    changes.push(
      'palette: retain preferred/max width while allowing the actual host width below the previous 520px minimum',
    );
    changes.push(
      'first_result.text: display the existing Open command label instead of inheriting the generic List item text',
    );
  } else {
    assert.ok(rootId.endsWith('_section_root'));
    assert.ok(
      root.component === 'VerticalGroup' || root.component === 'ScrollableBox',
    );
    root.component = 'ScrollableBox';
    root.layout = {
      ...root.layout,
      clip: true,
      input_policy: 'Receive',
      container: {
        kind: 'ScrollableBox',
        axis: 'Vertical',
        gap: '$editor.density.gap.medium',
        scrollbar_visibility: 'Auto',
      },
      width: { min: 0, stretch: 'Stretch' },
      height: { min: 0, stretch: 'Stretch' },
    };
    changes.push(
      `${rootId}: make the existing full ordered content a real vertical scroll container constrained by its host; no child is removed or hidden`,
    );
    changes.push(
      'All original controls and event routes remain; scroll-before/after and native interaction evidence are still required',
    );
  }
  let section = '';
  const bareKeys = (value: unknown) =>
    inline(value).replace(/"([A-Za-z_][A-Za-z0-9_]*)" = /g, '$1 = ');
  const output = source
    .split(/\r?\n/)
    .map((line) => {
      const header = line.match(/^\[([^\]]+)\]$/);
      if (header) section = header[1];
      const assignment = line.match(/^(component|props|layout)\s*=/);
      if (!assignment || !section.startsWith('nodes.')) return line;
      const id = section.slice(6),
        key = assignment[1];
      if (
        JSON.stringify(original.nodes![id]?.[key]) ===
        JSON.stringify(nodes[id]?.[key])
      )
        return line;
      return `${key} = ${bareKeys(nodes[id][key])}`;
    })
    .join(source.includes('\r\n') ? '\r\n' : '\n');
  assert.deepEqual(parseZuiDocument(output).document, document);
  for (const [id, before] of Object.entries(original.nodes!)) {
    assert.deepEqual(nodes[id].events, before.events);
    assert.deepEqual(nodes[id].children, before.children);
    assert.deepEqual(nodes[id]['bindings'], before['bindings']);
    assert.deepEqual(nodes[id].state, before.state);
    assert.equal(nodes[id].control_id, before.control_id);
  }
  return { output, changes: output === source ? [] : changes };
}

const workbenchSources = targets.map((suffix) => 'zircon_editor/assets/ui/editor/' + suffix);

export function runCurrentWorkbenchSourceRefinement(args: readonly string[] = [], contexts?: SourceRefinementContexts) {
  return runCurrentSourceRefinement({
    id: "workbench-composite", select: entry => workbenchSources.includes(entry.sourcePath),
    requiredSources: workbenchSources,
    transform: (source) => { const result = refineWorkbenchSource(source); return { source: result.output, changes: result.changes }; },
  }, args, contexts);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  void runCurrentWorkbenchSourceRefinement(process.argv.slice(2)).then((report) => {
    console.log(JSON.stringify({ ...report, records: undefined }));
  }).catch((error: unknown) => { console.error(error); process.exitCode = 1; });
}

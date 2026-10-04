import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { prepareComponentReviewHost } from '../../tools/zui-layout-component-hosts';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
} from './penpot-projection';
import {
  cloneZuiDocument,
  normalizeZuiDocument,
  parseZuiDocument,
  type ZuiDocument,
} from './zui-document';
import { reconcileReviewSource } from './zui-review-host';

describe('property editor slot review host', () => {
  const repo = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
  const sourcePath =
    'zircon_editor/assets/ui/editor/components/workbench/composites/inputs/workbench_property_editor_row.zui';
  const fieldPath =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/workbench_number_field.zui';
  const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const dependencies = new LayoutDependencies();
  let component: ZuiDocument;
  beforeAll(async () => {
    await dependencies.load(repo, [sourcePath, fieldPath, theme]);
    component = parseZuiDocument(
      await readFile(`${repo}/${sourcePath}`, 'utf8'),
    ).document;
  });

  it('mounts a declared native number-field instance into the required value slot', async () => {
    const before = normalizeZuiDocument(component);
    const result = (await prepareComponentReviewHost(
      repo,
      sourcePath,
      component,
      dependencies,
      theme,
    ))!;
    const host = parseZuiDocument(result.source).document;
    expect(host.nodes!['review_component'].children).toEqual([
      { node: 'review_value', slot: { name: 'value' } },
    ]);
    expect(host.nodes!['review_value']).toEqual({
      component:
        'res://ui/editor/components/workbench/primitives/inputs/workbench_number_field.zui#WorkbenchNumberField',
    });
    expect(
      Object.values(result.projection.nodes!).some(
        (node) => node.component === 'Slot',
      ),
    ).toBe(false);
    const projection = projectZuiDocument(result.projection);
    expect(projection.rootNodes[0].children.map((node) => node.nodeId)).toEqual(
      ['review_component__name_column', 'review_value'],
    );
    expect(
      projection.shapes.find((node) => node.nodeId === 'review_value')!.text
        ?.characters,
    ).toBe('42');
    expect(result.consumers.map((item) => item.sourcePath)).toEqual([
      sourcePath,
      fieldPath,
    ]);
    expect(normalizeZuiDocument(component)).toEqual(before);
    const source = { ...component, penpot_review_host: result.projection };
    expect(
      reconcileReviewSource(source, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(source);
  });

  it('rejects a changed required-slot contract instead of substituting content', async () => {
    const invalid = cloneZuiDocument(component);
    invalid.components!['WorkbenchPropertyEditorRow']['slots'] = {
      value: {
        required: true,
        multiple: false,
        accepts: ['WorkbenchCheckbox'],
      },
    };
    await expect(
      prepareComponentReviewHost(
        repo,
        sourcePath,
        invalid,
        dependencies,
        theme,
      ),
    ).rejects.toThrow('slot contract');
  });
});

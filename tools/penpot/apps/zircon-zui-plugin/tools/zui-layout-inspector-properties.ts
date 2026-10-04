import { sourceNodeByControl } from './zui-layout-source-control';
import type { ZuiDocument, ZuiNode } from '../src/bridge/zui-document';

const SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui';
type JsonObject = Record<string, unknown>;

function sourceControl(document: ZuiDocument, control: string): ZuiNode {
  return sourceNodeByControl(document, SOURCE, control);
}

/** Instances inherit the mounted virtual-list prototype after shared prefab expansion. */
export function prepareInspectorPropertyRows(
  document: ZuiDocument,
  value: unknown,
): void {
  if (value === null) return;
  const inspector = value as JsonObject;
  const container = sourceControl(document, 'WorkbenchInspectorMeshProperties');
  const prototype = sourceControl(
    document,
    'WorkbenchComponentPropertySlot04Row',
  );
  if (
    !container.children?.some(
      (mount) => document.nodes?.[mount.node] === prototype,
    )
  )
    throw new Error(
      'Inspector virtual-row prototype is not mounted in its authored container',
    );
  const components = inspector['components'] as JsonObject[];
  const rows = components.flatMap((component) =>
    (component['properties'] as JsonObject[]).map((property) => ({
      component,
      property,
    })),
  );
  const seen = new Set<string>();
  container.children = rows.map(({ component, property }) => {
    const identity = String(property['id']);
    if (seen.has(identity))
      throw new Error(`Inspector field identity is duplicated: ${identity}`);
    seen.add(identity);
    // This is the same field-key hash used by the native component-property list.
    let key = 0x6c62272e07bb014262b821756295c58dn;
    for (const byte of Buffer.from(identity, 'utf8'))
      key = BigInt.asUintN(
        128,
        (key ^ BigInt(byte)) * 0x0000000001000000000000000000013bn,
      );
    const id = `workbench_inspector_property_${key.toString(16)}`;
    if (document.nodes?.[id])
      throw new Error(`Inspector generated node ${id} already exists`);
    const row = structuredClone(prototype);
    // Generated rows have no authored callsite identity. The original prototype remains intact.
    delete row['penpot_review_source_path'];
    delete row['penpot_review_source_node_id'];
    delete row['penpot_review_instance_path'];
    row.control_id = `WorkbenchComponentPropertyVirtualRow${key.toString(16)}`;
    row['penpot_review_generated_product_row'] = {
      kind: 'inspectorProperty',
      componentId: component['id'],
      componentTitle: component['title'],
      componentCount: components.length,
      property: structuredClone(property),
      itemKey: key.toString(),
    };
    document.nodes![id] = row;
    return { node: id };
  });
}

/** Values bind to the real property-row prefab expanded by LayoutDependencies. */
export function applyInspectorPropertyRows(
  document: ZuiDocument,
  inspector: JsonObject,
): void {
  const components = inspector['components'] as JsonObject[];
  const count = components.reduce(
    (sum, component) => sum + (component['properties'] as unknown[]).length,
    0,
  );
  for (const control of [
    'WorkbenchInspectorMesh',
    'WorkbenchInspectorMeshProperties',
  ]) {
    const node = sourceControl(document, control);
    node.props = {
      ...(node.props ?? {}),
      visibility: count ? 'visible' : 'collapsed',
    };
  }
  const title = sourceControl(document, 'WorkbenchMeshLabel');
  title.props = {
    ...(title.props ?? {}),
    text:
      components.length === 1
        ? String(components[0]!['title'])
        : components.length
          ? 'Components'
          : '',
  };
  for (const row of Object.values(document.nodes ?? {})) {
    const generated = row['penpot_review_generated_product_row'] as
      JsonObject | undefined;
    if (generated?.['kind'] !== 'inspectorProperty') continue;
    const property = generated['property'] as JsonObject;
    const editable = property['editable'] === true;
    const label =
      generated['componentCount'] === 1
        ? String(property['label'])
        : `${String(generated['componentTitle'])} / ${String(property['label'])}`;
    const value =
      typeof property['value'] === 'string'
        ? property['value']
        : JSON.stringify(property['value']);
    row.props = {
      ...(row.props ?? {}),
      text: label,
      value,
      value_text: value.trim() ? value : '-',
      visibility: 'visible',
      inspector_property_field_id: String(property['id']),
      inspector_property_label: label,
      inspector_property_value_kind: String(property['kind']),
      inspector_property_editable: editable,
      read_only: !editable,
      editable_text: editable,
      input_focusable: editable,
      input_clickable: editable,
    };
    row['penpot_review_inspector_property'] = structuredClone(property);
  }
}

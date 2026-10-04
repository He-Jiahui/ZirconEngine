import type { ZuiDocument, ZuiNode, ZuiTable } from './zui-document';

type AnyRecord = Record<string, unknown>;

export function migrateLegacyDocument(
  raw: AnyRecord,
  sourcePath: string,
): ZuiDocument {
  const rawAsset = asRecord(raw['asset']) ?? {};
  const originalKind = stringValue(rawAsset['kind']) ?? 'unknown';
  const originalVersion = finiteNumber(rawAsset['version']) ?? 1;

  if (originalKind === 'style' || originalKind === 'theme_tokens') {
    return {
      ...raw,
      asset: {
        ...rawAsset,
        kind: originalKind === 'theme_tokens' ? 'theme_tokens' : 'style',
        version: 2,
        display_name: stringValue(rawAsset['display_name']) || sourcePath,
      },
      legacy_migration: {
        source_path: sourcePath,
        source_kind: originalKind,
        source_version: originalVersion,
      },
    } as unknown as ZuiDocument;
  }

  const nodes: Record<string, ZuiNode> = {};
  const parentIds = new Set<string>();
  const roots: string[] = [];

  const addNode = (value: unknown, fallbackId: string): string => {
    const sourceNode = asRecord(value) ?? {};
    const requestedId = stringValue(sourceNode['node_id']) || fallbackId;
    const nodeId = uniqueNodeId(requestedId, nodes);
    const node: AnyRecord = {
      component: componentName(sourceNode),
      ...(stringValue(sourceNode['control_id'])
        ? { control_id: sourceNode['control_id'] }
        : {}),
      ...(Array.isArray(sourceNode['classes'])
        ? {
            classes: sourceNode['classes'].filter(
              (item): item is string => typeof item === 'string',
            ),
          }
        : {}),
      ...(asRecord(sourceNode['params'])
        ? { params: sourceNode['params'] }
        : {}),
      ...(asRecord(sourceNode['props']) ? { props: sourceNode['props'] } : {}),
      ...(asRecord(sourceNode['layout'])
        ? { layout: sourceNode['layout'] }
        : {}),
      ...(asRecord(sourceNode['style_overrides'])
        ? { style: sourceNode['style_overrides'] }
        : {}),
      ...(stringValue(sourceNode['kind'])
        ? { legacy_kind: sourceNode['kind'] }
        : {}),
      ...(stringValue(sourceNode['type'])
        ? { legacy_type: sourceNode['type'] }
        : {}),
      ...(stringValue(sourceNode['component_ref'])
        ? { legacy_component_ref: sourceNode['component_ref'] }
        : {}),
      ...(stringValue(sourceNode['slot_name'])
        ? { legacy_slot_name: sourceNode['slot_name'] }
        : {}),
    };
    if (sourceNode['kind'] === 'slot')
      node['props'] = {
        ...asRecord(node['props']),
        name: sourceNode['slot_name'] ?? 'default',
      };
    const legacyBindings = sourceNode['bindings'];
    if (Array.isArray(legacyBindings) && legacyBindings.length > 0) {
      node['legacy_bindings'] = legacyBindings;
    }
    nodes[nodeId] = node as ZuiNode;

    const children = sourceNode['children'];
    if (Array.isArray(children)) {
      const mounts: Array<{ node: string; slot?: ZuiTable }> = [];
      for (const [index, childValue] of children.entries()) {
        const mount = asRecord(childValue) ?? {};
        // Older layout fixtures used `child = "node_id"` while the v2
        // contract uses `node = "node_id"`.  Treat both as references during
        // migration; dropping the legacy spelling leaves every visual child
        // detached from its root and produces a deceptively flat Penpot
        // screenshot even though all nodes were parsed successfully.
        const nested = mount['node'] ?? mount['child'];
        const childId = asRecord(nested)
          ? addNode(nested, `${nodeId}__${index}`)
          : stringValue(nested);
        // Keep forward references for the second pass below. Legacy tables
        // are not required to declare a parent after all of its children, so
        // the referenced node may not have been visited yet.
        if (!childId) continue;
        const slot = asRecord(mount['slot']);
        mounts.push({
          node: childId,
          ...(slot ? { slot: slot as unknown as ZuiTable } : {}),
        });
        const mountName = stringValue(mount['mount']);
        if (mountName) {
          mounts[mounts.length - 1].slot = {
            ...slot,
            name: mountName,
          } as ZuiTable;
          const childNode = asRecord(nodes[childId]);
          if (childNode) childNode['legacy_mount'] = mountName;
        }
      }
      if (mounts.length > 0) node['children'] = mounts;
    }
    return nodeId;
  };

  const rawNodes = asRecord(raw['nodes']);
  if (rawNodes) {
    for (const [nodeId, value] of Object.entries(rawNodes)) {
      addNode(value, nodeId);
    }
  }

  const rawRoot = asRecord(raw['root']);
  if (rawRoot && !(originalKind === 'widget' && asRecord(raw['components']))) {
    // Legacy layout/widget files encode the root as a reference (`node =
    // "root_id"`), while some older fixtures inline a node table. Resolve the
    // reference first; treating it as an inline node creates an empty
    // synthetic Container and makes the Penpot preview blank.
    const referencedRoot = stringValue(rawRoot['node']);
    const rootId =
      referencedRoot && nodes[referencedRoot]
        ? referencedRoot
        : addNode(rawRoot, 'root');
    roots.push(rootId);
  }

  const rawComponents = asRecord(raw['components']);
  const components: AnyRecord = {};
  if (rawComponents) {
    for (const [componentId, value] of Object.entries(rawComponents)) {
      const component = asRecord(value);
      if (!component) continue;
      const rootValue = component['root'];
      const rootId = asRecord(rootValue)
        ? addNode(rootValue, `${componentId}__root`)
        : stringValue(rootValue);
      if (rootId && nodes[rootId]) {
        roots.push(rootId);
        components[componentId] = { ...component, root: rootId };
      }
    }
  }

  // Resolve legacy forward references after the complete node table has been
  // collected. This both preserves valid `child`/`node` mounts and drops
  // malformed references with the same strictness as the canonical parser.
  parentIds.clear();
  for (const node of Object.values(nodes)) {
    const children = node.children;
    if (!Array.isArray(children)) continue;
    const validChildren = children.filter((child) => {
      const childId = stringValue(child.node);
      if (!childId || !nodes[childId]) return false;
      parentIds.add(childId);
      return true;
    });
    if (validChildren.length > 0) node.children = validChildren;
    else delete node.children;
  }

  const uniqueRoots = uniqueStrings(roots);
  const nodeEntries = Object.keys(nodes);
  const rootless = nodeEntries.filter((id) => !parentIds.has(id));
  const visualRoots = uniqueRoots.length > 0 ? uniqueRoots : rootless;
  const hasVisualNodes = Object.keys(nodes).length > 0;
  let root: { node: string } | undefined;
  if (hasVisualNodes) {
    const selectedRoots = uniqueStrings(visualRoots);
    if (selectedRoots.length === 1) {
      root = { node: selectedRoots[0] };
    } else {
      const syntheticId = uniqueNodeId('__penpot_root', nodes);
      nodes[syntheticId] = {
        // Legacy widget files may expose many component roots. The migration
        // is design-preview-only, so give that synthetic owner the same
        // responsive scroll semantics as the current editor hosts instead of
        // forcing a 1200x800 canvas that cannot fit the review viewports.
        component: 'ScrollableBox',
        control_id: 'PenpotCatalogRoot',
        props: { background_color: '#171a20' },
        layout: {
          clip: true,
          container: {
            kind: 'ScrollableBox',
            axis: 'Vertical',
            gap: 16,
            scrollbar_visibility: 'Auto',
          },
          padding: { top: 24, right: 24, bottom: 24, left: 24 },
          width: { min: 0, stretch: 'Stretch' },
          height: { min: 0, stretch: 'Stretch' },
        },
        children: selectedRoots.map((node) => ({ node })),
      };
      root = { node: syntheticId };
    }
  }

  const migrated: AnyRecord = {
    ...(asRecord(raw['imports']) ? { imports: raw['imports'] } : {}),
    ...(asRecord(raw['tokens']) ? { tokens: raw['tokens'] } : {}),
    ...(Array.isArray(raw['stylesheets'])
      ? { stylesheets: raw['stylesheets'] }
      : {}),
    asset: {
      ...rawAsset,
      kind: 'view',
      version: 2,
      display_name: stringValue(rawAsset['display_name']) || sourcePath,
    },
    ...(root ? { root } : {}),
    ...(hasVisualNodes ? { nodes } : {}),
    ...(Object.keys(components).length ? { components } : {}),
    legacy_migration: {
      source_path: sourcePath,
      source_kind: originalKind,
      source_version: originalVersion,
      source_component_ids: rawComponents ? Object.keys(rawComponents) : [],
    },
  };
  return migrated as unknown as ZuiDocument;
}

function componentName(node: AnyRecord): string {
  const type = stringValue(node['type']);
  if (type) return type;
  const reference = stringValue(node['component_ref']);
  if (reference) {
    const tail = reference.split('#').pop() ?? reference;
    return tail.split('/').pop() || 'Container';
  }
  const kind = stringValue(node['kind']);
  if (kind === 'slot') return 'Slot';
  return kind || 'Container';
}

function asRecord(value: unknown): AnyRecord | undefined {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as AnyRecord)
    : undefined;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' ? value : undefined;
}

function finiteNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function uniqueNodeId(
  requested: string,
  nodes: Record<string, ZuiNode>,
): string {
  const base = requested.trim() || 'node';
  if (!nodes[base]) return base;
  let index = 2;
  while (nodes[`${base}_${index}`]) index += 1;
  return `${base}_${index}`;
}

function uniqueStrings(values: string[]): string[] {
  return [...new Set(values.filter((value) => value.trim() !== ''))];
}

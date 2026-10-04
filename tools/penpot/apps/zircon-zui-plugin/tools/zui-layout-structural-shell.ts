export interface StructuralShellNode {
  nodeId: string;
  parentNodeId: string | null;
  component: string;
  visible: boolean;
  detached: boolean;
  text?: string;
  textParts?: Array<{ text: string }>;
}

export interface StructuralShellAudit {
  semanticNodes: StructuralShellNode[];
}

const STRUCTURAL_COMPONENTS = new Set([
  'Container',
  'DockHost',
  'GridBox',
  'HorizontalGroup',
  'Overlay',
  'Panel',
  'ScrollBox',
  'ScrollableBox',
  'Slot',
  'Space',
  'Splitter',
  'VerticalGroup',
]);

function isWorkbenchMountComponent(node: StructuralShellNode): boolean {
  const component = node.component;
  return (
    component.startsWith('Workbench') &&
    /(Workspaces?|Host|Body|Drawer|Panel|Module|Shell)$/.test(component)
  );
}

/**
 * A dynamic product host can intentionally describe only placement slots or
 * retire all of its former sample content. It needs a real product host, not
 * a substitute preview page. Keep that evidence pending after it is captured.
 */
export function structuralShellPendingReason(
  sourceClassification: string,
  layout: StructuralShellAudit,
): string | undefined {
  if (sourceClassification !== 'dynamic-host') return undefined;
  const visible = layout.semanticNodes.filter(
    (node) => node.visible && !node.detached,
  );
  if (!visible.length) return undefined;
  if (
    visible.some(
      (node) =>
        node.text?.trim() || node.textParts?.some((part) => part.text.trim()),
    )
  )
    return undefined;
  if (
    visible.some(
      (node) =>
        !STRUCTURAL_COMPONENTS.has(node.component) &&
        !isWorkbenchMountComponent(node),
    )
  )
    return undefined;

  const visibleNodeIds = new Set(visible.map((node) => node.nodeId));
  const leaves = visible.filter(
    (node) =>
      !layout.semanticNodes.some(
        (candidate) =>
          candidate.visible &&
          !candidate.detached &&
          candidate.parentNodeId === node.nodeId &&
          visibleNodeIds.has(candidate.nodeId),
      ),
  );
  const hasExposedSlot = leaves.some((node) => node.component === 'Slot');
  const hasHostMount =
    leaves.some((node) =>
      ['Container', 'Slot', 'Space'].includes(node.component) ||
      isWorkbenchMountComponent(node),
    ) ||
    (leaves.length === 1 &&
      ['HorizontalGroup', 'VerticalGroup'].includes(leaves[0]!.component));
  const hasRetiredContent = layout.semanticNodes.some(
    (node) => !node.visible,
  );
  if (!hasExposedSlot && !hasRetiredContent && !hasHostMount) return undefined;
  return hasExposedSlot
    ? 'Authored dynamic host exposes only structural Slot content; waiting for a fingerprinted product host with deterministic data.'
    : hasRetiredContent
      ? 'Authored dynamic host exposes only structural containers while its sample content is retired; waiting for a fingerprinted product host with deterministic data.'
      : 'Authored dynamic host exposes only structural host mounts; waiting for a fingerprinted product host with deterministic data.';
}

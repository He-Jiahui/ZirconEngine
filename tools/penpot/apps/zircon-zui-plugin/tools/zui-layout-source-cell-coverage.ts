import type { ZuiDocument } from '../src/bridge/zui-document';
import { isWorkbenchTable, tableCells } from '../src/bridge/zui-table-cells';
import type { TextStructureNode } from './zui-layout-text-structure';

/**
 * Compare authored, static Workbench row cells with the fragments a renderer
 * actually materialized. Comparing two rendered trees alone misses columns
 * when both renderers silently allocate them zero width.
 */
export function sourceCellCoverageErrors(
  source: ZuiDocument,
  geometry: unknown,
): string[] {
  if (source.asset.kind !== 'component') return [];
  const semanticNodes = (
    geometry as { layout?: { semanticNodes?: TextStructureNode[] } } | null
  )?.layout?.semanticNodes;
  if (!Array.isArray(semanticNodes)) return ['Missing semantic table geometry'];
  const errors: string[] = [];
  const componentRoots = new Set(
    Object.values(source.components ?? {}).map((component) => component.root),
  );
  for (const [nodeId, node] of Object.entries(source.nodes ?? {})) {
    const options = node.props?.['options'];
    if (!isWorkbenchTable(node) || !Array.isArray(options) || !options.length)
      continue;
    if (options.some((value) => typeof value !== 'string')) {
      errors.push(`${nodeId}: unsupported static table cell options`);
      continue;
    }
    const cells = tableCells({ ...node.props, ...node.state });
    if (cells.length !== options.length) {
      errors.push(`${nodeId}: source table cell mapping is incomplete`);
      continue;
    }
    const rendered = semanticNodes.find(
      (item) => item.nodeId === nodeId && item.visible && !item.detached,
    );
    if (!rendered) {
      if (componentRoots.has(nodeId))
        errors.push(`${nodeId}: missing rendered table node`);
      continue;
    }
    if (!Array.isArray(rendered.textParts)) {
      errors.push(`${nodeId}: missing rendered cell fragments`);
      continue;
    }
    const materialized = rendered.textParts.map((part) => part.text.trim());
    const missing = cells.flatMap((cell) => {
      const index = materialized.indexOf(cell.value);
      if (index < 0) return [cell.value];
      materialized.splice(index, 1);
      return [];
    });
    if (missing.length)
      errors.push(`${nodeId}: missing rendered cells: ${missing.join(', ')}`);
  }
  return errors;
}

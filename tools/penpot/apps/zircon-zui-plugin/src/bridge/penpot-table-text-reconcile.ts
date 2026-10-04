import { ZuiDocumentError, type ZuiNode } from './zui-document';
import type { ProjectionEditableState } from './penpot-projection-model';
import { normalizeTableCell, tableCells } from './zui-table-cells';

export function applyTableTextChanges(
  node: ZuiNode,
  nodeId: string,
  baseline: ProjectionEditableState,
  current: ProjectionEditableState,
  changes: string[],
): void {
  if (baseline.text || current.text)
    throw new ZuiDocumentError(
      `Table ${nodeId} has unexpected primary text metadata.`,
    );
  const cells = tableCells({ ...node.props, ...node.state });
  for (const [key, before] of Object.entries(baseline.textFragments ?? {})) {
    const after = current.textFragments?.[key];
    const cell = cells.find((cell) => cell.key === key);
    if (!after || !cell || before.property !== after.property)
      throw new ZuiDocumentError(`Table ${nodeId} has unmapped cell ${key}.`);
    for (const property of [
      'color',
      'colorOpacity',
      'fontFamily',
      'fontSize',
      'fontWeight',
      'lineHeight',
      'align',
    ] as const)
      if (before[property] !== after[property])
        throw new ZuiDocumentError(
          `Table ${nodeId} uses retained-host ${property}; independent cell style edits are unsupported.`,
        );
    if (before.characters === after.characters) continue;
    if (cell.sourceIndex === null)
      throw new ZuiDocumentError(
        `Table ${nodeId} must declare options before editing an archived text cell.`,
      );
    if (
      !after.characters ||
      normalizeTableCell(cell.column, after.characters) !== after.characters
    )
      throw new ZuiDocumentError(
        `Table ${nodeId} requires normalized, nonempty native cell text.`,
      );
    const target =
      node.state?.['options'] !== undefined ? node.state : (node.props ??= {});
    const values = target!['options'];
    if (!Array.isArray(values))
      throw new ZuiDocumentError(`Missing table options on ${nodeId}.`);
    values[cell.sourceIndex] = after.characters;
    changes.push(
      `nodes.${nodeId}.${target === node.state ? 'state' : 'props'}.options.${cell.sourceIndex}`,
    );
  }
  const projected = tableCells({ ...node.props, ...node.state });
  if (
    projected.length !== cells.length ||
    projected.some(
      (cell, index) =>
        cell.key !== cells[index].key ||
        cell.value !== current.textFragments?.[cell.key]?.characters,
    )
  )
    throw new ZuiDocumentError(
      `Table edit on ${nodeId} would change the native cell parsing contract.`,
    );
}

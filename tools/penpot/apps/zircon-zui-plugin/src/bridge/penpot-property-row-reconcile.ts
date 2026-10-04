import { ZuiDocumentError, type ZuiNode } from './zui-document';
import type { ProjectionEditableState } from './penpot-projection-model';
import {
  parsePropertyAxes,
  propertyRowValues,
} from './zui-property-row-values';

export function applyPropertyRowTextChanges(
  node: ZuiNode,
  nodeId: string,
  baseline: ProjectionEditableState,
  current: ProjectionEditableState,
  changes: string[],
): void {
  if (baseline.text || current.text)
    throw new ZuiDocumentError(
      `Property row ${nodeId} has unexpected primary text.`,
    );
  const values = propertyRowValues(node);
  const axes = values.axes.length >= 2 ? values.axes.slice(0, 4) : [];
  const replacements: Array<{ start: number; end: number; value: string }> = [];
  const write = (property: string, value: string) => {
    const target = Object.hasOwn(node.state ?? {}, property)
      ? node.state!
      : (node.props ??= {});
    target[property] = value;
    changes.push(
      `nodes.${nodeId}.${target === node.state ? 'state' : 'props'}.${property}`,
    );
  };
  for (const [key, before] of Object.entries(baseline.textFragments ?? {})) {
    const after = current.textFragments?.[key];
    if (!after || before.property !== after.property)
      throw new ZuiDocumentError(
        `Property row ${nodeId} has unmapped text ${key}.`,
      );
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
          `Property row ${nodeId} uses retained-host ${property}; edit its source theme or row style.`,
        );
    if (before.characters === after.characters) continue;
    if (!after.characters || after.characters.trim() !== after.characters)
      throw new ZuiDocumentError(
        `Property row ${nodeId} requires nonempty, trimmed text.`,
      );
    if (
      key === 'property-label' &&
      before.property === values.label.property &&
      before.property
    ) {
      write(before.property, after.characters);
    } else if (
      key === 'property-value' &&
      before.property === values.value.property &&
      before.property
    ) {
      if (parsePropertyAxes(after.characters).length >= 2)
        throw new ZuiDocumentError(
          `Property row ${nodeId} scalar edit would create axis groups; edit the source and re-import.`,
        );
      write(before.property, after.characters);
    } else {
      const axis = axes.find((item) => item.key === key);
      if (
        !axis ||
        !before.property ||
        before.property !== values.value.property
      )
        throw new ZuiDocumentError(
          `Property row ${nodeId} axis label or unmapped fragment ${key} cannot be edited.`,
        );
      if (
        after.characters.split(/\s+/).join(' ') !== after.characters ||
        after.characters.split(' ').some((token) => /^[XYZW]$/.test(token))
      )
        throw new ZuiDocumentError(
          `Property row ${nodeId} axis value must preserve native axis grouping.`,
        );
      replacements.push({
        start: axis.start,
        end: axis.end,
        value: after.characters,
      });
    }
  }
  if (replacements.length && values.value.property) {
    let value = values.value.raw;
    for (const replacement of replacements.sort((a, b) => b.start - a.start))
      value =
        value.slice(0, replacement.start) +
        replacement.value +
        value.slice(replacement.end);
    write(values.value.property, value);
  }
}

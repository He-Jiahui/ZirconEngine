import { nativePainterComponent } from './zui-native-painter-role';
import { treeRowTextColorProperty } from './zui-tree-row-visual';
import { ZuiDocumentError, type ZuiNode, type ZuiTable } from './zui-document';
import type {
  ProjectionEditableState,
  ProjectionText,
} from './penpot-projection-model';
import {
  writeAuthoredLineHeight,
  writeAuthoredPaintProperty,
} from './zui-paint-properties';
import { isSlider } from './zui-slider-projection';
import { isSegmentedControl } from './zui-segmented-projection';
import { applySegmentedTextChanges } from './penpot-segmented-text-reconcile';
import { isWorkbenchTable } from './zui-table-cells';
import { applyTableTextChanges } from './penpot-table-text-reconcile';
import { applyPropertyRowTextChanges } from './penpot-property-row-reconcile';
import { isPropertyRow } from './zui-property-row-projection';

export function applyNodeTextChanges(
  node: ZuiNode,
  nodeId: string,
  baseline: ProjectionEditableState,
  current: ProjectionEditableState,
  changes: string[],
  selfStyle: ZuiTable,
): void {
  const previous = baseline.textFragments ?? {};
  const next = current.textFragments ?? {};
  const keys = Object.keys(previous);
  if (
    keys.length !== Object.keys(next).length ||
    keys.some((key) => !Object.hasOwn(next, key))
  )
    throw new ZuiDocumentError(
      `Editable text fragments for node ${nodeId} were added, removed or retargeted.`,
    );
  const pairs = [
    { key: 'primary', before: baseline.text, after: current.text },
    ...keys.map((key) => ({ key, before: previous[key], after: next[key] })),
  ];
  if (
    isPropertyRow(node) &&
    keys.some((key) => key.startsWith('property-') || key.startsWith('axis-'))
  ) {
    applyPropertyRowTextChanges(node, nodeId, baseline, current, changes);
    return;
  }
  if (isWorkbenchTable(node)) {
    applyTableTextChanges(node, nodeId, baseline, current, changes);
    return;
  }
  if (isSegmentedControl(node)) {
    applySegmentedTextChanges(
      node,
      nodeId,
      baseline,
      current,
      changes,
      selfStyle,
    );
    return;
  }
  const slider = isSlider(node);
  if (slider) {
    for (const { before, after } of pairs) {
      if (!before || !after) continue;
      for (const key of ['fontFamily', 'fontWeight', 'align'] as const)
        if (before[key] !== after[key])
          throw new ZuiDocumentError(
            `Slider ${nodeId} does not map independent ${key} edits.`,
          );
      if (
        after.characters !== before.characters &&
        (!after.characters.trim() ||
          after.characters !== after.characters.trim())
      )
        throw new ZuiDocumentError(
          `Slider ${nodeId} text must be nonempty and trimmed.`,
        );
    }
    for (const key of ['fontSize', 'lineHeight'] as const) {
      if (
        pairs.some(({ before, after }) => before?.[key] !== after?.[key]) &&
        new Set(
          pairs.filter(({ after }) => after).map(({ after }) => after![key]),
        ).size > 1
      )
        throw new ZuiDocumentError(
          `Slider ${nodeId} shares ${key} across its text fragments.`,
        );
    }
  }
  const props = { ...node.props, ...node.state };
  const disabled =
    props['disabled'] === true ||
    props['loading'] === true ||
    props['enabled'] === false;
  if (slider) {
    const shared = pairs.filter(({ key }) => disabled || key !== 'label');
    for (const property of ['color', 'colorOpacity'] as const)
      if (
        shared.some(
          ({ before, after }) => before?.[property] !== after?.[property],
        ) &&
        new Set(shared.map(({ after }) => after?.[property])).size > 1
      )
        throw new ZuiDocumentError(
          `Slider ${nodeId} shares value ${property}; independent fragment edits cannot be mapped.`,
        );
  }
  for (const { key, before, after } of pairs)
    applyTextChanges(
      node,
      nodeId,
      before,
      after,
      changes,
      selfStyle,
      nativePainterComponent(node) === 'TreeRow'
        ? treeRowTextColorProperty(node)
        : slider && disabled
          ? 'disabled_foreground_color'
          : slider && key === 'label'
            ? 'label_color'
            : 'foreground_color',
    );
}

function applyTextChanges(
  node: ZuiNode,
  nodeId: string,
  baseline: ProjectionText | null,
  current: ProjectionText | null,
  changes: string[],
  selfStyle: ZuiTable,
  colorProperty:
    | 'foreground_color'
    | 'selected_foreground_color'
    | 'label_color'
    | 'disabled_foreground_color',
): void {
  if ((baseline === null) !== (current === null))
    throw new ZuiDocumentError(
      `Editable text metadata for node ${nodeId} was added or removed in Penpot.`,
    );
  if (!baseline || !current) return;
  if (current.property !== baseline.property)
    throw new ZuiDocumentError(
      `Text property metadata for node ${nodeId} changed from ${String(baseline.property)} to ${String(current.property)}.`,
    );
  const charactersChanged = current.characters !== baseline.characters;
  if (charactersChanged && baseline.property === null)
    throw new ZuiDocumentError(
      `Editable text for node ${nodeId} is a display-only Penpot label with no mapped .zui property.`,
    );
  if (charactersChanged && baseline.property) {
    node.props ??= {};
    node.props[baseline.property] = current.characters;
    changes.push(`nodes.${nodeId}.props.${baseline.property}`);
  }
  if (
    current.color !== baseline.color ||
    !nearlyEqual(current.colorOpacity, baseline.colorOpacity)
  )
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        colorProperty,
        serializedColor(current.color, current.colorOpacity),
        selfStyle,
      ),
    );
  const fontSizeChanged =
    current.fontSize !== baseline.fontSize && current.fontSize !== null;
  if (fontSizeChanged)
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'font_size',
        round(current.fontSize!),
        selfStyle,
      ),
    );
  if (
    current.fontWeight !== baseline.fontWeight &&
    current.fontWeight !== null
  ) {
    const weight =
      current.fontWeight === 'regular' ? 400 : Number(current.fontWeight);
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'font_weight',
        Number.isFinite(weight) ? weight : current.fontWeight,
        selfStyle,
      ),
    );
  }
  if (current.align !== baseline.align && current.align !== null)
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'text_align',
        current.align,
        selfStyle,
      ),
    );
  if (current.fontFamily !== baseline.fontFamily)
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'font_family',
        current.fontFamily,
        selfStyle,
      ),
    );
  if (!nearlyEqual(current.lineHeight, baseline.lineHeight) || fontSizeChanged)
    changes.push(
      writeAuthoredLineHeight(
        node,
        nodeId,
        current.lineHeight,
        current.fontSize ?? 14,
        selfStyle,
      ),
    );
}

export function serializedColor(color: string | null, opacity: number): string {
  if (!color) return 'transparent';
  if (opacity >= 1) return color;
  return `${color}${Math.round(Math.min(1, Math.max(0, opacity)) * 255)
    .toString(16)
    .padStart(2, '0')}`;
}
function nearlyEqual(left: number, right: number): boolean {
  return Math.abs(left - right) < 0.01;
}
function round(value: number): number {
  return Math.round(value * 1000) / 1000;
}

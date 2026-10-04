import { ZuiDocumentError, type ZuiNode, type ZuiTable } from './zui-document';
import type {
  ProjectionEditableState,
  ProjectionText,
} from './penpot-projection-model';
import {
  selectedSegmentProperty,
  segmentedOptions,
  segmentLabel,
} from './zui-segmented-projection';
import {
  writeAuthoredPaintProperty,
  writeAuthoredLineHeight,
  type AuthoredPaintProperty,
} from './zui-paint-properties';

export function applySegmentedTextChanges(
  node: ZuiNode,
  nodeId: string,
  baseline: ProjectionEditableState,
  current: ProjectionEditableState,
  changes: string[],
  selfStyle: ZuiTable,
): void {
  const tab = ['Tab', 'PanelTab'].includes(node.component);
  const props = { ...node.props, ...node.state };
  const options = segmentedOptions(props);
  const disabled =
    props['disabled'] === true ||
    props['loading'] === true ||
    props['enabled'] === false;
  const pairs = [
    { key: 'primary', before: baseline.text, after: current.text },
    ...Object.entries(baseline.textFragments ?? {}).map(([key, before]) => ({
      key,
      before,
      after: current.textFragments?.[key] ?? null,
    })),
  ];
  const colors = new Map<
    string,
    Array<{ before: ProjectionText; after: ProjectionText }>
  >();
  const optionTexts: Array<{ before: ProjectionText; after: ProjectionText }> =
    [];
  for (const { key, before, after } of pairs) {
    if (!before && !after) continue;
    if (!before || !after || before.property !== after.property)
      throw new ZuiDocumentError(
        `Segmented control ${nodeId} changed text property metadata.`,
      );
    const option = options.find((option) => option.key === key);
    const isOption = before.property === 'options';
    if (isOption && (!option || tab))
      throw new ZuiDocumentError(
        `Segmented control ${nodeId} has an unmapped option ${key}.`,
      );
    if (isOption) optionTexts.push({ before, after });
    for (const property of ['fontFamily', 'fontWeight', 'align'] as const)
      if (before[property] !== after[property])
        throw new ZuiDocumentError(
          `Segmented control ${nodeId} does not map independent ${property} edits.`,
        );
    if (
      !tab &&
      !isOption &&
      (before.fontSize !== after.fontSize ||
        before.lineHeight !== after.lineHeight)
    )
      throw new ZuiDocumentError(
        `Segmented group label ${nodeId} uses native caption typography.`,
      );
    if (before.characters !== after.characters) {
      if (
        !after.characters.trim() ||
        after.characters !== after.characters.trim() ||
        (isOption && after.characters !== segmentLabel(after.characters))
      )
        throw new ZuiDocumentError(
          `Segmented control ${nodeId} requires nonempty, trimmed native display text.`,
        );
      if (isOption && option) {
        const target =
          node.state?.['options'] !== undefined
            ? node.state
            : (node.props ??= {});
        const values = target!['options'];
        if (!Array.isArray(values))
          throw new ZuiDocumentError(`Missing source options on ${nodeId}.`);
        if (option.field)
          (values[option.index] as ZuiTable)[option.field] = after.characters;
        else values[option.index] = after.characters;
        changes.push(
          `nodes.${nodeId}.${target === node.state ? 'state' : 'props'}.options.${option.index}${option.field ? `.${option.field}` : ''}`,
        );
        if (option.selected) {
          if (options.filter((option) => option.selected).length !== 1)
            throw new ZuiDocumentError(
              `Segmented control ${nodeId} has ambiguous duplicate selected options.`,
            );
          const selectedProperty = selectedSegmentProperty(props)!;
          const selectedTarget =
            node.state?.[selectedProperty] !== undefined
              ? node.state
              : (node.props ??= {});
          selectedTarget![selectedProperty] = after.characters;
          changes.push(
            `nodes.${nodeId}.${selectedTarget === node.state ? 'state' : 'props'}.${selectedProperty}`,
          );
        }
      } else if (before.property) {
        const target =
          node.state?.[before.property] !== undefined
            ? node.state
            : (node.props ??= {});
        target![before.property] = after.characters;
        changes.push(
          `nodes.${nodeId}.${target === node.state ? 'state' : 'props'}.${before.property}`,
        );
      } else
        throw new ZuiDocumentError(
          `Segmented control ${nodeId} has no mapped source text.`,
        );
    }
    const colorProperty: AuthoredPaintProperty = disabled
      ? 'disabled_foreground_color'
      : !tab && !isOption
        ? 'label_color'
        : (
              tab
                ? props['selected'] === true || props['checked'] === true
                : option?.selected
            )
          ? 'selected_foreground_color'
          : 'idle_text_color';
    const group = colors.get(colorProperty) ?? [];
    group.push({ before, after });
    colors.set(colorProperty, group);
    if (
      (tab || isOption) &&
      (before.fontSize !== after.fontSize ||
        before.lineHeight !== after.lineHeight)
    ) {
      if (!after.fontSize || after.fontSize <= 0)
        throw new ZuiDocumentError(`Invalid segmented font size on ${nodeId}.`);
      if (before.fontSize !== after.fontSize)
        changes.push(
          writeAuthoredPaintProperty(
            node,
            nodeId,
            tab ? 'tab_font_size' : 'font_size',
            after.fontSize,
            selfStyle,
          ),
        );
      changes.push(
        tab
          ? writeAuthoredPaintProperty(
              node,
              nodeId,
              'tab_line_height',
              after.fontSize * after.lineHeight,
              selfStyle,
            )
          : writeAuthoredLineHeight(
              node,
              nodeId,
              after.lineHeight,
              after.fontSize,
              selfStyle,
            ),
      );
    }
  }
  for (const property of ['fontSize', 'lineHeight'] as const)
    if (
      optionTexts.some(
        ({ before, after }) => before[property] !== after[property],
      ) &&
      new Set(optionTexts.map(({ after }) => after[property])).size > 1
    )
      throw new ZuiDocumentError(
        `Segmented options in ${nodeId} share ${property}.`,
      );
  for (const [property, group] of colors) {
    if (
      !group.some(
        ({ before, after }) =>
          before.color !== after.color ||
          Math.abs(before.colorOpacity - after.colorOpacity) >= 0.01,
      )
    )
      continue;
    if (
      new Set(group.map(({ after }) => `${after.color}:${after.colorOpacity}`))
        .size > 1
    )
      throw new ZuiDocumentError(
        `Segmented options in ${nodeId} share ${property}.`,
      );
    const { after } = group[0];
    if (!after.color)
      throw new ZuiDocumentError(
        `Segmented text ${nodeId} requires a solid color.`,
      );
    const value =
      after.colorOpacity >= 1
        ? after.color
        : `${after.color}${Math.round(after.colorOpacity * 255)
            .toString(16)
            .padStart(2, '0')}`;
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        property as AuthoredPaintProperty,
        value,
        selfStyle,
      ),
    );
  }
  if (
    options.filter((option) => option.selected).length === 1 &&
    segmentedOptions({ ...node.props, ...node.state }).filter(
      (option) => option.selected,
    ).length !== 1
  )
    throw new ZuiDocumentError(
      `Segmented edit on ${nodeId} would create ambiguous selection.`,
    );
}

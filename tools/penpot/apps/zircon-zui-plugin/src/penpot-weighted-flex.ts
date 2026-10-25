import { allocateNativeLayoutSize } from './penpot-layout-allocation';
import type { Board } from '@penpot/plugin-types';
import type { ProjectionContainer } from './bridge/penpot-projection-model';
import type { ZuiDocument, ZuiTable } from './bridge/zui-document';
import { resolveDesignNumber } from './bridge/zui-prefab-system';
import {
  resolveLinearSlotConstraint,
  type AxisConstraintInput,
  type LinearSlotSizeInput,
} from './penpot-linear-slot';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_NODE,
} from './metadata';

const EPSILON = 0.001;

export interface WeightedFlexWidth {
  min: number;
  preferred: number;
  max?: number | null;
  priority: number;
  weight: number;
  stretch?: 'Fixed' | 'Stretch';
}

export function stretchedRowChildHeight(
  parentHeight: number,
  paddingTop: number,
  paddingBottom: number,
  currentHeight: number,
  constraint: {
    stretch?: 'Fixed' | 'Stretch';
    min?: number | null;
    max?: number | null;
  },
): number {
  const min = Math.max(0, constraint.min ?? 0);
  const max =
    constraint.max == null || constraint.max < 0
      ? Infinity
      : Math.max(min, constraint.max);
  return Math.max(
    min,
    Math.min(
      max,
      constraint.stretch === 'Stretch'
        ? parentHeight - paddingTop - paddingBottom
        : currentHeight,
    ),
  );
}

/** Keep the review projection aligned with Runtime's axis constraint solver. */
export function solveWeightedFlexWidths(
  available: number,
  constraints: WeightedFlexWidth[],
): number[] {
  const axes = constraints.map((axis) => {
    const min = Math.max(0, axis.min);
    const max =
      axis.max == null || axis.max < 0 ? undefined : Math.max(min, axis.max);
    return {
      min,
      max,
      resolved: Math.max(min, Math.min(max ?? Infinity, axis.preferred)),
      priority: axis.priority,
      weight: axis.weight > 0 ? axis.weight : 1,
      stretch: axis.stretch ?? 'Stretch',
    };
  });
  const room = Math.max(0, available);
  const total = axes.reduce((sum, axis) => sum + axis.resolved, 0);
  if (total + EPSILON < room) {
    let remaining = room - total;
    const priorities = [
      ...new Set(
        axes
          .filter(
            (axis) =>
              axis.stretch === 'Stretch' &&
              axis.resolved + EPSILON < (axis.max ?? Infinity),
          )
          .map((axis) => axis.priority),
      ),
    ].sort((left, right) => right - left);
    for (const priority of priorities) {
      remaining = distribute(axes, priority, remaining, 'grow');
      if (remaining <= EPSILON) break;
    }
  } else if (total > room + EPSILON) {
    let deficit = total - room;
    const priorities = [
      ...new Set(
        axes
          .filter((axis) => axis.resolved > axis.min + EPSILON)
          .map((axis) => axis.priority),
      ),
    ].sort((left, right) => left - right);
    for (const priority of priorities) {
      deficit = distribute(axes, priority, deficit, 'shrink');
      if (deficit <= EPSILON) break;
    }
    const excess = axes.reduce((sum, axis) => sum + axis.resolved, 0) - room;
    let finalDeficit = Math.max(0, excess);
    for (const axis of axes) {
      const amount = Math.min(finalDeficit, axis.resolved - axis.min);
      axis.resolved -= amount;
      finalDeficit -= amount;
      if (finalDeficit <= EPSILON) break;
    }
  }
  return axes.map(
    ({ resolved }) => Math.round(resolved * 1_000_000) / 1_000_000,
  );
}

export interface NativeLinearFlexAxis {
  min: number;
  max?: number | null;
  basis: number;
  grow: number;
  shrink: number;
  /** Fixed outer slot spacing; it never participates in the shrink basis. */
  margin?: number;
}

/** Arrange native linear slots using the current Taffy flex contract.
 * A node minimum is a clamp, while a Stretch slot's own minimum is its basis.
 * Freeze clamped items and redistribute remaining room exactly as flex layout.
 */
export function solveNativeLinearFlexWidths(
  available: number,
  constraints: readonly NativeLinearFlexAxis[],
): number[] {
  const room = Math.max(0, available);
  const axes = constraints.map((axis) => {
    const min = Math.max(0, axis.min);
    const max = axis.max == null || axis.max < 0
      ? Infinity : Math.max(min, axis.max);
    const basis = Math.max(0, axis.basis);
    return {
      min, max, basis,
      grow: Math.max(0, axis.grow),
      shrink: Math.max(0, axis.shrink),
      margin: Math.max(0, axis.margin ?? 0),
      hypothetical: Math.max(min, Math.min(max, basis)),
      target: basis,
      frozen: false,
    };
  });
  const growing = axes.reduce(
    (sum, axis) => sum + axis.hypothetical + axis.margin, 0,
  ) < room;
  for (const axis of axes) {
    const factor = growing ? axis.grow : axis.shrink;
    if (factor === 0 ||
        (growing ? axis.basis > axis.hypothetical : axis.basis < axis.hypothetical)) {
      axis.frozen = true;
      axis.target = axis.hypothetical;
    }
  }
  const freeSpace = () => room - axes.reduce(
    (sum, axis) => sum + (axis.frozen ? axis.target : axis.basis) + axis.margin, 0,
  );
  const initialFree = freeSpace();
  for (let iteration = 0; iteration <= axes.length; iteration++) {
    const active = axes.filter((axis) => !axis.frozen);
    if (!active.length) break;
    let remaining = freeSpace();
    const factorSum = active.reduce(
      (sum, axis) => sum + (growing ? axis.grow : axis.shrink), 0,
    );
    if (factorSum < 1) {
      const partial = initialFree * factorSum;
      if (Math.abs(partial) < Math.abs(remaining)) remaining = partial;
    }
    const scaledSum = active.reduce(
      (sum, axis) => sum + (growing ? axis.grow : axis.shrink * axis.basis), 0,
    );
    const violations: Array<{axis: (typeof axes)[number]; value: number}> = [];
    let totalViolation = 0;
    for (const axis of active) {
      const scaled = growing ? axis.grow : axis.shrink * axis.basis;
      const proposed = axis.basis +
        (scaledSum > 0 ? remaining * scaled / scaledSum : 0);
      axis.target = Math.max(axis.min, Math.min(axis.max, proposed));
      const value = axis.target - proposed;
      totalViolation += value;
      violations.push({axis, value});
    }
    if (Math.abs(totalViolation) <= 0.000001) {
      active.forEach((axis) => { axis.frozen = true; });
      break;
    }
    const freezeMin = totalViolation > 0;
    for (const {axis, value} of violations) {
      if (freezeMin ? value > 0 : value < 0) axis.frozen = true;
    }
  }
  return axes.map(({target}) => Math.round(target * 1_000_000) / 1_000_000);
}

type ResolvedAxis = {
  min: number;
  max?: number;
  resolved: number;
  priority: number;
  weight: number;
  stretch: 'Fixed' | 'Stretch';
};

function distribute(
  axes: ResolvedAxis[],
  priority: number,
  amount: number,
  mode: 'grow' | 'shrink',
): number {
  let remaining = amount;
  while (remaining > EPSILON) {
    const active = axes.filter(
      (axis) =>
        axis.priority === priority &&
        (mode === 'grow'
          ? axis.stretch === 'Stretch' &&
            axis.resolved + EPSILON < (axis.max ?? Infinity)
          : axis.resolved > axis.min + EPSILON),
    );
    if (!active.length) break;
    const totalWeight = active.reduce((sum, axis) => sum + axis.weight, 0);
    let consumed = 0;
    for (const axis of active) {
      const share =
        totalWeight <= EPSILON
          ? remaining / active.length
          : (remaining * axis.weight) / totalWeight;
      const capacity =
        mode === 'grow'
          ? (axis.max ?? Infinity) - axis.resolved
          : axis.resolved - axis.min;
      const delta = Math.min(share, Math.max(0, capacity));
      axis.resolved += mode === 'grow' ? delta : -delta;
      consumed += delta;
    }
    if (consumed <= EPSILON) break;
    remaining -= consumed;
  }
  return remaining;
}

function table(value: unknown): ZuiTable | undefined {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as ZuiTable)
    : undefined;
}

function designNumber(
  document: ZuiDocument,
  value: unknown,
  fallback: number,
): number {
  return resolveDesignNumber(document, value) ?? fallback;
}

export interface LinearDesiredSize {
  width: number;
  height: number;
}

function axisConstraint(
  document: ZuiDocument,
  value: ZuiTable | undefined,
): AxisConstraintInput {
  return {
    min: designNumber(document, value?.['min'], 0),
    preferred: designNumber(document, value?.['preferred'], 0),
    max: resolveDesignNumber(document, value?.['max']) ?? -1,
    priority: designNumber(document, value?.['priority'], 0),
    weight: designNumber(document, value?.['weight'], 1),
    stretchMode: value?.['stretch'] === 'Fixed' ? 'Fixed' : 'Stretch',
  };
}

function linearSizing(
  document: ZuiDocument,
  slot: ZuiTable | undefined,
): LinearSlotSizeInput | undefined {
  const size = table(table(slot?.['layout'])?.['linear_size']);
  if (!size) return undefined;
  const rule = size['rule'];
  if (
    rule !== undefined &&
    !['Auto', 'Stretch', 'StretchContent'].includes(String(rule))
  )
    throw new Error(`Unsupported native linear_size rule: ${String(rule)}`);
  return {
    rule: (rule ?? 'Stretch') as LinearSlotSizeInput['rule'],
    value: designNumber(document, size['value'], 1),
    shrink_value: designNumber(document, size['shrink_value'], 1),
    min: designNumber(document, size['min'], 0),
    max: designNumber(document, size['max'], -1),
  };
}

function defaultAxis(constraint: AxisConstraintInput): boolean {
  return (
    constraint.min === 0 &&
    constraint.max === -1 &&
    constraint.preferred === 0 &&
    constraint.priority === 0 &&
    constraint.weight === 1 &&
    constraint.stretchMode === 'Stretch'
  );
}

function finiteAxisValues(document: ZuiDocument, value: ZuiTable | undefined): boolean {
  return ['min', 'max', 'preferred', 'priority', 'weight'].every((key) => {
    if (value?.[key] === undefined) return true;
    const resolved = resolveDesignNumber(document, value[key]);
    return resolved !== undefined && Number.isFinite(resolved);
  });
}

function nativeFixedPlacementSupported(
  document: ZuiDocument,
  layout: ZuiTable | undefined,
  slotLayout: ZuiTable | undefined,
  direction: 'row' | 'column',
  crossConstraint: AxisConstraintInput,
): boolean {
  // Native merges placement tables slot-first, unlike its restored main axis.
  const defaultPlacement = ['anchor', 'pivot', 'position'].every((key) => {
    const value = slotLayout?.[key] ?? layout?.[key];
    if (value === undefined) return true;
    const point = table(value);
    return point !== undefined && ['x', 'y'].every((coordinate) =>
      point[coordinate] === undefined ||
      resolveDesignNumber(document, point[coordinate]) === 0,
    );
  });
  if (!defaultPlacement) return false;
  const rawAlignment = slotLayout?.['alignment'];
  const alignment = table(rawAlignment);
  if (rawAlignment !== undefined && !alignment) return false;
  const main = alignment?.[direction === 'row' ? 'horizontal' : 'vertical'] ?? 'Start';
  const cross = alignment?.[direction === 'row' ? 'vertical' : 'horizontal'] ?? 'Start';
  return (main === 'Start' || main === 'Fill') &&
    (cross === 'Start' || cross === 'Fill' ||
      (crossConstraint.stretchMode === 'Fixed' && (cross === 'Center' || cross === 'End')));
}

function nativeSlotCrossAlignment(
  slotLayout: ZuiTable | undefined,
  direction: 'row' | 'column',
  nodeId: string,
): 'start' | 'center' | 'end' | 'stretch' {
  const raw = slotLayout?.['alignment'];
  const alignment = table(raw);
  if (raw !== undefined && !alignment)
    throw new Error(`Invalid native slot alignment on ${nodeId}.`);
  for (const axis of ['horizontal', 'vertical']) {
    const value = alignment?.[axis];
    if (value !== undefined && (typeof value !== 'string' || !['Start', 'Center', 'End', 'Fill'].includes(value)))
      throw new Error(`Unsupported native slot alignment ${nodeId}.${axis}.`);
  }
  const value = alignment?.[direction === 'row' ? 'vertical' : 'horizontal'] ?? 'Start';
  return value === 'Center' ? 'center' : value === 'End' ? 'end' : value === 'Fill' ? 'stretch' : 'start';
}

/** Resolve native main-axis slots because Penpot fill has no weighted flex value. */
export function applyWeightedFlexLayout(
  parent: Board,
  document: ZuiDocument,
  container?: ProjectionContainer,
  desiredSizes?: ReadonlyMap<string, LinearDesiredSize>,
): void {
  if (
    container?.kind !== 'flex' ||
    container.wrap ||
    parent.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'container-kind') ===
      'scroll'
  )
    return;
  const axis = container.direction;
  const main = axis === 'row' ? 'width' : 'height';
  const cross = axis === 'row' ? 'height' : 'width';
  const parentId = parent.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_NODE_ID,
  );
  const parentNode = document.nodes?.[parentId];
  const mounts = parentNode?.children ?? [];
  const sourceKind = table(parentNode?.layout?.['container'])?.['kind'] ?? parentNode?.component;
  // Extend only ordinary native Fixed boxes. Other implicit flex contracts keep
  // their existing weighted projection policy, including priority fallbacks.
  const nativeLinearParent = axis === 'row'
    ? sourceKind === 'HorizontalBox' || sourceKind === 'HorizontalGroup'
    : sourceKind === 'VerticalBox' || sourceKind === 'VerticalGroup';
  const nativeFixedParent = nativeLinearParent &&
    [parent.x ?? 0, parent.y ?? 0, parent.width, parent.height].every(Number.isFinite) &&
    [container.rowGap, container.columnGap, ...Object.values(container.padding)]
      .every((value) => Number.isFinite(value) && value >= 0) &&
    !parent.children.some((child) => !child.hidden && child.type === 'board' &&
      child.layoutChild?.absolute === true &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE);
  const children = parent.children.filter(
    (shape): shape is Board =>
      shape.type === 'board' &&
      !shape.hidden &&
      shape.layoutChild?.absolute !== true &&
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) ===
        ZUI_ROLE_NODE,
  );
  if (!children.length) return;
  const entries = children.map((child) => {
    const nodeId = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_NODE_ID,
    );
    const node = document.nodes?.[nodeId];
    const mount = mounts.find(({ node: mounted }) => mounted === nodeId);
    if (!node || !mount || !child.layoutChild) return null;
    // Native Linear parents restore the node's own main-axis table as a whole.
    const slotLayout = table(mount.slot?.['layout']);
    const mainTable = table(node.layout?.[main]) ?? table(slotLayout?.[main]);
    const crossTable =
      table(slotLayout?.[cross]) ?? table(node.layout?.[cross]);
    const own = axisConstraint(document, mainTable);
    const crossConstraint = axisConstraint(document, crossTable);
    const padding = table(slotLayout?.['padding']);
    const mainPadding =
      axis === 'row'
        ? designNumber(document, padding?.['left'], 0) +
          designNumber(document, padding?.['right'], 0)
        : designNumber(document, padding?.['top'], 0) +
          designNumber(document, padding?.['bottom'], 0);
    const crossPadding =
      axis === 'row'
        ? designNumber(document, padding?.['top'], 0) +
          designNumber(document, padding?.['bottom'], 0)
        : designNumber(document, padding?.['left'], 0) +
          designNumber(document, padding?.['right'], 0);
    const measured = desiredSizes?.get(nodeId)?.[main] ?? child[main];
    const desiredCross = desiredSizes?.get(nodeId)?.[cross] ?? child[cross];
    const extent = Math.max(0, measured) + Math.max(0, mainPadding);
    const sizing = linearSizing(document, mount.slot);
    const inflated = {
      ...own,
      min: (own.min ?? 0) + Math.max(0, mainPadding),
      preferred: (own.preferred ?? 0) + Math.max(0, mainPadding),
      max:
        own.max == null || own.max < 0
          ? -1
          : own.max + Math.max(0, mainPadding),
    };
    if (!sizing && own.stretchMode === 'Fixed') {
      // Native Fixed axes use the greater of preferred and measured content.
      // Keep padding in the allocation until it is removed from the result.
      inflated.preferred = Math.max(inflated.preferred, extent);
    } else if (
      !sizing &&
      extent > 0 &&
      defaultAxis(own) &&
      mainTable?.['stretch'] !== 'Stretch'
    ) {
      inflated.preferred = extent;
      inflated.stretchMode = 'Fixed';
    }
    const constraint = sizing
      ? resolveLinearSlotConstraint(axis, inflated, sizing, extent)
      : inflated;
    // Taffy keeps the node's content bounds separate from slot margins and
    // resolves preferred against those bounds before using it as a flex basis.
    const contentMin = Math.max(0, own.min ?? 0);
    const rawMax = own.max ?? -1;
    const contentMax = rawMax < 0 ? -1 : Math.max(contentMin, rawMax);
    const preferred = Math.max(contentMin,
      Math.min(contentMax < 0 ? Infinity : contentMax, Math.max(0, own.preferred ?? 0)));
    const contentConstraint = sizing
      ? resolveLinearSlotConstraint(axis, {...own, min: contentMin, max: contentMax, preferred}, sizing, measured)
      : {min: contentMin, max: contentMax};
    const preserveStretch = mainTable?.['stretch'] === 'Stretch';
    const weight = own.weight ?? 1;
    const nativeFixed = !sizing && own.stretchMode === 'Fixed' && own.priority === 0 &&
      finiteAxisValues(document, mainTable) && finiteAxisValues(document, crossTable) &&
      [measured, desiredCross, mainPadding, crossPadding].every(Number.isFinite) &&
      ['left', 'right', 'top', 'bottom'].every((side) => {
        const value = padding?.[side] === undefined ? 0 : resolveDesignNumber(document, padding[side]);
        return typeof value === 'number' && Number.isFinite(value) && value >= 0;
      }) && nativeFixedPlacementSupported(document, node.layout, slotLayout, axis, crossConstraint);
    return {
      child,
      crossAlignment: nativeLinearParent ? nativeSlotCrossAlignment(slotLayout, axis, nodeId) : undefined,
      mainPadding: Math.max(0, mainPadding),
      crossPadding: Math.max(0, crossPadding),
      crossConstraint,
      desiredCross,
      constraint,
      explicitLinear: sizing !== undefined,
      nativeFixed,
      linearFlex: {
        min: contentConstraint.min,
        max: contentConstraint.max,
        basis: sizing?.rule === 'Stretch'
          ? Math.max(0, sizing.min ?? 0)
          : !sizing && own.stretchMode === 'Stretch' && (preserveStretch || measured <= 0)
            ? preferred : Math.max(0, measured, preferred),
        grow: sizing
          ? sizing.rule === 'Auto' ? 0 : Math.max(0, sizing.value ?? 1)
          : own.stretchMode === 'Stretch' && (preserveStretch || measured <= 0 || preferred > 0)
            ? weight <= 0 ? 1 : weight : 0,
        shrink: Math.max(0, sizing?.shrink_value ?? 1),
        margin: Math.max(0, mainPadding),
      },
    };
  });
  if (entries.some((entry) => entry === null)) return;
  const valid = entries as Array<NonNullable<(typeof entries)[number]>>;
  const available = Math.max(
    0,
    parent[main] -
      (axis === 'row'
        ? container.padding.left +
          container.padding.right +
          container.columnGap * (valid.length - 1)
        : container.padding.top +
          container.padding.bottom +
          container.rowGap * (valid.length - 1)),
  );
  if (!Number.isFinite(available)) return;
  const nativeLinear = valid.some(({explicitLinear}) => explicitLinear) ||
    (nativeFixedParent && valid.every(({nativeFixed}) => nativeFixed));
  const resolved = nativeLinear
    ? solveNativeLinearFlexWidths(available, valid.map(({linearFlex}) => linearFlex))
    : solveWeightedFlexWidths(
        available,
        valid.map(({ constraint }) => ({
          min: constraint.min,
          max: constraint.max,
          preferred: constraint.preferred ?? 0,
          priority: constraint.priority ?? 0,
          weight: constraint.weight ?? 1,
          stretch: constraint.stretchMode,
        })),
      );
  valid.forEach(
    (
      { child, mainPadding, crossPadding, crossConstraint, desiredCross, crossAlignment },
      index,
    ) => {
      const targetMain = Math.max(0.01, resolved[index] - (nativeLinear ? 0 : mainPadding));
      const crossAvailable = Math.max(
        0,
        parent[cross] -
          crossPadding -
          (axis === 'row'
            ? container.padding.top + container.padding.bottom
            : container.padding.left + container.padding.right),
      );
      const targetCross = stretchedRowChildHeight(
        crossAvailable,
        0,
        0,
        desiredCross,
        {
          stretch: crossConstraint.stretchMode,
          min: crossConstraint.min,
          max: crossConstraint.max,
        },
      );
      const width = axis === 'row' ? targetMain : Math.max(0.01, targetCross);
      const height = axis === 'row' ? Math.max(0.01, targetCross) : targetMain;
      allocateNativeLayoutSize(child, width, height);
      const layoutChild = child.layoutChild!;
      const currentAlignment = layoutChild.alignSelf as string | undefined;
      if (crossAlignment !== undefined && currentAlignment !== crossAlignment &&
          !(crossAlignment === 'start' && (currentAlignment === undefined || currentAlignment === 'auto')))
        layoutChild.alignSelf = crossAlignment;
      if (axis === 'row') {
        if (layoutChild.horizontalSizing !== 'fix')
          layoutChild.horizontalSizing = 'fix';
        if (layoutChild.verticalSizing !== 'fix')
          layoutChild.verticalSizing = 'fix';
      } else {
        if (layoutChild.verticalSizing !== 'fix')
          layoutChild.verticalSizing = 'fix';
        if (layoutChild.horizontalSizing !== 'fix')
          layoutChild.horizontalSizing = 'fix';
      }
    },
  );
}

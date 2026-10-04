/**
 * The linear slot sizing contract shared by Penpot projection and native ZUI.
 *
 * This is deliberately a pure conversion: arranging siblings still belongs to
 * the weighted flex pass. The result is the axis constraint that pass should
 * use for one mounted child.
 */

export type LinearMainAxis = 'row' | 'column';

export type LinearSlotSizeRule = 'Auto' | 'Stretch' | 'StretchContent';

export type LinearStretchMode = 'Fixed' | 'Stretch';

/** The native AxisConstraint fields before `resolved()` normalization. */
export interface AxisConstraintInput {
  min?: number;
  /** A negative max means unbounded, as it does in native AxisConstraint. */
  max?: number | null;
  preferred?: number;
  priority?: number;
  weight?: number;
  stretchMode?: LinearStretchMode;
}

/** The serialized values from UiLinearSlotSizing. */
export interface LinearSlotSizeInput {
  rule?: LinearSlotSizeRule;
  value?: number;
  shrink_value?: number;
  min?: number;
  /** A negative max means unbounded, as it does in native slot_contract.rs. */
  max?: number | null;
}

export interface ResolvedLinearSlotConstraint {
  axis: LinearMainAxis;
  rule: LinearSlotSizeRule;
  desired: number;
  min: number;
  max: number | null;
  preferred: number;
  priority: number;
  weight: number;
  stretchMode: LinearStretchMode;
  /** The normalized slot values, retaining explicit zero values for callers. */
  slot: {
    rule: LinearSlotSizeRule;
    value: number;
    shrink_value: number;
    min: number;
    max: number;
  };
}

const DEFAULT_AXIS: Required<AxisConstraintInput> = {
  min: 0,
  max: -1,
  preferred: 0,
  priority: 0,
  weight: 1,
  stretchMode: 'Stretch',
};

const DEFAULT_SLOT: Required<LinearSlotSizeInput> = {
  rule: 'Stretch',
  value: 1,
  shrink_value: 1,
  min: 0,
  max: -1,
};

function finiteOr(value: number | null | undefined, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function resolvedMax(
  current: number | null | undefined,
  slotMax: number,
): number | null {
  const currentMax = finiteOr(current, -1);
  const normalizedSlotMax = slotMax >= 0 ? Math.max(slotMax, 0) : null;
  if (currentMax >= 0 && normalizedSlotMax !== null) {
    return Math.min(currentMax, normalizedSlotMax);
  }
  if (currentMax >= 0) {
    return Math.max(currentMax, 0);
  }
  return normalizedSlotMax;
}

function clampPreferred(
  preferred: number,
  min: number,
  max: number | null,
): number {
  const nonNegative = Math.max(finiteOr(preferred, 0), 0);
  return max === null
    ? Math.max(nonNegative, min)
    : Math.min(Math.max(nonNegative, min), max);
}

/**
 * Resolve one mounted child's linear slot exactly like native `axis.rs`.
 *
 * `desired` is the measured extent including the slot's main-axis padding. The
 * caller can then feed `min`, `max`, `preferred`, `weight`, and `stretchMode`
 * into its sibling allocator.
 */
export function resolveLinearSlotConstraint(
  axis: LinearMainAxis,
  nodeConstraint: AxisConstraintInput | undefined,
  linearSize: LinearSlotSizeInput | undefined,
  desired: number,
): ResolvedLinearSlotConstraint {
  const node = { ...DEFAULT_AXIS, ...nodeConstraint };
  const slot = { ...DEFAULT_SLOT, ...linearSize };
  const rule = slot.rule;
  const desiredExtent = Math.max(finiteOr(desired, 0), 0);
  const min = Math.max(
    finiteOr(node.min, 0),
    0,
    Math.max(finiteOr(slot.min, 0), 0),
  );
  const max = resolvedMax(node.max, finiteOr(slot.max, -1));
  const boundedMax = max === null ? null : Math.max(max, min);
  const basePreferred = finiteOr(node.preferred, 0);

  let preferred: number;
  let weight: number;
  let stretchMode: LinearStretchMode;
  switch (rule) {
    case 'Auto':
      preferred = Math.max(desiredExtent, basePreferred, min);
      weight = finiteOr(slot.shrink_value, 1);
      stretchMode = 'Fixed';
      break;
    case 'StretchContent':
      preferred = Math.max(desiredExtent, basePreferred, min);
      weight = finiteOr(slot.value, 1);
      stretchMode = 'Stretch';
      break;
    case 'Stretch':
    default:
      preferred = min;
      weight = finiteOr(slot.value, 1);
      stretchMode = 'Stretch';
      break;
  }

  return {
    axis,
    rule,
    desired: desiredExtent,
    min,
    max: boundedMax,
    preferred: clampPreferred(preferred, min, boundedMax),
    priority: Math.trunc(finiteOr(node.priority, 0)),
    weight: weight <= 0 ? 1 : weight,
    stretchMode,
    slot: {
      rule,
      value: finiteOr(slot.value, DEFAULT_SLOT.value),
      shrink_value: finiteOr(slot.shrink_value, DEFAULT_SLOT.shrink_value),
      min: finiteOr(slot.min, DEFAULT_SLOT.min),
      max: finiteOr(slot.max, -1),
    },
  };
}

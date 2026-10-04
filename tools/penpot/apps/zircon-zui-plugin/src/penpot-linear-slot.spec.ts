import {
  resolveLinearSlotConstraint,
  type AxisConstraintInput,
  type LinearSlotSizeInput,
} from './penpot-linear-slot';
import { describe, expect, it } from 'vitest';

const nodeAxis = (
  overrides: Partial<AxisConstraintInput> = {},
): AxisConstraintInput => ({
  min: 0,
  max: -1,
  preferred: 0,
  priority: 0,
  weight: 1,
  stretchMode: 'Stretch',
  ...overrides,
});

const slot = (
  overrides: Partial<LinearSlotSizeInput> = {},
): LinearSlotSizeInput => ({ rule: 'Stretch', ...overrides });

describe('native linear slot sizing contract', () => {
  it('keeps Auto content at the measured extent', () => {
    const result = resolveLinearSlotConstraint(
      'column',
      nodeAxis({ min: 8, preferred: 12 }),
      slot({ rule: 'Auto', shrink_value: 0 }),
      190,
    );

    expect(result).toMatchObject({
      axis: 'column',
      rule: 'Auto',
      min: 8,
      max: null,
      preferred: 190,
      weight: 1,
      stretchMode: 'Fixed',
    });
  });

  it('uses the slot minimum and value when a child stretches', () => {
    const result = resolveLinearSlotConstraint(
      'column',
      nodeAxis({ min: 24, preferred: 90, stretchMode: 'Fixed' }),
      slot({ rule: 'Stretch', value: 4 }),
      31,
    );

    expect(result).toMatchObject({
      min: 24,
      max: null,
      preferred: 24,
      weight: 4,
      stretchMode: 'Stretch',
    });
  });

  it('keeps measured content as the preferred floor for StretchContent', () => {
    const result = resolveLinearSlotConstraint(
      'column',
      nodeAxis({ min: 12, preferred: 24 }),
      slot({ rule: 'StretchContent', value: 1 }),
      190,
    );

    expect(result).toMatchObject({
      min: 12,
      max: null,
      preferred: 190,
      weight: 1,
      stretchMode: 'Stretch',
    });
  });

  it('merges slot bounds and resolves non-positive weights like Runtime', () => {
    const result = resolveLinearSlotConstraint(
      'row',
      nodeAxis({ min: 12, max: 180, preferred: 160 }),
      slot({
        rule: 'StretchContent',
        value: 0,
        shrink_value: 0,
        min: 40,
        max: 120,
      }),
      190,
    );

    expect(result).toMatchObject({
      axis: 'row',
      min: 40,
      max: 120,
      preferred: 120,
      weight: 1,
      stretchMode: 'Stretch',
      slot: {
        rule: 'StretchContent',
        value: 0,
        shrink_value: 0,
        min: 40,
        max: 120,
      },
    });
  });
});

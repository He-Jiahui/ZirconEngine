import { describe, expect, it } from 'vitest';
import { readViewportSnapState, viewportSnapControlProps } from './zui-viewport-snap-state';

describe('same-present native viewport snap state', () => {
  it('retains each of the eight independent combinations including explicit false', () => {
    for (let mask = 0; mask < 8; mask += 1) {
      const factual = {
        translationEnabled: Boolean(mask & 1),
        rotationEnabled: Boolean(mask & 2),
        scaleEnabled: Boolean(mask & 4),
      };
      const state = readViewportSnapState(JSON.parse(JSON.stringify(factual)));
      expect(state).toEqual(factual);
      const controls = viewportSnapControlProps(state);
      expect(controls.ToggleTranslateSnap).toEqual({ checked: factual.translationEnabled, selected: factual.translationEnabled, value: factual.translationEnabled });
      expect(controls.ToggleRotateSnap).toEqual({ checked: factual.rotationEnabled, selected: factual.rotationEnabled, value: factual.rotationEnabled });
      expect(controls.ToggleScaleSnap).toEqual({ checked: factual.scaleEnabled, selected: factual.scaleEnabled, value: factual.scaleEnabled });
    }
  });
  it('rejects old/status-only values, a missing axis, null/numeric/string booleans and extra axes', () => {
    for (const value of [
      undefined, null, [], 'Grid|Snap On',
      { gridMode: 'VisibleAndSnap' },
      { translationEnabled: true, rotationEnabled: false },
      { translationEnabled: true, rotationEnabled: false, scaleEnabled: null },
      { translationEnabled: true, rotationEnabled: false, scaleEnabled: 0 },
      { translationEnabled: true, rotationEnabled: false, scaleEnabled: 'false' },
      { translationEnabled: true, rotationEnabled: false, scaleEnabled: false, gridVisible: true },
    ]) expect(() => readViewportSnapState(value)).toThrow();
  });
  it('returns a detached snapshot so later input mutation cannot change checked state', () => {
    const input = { translationEnabled: false, rotationEnabled: true, scaleEnabled: false };
    const state = readViewportSnapState(input);
    input.translationEnabled = true;
    expect(viewportSnapControlProps(state).ToggleTranslateSnap.checked).toBe(false);
  });
});

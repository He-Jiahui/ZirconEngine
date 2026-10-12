/** Actual prepared viewport snap state; no grid/step/status inference. */
export interface ViewportSnapState {
  translationEnabled: boolean;
  rotationEnabled: boolean;
  scaleEnabled: boolean;
}

const FIELDS = ['translationEnabled', 'rotationEnabled', 'scaleEnabled'] as const;

export function readViewportSnapState(value: unknown): ViewportSnapState {
  if (typeof value !== 'object' || value === null || Array.isArray(value))
    throw new Error('workbenchPresentation.viewportSnap must be an object');
  const record = value as Record<string, unknown>;
  for (const key of Object.keys(record))
    if (!(FIELDS as readonly string[]).includes(key))
      throw new Error(`Unknown workbenchPresentation.viewportSnap field: ${key}`);
  for (const key of FIELDS)
    if (!Object.hasOwn(record, key) || typeof record[key] !== 'boolean')
      throw new Error(`workbenchPresentation.viewportSnap.${key} must be an explicit boolean`);
  return {
    translationEnabled: record['translationEnabled'] as boolean,
    rotationEnabled: record['rotationEnabled'] as boolean,
    scaleEnabled: record['scaleEnabled'] as boolean,
  };
}

export function viewportSnapControlProps(state: ViewportSnapState): Record<
  'ToggleTranslateSnap' | 'ToggleRotateSnap' | 'ToggleScaleSnap',
  { checked: boolean; selected: boolean; value: boolean }
> {
  const props = (enabled: boolean) => ({ checked: enabled, selected: enabled, value: enabled });
  return {
    ToggleTranslateSnap: props(state.translationEnabled),
    ToggleRotateSnap: props(state.rotationEnabled),
    ToggleScaleSnap: props(state.scaleEnabled),
  };
}

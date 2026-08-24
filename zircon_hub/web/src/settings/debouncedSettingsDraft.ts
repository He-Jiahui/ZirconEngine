import { useEffect, useMemo, useRef } from "react";

export const SETTINGS_DRAFT_QUIET_WINDOW_MS = 200;

export interface DebounceTimer {
  schedule(callback: () => void, delayMs: number): unknown;
  cancel(handle: unknown): void;
}

const systemTimer: DebounceTimer = {
  schedule: (callback, delayMs) => setTimeout(callback, delayMs),
  cancel: (handle) => clearTimeout(handle as ReturnType<typeof setTimeout>),
};

export class DebouncedSettingsDraft<Draft> {
  readonly #dispatch: (draft: Draft) => void;
  readonly #delayMs: number;
  readonly #timer: DebounceTimer;
  #generation = 0;
  #pendingDraft: Draft | undefined;
  #pendingHandle: unknown;

  constructor(dispatch: (draft: Draft) => void, delayMs: number, timer: DebounceTimer = systemTimer) {
    this.#dispatch = dispatch;
    this.#delayMs = delayMs;
    this.#timer = timer;
  }

  schedule(draft: Draft) {
    this.cancelPending();
    const generation = this.#generation;
    this.#pendingDraft = draft;
    this.#pendingHandle = this.#timer.schedule(() => {
      if (generation !== this.#generation || this.#pendingDraft === undefined) {
        return;
      }

      const pendingDraft = this.#pendingDraft;
      this.#pendingDraft = undefined;
      this.#pendingHandle = undefined;
      this.#dispatch(pendingDraft);
    }, this.#delayMs);
  }

  cancel() {
    this.cancelPending();
  }

  private cancelPending() {
    this.#generation += 1;
    this.#pendingDraft = undefined;
    if (this.#pendingHandle !== undefined) {
      this.#timer.cancel(this.#pendingHandle);
      this.#pendingHandle = undefined;
    }
  }
}

export function useDebouncedSettingsDraft<Draft>(dispatch: (draft: Draft) => void) {
  const dispatchRef = useRef(dispatch);
  dispatchRef.current = dispatch;
  const dispatcher = useMemo(
    () =>
      new DebouncedSettingsDraft<Draft>(
        (draft) => dispatchRef.current(draft),
        SETTINGS_DRAFT_QUIET_WINDOW_MS,
      ),
    [],
  );

  useEffect(() => () => dispatcher.cancel(), [dispatcher]);
  return useMemo(
    () => ({
      scheduleDraftPublication: (draft: Draft) => dispatcher.schedule(draft),
      cancelPendingDraft: () => dispatcher.cancel(),
    }),
    [dispatcher],
  );
}

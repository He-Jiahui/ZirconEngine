import { useCallback, useEffect, useMemo, useRef } from "react";

export const PROJECT_SEARCH_QUIET_WINDOW_MS = 200;

export interface DebounceTimer {
  schedule(callback: () => void, delayMs: number): unknown;
  cancel(handle: unknown): void;
}

const systemTimer: DebounceTimer = {
  schedule: (callback, delayMs) => setTimeout(callback, delayMs),
  cancel: (handle) => clearTimeout(handle as ReturnType<typeof setTimeout>),
};

export class DebouncedProjectSearch {
  readonly #dispatch: (query: string) => void;
  readonly #delayMs: number;
  readonly #timer: DebounceTimer;
  #generation = 0;
  #pendingHandle: unknown;

  constructor(dispatch: (query: string) => void, delayMs: number, timer: DebounceTimer = systemTimer) {
    this.#dispatch = dispatch;
    this.#delayMs = delayMs;
    this.#timer = timer;
  }

  schedule(query: string) {
    this.cancelPending();
    const generation = this.#generation;
    this.#pendingHandle = this.#timer.schedule(() => {
      if (generation !== this.#generation) {
        return;
      }
      this.#pendingHandle = undefined;
      this.#dispatch(query);
    }, this.#delayMs);
  }

  cancel() {
    this.cancelPending();
  }

  private cancelPending() {
    this.#generation += 1;
    if (this.#pendingHandle !== undefined) {
      this.#timer.cancel(this.#pendingHandle);
      this.#pendingHandle = undefined;
    }
  }
}

export function useDebouncedProjectSearch(dispatch: (query: string) => void) {
  const dispatchRef = useRef(dispatch);
  dispatchRef.current = dispatch;
  const dispatcher = useMemo(
    () => new DebouncedProjectSearch((query) => dispatchRef.current(query), PROJECT_SEARCH_QUIET_WINDOW_MS),
    [],
  );

  useEffect(() => () => dispatcher.cancel(), [dispatcher]);
  return useCallback((query: string) => dispatcher.schedule(query), [dispatcher]);
}

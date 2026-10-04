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

export class SettingsActionScheduler<Draft> {
  readonly #drafts: DebouncedSettingsDraft<Draft>;
  readonly #publishDraft: (draft: Draft) => void | Promise<void>;
  #tail: Promise<void> = Promise.resolve();

  constructor(
    publishDraft: (draft: Draft) => void | Promise<void>,
    delayMs: number = SETTINGS_DRAFT_QUIET_WINDOW_MS,
    timer: DebounceTimer = systemTimer,
  ) {
    this.#publishDraft = publishDraft;
    this.#drafts = new DebouncedSettingsDraft((draft) => {
      void this.enqueue(() => this.#publishDraft(draft));
    }, delayMs, timer);
  }

  schedule(draft: Draft) {
    this.#drafts.schedule(draft);
  }

  cancelPending() {
    this.#drafts.cancel();
  }

  runBarrier<Result>(action: () => Result | Promise<Result>): Promise<Result> {
    this.#drafts.cancel();
    return this.enqueue(action);
  }

  private enqueue<Result>(action: () => Result | Promise<Result>): Promise<Result> {
    const result = this.#tail.then(action);
    this.#tail = result.then(
      () => undefined,
      () => undefined,
    );
    return result;
  }
}

export function useDebouncedSettingsDraft<Draft>(dispatch: (draft: Draft) => void | Promise<void>) {
  const dispatchRef = useRef(dispatch);
  dispatchRef.current = dispatch;
  const dispatcher = useMemo(
    () => new SettingsActionScheduler<Draft>((draft) => dispatchRef.current(draft)),
    [],
  );

  useEffect(() => () => dispatcher.cancelPending(), [dispatcher]);
  return useMemo(
    () => ({
      scheduleDraftPublication: (draft: Draft) => dispatcher.schedule(draft),
      cancelPendingDraft: () => dispatcher.cancelPending(),
      runSettingsBarrier: <Result>(action: () => Result | Promise<Result>) => dispatcher.runBarrier(action),
    }),
    [dispatcher],
  );
}

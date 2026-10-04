import { useCallback, useEffect, useMemo, useRef } from "react";

// 本地结果立即响应输入；静默窗口只用于减少向后端发布查询的频率。
export const PROJECT_SEARCH_QUIET_WINDOW_MS = 200;

// 计时器适配点供确定性生命周期测试使用；调度须异步返回可取消的句柄，取消后已排队回调仍可能到达。
export interface DebounceTimer {
  schedule(callback: () => void, delayMs: number): unknown;
  cancel(handle: unknown): void;
}

const systemTimer: DebounceTimer = {
  schedule: (callback, delayMs) => setTimeout(callback, delayMs),
  cancel: (handle) => clearTimeout(handle as ReturnType<typeof setTimeout>),
};

// 发布连续输入的最后一个查询；持有实例的页面或测试负责在生命周期结束时取消，实例不持有业务状态。
export class DebouncedProjectSearch {
  readonly #dispatch: (query: string) => void;
  readonly #delayMs: number;
  readonly #timer: DebounceTimer;
  #generation = 0;
  #pendingHandle: unknown;

  // 发布目标和静默时长在实例生命周期内固定；测试可注入计时器，而页面使用浏览器调度。
  constructor(dispatch: (query: string) => void, delayMs: number, timer: DebounceTimer = systemTimer) {
    this.#dispatch = dispatch;
    this.#delayMs = delayMs;
    this.#timer = timer;
  }

  // 新输入取代尚未发布的旧输入，不能用作必须逐条送达的命令队列。
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

  // 生命周期清理只撤销待发布输入，不撤回已经发出的后端请求。
  cancel() {
    this.cancelPending();
  }

  // 代际同时废弃已排队的旧回调，不能只依赖取消句柄保证卸载后不会发布。
  private cancelPending() {
    this.#generation += 1;
    if (this.#pendingHandle !== undefined) {
      this.#timer.cancel(this.#pendingHandle);
      this.#pendingHandle = undefined;
    }
  }
}

// TODO: [CR-HUBWEB-0001] 确认静默期内筛选或排序回包同步查询时应保留还是撤销未发布输入；缺少交互回包顺序证据；下一步核查两处项目页面的本地查询同步并补充回归。
// 页面挂载期间复用调度器，回调始终读取最新动作入口；卸载清理防止旧页面继续发布查询。
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

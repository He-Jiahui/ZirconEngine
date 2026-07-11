import type { ControlEvent } from "./contracts";
import { parseControlEvent } from "./validation";

export interface EventCallbacks {
  onEvent: (event: ControlEvent) => void;
  onResync: () => void;
  onConnection: (connected: boolean) => void;
  onError: (message: string) => void;
}

export function openControlEvents(cursor: number, callbacks: EventCallbacks): () => void {
  const source = new EventSource(`/control/v1/events/stream?cursor=${encodeURIComponent(cursor)}`, { withCredentials: true });
  source.onopen = () => callbacks.onConnection(true);
  source.addEventListener("coordinator", (raw) => {
    try {
      const event = raw as MessageEvent<string>;
      callbacks.onEvent(parseControlEvent(event.lastEventId, event.data));
    } catch (error) {
      callbacks.onError(error instanceof Error ? error.message : "事件格式无效");
      callbacks.onResync();
    }
  });
  source.addEventListener("resync_required", callbacks.onResync);
  source.onerror = () => {
    callbacks.onConnection(false);
    callbacks.onError("实时连接已断开，正在等待重连");
  };
  return () => source.close();
}

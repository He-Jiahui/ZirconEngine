export type WindowActionKind = "close" | "minimize" | "toggle-maximize";

export type WindowActionFailureHandler = (action: WindowActionKind, error: unknown) => void;

export interface WindowActionScheduler {
  inFlightCount(): number;
  run(action: WindowActionKind, invoke: () => Promise<void>): Promise<boolean>;
}

export function createWindowActionScheduler(onFailure: WindowActionFailureHandler): WindowActionScheduler {
  const inFlight = new Map<WindowActionKind, Promise<boolean>>();

  return {
    inFlightCount: () => inFlight.size,
    run(action, invoke) {
      const existing = inFlight.get(action);
      if (existing) {
        return existing;
      }

      const receipt = Promise.resolve()
        .then(invoke)
        .then(
          () => true,
          (error: unknown) => {
            onFailure(action, error);
            return false;
          },
        )
        .finally(() => {
          if (inFlight.get(action) === receipt) {
            inFlight.delete(action);
          }
        });
      inFlight.set(action, receipt);
      return receipt;
    },
  };
}

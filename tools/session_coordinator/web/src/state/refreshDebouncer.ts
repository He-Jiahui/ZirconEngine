export interface ResyncDebouncer {
  schedule(): void;
  flush(): void;
  cancel(): void;
}

export function createResyncDebouncer(
  onResync: () => void,
  delayMs = 250,
): ResyncDebouncer {
  let pending: ReturnType<typeof setTimeout> | undefined;

  const cancel = () => {
    if (pending === undefined) return;
    clearTimeout(pending);
    pending = undefined;
  };

  return {
    schedule: () => {
      if (pending !== undefined) return;
      pending = setTimeout(() => {
        pending = undefined;
        onResync();
      }, delayMs);
    },
    flush: () => {
      cancel();
      onResync();
    },
    cancel,
  };
}

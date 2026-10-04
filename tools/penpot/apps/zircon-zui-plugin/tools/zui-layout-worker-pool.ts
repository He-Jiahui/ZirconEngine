import assert from 'node:assert/strict';

export function serializedWriter<T>(write: (value: T) => Promise<void>) {
  let tail = Promise.resolve();
  return (value: T): Promise<void> => {
    const snapshot = structuredClone(value);
    tail = tail.then(() => write(snapshot));
    return tail;
  };
}

export async function runReviewPhases<T>(
  phases: T[][],
  workers: number,
  runWorker: (take: () => T | undefined) => Promise<void>,
): Promise<void> {
  assert.ok(
    Number.isInteger(workers) && workers >= 1 && workers <= 4,
    '--workers must be an integer from 1 to 4',
  );
  for (const phase of phases) {
    let next = 0;
    const take = () => phase[next++];
    const results = await Promise.allSettled(
      Array.from({ length: Math.min(workers, phase.length) }, () =>
        runWorker(take),
      ),
    );
    const failures = results.filter(
      (result): result is PromiseRejectedResult => result.status === 'rejected',
    );
    if (failures.length)
      throw new AggregateError(
        failures.map((result) => result.reason),
        'Penpot workers failed',
      );
  }
}

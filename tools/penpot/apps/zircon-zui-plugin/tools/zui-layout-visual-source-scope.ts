import assert from 'node:assert/strict';

/** Prepare only requested sources before applying pending and batch filters. */
export async function prepareVisualSourceEntries<
  T extends { sourcePath: string },
>(
  entries: readonly T[],
  args: readonly string[],
  prepare: (entry: T) => Promise<void>,
): Promise<T[]> {
  const filters = args.flatMap((argument, index) => {
    if (argument !== '--source') return [];
    const value = args[index + 1];
    assert.ok(value && !value.startsWith('--'), '--source requires a path');
    return [value];
  });
  const selected = entries.filter(
    (entry) =>
      !filters.length ||
      filters.some((source) => entry.sourcePath.includes(source)),
  );
  for (const entry of selected) await prepare(entry);
  return selected;
}

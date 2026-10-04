/** Select exact authored sources while keeping every other catalog record intact. */
export function catalogSourceSelection(args: readonly string[]): string[] {
  const result: string[] = [];
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument !== '--source' && !argument?.startsWith('--source=')) continue;
    const value = argument === '--source' ? args[++index] : argument.slice(9);
    if (!value || value.startsWith('--'))
      throw new Error(
        '--source requires an exact repository-relative .zui path',
      );
    const normalized = value.replace(/\\/g, '/');
    if (
      !normalized.endsWith('.zui') ||
      normalized.startsWith('/') ||
      normalized.includes(':') ||
      normalized.split('/').some((segment) => ['', '.', '..'].includes(segment))
    )
      throw new Error(`Invalid catalog source path: ${value}`);
    result.push(normalized);
  }
  return [...new Set(result)];
}

export function selectCatalogSources(
  inventory: readonly string[],
  requested: readonly string[],
): string[] {
  if (requested.length === 0) return [...inventory];
  const available = new Set(inventory);
  for (const source of requested)
    if (!available.has(source))
      throw new Error(`Catalog source not found: ${source}`);
  const selected = new Set(requested);
  return inventory.filter((source) => selected.has(source));
}

/** Retain unselected evidence by reference; a focused refresh owns only its sources. */
export function mergeSelectedCatalogEntries<T extends { sourcePath: string }>(
  previous: readonly T[],
  updated: readonly T[],
): T[] {
  const replacements = new Map(
    updated.map((entry) => [entry.sourcePath, entry]),
  );
  if (replacements.size !== updated.length)
    throw new Error(
      'Focused catalog refresh produced duplicate source records',
    );
  const result = previous.map((entry) => {
    const replacement = replacements.get(entry.sourcePath);
    replacements.delete(entry.sourcePath);
    return replacement ?? entry;
  });
  result.push(...replacements.values());
  return result;
}

export function findPenpotShapeAncestor<T extends { readonly id: string }>(
  start: T | null | undefined,
  parentOf: (shape: T) => T | null,
  matches: (shape: T) => boolean,
): T | null {
  const visitedIds = new Set<string>();
  let current = start ?? null;

  while (current) {
    const id = current.id;
    if (typeof id !== 'string' || id.length === 0 || visitedIds.has(id))
      return null;
    visitedIds.add(id);
    if (matches(current)) return current;
    current = parentOf(current);
  }

  return null;
}

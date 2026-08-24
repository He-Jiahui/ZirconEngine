export function groupBy<T>(items: readonly T[], key: (item: T) => string): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  let previousGroupKey: string | undefined;
  let previousGroup: T[] | undefined;

  for (const item of items) {
    const groupKey = key(item);
    if (previousGroup !== undefined && groupKey === previousGroupKey) {
      previousGroup.push(item);
      continue;
    }

    const group = groups.get(groupKey);
    if (group === undefined) {
      previousGroup = [item];
      groups.set(groupKey, previousGroup);
    } else {
      group.push(item);
      previousGroup = group;
    }
    previousGroupKey = groupKey;
  }

  return groups;
}

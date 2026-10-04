// 为目录树保留分类首次出现及组内原行顺序；每项键只求一次，结果组数组归调用方所有，输入行不被复制。
export function groupBy<T>(items: readonly T[], key: (item: T) => string): Map<string, T[]> {
  const groups = new Map<string, T[]>();
  // 连续分类缓存服务大目录常见的同组输入；非连续重复键仍须回到已有组，不能要求调用方预先排序。
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

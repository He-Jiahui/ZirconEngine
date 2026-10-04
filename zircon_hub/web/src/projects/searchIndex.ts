// 索引保留原项目对象供表格与卡片使用；预处理文本只对应构建时的内容快照。
export interface SearchIndexEntry<T> {
  item: T;
  normalizedText: string;
}

// 数据快照变化时构建，查询输入复用；页面负责选择可搜索字段并在字段变化时重建索引。
export function buildSearchIndex<T>(items: T[], searchableText: (item: T) => string): SearchIndexEntry<T>[] {
  return items.map((item) => ({
    item,
    normalizedText: searchableText(item).toLowerCase(),
  }));
}

// 项目数组和索引须配对使用；空查询保留原数组身份，其余结果保留索引顺序和原对象。
export function filterSearchIndex<T>(items: T[], index: SearchIndexEntry<T>[], query: string): T[] {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) {
    return items;
  }

  const matches: T[] = [];
  for (const entry of index) {
    if (entry.normalizedText.includes(normalizedQuery)) {
      matches.push(entry.item);
    }
  }
  return matches;
}

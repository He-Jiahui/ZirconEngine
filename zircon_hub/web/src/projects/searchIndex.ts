export interface SearchIndexEntry<T> {
  item: T;
  normalizedText: string;
}

export function buildSearchIndex<T>(items: T[], searchableText: (item: T) => string): SearchIndexEntry<T>[] {
  return items.map((item) => ({
    item,
    normalizedText: searchableText(item).toLowerCase(),
  }));
}

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

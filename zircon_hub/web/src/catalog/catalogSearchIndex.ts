export type CatalogMode = "assets" | "plugins" | "learn";

export interface CatalogSearchableRow {
  title: string;
  detail: string;
  meta: string;
  category: string;
  categoryKey: string;
  scope: string;
  scopeKey: string;
  path: string;
}

export interface CatalogSearchIndexEntry<T extends CatalogSearchableRow> {
  row: T;
  normalizedText: string;
}

const FIELD_SEPARATOR = "\0";

export function buildCatalogSearchIndex<T extends CatalogSearchableRow>(rows: readonly T[]): CatalogSearchIndexEntry<T>[] {
  return rows.map((row) => ({
    row,
    normalizedText: searchableFields(row).join(FIELD_SEPARATOR).toLowerCase(),
  }));
}

export function filterCatalogSearchIndex<T extends CatalogSearchableRow>(
  rows: readonly T[],
  searchIndex: readonly CatalogSearchIndexEntry<T>[],
  mode: CatalogMode,
  tab: string,
  query: string,
): T[] {
  const normalizedQuery = query.trim().toLowerCase();
  if (normalizedQuery.includes(FIELD_SEPARATOR)) {
    return rows.filter(
      (row) =>
        matchesCatalogTab(row, mode, tab) &&
        searchableFields(row).some((value) => value.toLowerCase().includes(normalizedQuery)),
    );
  }

  const matches: T[] = [];
  for (const entry of searchIndex) {
    if (
      matchesCatalogTab(entry.row, mode, tab) &&
      (normalizedQuery.length === 0 || entry.normalizedText.includes(normalizedQuery))
    ) {
      matches.push(entry.row);
    }
  }
  return matches;
}

function searchableFields(row: CatalogSearchableRow) {
  return [row.title, row.detail, row.meta, row.category, row.scope, row.path];
}

function matchesCatalogTab(row: CatalogSearchableRow, mode: CatalogMode, tab: string) {
  return (
    tab === "all" ||
    (mode === "learn"
      ? row.categoryKey === tab
      : tab === "project"
        ? row.scopeKey === "project"
        : row.scopeKey === "engine")
  );
}

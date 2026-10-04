// 目录页面的三类本地投影共用搜索契约；模式只影响标签含义，展示文案与稳定分类键分别传入。
export type CatalogMode = "assets" | "plugins" | "learn";

// 调用方须先完成本地化投影；搜索覆盖可见说明及路径，分类与作用域的键用于筛选，不能以翻译文案替代。
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

// 条目保留原行身份以便页面继续选中和展示；缓存文本只对构建时的行内容有效。
export interface CatalogSearchIndexEntry<T extends CatalogSearchableRow> {
  row: T;
  normalizedText: string;
}

// 字段边界阻止普通查询跨两段文案拼接命中；带边界字符的真实查询须保留逐字段匹配语义。
const FIELD_SEPARATOR = "\0";

// 在行快照改变时构建一次，供连续输入复用；调用方若原地修改行内容，必须同时重建索引。
export function buildCatalogSearchIndex<T extends CatalogSearchableRow>(rows: readonly T[]): CatalogSearchIndexEntry<T>[] {
  return rows.map((row) => ({
    row,
    normalizedText: searchableFields(row).join(FIELD_SEPARATOR).toLowerCase(),
  }));
}

// 行数组与索引须来自同一快照；结果保留索引顺序和原行对象，模式及标签由页面选项提供。
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

// 字段集合与页面的旧搜索范围一致，稳定键不参与用户可见文本搜索。
function searchableFields(row: CatalogSearchableRow) {
  return [row.title, row.detail, row.meta, row.category, row.scope, row.path];
}

// 学习目录按分类键筛选；其余目录仅区分项目与引擎作用域，调用方负责提供相应标签选项。
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

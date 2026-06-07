import AutoStoriesOutlinedIcon from "@mui/icons-material/AutoStoriesOutlined";
import CategoryOutlinedIcon from "@mui/icons-material/CategoryOutlined";
import ExtensionOutlinedIcon from "@mui/icons-material/ExtensionOutlined";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import OpenInNewOutlinedIcon from "@mui/icons-material/OpenInNewOutlined";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import { Box, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import { EmptyStateBlock, HubList, HubPanel, HubTreeView, MetricCard, QuickActions, SourceEngineList, StatusBadge } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubSearchField, HubTabs } from "../components/inputs";
import { formatCountText } from "../text/counts";
import { quickActionProjectTargetPayload } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubShellState, StatusTone } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface CatalogPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

interface CatalogRow {
  id: string;
  title: string;
  detail: string;
  meta: string;
  category: string;
  categoryKey: string;
  scope: string;
  scopeKey: string;
  path: string;
  status: string;
  tone: StatusTone;
}

const pageIcon = {
  assets: Inventory2OutlinedIcon,
  plugins: ExtensionOutlinedIcon,
  learn: AutoStoriesOutlinedIcon,
};

export function CatalogPage({ state, onAction }: CatalogPageProps) {
  const mode: "assets" | "plugins" | "learn" = state.activePage === "plugins" || state.activePage === "learn" ? state.activePage : "assets";
  const [query, setQuery] = useState("");
  const [tab, setTab] = useState("all");
  const [selectedRowId, setSelectedRowId] = useState<string | null>(null);
  const common = state.ui.common;
  const text = state.ui.catalog;
  const Icon = pageIcon[mode];
  const comingSoonRows = useMemo(() => state.comingSoon.filter((entry) => entry.category === mode), [mode, state.comingSoon]);
  const project = state.selectedProject;
  const quickActionProjectTarget = quickActionProjectTargetPayload(project);

  const rows = useMemo(() => catalogRows(state, mode, text), [mode, state, text]);
  const visibleRows = useMemo(() => filterRows(rows, mode, tab, query), [mode, query, rows, tab]);
  const categoryCount = new Set(rows.map((row) => row.category)).size;
  const scopeCount = new Set(rows.map((row) => row.scope)).size;
  const selectedRow = useMemo(() => {
    const selectedCandidates = visibleRows.length > 0 ? visibleRows : rows;
    return selectedCandidates.find((row) => row.id === selectedRowId) ?? selectedCandidates[0];
  }, [rows, selectedRowId, visibleRows]);
  const openLearnResource = (row: CatalogRow) => {
    void onAction(HUB_ACTION.openResource, undefined, { resourceId: row.id, path: row.path });
  };
  const treeNodes = useMemo(
    () => [
      {
        id: `${mode}-catalog`,
        label: state.pageTitle,
        detail: formatCountText(common.entryCountTemplate, rows.length),
        children: Array.from(groupBy(rows, (row) => row.category)).map(([category, groupedRows]) => ({
          id: `${mode}-${category}`,
          label: category,
          detail: formatCountText(common.entryCountTemplate, groupedRows.length),
          children: groupedRows.map((row) => ({
            id: `${mode}-${category}-${row.id}`,
            label: row.title,
            detail: row.scope,
          })),
        })),
      },
    ],
    [common.entryCountTemplate, mode, rows, state.pageTitle],
  );

  return (
    <Box
      sx={{
        height: "100%",
        minHeight: 0,
        overflow: "auto",
        px: `${hubTokens.window.pagePaddingX}px`,
        py: `${hubTokens.window.pagePaddingY}px`,
        "@media (max-width: 980px)": { px: 2, py: 2 },
      }}
    >
      <Box sx={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 2, mb: 2.5 }}>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="h4">{state.pageTitle}</Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mt: 0.9 }}>
            {state.pageSubtitle}
          </Typography>
        </Box>
        <Box sx={{ width: 320, maxWidth: "100%", "@media (max-width: 760px)": { width: "100%" } }}>
          <HubSearchField value={query} placeholder={`${text.searchPlaceholderPrefix}${text.searchPlaceholderSeparator}${state.pageTitle}${text.searchPlaceholderSuffix}`} onChange={setQuery} />
        </Box>
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubStatusBanner task={state.taskSummary} />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "repeat(3, minmax(0, 1fr))",
          gap: 1.2,
          mb: 1.4,
          "@media (max-width: 980px)": { gridTemplateColumns: "1fr" },
        }}
      >
        <MetricCard label={text.entries} value={`${rows.length}`} detail={state.pageTitle} icon={<Icon />} tone="accent" />
        <MetricCard label={text.categories} value={`${categoryCount}`} detail={selectedRow?.category ?? text.noCatalog} icon={<CategoryOutlinedIcon />} />
        <MetricCard label={text.scopes} value={`${scopeCount}`} detail={selectedRow?.scope ?? text.noScope} icon={<StorageOutlinedIcon />} tone="success" />
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubTabs value={tab} onChange={setTab} options={catalogTabs(mode, text)} />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) minmax(330px, 0.55fr)",
          gap: 1.4,
          "@media (max-width: 1180px)": { gridTemplateColumns: "1fr" },
        }}
      >
        <HubPanel title={catalogPanelTitle(mode, text)}>
          {visibleRows.length > 0 ? (
            <HubList
              items={visibleRows.map((row) => ({
                id: row.id,
                title: row.title,
                detail: row.detail,
                meta: row.meta,
                selected: selectedRow?.id === row.id,
                icon: iconForMode(mode),
              }))}
              onSelect={(item) => setSelectedRowId(item.id)}
            />
          ) : (
            <EmptyStateBlock title={text.noEntriesFound} detail={text.noEntriesFoundDetail} icon={<FolderOutlinedIcon />} />
          )}
        </HubPanel>

        <HubPanel title={text.selectedEntry}>
          {selectedRow ? (
            <Box sx={{ display: "grid", gap: 1.1 }}>
              <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1.2 }}>
                <Typography variant="body2" noWrap sx={{ fontWeight: 700, color: hubTokens.colors.text }}>
                  {selectedRow.title}
                </Typography>
                <StatusBadge label={selectedRow.status} tone={selectedRow.tone} />
              </Box>
              <HubList
                items={[
                  { id: "category", title: common.category, detail: selectedRow.category },
                  { id: "scope", title: common.scope, detail: selectedRow.scope },
                  { id: "path", title: common.path, detail: selectedRow.path },
                ]}
              />
              {mode === "learn" ? (
                <Box sx={{ display: "flex", justifyContent: "flex-end" }}>
                  <HubButton tone="primary" startIcon={<OpenInNewOutlinedIcon />} onClick={() => openLearnResource(selectedRow)}>
                    {state.ui.actions.openResource}
                  </HubButton>
                </Box>
              ) : null}
            </Box>
          ) : (
            <EmptyStateBlock title={text.noCatalogEntrySelected} detail={text.noCatalogEntrySelectedDetail} />
          )}
        </HubPanel>

        <HubPanel title={text.catalogTree}>
          <HubTreeView nodes={treeNodes} defaultExpanded={[`${mode}-catalog`]} />
        </HubPanel>

        {comingSoonRows.length > 0 ? (
          <HubPanel title={text.comingSoonPanel}>
            <HubList
              items={comingSoonRows.map((entry) => ({
                id: entry.id,
                title: entry.title,
                detail: entry.detail,
                meta: entry.meta,
                disabled: entry.disabled,
                icon: iconForMode(mode),
              }))}
            />
          </HubPanel>
        ) : null}

        <HubPanel title={common.quickActions}>
          <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
        </HubPanel>

        <HubPanel title={common.sourceEngines}>
          <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
        </HubPanel>
      </Box>
    </Box>
  );
}

function catalogRows(state: HubShellState, mode: "assets" | "plugins" | "learn", text: HubShellState["ui"]["catalog"]): CatalogRow[] {
  if (mode === "plugins") {
    return state.plugins.map((plugin) => ({
      id: plugin.id,
      title: plugin.displayName,
      detail: plugin.description,
      meta: formatCountText(text.moduleCountTemplate, plugin.moduleCount),
      category: plugin.category,
      categoryKey: "local",
      scope: plugin.scope,
      scopeKey: plugin.scopeKey,
      path: plugin.manifestPath || plugin.packageRoot,
      status: plugin.maturity,
      tone: plugin.maturityTone,
    }));
  }

  if (mode === "learn") {
    return state.learnResources.map((resource) => ({
      id: resource.id,
      title: resource.title,
      detail: resource.summary,
      meta: resource.source,
      category: resource.category,
      categoryKey: resource.categoryKey,
      scope: resource.source,
      scopeKey: resource.sourceKey,
      path: resource.path,
      status: resource.category,
      tone: "neutral",
    }));
  }

  return state.assets.map((asset) => ({
    id: asset.id,
    title: asset.name,
    detail: asset.detail,
    meta: asset.size,
    category: asset.kind,
    categoryKey: "local",
    scope: asset.source,
    scopeKey: asset.sourceKey,
    path: asset.path,
    status: asset.source,
    tone: asset.sourceKey === "project" ? "success" : "neutral",
  }));
}

function catalogPanelTitle(mode: "assets" | "plugins" | "learn", text: HubShellState["ui"]["catalog"]) {
  if (mode === "plugins") {
    return text.pluginsCatalogPanelTitle;
  }

  if (mode === "learn") {
    return text.learnCatalogPanelTitle;
  }

  return text.assetsCatalogPanelTitle;
}

function catalogTabs(mode: "assets" | "plugins" | "learn", text: HubShellState["ui"]["catalog"]) {
  if (mode === "learn") {
    return [
      { value: "all", label: text.all },
      { value: "guide", label: text.guides },
      { value: "reference", label: text.reference },
    ];
  }

  return [
    { value: "all", label: text.all },
    { value: "project", label: text.project },
    { value: "engine", label: text.engine },
  ];
}

function filterRows(rows: CatalogRow[], mode: "assets" | "plugins" | "learn", tab: string, query: string) {
  const normalizedQuery = query.trim().toLowerCase();
  return rows.filter((row) => {
    const inTab =
      tab === "all" ||
      (mode === "learn"
        ? row.categoryKey === tab
        : tab === "project"
          ? row.scopeKey === "project"
          : row.scopeKey === "engine");
    const inQuery =
      normalizedQuery.length === 0 ||
      [row.title, row.detail, row.meta, row.category, row.scope, row.path].some((value) => value.toLowerCase().includes(normalizedQuery));
    return inTab && inQuery;
  });
}

function iconForMode(mode: "assets" | "plugins" | "learn") {
  const Icon = pageIcon[mode];
  return <Icon fontSize="small" />;
}

function groupBy<T>(items: T[], key: (item: T) => string): Map<string, T[]> {
  return items.reduce((groups, item) => {
    const groupKey = key(item);
    groups.set(groupKey, [...(groups.get(groupKey) ?? []), item]);
    return groups;
  }, new Map<string, T[]>());
}

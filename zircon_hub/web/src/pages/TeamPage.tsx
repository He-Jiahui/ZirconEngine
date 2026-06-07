import AccountTreeOutlinedIcon from "@mui/icons-material/AccountTreeOutlined";
import GroupsOutlinedIcon from "@mui/icons-material/GroupsOutlined";
import HistoryOutlinedIcon from "@mui/icons-material/HistoryOutlined";
import PersonOutlineOutlinedIcon from "@mui/icons-material/PersonOutlineOutlined";
import { Box, Typography } from "@mui/material";
import { useMemo, useState } from "react";
import { EmptyStateBlock, HubList, HubPanel, HubTreeView, MetricCard, QuickActions, SourceEngineList, StatusBadge } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubTabs } from "../components/inputs";
import { formatCountText } from "../text/counts";
import { quickActionProjectTargetPayload } from "../tauri/projectTarget";
import { hubTokens } from "../theme/tokens";
import type { HubActionHandler, HubActionHistoryItem, HubShellState } from "../types/hub";
import { HUB_ACTION } from "../types/hub";

export interface TeamPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function TeamPage({ state, onAction }: TeamPageProps) {
  const [tab, setTab] = useState("overview");
  const common = state.ui.common;
  const text = state.ui.team;
  const quickActionProjectTarget = quickActionProjectTargetPayload(state.selectedProject);
  const reservedCollaboration = useMemo(
    () => state.comingSoon.filter((entry) => entry.category === "team"),
    [state.comingSoon],
  );
  const memberRows = useMemo(
    () =>
      state.team.members.map((member) => ({
        id: member.id,
        title: member.name || text.unknownContributor,
        detail: member.email || text.noEmailConfigured,
        meta: member.commitsLabel,
        icon: <PersonOutlineOutlinedIcon fontSize="small" />,
      })),
    [state.team.members, text],
  );
  const actionRows = useMemo(
    () =>
      state.actionHistory.map((action) => ({
        id: action.id,
        title: action.action,
        detail: action.detail,
        meta: action.finished,
        icon: <HistoryOutlinedIcon fontSize="small" />,
      })),
    [state.actionHistory],
  );
  const teamTree = useMemo(
    () => [
      {
        id: "repository",
        label: text.repository,
        detail: state.team.repositoryPath,
        children: [
          {
            id: "identity",
            label: text.identity,
            detail: state.team.identityName || common.notConfigured,
            children: [
              { id: "identity-name", label: text.name, detail: state.team.identityName || common.notConfigured },
              { id: "identity-email", label: text.email, detail: state.team.identityEmail || common.notConfigured },
            ],
          },
          {
            id: "contributors",
            label: text.contributors,
            detail: formatCountText(common.memberCountTemplate, state.team.members.length),
            children: state.team.members.map((member) => ({
              id: member.id,
              label: member.name || member.email || text.unknownContributor,
              detail: member.commitsLabel,
            })),
          },
        ],
      },
    ],
    [common, state.team, text],
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
        <StatusBadge label={state.taskSummary.label} tone={state.taskSummary.tone} />
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
        <MetricCard
          label={text.repository}
          value={state.team.repositoryAvailable ? common.connected : common.notConfigured}
          detail={state.team.repositoryPath}
          icon={<AccountTreeOutlinedIcon />}
          tone="accent"
        />
        <MetricCard
          label={text.identity}
          value={state.team.identityName || common.notConfigured}
          detail={state.team.identityEmail || text.noEmailConfigured}
          icon={<PersonOutlineOutlinedIcon />}
        />
        <MetricCard
          label={text.contributors}
          value={`${state.team.members.length}`}
          detail={formatCountText(text.recentActionCountTemplate, state.actionHistory.length)}
          icon={<GroupsOutlinedIcon />}
          tone="success"
        />
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubTabs
          value={tab}
          onChange={setTab}
          options={[
            { value: "overview", label: common.overview },
            { value: "activity", label: common.activity },
            { value: "toolchain", label: common.toolchain },
          ]}
        />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) minmax(330px, 0.55fr)",
          gap: 1.4,
          "@media (max-width: 1180px)": { gridTemplateColumns: "1fr" },
        }}
      >
        {tab === "overview" ? (
          <>
            <HubPanel title={text.teamMembers}>
              {memberRows.length > 0 ? (
                <HubList items={memberRows} />
              ) : (
                <EmptyStateBlock title={text.noTeamMembersFound} detail={text.noTeamMembersFoundDetail} />
              )}
            </HubPanel>
            <HubPanel title={text.repositoryIdentity}>
              <HubList
                items={[
                  { id: "repo", title: text.repository, detail: state.team.repositoryPath },
                  { id: "name", title: text.gitName, detail: state.team.identityName || common.notConfigured },
                  { id: "email", title: text.gitEmail, detail: state.team.identityEmail || common.notConfigured },
                ]}
              />
            </HubPanel>
            <HubPanel title={text.teamTree}>
              <HubTreeView nodes={teamTree} defaultExpanded={["repository", "identity", "contributors"]} />
            </HubPanel>
            <HubPanel title={text.comingSoonPanel}>
              <HubList
                items={reservedCollaboration.map((entry) => ({
                  id: entry.id,
                  title: entry.title,
                  detail: entry.detail,
                  meta: entry.meta,
                  disabled: entry.disabled,
                  icon: <GroupsOutlinedIcon fontSize="small" />,
                }))}
              />
            </HubPanel>
          </>
        ) : null}

        {tab === "activity" ? (
          <>
            <HubPanel title={text.actionHistory}>
              {actionRows.length > 0 ? <HubList items={actionRows} /> : <EmptyStateBlock title={text.noRecentActions} detail={text.noRecentActionsDetail} />}
            </HubPanel>
            <HubPanel title={text.latestAction}>
              {state.actionHistory[0] ? <ActionDetail action={state.actionHistory[0]} /> : <EmptyStateBlock title={text.noActionSelected} detail={text.noActionSelectedDetail} />}
            </HubPanel>
          </>
        ) : null}

        {tab === "toolchain" ? (
          <>
            <HubPanel title={common.sourceEngines}>
              <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
            </HubPanel>
            <HubPanel title={common.quickActions}>
              <QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)} />
            </HubPanel>
          </>
        ) : null}
      </Box>
    </Box>
  );
}

function ActionDetail({ action }: { action: HubActionHistoryItem }) {
  return (
    <Box sx={{ display: "grid", gap: 1.1 }}>
      <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1.2 }}>
        <Typography variant="body2" noWrap sx={{ fontWeight: 700, color: hubTokens.colors.text }}>
          {action.action}
        </Typography>
        <StatusBadge label={action.status} tone={action.tone} />
      </Box>
      <HubList items={action.detailRows} />
    </Box>
  );
}

import AccountTreeOutlinedIcon from "@mui/icons-material/AccountTreeOutlined";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import HealthAndSafetyOutlinedIcon from "@mui/icons-material/HealthAndSafetyOutlined";
import RestartAltOutlinedIcon from "@mui/icons-material/RestartAltOutlined";
import SaveOutlinedIcon from "@mui/icons-material/SaveOutlined";
import SettingsOutlinedIcon from "@mui/icons-material/SettingsOutlined";
import TuneOutlinedIcon from "@mui/icons-material/TuneOutlined";
import UndoOutlinedIcon from "@mui/icons-material/UndoOutlined";
import { Box, Typography } from "@mui/material";
import { useEffect, useMemo, useState } from "react";
import { MetricCard, metricToneFromStatus, SettingsSection } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubTabs } from "../components/inputs";
import { useDebouncedSettingsDraft } from "../settings/debouncedSettingsDraft";
import { settingsJobCountLabel, settingsOptionLabel } from "../settings/options";
import { hubTokens } from "../theme/tokens";
import { HUB_ACTION } from "../types/hub";
import type { HubActionHandler, HubSettingsFolderField, HubSettingsSummary, HubShellState } from "../types/hub";

type SettingsDraft = Pick<
  HubSettingsSummary,
  | "pythonPath"
  | "cargoPath"
  | "rustupPath"
  | "defaultProjectDir"
  | "defaultSourceDir"
  | "defaultBuildOutputDir"
  | "defaultDeviceInstallDir"
  | "buildProfile"
  | "jobs"
  | "language"
>;

export interface SettingsPageProps {
  state: HubShellState;
  onAction: HubActionHandler;
}

export function SettingsPage({ state, onAction }: SettingsPageProps) {
  const [tab, setTab] = useState("overview");
  const [draft, setDraft] = useState<SettingsDraft>(() => settingsDraftFromState(settingsDraftState(state)));
  const draftSettings = settingsDraftState(state);
  const settingsText = draftSettings.text;
  const labels = settingsText.labels;
  const buildProfileLabel = settingsOptionLabel(settingsText.buildProfileOptions, draft.buildProfile);
  const languageLabel = settingsOptionLabel(settingsText.languageOptions, draft.language);
  const draftJobsLabel = settingsJobCountLabel(settingsText, draft.jobs);
  const healthTone = metricToneFromStatus(draftSettings.health.tone);
  const { scheduleDraftPublication, cancelPendingDraft } = useDebouncedSettingsDraft(
    (nextDraft: SettingsDraft) => {
      void onAction(HUB_ACTION.updateSettingsDraft, undefined, { settings: nextDraft });
    },
  );

  useEffect(() => {
    setDraft(settingsDraftFromState(settingsDraftState(state)));
  }, [state.settings, state.settingsDraft]);

  const healthRows = useMemo(
    () => draftSettings.health.rows.map((row) => ({ ...row, disabled: false })),
    [draftSettings.health.rows],
  );
  const pathTree = useMemo(
    () => [
      {
        id: "settings-root",
        label: settingsText.pathDefaultsPanel,
        detail: languageLabel,
        children: [
          { id: "projects", label: labels.defaultProjectDir, detail: draft.defaultProjectDir },
          { id: "source", label: labels.defaultSourceDir, detail: draft.defaultSourceDir },
          { id: "build", label: labels.defaultBuildOutputDir, detail: draft.defaultBuildOutputDir },
          { id: "device", label: labels.defaultDeviceInstallDir, detail: draft.defaultDeviceInstallDir },
        ],
      },
    ],
    [draft, labels, languageLabel, settingsText.pathDefaultsPanel],
  );
  const updateDraft = <Key extends keyof SettingsDraft>(key: Key, value: SettingsDraft[Key]) => {
    const nextDraft = { ...draft, [key]: value };
    setDraft(nextDraft);
    scheduleDraftPublication(nextDraft);
  };
  const saveDraft = () => {
    cancelPendingDraft();
    void onAction(HUB_ACTION.saveSettings, undefined, { settings: draft });
  };
  const browseFolder = (field: HubSettingsFolderField, initialDir: string) => {
    cancelPendingDraft();
    void onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft });
  };
  const discardDraft = () => {
    cancelPendingDraft();
    void onAction(HUB_ACTION.discardSettingsDraft);
  };
  const restoreDefaultSettings = () => {
    cancelPendingDraft();
    void onAction(HUB_ACTION.restoreDefaultSettings);
  };

  return (
    <Box
      sx={{
        height: "100%",
        minHeight: 0,
        overflow: "auto",
        px: `${hubTokens.window.pagePaddingX}px`,
        py: `${hubTokens.window.pagePaddingY}px`,
        "@media (max-width: 980px)": {
          px: 2,
          py: 2,
        },
      }}
    >
      <Box sx={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 2, mb: 2.4 }}>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="h4">{settingsText.heading}</Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mt: 0.9 }}>
            {state.pageSubtitle}
          </Typography>
        </Box>
        <Box sx={{ display: "flex", gap: 1.2, flexWrap: "wrap", justifyContent: "flex-end" }}>
          <HubButton startIcon={<FolderOutlinedIcon />} onClick={() => void onAction(HUB_ACTION.showPage, "projects")}>
            {settingsText.projectsButton}
          </HubButton>
          <HubButton startIcon={<UndoOutlinedIcon />} onClick={discardDraft}>
            {settingsText.discardButton}
          </HubButton>
          <HubButton startIcon={<RestartAltOutlinedIcon />} onClick={restoreDefaultSettings}>
            {settingsText.restoreDefaultsButton}
          </HubButton>
          <HubButton tone="primary" startIcon={<SaveOutlinedIcon />} onClick={saveDraft}>
            {settingsText.saveButton}
          </HubButton>
        </Box>
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubStatusBanner task={state.taskSummary} />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "repeat(4, minmax(0, 1fr))",
          gap: 1.2,
          mb: 1.4,
          "@media (max-width: 1180px)": { gridTemplateColumns: "repeat(2, minmax(0, 1fr))" },
          "@media (max-width: 720px)": { gridTemplateColumns: "1fr" },
        }}
      >
        <MetricCard label={settingsText.sourceEnginesPanel} value={state.engineVersion} detail={draft.defaultSourceDir} icon={<AccountTreeOutlinedIcon />} tone="accent" />
        <MetricCard label={labels.buildProfile} value={buildProfileLabel} detail={draftJobsLabel} icon={<TuneOutlinedIcon />} />
        <MetricCard label={labels.language} value={languageLabel} detail={state.productName} icon={<SettingsOutlinedIcon />} />
        <MetricCard label={settingsText.configurationHealthPanel} value={draftSettings.health.label} detail={draftSettings.health.detail} icon={<HealthAndSafetyOutlinedIcon />} tone={healthTone} />
      </Box>

      <Box sx={{ mb: 1.4 }}>
        <HubTabs value={tab} onChange={setTab} options={settingsText.tabs} />
      </Box>

      <Box
        sx={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr) minmax(330px, 0.42fr)",
          gap: 1.4,
          "@media (max-width: 1180px)": {
            gridTemplateColumns: "1fr",
          },
        }}
      >
        <SettingsSection
          tab={tab}
          draft={draft}
          draftSettings={draftSettings}
          healthRows={healthRows}
          pathTree={pathTree}
          buildProfileLabel={buildProfileLabel}
          languageLabel={languageLabel}
          healthTone={healthTone}
          state={state}
          updateDraft={updateDraft}
          browseFolder={browseFolder}
          onAction={onAction}
        />
      </Box>
    </Box>
  );
}

function settingsDraftState(state: HubShellState): HubSettingsSummary {
  return state.settingsDraft ?? state.settings;
}

function settingsDraftFromState(settings: HubSettingsSummary): SettingsDraft {
  return {
    pythonPath: settings.pythonPath,
    cargoPath: settings.cargoPath,
    rustupPath: settings.rustupPath,
    defaultProjectDir: settings.defaultProjectDir,
    defaultSourceDir: settings.defaultSourceDir,
    defaultBuildOutputDir: settings.defaultBuildOutputDir,
    defaultDeviceInstallDir: settings.defaultDeviceInstallDir,
    buildProfile: settings.buildProfile,
    jobs: settings.jobs,
    language: settings.language,
  };
}

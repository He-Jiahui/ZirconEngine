import AccountTreeOutlinedIcon from "@mui/icons-material/AccountTreeOutlined";
import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import HealthAndSafetyOutlinedIcon from "@mui/icons-material/HealthAndSafetyOutlined";
import SaveOutlinedIcon from "@mui/icons-material/SaveOutlined";
import SettingsOutlinedIcon from "@mui/icons-material/SettingsOutlined";
import TuneOutlinedIcon from "@mui/icons-material/TuneOutlined";
import { Box, LinearProgress, Typography } from "@mui/material";
import { useEffect, useMemo, useState } from "react";
import { HubList, HubPanel, HubTreeView, MetricCard, SourceEngineList, StatusBadge } from "../components/data";
import { HubStatusBanner } from "../components/feedback";
import { HubButton, HubCheckbox, HubComboBox, HubIconButton, HubSwitch, HubTabs, HubTextField } from "../components/inputs";
import { settingsJobCountLabel, settingsOptionLabel } from "../settings/options";
import { hubTokens } from "../theme/tokens";
import { HUB_ACTION } from "../types/hub";
import type { HubActionHandler, HubSettingsFolderField, HubSettingsSummary, HubShellState, StatusTone } from "../types/hub";

type MetricTone = "neutral" | "accent" | "success" | "warning" | "error";

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
    setDraft((current) => ({ ...current, [key]: value }));
  };
  const saveDraft = () => {
    void onAction(HUB_ACTION.saveSettings, undefined, { settings: draft });
  };
  const browseFolder = (field: HubSettingsFolderField, initialDir: string) => {
    void onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft });
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
        <Box sx={{ minWidth: 0, display: "grid", gap: 1.4 }}>
          {tab === "overview" ? (
            <>
              <HubPanel title={settingsText.buildDefaultsPanel}>
                <Box sx={{ display: "grid", gridTemplateColumns: "repeat(2, minmax(0, 1fr))", gap: 1.2, "@media (max-width: 760px)": { gridTemplateColumns: "1fr" } }}>
                  <HubComboBox
                    value={draft.buildProfile}
                    minWidth={0}
                    options={settingsText.buildProfileOptions}
                    onChange={(value) => updateDraft("buildProfile", value)}
                  />
                  <HubTextField
                    label={labels.jobs}
                    value={draft.jobs}
                    type="number"
                    slotProps={{ htmlInput: { min: 1, step: 1 } }}
                    onChange={(event) => updateDraft("jobs", Math.max(1, Number.parseInt(event.target.value, 10) || 1))}
                  />
                  <HubSwitch checked={draft.buildProfile === "release"} label={labels.releaseBuild} detail={buildProfileLabel} onChange={(checked) => updateDraft("buildProfile", checked ? "release" : "debug")} />
                  <HubComboBox
                    value={draft.language}
                    minWidth={0}
                    options={settingsText.languageOptions}
                    onChange={(value) => updateDraft("language", value)}
                  />
                </Box>
              </HubPanel>
              <HubPanel title={settingsText.configurationPathsPanel}>
                <HubList items={healthRows} />
              </HubPanel>
            </>
          ) : null}

          {tab === "toolchain" ? (
            <HubPanel title={settingsText.sourceEnginesPanel}>
              <Box sx={{ display: "grid", gap: 1.2, mb: 1.2 }}>
                <HubTextField label={labels.pythonPath} value={draft.pythonPath} onChange={(event) => updateDraft("pythonPath", event.target.value)} />
                <HubTextField label={labels.cargoPath} value={draft.cargoPath} onChange={(event) => updateDraft("cargoPath", event.target.value)} />
                <HubTextField label={labels.rustupPath} value={draft.rustupPath} onChange={(event) => updateDraft("rustupPath", event.target.value)} />
              </Box>
              <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
            </HubPanel>
          ) : null}

          {tab === "paths" ? (
            <HubPanel title={settingsText.pathDefaultsPanel}>
              <Box sx={{ display: "grid", gap: 1.2 }}>
                <SettingsPathField
                  label={labels.defaultProjectDir}
                  value={draft.defaultProjectDir}
                  browseLabel={state.ui.actions.browseFolder}
                  onChange={(value) => updateDraft("defaultProjectDir", value)}
                  onBrowse={() => browseFolder("defaultProjectDir", draft.defaultProjectDir)}
                />
                <SettingsPathField
                  label={labels.defaultSourceDir}
                  value={draft.defaultSourceDir}
                  browseLabel={state.ui.actions.browseFolder}
                  onChange={(value) => updateDraft("defaultSourceDir", value)}
                  onBrowse={() => browseFolder("defaultSourceDir", draft.defaultSourceDir)}
                />
                <SettingsPathField
                  label={labels.defaultBuildOutputDir}
                  value={draft.defaultBuildOutputDir}
                  browseLabel={state.ui.actions.browseFolder}
                  onChange={(value) => updateDraft("defaultBuildOutputDir", value)}
                  onBrowse={() => browseFolder("defaultBuildOutputDir", draft.defaultBuildOutputDir)}
                />
                <SettingsPathField
                  label={labels.defaultDeviceInstallDir}
                  value={draft.defaultDeviceInstallDir}
                  browseLabel={state.ui.actions.browseFolder}
                  onChange={(value) => updateDraft("defaultDeviceInstallDir", value)}
                  onBrowse={() => browseFolder("defaultDeviceInstallDir", draft.defaultDeviceInstallDir)}
                />
              </Box>
            </HubPanel>
          ) : null}

          {tab === "advanced" ? (
            <HubPanel title={settingsText.advancedConfigurationPanel}>
              <Box sx={{ display: "grid", gap: 1, mb: 1.2 }}>
                <HubCheckbox checked={draft.language === "Chinese"} label={labels.localizedUi} detail={languageLabel} onChange={(checked) => updateDraft("language", checked ? "Chinese" : "English")} />
                <HubCheckbox checked={state.sourceEngines.length > 0} label={settingsText.sourceEnginesPanel} detail={`${state.sourceEngines.length}`} disabled />
              </Box>
              <HubTreeView nodes={pathTree} defaultExpanded={["settings-root"]} />
            </HubPanel>
          ) : null}
        </Box>

        <Box sx={{ minWidth: 0, display: "grid", gap: 1.4, alignContent: "start" }}>
          <HubPanel title={settingsText.configurationHealthPanel} action={<StatusBadge label={draftSettings.health.label} tone={draftSettings.health.tone} />}>
            <Box sx={{ display: "grid", gap: 1.2 }}>
              <Box>
                <Box sx={{ display: "flex", justifyContent: "space-between", mb: 0.7 }}>
                  <Typography variant="body2">{settingsText.completenessLabel}</Typography>
                  <Typography variant="body2" sx={{ color: healthTone === "success" ? hubTokens.colors.success : hubTokens.colors.warning }}>
                    {draftSettings.health.completion}%
                  </Typography>
                </Box>
                <LinearProgress variant="determinate" value={draftSettings.health.completion} />
              </Box>
              <HubList items={healthRows} />
            </Box>
          </HubPanel>
          <HubPanel title={settingsText.activeSourceEnginePanel}>
            <SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)} />
          </HubPanel>
        </Box>
      </Box>
    </Box>
  );
}

interface SettingsPathFieldProps {
  label: string;
  value: string;
  browseLabel: string;
  onChange: (value: string) => void;
  onBrowse: () => void;
}

function SettingsPathField({ label, value, browseLabel, onChange, onBrowse }: SettingsPathFieldProps) {
  return (
    <Box sx={{ display: "grid", gridTemplateColumns: "minmax(0, 1fr) auto", gap: 0.8, alignItems: "center" }}>
      <HubTextField label={label} value={value} onChange={(event) => onChange(event.target.value)} />
      <HubIconButton label={browseLabel} onClick={onBrowse}>
        <FolderOutlinedIcon />
      </HubIconButton>
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

function metricToneFromStatus(tone: StatusTone): MetricTone {
  return tone === "running" ? "neutral" : tone;
}

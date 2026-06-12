import FolderOutlinedIcon from "@mui/icons-material/FolderOutlined";
import { Box, LinearProgress, Typography } from "@mui/material";
import type {
  HubActionHandler,
  HubSettingsFolderField,
  HubSettingsHealthRow,
  HubSettingsSummary,
  HubShellState,
  StatusTone,
} from "../../types/hub";
import { HUB_ACTION } from "../../types/hub";
import { hubTokens } from "../../theme/tokens";
import { HubCheckbox, HubComboBox, HubIconButton, HubSwitch, HubTextField } from "../inputs";
import { HubList } from "./HubList";
import { HubPanel } from "./HubPanel";
import { HubTreeView } from "./HubTreeView";
import { SourceEngineList } from "./SourceEngineList";
import { StatusBadge } from "./StatusBadge";

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

type MetricTone = "neutral" | "accent" | "success" | "warning" | "error";

export interface SettingsSectionProps {
  tab: string;
  draft: SettingsDraft;
  draftSettings: HubSettingsSummary;
  healthRows: Array<HubSettingsHealthRow & { disabled: boolean }>;
  pathTree: Parameters<typeof HubTreeView>[0]["nodes"];
  buildProfileLabel: string;
  languageLabel: string;
  healthTone: MetricTone;
  state: HubShellState;
  updateDraft: <Key extends keyof SettingsDraft>(key: Key, value: SettingsDraft[Key]) => void;
  browseFolder: (field: HubSettingsFolderField, initialDir: string) => void;
  onAction: HubActionHandler;
}

export function SettingsSection({
  tab,
  draft,
  draftSettings,
  healthRows,
  pathTree,
  buildProfileLabel,
  languageLabel,
  healthTone,
  state,
  updateDraft,
  browseFolder,
  onAction,
}: SettingsSectionProps) {
  const settingsText = draftSettings.text;
  const labels = settingsText.labels;

  return (
    <>
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
    </>
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

export function metricToneFromStatus(tone: StatusTone): MetricTone {
  return tone === "running" ? "neutral" : tone;
}

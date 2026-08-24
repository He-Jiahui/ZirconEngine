import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ChevronRightIcon from "@mui/icons-material/ChevronRight";
import RadioButtonUncheckedIcon from "@mui/icons-material/RadioButtonUnchecked";
import SettingsOutlinedIcon from "@mui/icons-material/SettingsOutlined";
import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import { Box, ButtonBase, Divider, Typography } from "@mui/material";
import { useMemo } from "react";
import { selectSourceEngineChoices } from "../../projections/sourceEngineChoices";
import { hubTokens } from "../../theme/tokens";
import type { HubSettingsSummary, HubShellText, HubSourceEngineSummary } from "../../types/hub";
import { StatusBadge } from "../data";
import { HubPopover } from "./HubPopover";

export interface SourceEnginePopoverProps {
  anchorEl: HTMLElement | null;
  open: boolean;
  engines: HubSourceEngineSummary[];
  activeEngineId?: string | null;
  settings: HubSettingsSummary;
  text: HubShellText;
  onClose: () => void;
  onSelect: (engineId: string) => void;
  onManage: () => void;
}

export function SourceEnginePopover({
  anchorEl,
  open,
  engines,
  activeEngineId,
  settings,
  text,
  onClose,
  onSelect,
  onManage,
}: SourceEnginePopoverProps) {
  const { activeEngines, fallbackEngines } = useMemo(
    () => selectSourceEngineChoices(engines, activeEngineId),
    [activeEngineId, engines],
  );

  return (
    <HubPopover anchorEl={anchorEl} open={open} width={388} onClose={onClose}>
      <Typography variant="caption" sx={sectionLabelSx}>
        {text.activeEngine}
      </Typography>
      <Box sx={{ display: "grid", gap: 0.75 }}>
        {activeEngines.map((engine) => (
          <EngineRow key={engine.id} engine={engine} active activeLabel={text.active} onSelect={onSelect} />
        ))}
        {activeEngines.length === 0 ? (
          <Box sx={{ px: 1, py: 1.2, color: hubTokens.colors.textMuted }}>
            <Typography variant="body2">{text.noSourceEngineRegistered}</Typography>
          </Box>
        ) : null}
      </Box>

      <Typography variant="caption" sx={{ ...sectionLabelSx, mt: 1.3 }}>
        {text.readyFallback}
      </Typography>
      <Box sx={{ display: "grid", gap: 0.75 }}>
        {fallbackEngines.map((engine) => (
          <EngineRow key={engine.id} engine={engine} active={false} activeLabel={text.active} onSelect={onSelect} />
        ))}
        {fallbackEngines.length === 0 ? (
          <Box sx={{ px: 1, py: 0.8, color: hubTokens.colors.textMuted }}>
            <Typography variant="caption">{text.noFallbackEngineConfigured}</Typography>
          </Box>
        ) : null}
      </Box>

      <Typography variant="caption" sx={{ ...sectionLabelSx, mt: 1.3 }}>
        {text.localDefaults}
      </Typography>
      <Box sx={{ display: "grid", gap: 0.75 }}>
        <PathRow label={text.source} value={settings.defaultSourceDir} />
        <PathRow label={text.buildOutput} value={settings.defaultBuildOutputDir} />
      </Box>

      <Divider sx={{ my: 1.1, borderColor: hubTokens.colors.line }} />
      <ButtonBase
        onClick={onManage}
        sx={{
          width: "100%",
          minHeight: 38,
          justifyContent: "space-between",
          px: 1,
          borderRadius: `${hubTokens.radius.compact}px`,
          color: hubTokens.colors.textSoft,
          "&:hover": { backgroundColor: "rgba(255,255,255,0.05)" },
        }}
      >
        <Typography variant="body2">{text.manageEngines}</Typography>
        <SettingsOutlinedIcon fontSize="small" />
      </ButtonBase>
    </HubPopover>
  );
}

function EngineRow({
  engine,
  active,
  activeLabel,
  onSelect,
}: {
  engine: HubSourceEngineSummary;
  active: boolean;
  activeLabel: string;
  onSelect: (engineId: string) => void;
}) {
  const Icon = active ? CheckCircleIcon : RadioButtonUncheckedIcon;

  return (
    <ButtonBase
      onClick={() => onSelect(engine.id)}
      sx={{
        width: "100%",
        minHeight: 72,
        display: "grid",
        gridTemplateColumns: "34px minmax(0, 1fr) auto",
        alignItems: "center",
        gap: 1,
        px: 1,
        py: 0.9,
        borderRadius: `${hubTokens.radius.compact}px`,
        border: `1px solid ${active ? "rgba(45,212,207,0.36)" : hubTokens.colors.lineStrong}`,
        backgroundColor: active ? "rgba(18,82,80,0.5)" : "rgba(32,32,32,0.52)",
        color: hubTokens.colors.text,
        textAlign: "left",
        "&:hover": {
          borderColor: "rgba(45,212,207,0.34)",
          backgroundColor: active ? "rgba(18,82,80,0.58)" : "rgba(38,38,38,0.78)",
        },
      }}
    >
      <Box sx={{ display: "grid", placeItems: "center", color: active ? hubTokens.colors.success : hubTokens.colors.textMuted }}>
        <Icon fontSize="small" />
      </Box>
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="body2" noWrap sx={{ fontWeight: 700 }}>
          {engine.name}
        </Typography>
        <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
          {engine.sourcePath}
        </Typography>
        <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
          {engine.outputPath}
        </Typography>
      </Box>
      {active ? <StatusBadge label={activeLabel} tone="success" /> : <ChevronRightIcon sx={{ color: hubTokens.colors.textSoft }} />}
    </ButtonBase>
  );
}

function PathRow({ label, value }: { label: string; value: string }) {
  return (
    <Box
      sx={{
        minHeight: 48,
        display: "grid",
        gridTemplateColumns: "30px minmax(0, 1fr)",
        alignItems: "center",
        gap: 1,
        px: 1,
        py: 0.7,
        borderRadius: `${hubTokens.radius.compact}px`,
        border: `1px solid ${hubTokens.colors.lineStrong}`,
        backgroundColor: "rgba(32,32,32,0.44)",
      }}
    >
      <StorageOutlinedIcon sx={{ color: hubTokens.colors.accent, fontSize: 20 }} />
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="caption" noWrap sx={{ display: "block", color: hubTokens.colors.textMuted }}>
          {label}
        </Typography>
        <Typography variant="body2" noWrap>
          {value}
        </Typography>
      </Box>
    </Box>
  );
}

const sectionLabelSx = {
  display: "block",
  px: 1,
  py: 0.7,
  color: hubTokens.colors.accent,
  fontWeight: 700,
  textTransform: "uppercase",
};

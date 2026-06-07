import StorageOutlinedIcon from "@mui/icons-material/StorageOutlined";
import { Box, ButtonBase, Typography } from "@mui/material";
import { hubTokens } from "../../theme/tokens";
import type { HubSourceEngineSummary } from "../../types/hub";
import { StatusBadge } from "./StatusBadge";

export interface SourceEngineListProps {
  engines: HubSourceEngineSummary[];
  emptyLabel: string;
  onSelect?: (engine: HubSourceEngineSummary) => void;
}

export function SourceEngineList({ engines, emptyLabel, onSelect }: SourceEngineListProps) {
  const hasSelectHandler = Boolean(onSelect);

  if (engines.length === 0) {
    return (
      <Box sx={{ minHeight: 72, display: "grid", alignItems: "center", color: hubTokens.colors.textMuted }}>
        <Typography variant="body2">{emptyLabel}</Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ display: "grid", gap: 0.85 }}>
      {engines.map((engine) => (
        <ButtonBase
          key={engine.id}
          disabled={!hasSelectHandler}
          onClick={() => onSelect?.(engine)}
          sx={{
            minWidth: 0,
            minHeight: 72,
            display: "grid",
            gridTemplateColumns: "40px minmax(0, 1fr) auto",
            alignItems: "center",
            gap: 1.2,
            px: 1.2,
            py: 1,
            borderRadius: `${hubTokens.radius.compact}px`,
            border: `1px solid ${engine.active ? "rgba(45,212,207,0.34)" : hubTokens.colors.lineStrong}`,
            backgroundColor: engine.active ? "rgba(18,82,80,0.38)" : "rgba(32,32,32,0.64)",
            color: hubTokens.colors.text,
            cursor: hasSelectHandler ? "pointer" : "default",
            textAlign: "left",
            "&:hover": {
              borderColor: hasSelectHandler ? "rgba(45,212,207,0.34)" : engine.active ? "rgba(45,212,207,0.34)" : hubTokens.colors.lineStrong,
              backgroundColor: hasSelectHandler ? "rgba(40,40,40,0.84)" : engine.active ? "rgba(18,82,80,0.38)" : "rgba(32,32,32,0.64)",
            },
            "&.Mui-disabled": {
              opacity: 1,
              color: hubTokens.colors.text,
            },
          }}
        >
          <Box
            sx={{
              width: 34,
              height: 34,
              display: "grid",
              placeItems: "center",
              borderRadius: `${hubTokens.radius.compact}px`,
              color: hubTokens.colors.accent,
              backgroundColor: "rgba(17,127,124,0.22)",
            }}
          >
            <StorageOutlinedIcon fontSize="small" />
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
          <StatusBadge label={engine.status} tone={engine.active ? "success" : "neutral"} />
        </ButtonBase>
      ))}
    </Box>
  );
}

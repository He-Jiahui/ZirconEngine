import { Box, Stack, Typography } from "@mui/material";

export type StageState = "done" | "active" | "queued" | "blocked";
export interface StageRailItem { label: string; state: StageState; detail: string; }

const stateColor: Record<StageState, string> = {
  done: "success.main",
  active: "primary.main",
  queued: "warning.main",
  blocked: "error.main",
};

export function StageRail({ stages, ariaLabel = "流程阶段" }: { stages: StageRailItem[]; ariaLabel?: string }) {
  return <Box component="ol" aria-label={ariaLabel} sx={{ display: "grid", gridTemplateColumns: { xs: "1fr", md: `repeat(${Math.max(stages.length, 1)}, minmax(0, 1fr))` }, gap: { xs: 1, md: 0 }, listStyle: "none", p: 0, m: 0 }}>
    {stages.map((stage, index) => <Box component="li" key={`${stage.label}-${index}`} sx={{ position: "relative", minWidth: 0, pr: { md: index === stages.length - 1 ? 0 : 2 }, pb: { xs: 1, md: 0 }, borderBottom: { xs: index === stages.length - 1 ? 0 : 1, md: 0 }, borderColor: "divider" }}>
      {index < stages.length - 1 && <Box sx={{ display: { xs: "none", md: "block" }, position: "absolute", top: 9, left: 18, right: 0, borderTop: 1, borderColor: "divider" }} />}
      <Stack direction="row" spacing={1} sx={{ position: "relative", alignItems: "flex-start" }}>
        <Box sx={{ flex: "0 0 auto", width: 18, height: 18, borderRadius: "50%", bgcolor: stateColor[stage.state], boxShadow: stage.state === "active" ? "0 0 0 4px rgba(33,213,207,.16)" : "none" }} />
        <Stack spacing={0.25} sx={{ minWidth: 0 }}>
          <Typography variant="body2" sx={{ fontWeight: 700 }}>{stage.label}</Typography>
          <Typography variant="caption" color="text.secondary" sx={{ overflowWrap: "anywhere" }}>{stage.detail}</Typography>
        </Stack>
      </Stack>
    </Box>)}
  </Box>;
}


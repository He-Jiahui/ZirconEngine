import { Box, Stack, Typography } from "@mui/material";

export type DashboardTone = "neutral" | "info" | "success" | "warning" | "danger";

const toneColor: Record<DashboardTone, string> = {
  neutral: "divider",
  info: "primary.main",
  success: "success.main",
  warning: "warning.main",
  danger: "error.main",
};

export function DashboardKpi({ label, value, detail, tone = "neutral" }: { label: string; value: string | number; detail: string; tone?: DashboardTone }) {
  return <Box component="article" sx={{ minWidth: 0, p: 1.5, border: 1, borderColor: "divider", borderTop: 3, borderTopColor: toneColor[tone], bgcolor: "background.paper", borderRadius: 1 }}>
    <Stack spacing={0.35}>
      <Typography variant="caption" color="text.secondary" sx={{ textTransform: "uppercase", letterSpacing: "0.06em" }}>{label}</Typography>
      <Typography variant="h4" sx={{ fontVariantNumeric: "tabular-nums", lineHeight: 1.05 }}>{value}</Typography>
      <Typography variant="caption" color="text.secondary" sx={{ overflowWrap: "anywhere" }}>{detail}</Typography>
    </Stack>
  </Box>;
}


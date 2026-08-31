import { Box, Stack, Typography } from "@mui/material";

export function SignalBar({ label, value, total, detail, tone = "primary.main" }: { label: string; value: number; total: number; detail?: string; tone?: string }) {
  const percent = total > 0 ? Math.min(100, Math.round((value / total) * 100)) : 0;
  return <Stack spacing={0.5} sx={{ minWidth: 0 }}>
    <Stack direction="row" spacing={1} sx={{ alignItems: "baseline" }}>
      <Typography variant="body2" sx={{ flex: 1 }}>{label}</Typography>
      <Typography variant="body2" sx={{ fontWeight: 700, fontVariantNumeric: "tabular-nums" }}>{value}/{total}</Typography>
    </Stack>
    <Box sx={{ height: 8, bgcolor: "action.hover", borderRadius: 999, overflow: "hidden" }}><Box sx={{ width: `${percent}%`, height: "100%", bgcolor: tone, transition: "width 220ms ease-out" }} /></Box>
    {detail && <Typography variant="caption" color="text.secondary">{detail}</Typography>}
  </Stack>;
}


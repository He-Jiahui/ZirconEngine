import { Box, Grid, Stack, Typography } from "@mui/material";
import type { ArtifactLifecycleProjection } from "../../api/contracts";

export function artifactLifecycleCounts(
  lifecycle: ArtifactLifecycleProjection,
): ArtifactLifecycleProjection {
  return lifecycle;
}

export function ArtifactLifecycleSummary({ lifecycle }: { lifecycle: ArtifactLifecycleProjection }) {
  const counts = artifactLifecycleCounts(lifecycle);
  const metrics = [
    ["可复用池", counts.reusablePools],
    ["用后即删", counts.ephemeralTargets],
    ["待清理", counts.pendingCleanup],
    ["清理失败", counts.failedCleanup],
  ] as const;
  return <Stack spacing={1.5} sx={{ mb: 2 }}>
    <Grid container spacing={1}>
      {metrics.map(([label, count]) => <Grid key={label} size={{ xs: 6, md: 3 }}>
        <Box aria-label={`${label} ${count}`} sx={{ border: 1, borderColor: "divider", borderRadius: 1, p: 1.5 }}>
          <Typography variant="body2" color="text.secondary">{label}</Typography>
          <Typography variant="h5">{count}</Typography>
        </Box>
      </Grid>)}
    </Grid>
    <Typography variant="caption" color="text.secondary">
      数量来自服务确认当前存在的唯一 Cargo 目录；历史作业与已删除目录不参与统计。
    </Typography>
  </Stack>;
}

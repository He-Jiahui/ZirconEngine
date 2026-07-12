import { Box, Grid, Stack, Typography } from "@mui/material";
import type { CargoJobProjection } from "../../api/contracts";

export interface ArtifactLifecycleCounts {
  reusablePools: number;
  ephemeralJobs: number;
  pendingCleanup: number;
  failedCleanup: number;
}

export function artifactLifecycleCounts(jobs: CargoJobProjection[]): ArtifactLifecycleCounts {
  const reusableKeys = new Set(
    jobs
      .filter((job) =>
        job.cleanup_policy === "retained"
        && job.cleanup_status === "retained"
        && job.compatibility_key
      )
      .map((job) => job.compatibility_key as string),
  );
  return {
    reusablePools: reusableKeys.size,
    ephemeralJobs: jobs.filter((job) => job.cleanup_policy === "delete_on_release").length,
    pendingCleanup: jobs.filter((job) => job.cleanup_status === "pending").length,
    failedCleanup: jobs.filter((job) => job.cleanup_status === "failed").length,
  };
}

export function ArtifactLifecycleSummary({ jobs }: { jobs: CargoJobProjection[] }) {
  const counts = artifactLifecycleCounts(jobs);
  const metrics = [
    ["可复用池", counts.reusablePools],
    ["用后即删", counts.ephemeralJobs],
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
      数量来自当前有界协调器快照，不代表独立磁盘扫描；清理与回收仍由服务受控执行。
    </Typography>
  </Stack>;
}

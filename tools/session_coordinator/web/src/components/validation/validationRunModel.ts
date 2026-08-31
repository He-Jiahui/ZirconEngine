import type { CargoLaneProjection, CargoRunHealthProjection } from "../../api/contracts";

export type ValidationRunProgress = {
  jobId: string;
  sessionId: string;
  lane: CargoLaneProjection["lane_kind"];
  state: CargoLaneProjection["status"];
  elapsed: string;
  stepIndex: number;
  stepCount: number;
  stepLabel: string;
  outputLabel: string;
};

const stepCount = 4;

function elapsedBetween(startedAt: string | null, finishedAt: string | null, createdAt: string, now: Date): string {
  const started = Date.parse(startedAt ?? createdAt);
  const finished = finishedAt ? Date.parse(finishedAt) : now.getTime();
  if (!Number.isFinite(started) || !Number.isFinite(finished) || finished < started) return "时间未知";
  const seconds = Math.floor((finished - started) / 1000);
  return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
}

function stageFor(job: CargoLaneProjection): Pick<ValidationRunProgress, "stepIndex" | "stepLabel"> {
  switch (job.status) {
    case "leased": return { stepIndex: 2, stepLabel: "准备验证副本" };
    case "running": return { stepIndex: 3, stepLabel: "执行验证命令" };
    case "succeeded": return { stepIndex: 4, stepLabel: "验证完成" };
    case "failed": return { stepIndex: 4, stepLabel: "验证失败" };
    case "orphaned": return { stepIndex: 4, stepLabel: "等待进程收束" };
    case "released": return { stepIndex: 4, stepLabel: "验证已释放" };
  }
}

function outputFor(job: CargoLaneProjection, health: CargoRunHealthProjection | undefined): string {
  if (job.status !== "running") return "输出已收束";
  if (!health) return "等待进程观察";
  if (health.outputState === "output_observed") return "已有实时输出";
  if (health.outputState === "awaiting_output") return "命令启动中";
  return "日志暂不可读";
}

export function validationRunProgress(
  jobs: CargoLaneProjection[],
  runHealth: CargoRunHealthProjection[],
  now = new Date(),
): ValidationRunProgress[] {
  const healthByJob = new Map(runHealth.map((item) => [item.jobId, item]));
  return jobs.map((job) => ({
    jobId: job.job_id,
    sessionId: job.session_id,
    lane: job.lane_kind,
    state: job.status,
    elapsed: elapsedBetween(job.started_at, job.finished_at, job.created_at, now),
    ...stageFor(job),
    stepCount,
    outputLabel: outputFor(job, healthByJob.get(job.job_id)),
  }));
}

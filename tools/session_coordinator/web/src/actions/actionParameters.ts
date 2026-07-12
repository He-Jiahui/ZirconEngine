import type { JsonObject, ServiceProjection } from "../api/contracts";

const lifecycleKinds = new Set([
  "service.drain",
  "service.resume",
  "service.stop",
  "service.restart",
  "service.force_stop",
]);

export interface ReviewParametersInput {
  reviewerSessionId: string;
  executorSessionId: string;
  criticalCount: number;
  importantCount: number;
  summary: string;
}

export interface ActionParameterInput {
  sessionId: string;
  template: string;
  jobId: string;
  runId: string;
  milestoneId: string;
  lifecycleTimeoutSeconds: number;
  review: ReviewParametersInput | null;
}

export function isLifecycleAction(kind: string): boolean {
  return lifecycleKinds.has(kind);
}

export function actionMutationBlockReason(service: ServiceProjection): string | null {
  if (service.mode === "read_only" || service.supervision?.state === "read_only") {
    return "服务处于只读模式，所有受控变更已禁用。";
  }
  if (service.supervision?.state === "identity_mismatch") {
    return "服务进程身份不匹配，受控变更已禁用，请先从托盘完成身份恢复。";
  }
  if (service.supervision?.state === "fatal_integrity_error") {
    return "服务检测到致命完整性错误，受控变更已禁用，仅保留诊断读取。";
  }
  return null;
}

export function buildActionParameters(kind: string, input: ActionParameterInput): JsonObject {
  if (isLifecycleAction(kind)) {
    if (!Number.isInteger(input.lifecycleTimeoutSeconds)
      || input.lifecycleTimeoutSeconds < 1
      || input.lifecycleTimeoutSeconds > 300) {
      throw new Error("服务操作等待时间必须为 1–300 秒的整数");
    }
    return { timeoutSeconds: input.lifecycleTimeoutSeconds };
  }

  if (kind === "validation.start") {
    return {
      sessionId: input.sessionId,
      template: input.template,
      runId: input.runId,
      milestoneId: input.milestoneId,
    };
  }
  if (kind === "validation.cancel") {
    return { sessionId: input.sessionId, jobId: input.jobId };
  }
  if (kind === "milestone.commit") {
    return { sessionId: input.sessionId, runId: input.runId, milestoneId: input.milestoneId };
  }
  if (kind === "session.complete") {
    return { sessionId: input.sessionId, runId: input.runId };
  }
  if (kind === "topology.refresh" && input.review) {
    return {
      sessionId: input.review.reviewerSessionId,
      executorSessionId: input.review.executorSessionId,
      runId: input.runId,
      milestoneId: input.milestoneId,
      criticalCount: input.review.criticalCount,
      importantCount: input.review.importantCount,
      summary: input.review.summary,
    };
  }
  return { sessionId: input.sessionId };
}

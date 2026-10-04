import { decimal, record, uuid } from "./protocol";

export type CloudStageResult = {
  stageId: string;
  revision: string;
  manifestDigest: string;
  fileCount: number;
  totalBytes: string;
};

export type CloudApplyResult =
  | { status: "applied"; stageId: string; revision: string; appliedFiles: number; unchangedFiles: number }
  | { status: "conflict"; stageId: string; revision: string; conflictCount: number; paths: string[] };

export type CloudDiscardResult = { status: "discarded"; stageId: string; revision: string };

export function parseCloudStageResult(value: unknown, expectedRevision: string): CloudStageResult {
  const result = record(value);
  exactFields(result, ["status", "stageId", "revision", "manifestDigest", "fileCount", "totalBytes"]);
  decimal(expectedRevision);
  uuid(result.stageId); decimal(result.revision); decimal(result.totalBytes);
  if (result.status !== "staged" || result.revision !== expectedRevision
    || typeof result.manifestDigest !== "string" || !/^[0-9a-f]{64}$/.test(result.manifestDigest)
    || !Number.isSafeInteger(result.fileCount) || Number(result.fileCount) < 0) fail();
  return result as unknown as CloudStageResult;
}

export function parseCloudApplyResult(value: unknown, expectedStageId: string, expectedRevision: string): CloudApplyResult {
  const result = record(value);
  uuid(expectedStageId); decimal(expectedRevision);
  if (result.stageId !== expectedStageId || result.revision !== expectedRevision) fail();
  if (result.status === "applied") {
    exactFields(result, ["status", "stageId", "revision", "appliedFiles", "unchangedFiles"]);
    if (!Number.isSafeInteger(result.appliedFiles) || Number(result.appliedFiles) < 0
      || !Number.isSafeInteger(result.unchangedFiles) || Number(result.unchangedFiles) < 0) fail();
    return result as unknown as CloudApplyResult;
  }
  if (result.status === "conflict") {
    exactFields(result, ["status", "stageId", "revision", "conflictCount", "paths"]);
    if (!Number.isSafeInteger(result.conflictCount) || Number(result.conflictCount) < 1) fail();
    if (!Array.isArray(result.paths) || result.paths.length > 256
      || result.paths.some(path => typeof path !== "string" || !safeRelativePath(path))) fail();
    if (new Set(result.paths).size !== result.paths.length) fail();
    return result as unknown as CloudApplyResult;
  }
  fail();
}

export function parseCloudDiscardResult(value: unknown, expectedRevision: string): CloudDiscardResult {
  const result = record(value);
  exactFields(result, ["status", "stageId", "revision"]);
  decimal(expectedRevision);
  uuid(result.stageId); decimal(result.revision);
  if (result.status !== "discarded" || result.revision !== expectedRevision) fail();
  return result as unknown as CloudDiscardResult;
}

function exactFields(value: Record<string, unknown>, fields: string[]) {
  if (Object.keys(value).length !== fields.length || fields.some(field => !Object.prototype.hasOwnProperty.call(value, field))) fail();
}

function safeRelativePath(value: string) {
  return value.length > 0 && value.length <= 512 && !value.includes("\\") && !value.includes(":")
    && value.split("/").every(part => part.length > 0 && part !== "." && part !== "..");
}

function fail(): never { throw new Error("account_protocol_invalid"); }

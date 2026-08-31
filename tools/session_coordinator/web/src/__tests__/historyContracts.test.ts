import assert from "node:assert/strict";
import test from "node:test";
import type { ValidationHistoryTicket } from "../api/contracts.js";
import { parseFailureHistory, parseValidationHistory } from "../api/validation.js";
import { ticketDuration } from "../components/validation/ValidationHistory.js";

test("validation history accepts bounded ticket timelines", () => {
  const projection = parseValidationHistory({
    tickets: [{
      ticketId: "ticket-a", sessionId: "session-a", planPath: "docs/plans/a.md",
      status: "failed", sourceManifestHash: "a".repeat(64), command: ["cargo", "test"],
      commandTruncated: false, createdAt: "2026-08-24T01:00:00+00:00",
      updatedAt: "2026-08-24T01:01:00+00:00",
      eventsTruncated: false,
      events: [{
        eventId: 1, type: "validation.ticket_status_changed",
        createdAt: "2026-08-24T01:01:00+00:00", fromStatus: "running",
        toStatus: "failed", phase: "test", errorCode: "assertion_failed",
        jobId: null, runId: null, exitCode: 1,
      }],
    }],
    statusCounts: { queued: 0, materializing: 0, running: 0, passed: 0, failed: 1, snapshot_stale: 0 },
    truncated: false,
  });
  assert.equal(projection.tickets[0]?.events[0]?.toStatus, "failed");
});

test("failure history accepts added and fixed lifecycle events", () => {
  const projection = parseFailureHistory({
    chains: [{
      lifecycleKey: "chain-a", summarySlug: "failure-a", status: "fixed", priority: 10,
      originPlan: "docs/plans/a.md", fixingPlan: "docs/plans/b.md",
      artifactPath: "docs/plans/a/fixed-a.md", createdAt: "2026-08-24",
      resolvedAt: "2026-08-25",
      events: [
        { kind: "added", createdAt: "2026-08-24", artifactPath: "docs/plans/a/fixed-a.md" },
        { kind: "fixed", createdAt: "2026-08-25", artifactPath: "docs/plans/a/fixed-a.md" },
      ],
    }],
    statusCounts: { open: 0, fixed: 1 },
    truncated: false,
  });
  assert.deepEqual(projection.chains[0]?.events.map((event) => event.kind), ["added", "fixed"]);
});

test("ticket duration keeps running work live and terminal work fixed", () => {
  const ticket: ValidationHistoryTicket = {
    ticketId: "ticket-a", sessionId: "session-a", planPath: "docs/plans/a.md",
    status: "running", sourceManifestHash: "a".repeat(64), command: ["cargo", "test"],
    commandTruncated: false, createdAt: "2026-08-24T01:00:00Z",
    updatedAt: "2026-08-24T01:01:00Z", events: [], eventsTruncated: false,
  };
  assert.equal(ticketDuration(ticket, new Date("2026-08-24T02:02:05Z")), "1 小时 2 分");
  assert.equal(ticketDuration({ ...ticket, status: "passed" }, new Date("2026-08-25T00:00:00Z")), "1 分 0 秒");
});

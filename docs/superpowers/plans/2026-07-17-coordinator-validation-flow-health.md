# Coordinator Validation Flow Health Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the coordinator board show validation-lane WIP, queue order, queue age, and the next owning Session without presenting a resource wait as global Session admission blocking.

**Architecture:** Reuse the bounded `validation.cargoReservations` projection already delivered by the coordinator. Pure front-end selectors derive a CPU/GPU lane summary from that data and render the same resource-local state in the Overview and Validation pages; no command, permission, scheduler, or global supervision state changes.

**Tech Stack:** React, TypeScript, Material UI, Node test runner, existing control-snapshot contracts.

---

## File map

- `tools/session_coordinator/web/src/pages/OverviewPage.tsx`: derives and renders compact lane-level WIP/queue health.
- `tools/session_coordinator/web/src/pages/ValidationPage.tsx`: renders readable elapsed wait and lane summary beside the exact FIFO rows.
- `tools/session_coordinator/web/src/__tests__/components.test.tsx`: locks the derived ordering, elapsed wording, and non-admission language.
- `docs/tools/session_coordinator/validation-queue.md`: documents the projection and its bounded, resource-local interpretation.

## M1 — Resource-local queue visibility

### Implementation slice

- [x] Add a pure `validationFlowHealth(snapshot, now)` selector in `OverviewPage.tsx`. It groups `cargoReservations` by `laneScope`, uses the existing one-based `queuePosition` to select the next item, counts running/leased and pending rows separately, and derives a non-negative elapsed minute count from `createdAt`. Invalid timestamps render as an unknown age rather than throwing.
- [x] Add a `验证流速 · 仅影响独占资源` Overview panel. For each occupied lane, it displays running count, queue count, next Session, and oldest queued elapsed age. Empty queues explicitly say that no validation lane is waiting; the panel does not infer or display a Session-admission closure.
- [x] Add `reservationAge` and lane-summary helpers in `ValidationPage.tsx`. Each existing FIFO reservation row keeps its lane, position, state, and Session, then appends its bounded elapsed wait. Pending/leased rows show expiry; running rows show process-health wording because their original reservation expiry is not a running-job deadline. The page-level summary distinguishes active lane work from pending rows.
- [x] Update `components.test.tsx` with a snapshot that has one running CPU reservation and two pending reservations. It asserts stable CPU ordering, age text, next Session, and the retained phrase that validation does not block Session registration.
- [x] Update `validation-queue.md` with the exact display contract: `queuePosition` is lane-local, age is informational, and the board never claims ETA or global admission state.

### Testing stage

- [x] Run `npm --prefix tools/session_coordinator/web run check`.
- [x] TypeScript, component, build, and asset validation passed without a correction loop.
- [x] Run `git diff --check --` for the five files in this plan.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|

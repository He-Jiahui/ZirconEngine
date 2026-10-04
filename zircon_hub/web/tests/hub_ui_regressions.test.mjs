import assert from "node:assert/strict";
import test from "node:test";

import { selectVisibleCatalogRow } from "../src/catalog/catalogSelection.ts";
import { canDispatchHubAction, resolveHubBootstrap, HubStateLoadError } from "../src/tauri/hubBootstrap.ts";
import { HubBootstrapRetryBudget, HubStateChronology } from "../src/tauri/hubStateChronology.ts";
import { assertHubShellState } from "../src/tauri/hubStateValidator.ts";

test("rejected backend bootstrap blocks Hub business actions", async () => {
  let businessActionInvocations = 0;
  const outcome = await resolveHubBootstrap(async () => {
    throw new HubStateLoadError("backend-unavailable", new Error("backend offline"));
  });

  if (canDispatchHubAction(outcome)) {
    businessActionInvocations += 1;
  }

  assert.equal(outcome.status, "backend-unavailable");
  assert.equal(businessActionInvocations, 0);
});

test("invalid bootstrap payload is reported as a protocol mismatch", async () => {
  const outcome = await resolveHubBootstrap(async () => {
    throw new HubStateLoadError("protocol-mismatch", new Error("missing project rows"));
  });

  assert.equal(outcome.status, "protocol-mismatch");
  assert.equal(canDispatchHubAction(outcome), false);
});

test("catalog selection never exposes a row hidden by the current filter", () => {
  const rows = [{ id: "visible" }, { id: "hidden" }];

  assert.equal(selectVisibleCatalogRow(rows.slice(0, 1), "hidden")?.id, "visible");
  assert.equal(selectVisibleCatalogRow([], "hidden"), undefined);
});

test("Hub chronology rejects delayed duplicate and retired-epoch states", () => {
  const chronology = new HubStateChronology();
  const bootstrap = { backendEpoch: "epoch-a", stateRevision: "18446744073709551617" };
  const newer = { ...bootstrap, stateRevision: "18446744073709551618" };
  const delayed = { ...bootstrap, stateRevision: "18446744073709551617" };
  const restarted = { backendEpoch: "epoch-b", stateRevision: "1" };

  assert.equal(chronology.acceptBootstrap(bootstrap), bootstrap);
  assert.equal(chronology.accept(newer, "invoke"), newer);
  assert.equal(chronology.accept(delayed, "event"), undefined);
  assert.equal(chronology.accept(restarted, "event"), undefined);
  assert.equal(chronology.acceptBootstrap(restarted), restarted);
  assert.equal(chronology.accept(newer, "event"), undefined);
  assert.equal(chronology.acceptBootstrap(restarted), undefined);
});

test("Hub chronology ignores malformed payloads only when their canonical event envelope is provably stale", () => {
  const chronology = new HubStateChronology();
  const accepted = minimalHubState("epoch-a", "5");
  const malformedOldEvent = { ...accepted, stateRevision: "4", taskSummary: null };

  assert.equal(chronology.acceptBootstrap(accepted), accepted);
  assert.throws(() => assertHubShellState(malformedOldEvent), /taskSummary/);
  assert.equal(chronology.isProvablyStaleEvent(malformedOldEvent), true);
  assert.equal(chronology.isProvablyStaleEvent({ ...malformedOldEvent, stateRevision: "5" }), true);
  assert.equal(chronology.isProvablyStaleEvent({ ...malformedOldEvent, stateRevision: "6" }), false);
  assert.equal(chronology.isProvablyStaleEvent({ ...malformedOldEvent, stateRevision: "04" }), false);
  assert.equal(chronology.isProvablyStaleEvent({ ...malformedOldEvent, backendEpoch: "epoch-b" }), false);
  assert.equal(chronology.isProvablyStaleEvent({ invalid: true }), false);
  assert.equal(chronology.isProvablyStaleEvent(Object.create({ backendEpoch: "epoch-a", stateRevision: "4" })), false);

  chronology.beginBootstrap();
  assert.equal(chronology.isProvablyStaleEvent(malformedOldEvent), false);
  chronology.cancelBootstrap();
  chronology.quarantine();
  assert.equal(chronology.isProvablyStaleEvent(malformedOldEvent), false);
});

test("Hub chronology requires bootstrap before events or invoke responses", () => {
  const chronology = new HubStateChronology();
  const state = { backendEpoch: "epoch-a", stateRevision: "1" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(state, "event"), undefined);
  assert.equal(chronology.accept(state, "invoke"), undefined);
  assert.equal(chronology.acceptBootstrap(state), state);
});

test("Hub chronology reconciles the newest matching event staged during bootstrap", () => {
  const chronology = new HubStateChronology();
  const bootstrap = { backendEpoch: "epoch-a", stateRevision: "1", page: "projects" };
  const staged = { ...bootstrap, stateRevision: "3", page: "editor" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(staged, "event"), undefined);
  assert.equal(chronology.accept({ ...staged, stateRevision: "2" }, "event"), undefined);
  assert.equal(chronology.acceptBootstrap(bootstrap), staged);
  assert.equal(chronology.accept({ ...staged, backendEpoch: "retired", stateRevision: "100" }, "event"), undefined);
});

test("Hub chronology does not publish same-epoch events while bootstrap is in flight", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "1" });
  const event = { backendEpoch: "epoch-a", stateRevision: "2" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(event, "event"), undefined);
  assert.equal(chronology.accept(event, "invoke"), undefined);
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "1" }), event);
});

test("Hub chronology retains a new-epoch event and rejects an old bootstrap response", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });
  const newerEpochEvent = { backendEpoch: "epoch-b", stateRevision: "2", page: "editor" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(newerEpochEvent, "event"), undefined);
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "5" }), undefined);

  chronology.beginBootstrap();
  const reconciled = chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" });
  assert.equal(reconciled, newerEpochEvent);
});

test("Hub chronology retains foreign-epoch evidence across a superseding bootstrap", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "1" });

  chronology.beginBootstrap();
  const newEpochEvent = { backendEpoch: "epoch-b", stateRevision: "9" };
  assert.equal(chronology.accept(newEpochEvent, "event"), undefined);
  chronology.beginBootstrap();

  const currentBootstrap = { backendEpoch: "epoch-a", stateRevision: "2" };
  assert.equal(chronology.acceptBootstrap(currentBootstrap), undefined);
  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" }), newEpochEvent);
});

test("Hub chronology keeps both unknown epochs when A and B are staged before stale A bootstrap", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-old", stateRevision: "1" });
  const stagedA = { backendEpoch: "epoch-a", stateRevision: "2" };
  const stagedB = { backendEpoch: "epoch-b", stateRevision: "1" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(stagedA, "event"), undefined);
  assert.equal(chronology.accept(stagedB, "event"), undefined);
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "1" }), undefined);

  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "0" }), undefined);
  assert.equal(chronology.hasPendingBootstrap(), true);
});

test("Hub chronology accepts a new bootstrap despite a staged event from the retired epoch", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });
  const retiredEvent = { backendEpoch: "epoch-a", stateRevision: "5" };
  const currentEvent = { backendEpoch: "epoch-b", stateRevision: "2" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(retiredEvent, "event"), undefined);
  assert.equal(chronology.accept(currentEvent, "event"), undefined);
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" }), currentEvent);
  assert.equal(chronology.hasPendingBootstrap(), false);
  assert.equal(chronology.accept({ backendEpoch: "epoch-a", stateRevision: "6" }, "event"), undefined);
  assert.equal(chronology.accept({ backendEpoch: "epoch-b", stateRevision: "3" }, "invoke")?.stateRevision, "3");
});

test("Hub chronology retains new-epoch evidence through retry exhaustion and manual recovery", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });
  const newEpochEvent = { backendEpoch: "epoch-b", stateRevision: "2" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(newEpochEvent, "event"), undefined);
  for (const [index, staleRevision] of ["5", "6", "7"].entries()) {
    assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: staleRevision }), undefined);
    if (index < 2) {
      chronology.beginBootstrap();
    }
  }
  chronology.quarantine();
  assert.equal(chronology.hasPendingBootstrap(), true);
  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "8" }), undefined);
  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" }), newEpochEvent);
});

test("Hub chronology retains foreign-epoch evidence when a bootstrap is cancelled or superseded", () => {
  for (const transition of ["cancel", "supersede"]) {
    const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });
    const newEpochEvent = { backendEpoch: "epoch-b", stateRevision: "2" };
    chronology.beginBootstrap();
    assert.equal(chronology.accept(newEpochEvent, "event"), undefined);
    if (transition === "cancel") {
      chronology.cancelBootstrap();
      chronology.beginBootstrap();
    } else {
      chronology.beginBootstrap();
    }
    assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "5" }), undefined);
    chronology.beginBootstrap();
    assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" }), newEpochEvent);
  }
});

test("Hub chronology preserves ambiguous new-epoch evidence across repeated bootstrap responses", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });
  const epochB = { backendEpoch: "epoch-b", stateRevision: "2" };
  const epochC = { backendEpoch: "epoch-c", stateRevision: "1" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(epochB, "event"), undefined);
  assert.equal(chronology.accept(epochC, "event"), undefined);
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" }), undefined);
  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-c", stateRevision: "1" }), undefined);
  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "3" }), undefined);
  assert.equal(chronology.hasPendingBootstrap(), true);
});

test("Hub chronology quarantines retired-epoch events while foreign bootstrap evidence is pending", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });
  const stagedB = { backendEpoch: "epoch-b", stateRevision: "2", page: "editor" };

  chronology.beginBootstrap();
  assert.equal(chronology.accept(stagedB, "event"), undefined);
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "5" }), undefined);

  // The stale A response must not reopen the retired A event lane while B is
  // waiting for its own authoritative bootstrap response.
  assert.equal(chronology.accept({ backendEpoch: "epoch-a", stateRevision: "6" }, "event"), undefined);
  assert.equal(chronology.accept(stagedB, "event"), undefined);

  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" }), stagedB);
});

test("Hub chronology exposes a retry requirement after a stale bootstrap", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });

  chronology.beginBootstrap();
  assert.equal(chronology.accept({ backendEpoch: "epoch-b", stateRevision: "2" }, "event"), undefined);
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "5" }), undefined);
  assert.equal(chronology.hasPendingBootstrap(), true);

  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-b", stateRevision: "1" }).backendEpoch, "epoch-b");
  assert.equal(chronology.hasPendingBootstrap(), false);
});

test("Hub bootstrap retry budget is bounded and resettable", () => {
  const budget = new HubBootstrapRetryBudget();

  assert.equal(budget.tryAcquire(), true);
  assert.equal(budget.tryAcquire(), true);
  assert.equal(budget.tryAcquire(), false);

  budget.reset();
  assert.equal(budget.tryAcquire(), true);
});

test("Hub chronology quarantines events after a protocol fault until bootstrap succeeds", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });
  const delayedEvent = { backendEpoch: "epoch-a", stateRevision: "5" };

  chronology.quarantine();
  assert.equal(chronology.accept(delayedEvent, "event"), undefined);

  chronology.beginBootstrap();
  assert.equal(chronology.accept(delayedEvent, "invoke"), undefined);
  assert.deepEqual(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "4" }), {
    backendEpoch: "epoch-a",
    stateRevision: "4",
  });
  assert.equal(chronology.accept(delayedEvent, "event"), delayedEvent);
});

test("Hub chronology keeps quarantine for a lower-revision recovery bootstrap", () => {
  const chronology = new HubStateChronology({ backendEpoch: "epoch-a", stateRevision: "4" });

  chronology.quarantine();
  chronology.beginBootstrap();
  assert.equal(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "3" }), undefined);
  assert.equal(chronology.accept({ backendEpoch: "epoch-a", stateRevision: "5" }, "event"), undefined);

  chronology.beginBootstrap();
  assert.deepEqual(chronology.acceptBootstrap({ backendEpoch: "epoch-a", stateRevision: "4" }), {
    backendEpoch: "epoch-a",
    stateRevision: "4",
  });
});

test("Hub state parser requires lossless chronology strings", () => {
  const state = minimalHubState("epoch-a", "18446744073709551617");

  assert.equal(assertHubShellState(state), state);
  assert.throws(() => assertHubShellState({ ...state, stateRevision: 18446744073709551617n }));
  assert.throws(() => assertHubShellState({ ...state, stateRevision: "01" }));
  assert.throws(() => assertHubShellState({ ...state, backendEpoch: " " }));
  assert.throws(
    () => assertHubShellState({
      ...state,
      ui: { ...state.ui, shell: { ...state.ui.shell, navItems: null } },
    }),
    /ui\.shell\.navItems/,
  );
});

test("Hub state parser rejects unknown route and project-view discriminants", () => {
  const state = minimalHubState("epoch-a", "1");
  const invalidValues = {
    activePage: "future-page",
    projectFilter: "future-filter",
    projectSort: "future-sort",
    projectViewMode: "future-view",
    projectSubpage: "future-subpage",
  };

  for (const [field, invalidValue] of Object.entries(invalidValues)) {
    assert.throws(
      () => assertHubShellState({ ...state, [field]: invalidValue }),
      new RegExp(field),
    );
  }
});

test("Hub state parser rejects malformed element tones, duplicate ids, and ranges", () => {
  const state = minimalHubState("epoch-a", "1");

  assert.throws(
    () => assertHubShellState({
      ...state,
      taskSummary: { ...state.taskSummary, progressPercent: 101 },
    }),
    /taskSummary\.progressPercent/,
  );
  assert.throws(
    () => assertHubShellState({
      ...state,
      taskStatus: [{ id: "running", label: "Running", tone: "danger" }],
    }),
    /taskStatus\[0\]\.tone/,
  );
  assert.throws(
    () => assertHubShellState({
      ...state,
      projects: [{ id: "same" }, { id: "same" }],
    }),
    /projects.*unique/,
  );
  assert.throws(
    () => assertHubShellState({
      ...state,
      sourceEngines: [{ id: "engine", buildHistory: [{ id: "build" }, { id: "build" }] }],
    }),
    /sourceEngines\[0\]\.buildHistory.*unique/,
  );
});

function minimalHubState(backendEpoch, stateRevision) {
  return {
    backendEpoch,
    stateRevision,
    productName: "Zircon Hub",
    engineVersion: "test",
    activePage: "projects",
    pageTitle: "Projects",
    pageSubtitle: "",
    projectFilter: "all",
    projectSort: "last-modified",
    projectViewMode: "grid",
    projectSubpage: "dashboard",
    projectTemplates: [],
    taskStatus: [],
    projects: [],
    browserProjects: [],
    recentProjects: [],
    quickActions: [],
    sourceEngines: [],
    assets: [],
    plugins: [],
    learnResources: [],
    actionHistory: [],
    comingSoon: [],
    selectedProjectId: null,
    activeSourceEngineId: null,
    selectedProject: null,
    settingsDraft: null,
    taskSummary: {
      label: "Ready",
      detail: "",
      tone: "neutral",
      running: false,
      cancellable: false,
      recovery: null,
      operation: "Hub",
      progressPercent: 0,
      taskId: 0,
      queued: 0,
    },
    team: {},
    settings: {},
    ui: { shell: { demoModeBadge: "Demo", navItems: [] }, common: {} },
  };
}

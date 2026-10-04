import assert from "node:assert/strict";
import test from "node:test";

import {
  admittedSourceEngineId,
  selectSourceEngineChoices,
} from "../src/projections/sourceEngineChoices.ts";

const engines = [
  { id: "engine-a", active: false },
  { id: "engine-b", active: true },
  { id: "engine-c", active: false },
];

test("source-engine projection never substitutes a first or flagged engine for a missing active id", () => {
  assert.equal(admittedSourceEngineId(engines, "engine-b"), "engine-b");
  assert.equal(admittedSourceEngineId(engines, "missing-engine"), undefined);
  assert.equal(admittedSourceEngineId(engines, null), "engine-b");

  const missing = selectSourceEngineChoices(engines, "missing-engine");
  assert.deepEqual(missing.activeEngines, []);
  assert.deepEqual(missing.fallbackEngines.map((engine) => engine.id), ["engine-a", "engine-b"]);

  const unspecified = selectSourceEngineChoices(engines, null);
  assert.deepEqual(unspecified.activeEngines.map((engine) => engine.id), ["engine-b"]);
  assert.deepEqual(unspecified.fallbackEngines.map((engine) => engine.id), ["engine-a", "engine-c"]);

  const noFlag = engines.map((engine) => ({ ...engine, active: false }));
  assert.equal(admittedSourceEngineId(noFlag, null), undefined);
  assert.deepEqual(selectSourceEngineChoices(noFlag, null).activeEngines, []);
});

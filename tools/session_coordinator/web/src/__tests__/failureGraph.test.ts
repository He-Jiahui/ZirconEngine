import test from "node:test";
import assert from "node:assert/strict";
import { failureClass } from "../components/failure/failureModel";
test("failure classification keeps foreign diagnostics non-applicable", () => assert.equal(failureClass({ status: "open", applicable: false }), "foreign"));
test("failure classification shows fixed nodes", () => assert.equal(failureClass({ status: "fixed" }), "fixed"));

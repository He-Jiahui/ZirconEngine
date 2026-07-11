import test from "node:test";
import assert from "node:assert/strict";
import { routeForPath, routes } from "../navigation";

test("control navigation has ten stable routes", () => assert.equal(routes.length, 10));
test("control navigation maps nested workflow URLs", () => assert.equal(routeForPath("/ui/workflows/run-a"), "workflows"));
test("control navigation falls back safely", () => assert.equal(routeForPath("/ui/unknown"), "overview"));

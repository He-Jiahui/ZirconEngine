import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("shared HubSearchField publishes a stable accessible name independent of placeholder styling", async () => {
  const source = await readFile(new URL("../src/components/inputs/HubSearchField.tsx", import.meta.url), "utf8");

  assert.match(source, /ariaLabel\?: string/);
  assert.match(source, /ariaLabel = placeholder/);
  assert.match(source, /htmlInput: \{ "aria-label": ariaLabel \}/);
});

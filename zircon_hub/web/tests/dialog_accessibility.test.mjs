import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("shared HubDialog explicitly associates its title and content description", async () => {
  const source = await readFile(new URL("../src/components/overlays/HubDialog.tsx", import.meta.url), "utf8");

  assert.match(source, /useId/);
  assert.match(source, /aria-labelledby=\{titleId\}/);
  assert.match(source, /aria-describedby=\{descriptionId\}/);
  assert.match(source, /<DialogTitle id=\{titleId\}>/);
  assert.match(source, /<DialogContent id=\{descriptionId\}>/);
});

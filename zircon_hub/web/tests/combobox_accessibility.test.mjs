import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const root = new URL("../src/", import.meta.url);

test("shared HubComboBox exposes an explicit accessible label while preserving its placeholder", async () => {
  const source = await readFile(new URL("components/inputs/HubComboBox.tsx", root), "utf8");

  assert.match(source, /label\?: string/);
  assert.match(source, /options, label, placeholder, minWidth/);
  assert.match(source, /<TextField \{\.\.\.params\} label=\{label\} placeholder=\{placeholder\} \/>/);
});

test("settings and new-project combo boxes pass localized labels to the shared input", async () => {
  const [settings, createProject] = await Promise.all([
    readFile(new URL("components/data/SettingsSection.tsx", root), "utf8"),
    readFile(new URL("components/overlays/CreateProjectDialog.tsx", root), "utf8"),
  ]);

  assert.match(settings, /<HubComboBox[\s\S]*?label=\{labels\.buildProfile\}[\s\S]*?options=\{settingsText\.buildProfileOptions\}/);
  assert.match(settings, /<HubComboBox[\s\S]*?label=\{labels\.language\}[\s\S]*?options=\{settingsText\.languageOptions\}/);
  assert.match(createProject, /<HubComboBox[\s\S]*?label=\{text\.sourceEngine\}[\s\S]*?placeholder=\{text\.sourceEngine\}/);
  assert.match(createProject, /<HubComboBox[\s\S]*?label=\{text\.template\}[\s\S]*?placeholder=\{text\.template\}/);
});

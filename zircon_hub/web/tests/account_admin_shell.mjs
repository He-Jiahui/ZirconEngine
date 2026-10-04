import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const nativeRoot = new URL("../../src/", import.meta.url);
const literal = '"(?:[^"\\\\]|\\\\.)*"';
const camelCase = value => value.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
const quoted = value => JSON.parse(value);

// These fixtures read the native text literals, not a second translation catalog.
// Accept only the current literal-only forms and fail if the native schema changes.
export async function accountAdminShell(language) {
  assert.ok(["English", "Chinese"].includes(language));
  const [uiSource, localizedSource, messageSource, reservedSource] = await Promise.all([
    "tauri_app/view_model/ui_text.rs", "tauri_app/view_model/localized.rs",
    "state/hub_message/shell.rs", "tauri_app/view_model/coming_soon.rs",
  ].map(file => readFile(new URL(file, nativeRoot), "utf8")));
  const select = (english, chinese) => quoted(language === "English" ? english : chinese);
  const pair = `text\\s*\\.pair\\(\\s*(${literal})\\s*,\\s*(${literal})\\s*,?\\s*\\)`;
  const ui = {};
  for (const section of ["shell", "actions", "projects", "common", "editor", "builds", "catalog", "cloud", "team"]) {
    const type = section === "actions" ? "HubActionText" : `Hub${section[0].toUpperCase()}${section.slice(1)}Text`;
    const fields = uiSource.match(new RegExp(`struct ${type} \\{([^}]+)\\}`))?.[1];
    assert.ok(fields, `missing native ${type} schema`);
    const expected = [...fields.matchAll(/pub (\w+):/g)].map(match => camelCase(match[1])).sort();
    const body = uiSource.match(new RegExp(`\\b${type} \\{\\s*\\w+:([\\s\\S]*?)\\n\\s*\\}`))?.[0];
    assert.ok(body, `missing native ${type} literals`);
    ui[section] = Object.fromEntries([...body.matchAll(new RegExp(`(\\w+):\\s*${pair}\\s*\\.to_string\\(\\)`, "g"))]
      .map(([, key, english, chinese]) => [camelCase(key), select(english, chinese)]));
    if (section === "shell") {
      const navigation = uiSource.slice(uiSource.indexOf("fn nav_items("), uiSource.indexOf("#[cfg(test)]"));
      ui.shell.navItems = [...navigation.matchAll(new RegExp(`\\((${literal}),\\s*(${literal}),\\s*(${literal})\\)`, "g"))]
        .map(([, id, english, chinese]) => ({ id: quoted(id), label: select(english, chinese) }));
      assert.equal(ui.shell.navItems.length, 9);
    }
    assert.deepEqual(Object.keys(ui[section]).sort(), expected, `unprojected native ${type} fields`);
  }
  const subtitleSource = localizedSource.split("fn page_subtitle(")[1]?.split("fn status_label(")[0];
  const languageSource = subtitleSource?.split(`HubLanguage::${language} => match page {`)[1];
  const subtitle = languageSource?.match(new RegExp(`HubPage::Team => (${literal})`))?.[1];
  assert.ok(subtitle, "missing native Team subtitle");
  const ready = messageSource.match(new RegExp(`\\(HubLanguage::${language}, Self::HubReady\\) => (${literal})`))?.[1];
  assert.ok(ready, "missing native HubReady message");
  const status = reservedSource.match(new RegExp(`let status = ${pair}`));
  const separator = reservedSource.match(new RegExp(`category_label, ${pair}`));
  const categorySource = reservedSource.split("fn coming_soon_category_label(")[1]?.split("#[cfg(test)]")[0];
  assert.ok(status && separator && categorySource, "missing native reserved-entry literals");
  const categories = Object.fromEntries([...categorySource.matchAll(new RegExp(`(${literal}) => ${pair}`, "g"))]
    .map(([, id, english, chinese]) => [quoted(id), select(english, chinese)]));
  const comingSoon = [...reservedSource.matchAll(new RegExp(`\\(\\s*(${literal}),\\s*(${literal}),\\s*${pair},\\s*${pair},?\\s*\\)`, "g"))]
    .map(([, id, category, titleEnglish, titleChinese, detailEnglish, detailChinese]) => {
      const categoryLabel = categories[quoted(category)];
      assert.ok(categoryLabel, `missing native category ${category}`);
      const statusLabel = select(status[1], status[2]);
      return {
        id: quoted(id), category: quoted(category), categoryLabel,
        title: select(titleEnglish, titleChinese), detail: select(detailEnglish, detailChinese),
        status: statusLabel, meta: `${categoryLabel}${select(separator[1], separator[2])}${statusLabel}`, disabled: true,
      };
    });
  assert.equal(comingSoon.length, 15);
  return {
    ui, comingSoon, pageTitle: ui.shell.navItems.find(item => item.id === "team").label,
    pageSubtitle: quoted(subtitle), readyDetail: quoted(ready),
  };
}

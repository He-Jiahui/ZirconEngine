import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import test from "node:test";

const require = createRequire(import.meta.url);
const playwrightRoot = process.env.ZIRCON_PLAYWRIGHT_NODE_MODULES;
const { chromium } = playwrightRoot ? require(path.join(playwrightRoot, "playwright")) : { chromium: undefined };
const executablePath = process.env.ZIRCON_PLAYWRIGHT_EXECUTABLE_PATH || (chromium ? chromium.executablePath() : undefined);
const browserAvailable = Boolean(chromium && executablePath && existsSync(executablePath));
const launchOptions = executablePath ? { executablePath } : {};
const baseUrl = process.env.ZIRCON_HUB_TEST_URL;

test("workflow controls stay disabled until an explicit project is selected", { skip: !baseUrl || !browserAvailable }, async () => {
  const browser = await chromium.launch({ headless: true, ...launchOptions });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    await page.goto(baseUrl);

    const navigation = page.getByTestId("hub-navigation-drawer");
    await navigation.getByRole("button", { name: "构建", exact: true }).click();
    const header = page.getByTestId("hub-page-header");
    for (const label of ["构建", "打包", "安装"]) {
      assert.equal(await header.getByRole("button", { name: label, exact: true }).isDisabled(), true);
    }

    await navigation.getByRole("button", { name: "本地交付", exact: true }).click();
    const cloudHeader = page.getByTestId("hub-page-header");
    for (const label of ["打包项目", "安装到设备"]) {
      assert.equal(await cloudHeader.getByRole("button", { name: label, exact: true }).isDisabled(), true);
    }
  } finally {
    await browser.close();
  }
});

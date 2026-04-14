import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { extname, join, resolve } from "node:path";

const host = "127.0.0.1";
const port = 5173;
const rootDir = resolve(fileURLToPath(new URL(".", import.meta.url)));
const fixtureDir = resolve(rootDir, "../../zircon_editor/fixtures/workbench");
const iconDir = resolve(rootDir, "../../zircon_editor/assets/icons/ionicons");

const mime = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
};

createServer(async (req, res) => {
  try {
    const url = new URL(req.url ?? "/", `http://${host}:${port}`);
    let filePath;
    if (url.pathname === "/") {
      filePath = join(rootDir, "index.html");
    } else if (url.pathname.startsWith("/fixtures/")) {
      filePath = join(fixtureDir, url.pathname.replace("/fixtures/", ""));
    } else if (url.pathname.startsWith("/assets/icons/")) {
      filePath = join(iconDir, url.pathname.replace("/assets/icons/", ""));
    } else {
      filePath = join(rootDir, url.pathname.slice(1));
    }

    const body = await readFile(filePath);
    res.writeHead(200, {
      "Content-Type": mime[extname(filePath)] ?? "application/octet-stream",
      "Cache-Control": "no-cache",
    });
    res.end(body);
  } catch (error) {
    res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    res.end(`Not found: ${error}`);
  }
}).listen(port, host, () => {
  console.log(`Zircon editor workbench preview: http://${host}:${port}/`);
});

import { readFileSync, readdirSync, statSync } from "node:fs";
import { extname, relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const dist = resolve(root, "dist");
const indexPath = resolve(dist, "index.html");
const index = readFileSync(indexPath, "utf8");
if (!/<base\s+href=["']\/ui\/["']\s*\/?>/i.test(index)) fail("control console base path must be /ui/");
const files = walk(dist).filter((path) => path !== indexPath);
const sourceFiles = walk(resolve(root, "src")).filter((path) => /\.[cm]?[jt]sx?$/.test(path));
const forbidden = [
  /https?:\/\/(?:localhost|127\.0\.0\.1):\d+/i,
  /ZIRCON_COORDINATOR_(?:TOKEN|MAINTENANCE)/i,
  /maintenance[_-]?capability/i,
  /we(?:com|chat)[_-]?(?:webhook|key)/i,
];

if (files.some((path) => extname(path) === ".map")) fail("production source map found");
for (const path of files) {
  const name = relative(dist, path).replaceAll("\\", "/");
  if (!/[.-][0-9A-Za-z_-]{8,}\.[^.]+$/.test(name)) fail(`asset is not content-hashed: ${name}`);
  if (!index.includes(name)) fail(`asset is not referenced by index.html: ${name}`);
  const text = readFileSync(path, "utf8");
  for (const pattern of forbidden) if (pattern.test(text)) fail(`forbidden runtime material in ${name}`);
}
for (const pattern of forbidden) if (pattern.test(index)) fail("forbidden runtime material in index.html");
for (const path of sourceFiles) {
  const text = readFileSync(path, "utf8");
  const hubImports = [...text.matchAll(/from\s+["']([^"']*zircon_hub\/web\/src\/[^"']+)["']/g)].map((match) => match[1].replaceAll("\\", "/"));
  for (const imported of hubImports) {
    if (!/(?:theme\/(?:tokens|muiTheme)|components\/(?:data\/HubPanel|inputs\/HubButton))$/.test(imported))
      fail(`forbidden Hub runtime import: ${imported}`);
  }
  if (/zircon_hub\/web\/src\/(?:tauri|types|data|pages|settings)\//.test(text)) fail(`forbidden Hub coupling in ${relative(root, path)}`);
}
if (!files.length) fail("no production assets were emitted");
console.log(`verified ${files.length} hashed control-console assets`);

function walk(directory) {
  return readdirSync(directory).flatMap((name) => {
    const path = resolve(directory, name);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

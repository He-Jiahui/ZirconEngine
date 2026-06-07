import { createReadStream, existsSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize, resolve, sep } from "node:path";

const root = resolve(process.cwd());
const host = process.env.HOST || "127.0.0.1";
const preferredPort = Number.parseInt(process.env.PORT || "5173", 10);
const maxPortOffset = 20;

const mimeTypes = new Map([
  [".css", "text/css; charset=utf-8"],
  [".gif", "image/gif"],
  [".html", "text/html; charset=utf-8"],
  [".ico", "image/x-icon"],
  [".jpeg", "image/jpeg"],
  [".jpg", "image/jpeg"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".mjs", "text/javascript; charset=utf-8"],
  [".png", "image/png"],
  [".svg", "image/svg+xml; charset=utf-8"],
  [".webp", "image/webp"]
]);

function requestFilePath(requestUrl = "/") {
  let pathname;
  try {
    pathname = decodeURIComponent(new URL(requestUrl, `http://${host}`).pathname);
  } catch {
    return { status: 400, path: null };
  }

  const safePath = normalize(pathname).replace(/^(\.\.(\/|\\|$))+/, "");
  const candidate = resolve(root, `.${safePath}`);
  if (candidate !== root && !candidate.startsWith(`${root}${sep}`)) {
    return { status: 403, path: null };
  }

  if (existsSync(candidate) && statSync(candidate).isDirectory()) {
    return { status: 200, path: join(candidate, "index.html") };
  }
  return { status: 200, path: candidate };
}

function handleRequest(request, response) {
  const result = requestFilePath(request.url);
  if (result.status !== 200) {
    response.writeHead(result.status);
    response.end(result.status === 400 ? "Bad request" : "Forbidden");
    return;
  }

  if (!result.path || !existsSync(result.path)) {
    response.writeHead(404);
    response.end("Not found");
    return;
  }

  response.writeHead(200, {
    "Content-Type": mimeTypes.get(extname(result.path)) || "application/octet-stream"
  });
  createReadStream(result.path).pipe(response);
}

function listen(port, remainingRetries = maxPortOffset) {
  const server = createServer(handleRequest);
  server.on("error", (error) => {
    if (error.code === "EADDRINUSE" && remainingRetries > 0) {
      listen(port + 1, remainingRetries - 1);
      return;
    }
    console.error(error.message);
    process.exit(1);
  });
  server.listen(port, host, () => {
    console.log(`Workbench component prototype: http://${host}:${port}/`);
  });
}

listen(preferredPort);

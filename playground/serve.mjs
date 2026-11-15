// Zero-dependency static server for the playground.
//
// Usage:
//   node playground/serve.mjs            # http://127.0.0.1:5173
//   PORT=8080 node playground/serve.mjs
//
// Serving over HTTP (rather than opening the page from disk) matters: browsers
// refuse `WebAssembly.instantiateStreaming` for `file://` URLs, and the module
// must be delivered with `application/wasm`.

import { createReadStream, statSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join, normalize, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL(".", import.meta.url)));
const host = process.env.HOST ?? "127.0.0.1";
// Some environments export PORT=0 or an empty PORT; only an explicit positive
// value overrides the playground default.
const requestedPort = Number(process.env.PORT);
const port =
  Number.isInteger(requestedPort) && requestedPort > 0 ? requestedPort : 5173;

const MIME_TYPES = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
  ".rupa": "text/plain; charset=utf-8",
};

/**
 * Resolve a request path inside the served root.
 *
 * Returns the file to send with its size, or null when the path escapes the
 * root or does not exist.
 */
function resolveRequestPath(url) {
  const pathname = decodeURIComponent(new URL(url, "http://localhost").pathname);
  const requested = normalize(join(root, pathname));
  if (requested !== root && !requested.startsWith(root + sep)) return null;
  try {
    const info = statSync(requested);
    if (!info.isDirectory()) return { path: requested, size: info.size };
    const index = join(requested, "index.html");
    return { path: index, size: statSync(index).size };
  } catch {
    return null;
  }
}

const server = createServer((request, response) => {
  const target = resolveRequestPath(request.url ?? "/");
  if (!target) {
    response.writeHead(404, { "content-type": "text/plain; charset=utf-8" });
    response.end("Not found\n");
    return;
  }

  const type =
    MIME_TYPES[extname(target.path).toLowerCase()] ?? "application/octet-stream";
  response.writeHead(200, {
    "content-type": type,
    "content-length": target.size,
    "cache-control": "no-store",
    // Harmless for this page, and already in place if COEP is ever enabled.
    "cross-origin-opener-policy": "same-origin",
  });
  if (request.method === "HEAD") {
    response.end();
    return;
  }
  createReadStream(target.path).pipe(response);
});

server.listen(port, host, () => {
  const bound = server.address();
  const actualPort = typeof bound === "object" && bound ? bound.port : port;
  console.log(`Rupa playground: http://${host}:${actualPort}/`);
  console.log(`serving ${root}`);
});

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => {
    server.close(() => process.exit(0));
  });
}

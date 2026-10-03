// velquview.rizeva.my.id — static file server for the VelquView landing.
// Serves VELQU_LANDING_DIST (absolute path, pinned by the systemd unit)
// on 127.0.0.1:$VELQU_LANDING_PORT. No deps: plain node:http.
const http = require("http");
const zlib = require("zlib");
const fs = require("fs");
const path = require("path");

const DIST = process.env.VELQU_LANDING_DIST;
const PORT = Number(process.env.VELQU_LANDING_PORT || 8120);
if (!DIST) { console.error("VELQU_LANDING_DIST is required"); process.exit(1); }

const TYPES = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
  ".json": "application/json",
  ".txt": "text/plain; charset=utf-8",
  ".xml": "application/xml; charset=utf-8",
  ".webp": "image/webp",
  ".woff2": "font/woff2",
  ".wasm": "application/wasm",
};

// Compressible types get on-the-fly gzip (the wasm engine is ~3 MB
// raw, ~0.8 MB gzipped); only when the client advertises support.
const GZIP = new Set([".wasm", ".js", ".css", ".svg", ".html"]);

const server = http.createServer((req, res) => {
  const url = new URL(req.url, "http://localhost");
  let rel = url.pathname === "/" ? "/index.html" : url.pathname;
  const file = path.join(DIST, path.normalize(rel));
  // path traversal guard: must stay inside DIST
  if (!file.startsWith(path.resolve(DIST) + path.sep)) {
    res.writeHead(403).end("forbidden");
    return;
  }
  const headers = {
    "x-content-type-options": "nosniff",
    "x-frame-options": "DENY",
    "referrer-policy": "strict-origin-when-cross-origin",
  };
  fs.readFile(file, (err, buf) => {
    if (err) {
      // Branded 404 for pages, plain for missing assets
      const notFound = path.join(DIST, "404.html");
      fs.readFile(notFound, (e2, page) => {
        if (e2) { res.writeHead(404, headers).end("not found"); return; }
        res.writeHead(404, { ...headers, "content-type": "text/html; charset=utf-8" }).end(page);
      });
      return;
    }
    const ext = path.extname(file).toLowerCase();
    const common = {
      ...headers,
      "content-type": TYPES[ext] || "application/octet-stream",
      "cache-control": ext === ".html" ? "no-cache" : "public, max-age=3600",
    };
    if (GZIP.has(ext) && /gzip/.test(req.headers["accept-encoding"] || "")) {
      zlib.gzip(buf, { level: 9 }, (gzErr, gz) => {
        if (gzErr) { res.writeHead(200, common).end(buf); return; }
        res.writeHead(200, { ...common, "content-encoding": "gzip", vary: "Accept-Encoding" }).end(gz);
      });
      return;
    }
    res.end(buf);
  });
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`velqu-landing serving ${DIST} on 127.0.0.1:${PORT}`);
});

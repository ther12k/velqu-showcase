// velquview.rizeva.my.id — static file server for the VelquView landing.
// Serves VELQU_LANDING_DIST (absolute path, pinned by the systemd unit)
// on 127.0.0.1:$VELQU_LANDING_PORT. No deps: plain node:http.
const http = require("http");
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
};

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
    res.writeHead(200, {
      ...headers,
      "content-type": TYPES[path.extname(file).toLowerCase()] || "application/octet-stream",
      "cache-control": path.extname(file) === ".html" ? "no-cache" : "public, max-age=3600",
    });
    res.end(buf);
  });
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`velqu-landing serving ${DIST} on 127.0.0.1:${PORT}`);
});

// A/B browser reference capture for the sample sets (login/ab/chat/...).
// Mirrors the velqu side exactly: same index.html, same app.css, real
// Tailwind v3.4.17 utilities compiled from the page's classes plus the
// documented parity base (box-sizing/border + the renderer's font face),
// captured at deviceScaleFactor 2 to match `--scale 2.0`.
//
//   node tools/ab-shot.mjs <set> <logical-height>   # e.g. ab-shot.mjs login 800
//   → out/ab-<set>-browser.png (2560 × 2×height)
import { chromium } from 'playwright';
import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, cpSync, rmSync } from 'node:fs';
import { mkdirSync as _m } from 'node:fs';

const root = new URL('..', import.meta.url).pathname; // crate root
const set = process.argv[2] ?? 'login';
const height = parseInt(process.argv[3] ?? '800', 10);
const src = `${root}${set}`;
const tmp = `/tmp/abset-${set}`;
const viewport = { width: 1280, height };

rmSync(tmp, { recursive: true, force: true });
mkdirSync(tmp, { recursive: true });
cpSync(`${src}/index.html`, `${tmp}/index.html`);
cpSync(`${src}/app.css`, `${tmp}/app.css`);

// real Tailwind utilities from this set's exact classes + parity base
execSync(
  `printf '@tailwind utilities;\\n' > /tmp/ab-in.css && ` +
    `npx -y tailwindcss@3.4.17 -i /tmp/ab-in.css -o /tmp/ab-core.css --content ${tmp}/index.html --minify`,
  { stdio: 'pipe' },
);
writeFileSync(
  `${tmp}/tw.css`,
  readFileSync('/tmp/ab-core.css') +
`
/* parity base (same as tools/build-browser-css.sh) */
*,::before,::after{box-sizing:border-box;border-width:0;border-style:solid;border-color:#e5e7eb}
body{font-family:'DejaVu Sans',sans-serif}
`,
);

// inject both stylesheets into the copy (velqu auto-loads *.css; the
// browser copy links them explicitly — app.css last, as an author sheet)
let html = readFileSync(`${tmp}/index.html`, 'utf8');
html = html.replace(
  '</head>',
  '  <link rel="stylesheet" href="tw.css">\n  <link rel="stylesheet" href="app.css">\n</head>',
);
writeFileSync(`${tmp}/index.html`, html);

const browser = await chromium.launch();
const page = await browser.newPage({ viewport, deviceScaleFactor: 2 });
await page.goto(`file://${tmp}/index.html`);
await page.evaluate(() => document.fonts.ready);
await page.waitForTimeout(150);
mkdirSync(`${root}out`, { recursive: true });
await page.screenshot({ path: `${root}out/ab-${set}-browser.png` });
console.log(`out/ab-${set}-browser.png written (viewport 1280x${height} @2x)`);
await browser.close();

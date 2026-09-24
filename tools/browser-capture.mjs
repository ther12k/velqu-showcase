// Browser reference capture.
//
// Geometry facts are collected at the SAME fixed logical viewport as the
// velqu side (1280x4096, deviceScaleFactor 1) — full-page screenshots are
// capture mechanics and never resize the layout inputs. Waits for font
// loading to finish before measuring. Records the pinned environment
// (browser build, UA, CSS/HTML hashes, viewport, scale) into the facts file
// so comparisons are attributable to exact inputs.
import { chromium } from 'playwright';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';

const root = new URL('..', import.meta.url).pathname; // crate root
mkdirSync(root + 'out', { recursive: true });

const sha = (p) => createHash('sha256').update(readFileSync(p)).digest('hex');
const htmlSha = sha(root + 'showcase/index.html');
const cssSha = sha(root + 'showcase/tw.css');

const FACTS_VIEWPORT = { width: 1280, height: 4096 };

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: FACTS_VIEWPORT,
  deviceScaleFactor: 1,
});
await page.goto('file://' + root + 'showcase/index.html');
await page.addStyleTag({ path: root + 'showcase/tw.css' });
await page.evaluate(() => document.fonts.ready);
await page.waitForTimeout(150);

// Duplicate data-vv-test ids would silently collide in any facts join.
const duplicates = await page.evaluate(() => {
  const seen = new Set();
  const dupes = new Set();
  document.querySelectorAll('[data-vv-test]').forEach((el) => {
    const id = el.getAttribute('data-vv-test');
    if (seen.has(id)) dupes.add(id);
    seen.add(id);
  });
  return [...dupes];
});
if (duplicates.length) {
  console.error('DUPLICATE data-vv-test ids:', duplicates);
  process.exit(2);
}

const env = await page.evaluate(() => ({
  userAgent: navigator.userAgent,
  browserVersion: navigator.userAgent.match(/Chrome\/(\S+)/)?.[1] ?? 'unknown',
  devicePixelRatio: window.devicePixelRatio,
}));

const facts = await page.evaluate(() => {
  const nodes = [];
  document.querySelectorAll('[data-vv-test]').forEach((el) => {
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    nodes.push({
      id: el.getAttribute('data-vv-test'),
      tag: el.tagName.toLowerCase(),
      display: cs.display,
      x: +(r.x + window.scrollX).toFixed(2),
      y: +(r.y + window.scrollY).toFixed(2),
      w: +r.width.toFixed(2),
      h: +r.height.toFixed(2),
      padding: [parseFloat(cs.paddingTop), parseFloat(cs.paddingRight), parseFloat(cs.paddingBottom), parseFloat(cs.paddingLeft)],
      border: [parseFloat(cs.borderTopWidth), parseFloat(cs.borderRightWidth), parseFloat(cs.borderBottomWidth), parseFloat(cs.borderLeftWidth)],
      borderStyle: cs.borderTopStyle,
      lineHeight: cs.lineHeight,
      fontSize: cs.fontSize,
      margin: [parseFloat(cs.marginTop), parseFloat(cs.marginRight), parseFloat(cs.marginBottom), parseFloat(cs.marginLeft)],
      text: (el.innerText || '').replace(/\n/g, ' ').slice(0, 80),
    });
  });
  return { nodes };
});

const meta = {
  engine: 'chromium',
  factsViewport: [FACTS_VIEWPORT.width, FACTS_VIEWPORT.height],
  deviceScaleFactor: 1,
  font: "DejaVu Sans (via CSS font-family; loaded via document.fonts.ready)",
  htmlSha256: htmlSha,
  cssSha256: cssSha,
  tailwind: 'v3.4.17 (npx tailwindcss CLI, utilities only + documented parity base)',
  ...env,
  capturedAt: new Date().toISOString(),
};
console.log('browser content height:', await page.evaluate(() => document.documentElement.scrollHeight));

// Full-page screenshot: capture mechanics only (layout inputs above are the
// fixed facts viewport).
await page.screenshot({ path: root + 'out/browser.png', fullPage: true });

writeFileSync(root + 'out/browser-facts.json', JSON.stringify({ meta, ...facts }, null, 2));
console.log('captured', facts.nodes.length, 'fact nodes · chromium', env.browserVersion);
await browser.close();

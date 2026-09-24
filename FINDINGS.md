# Showcase findings — VelquView (1d0a562) vs Chromium 153 + Tailwind v3.4.17

Second revision (2026-09-21 evening). Supersedes the first revision, which
contained two review-identified errors, both corrected here with evidence:

1. **`max-w-64` was misclassified as "v4-only".** The Tailwind v3.4.17 CLI
   emits `.max-w-64 { max-width: 16rem }` — v3.4 added the spacing scale to
   max/min-width. It is a **valid upstream utility outside velqu-tailwind's
   declared v0 subset** (which implements named max-w breakpoints only).
   Corrected classification; the page itself stays inside the supported
   subset, and the boundary is now pinned by the profile check's deferred
   table, not by an authoring anecdote.
2. **The reported "+2px auto-size border-box engine difference" was a
   comparison-setup error, not an engine difference.** The browser reference
   had no preflight, so Chromium computed every border-width utility to an
   effective 0 (border-style: none ⇒ used width 0 — captured border=[0,0,0,0]
   on every element) while velqu applied real 1px borders. With Tailwind's
   own preflight border-base rule added to the parity CSS (the one rule its
   border utilities presuppose), the difference **disappears completely**:
   borders/spacing/sizing/flex/grid/images/overflow now compare at exactly
   0.0 on every property. First revision's attribution of residual deltas to
   "auto-size clamping" is retracted.

## What the suite is (three complementary checks)

- **Profile check** (`cargo run --release --bin profile-check`): 202 page
  classes pinned to exact declarations emitted by velqu-tailwind; 22
  valid-upstream/deferred classes confirmed to produce diagnostics and no
  silent rules; any unclassified page class fails. Color expectations were
  spot-verified against real Tailwind v3.4.17 output (rgb triplets match).
- **Structural comparison** (`python3 tools/compare.py`): gated against
  reviewed `baselines.json` — completeness (every page id present in both
  engines or on the reviewed absence record), per-element section-local
  geometry (regressions AND unexpected improvements fail; every accepted
  difference is a recorded deviation), global section structure, and input
  pinning (page/CSS hashes, fixed 1280x4096 facts viewport both engines,
  font-loading waited, browser build + CSS sha recorded).
- **Velqu raster regression** (`baselines.json` velquRaster): whole-page +
  per-section pixel digests of the velqu render, plus an in-process
  double-render determinism digest. This answers "did the pinned velqu
  rendering change", not "is velqu equal to Chromium" — cross-engine raster
  equality is deliberately NOT a gate; the side-by-side montages in
  `out/report.html` remain the human-review evidence for paint-level
  differences (color, radius, weight), which geometry alone cannot
  establish.

## Conformance result (corrected setup, measured 2026-09-21)

**Defensible headline: under the pinned comparison setup, the former
border-related geometry differences disappear. Remaining measured text,
control, and semantic differences are individually recorded and
review-gated.**

Section accounting is DERIVED by the gate from its own classification
(`SECTION_KIND` in `tools/compare.py`; `derived_summary()` prints it on
every run and embeds it in `out/report.html` — never hand-counted):

- Sections tested: **13**
- Conformance-designated: **10** (type, colors, borders, spacing, sizing,
  flex, grid, styled controls, images, overflow)
- Difference-designated: **3** (sec-native, sec-controls-native,
  sec-inline-flex)
- Page ids: **206**; compared in velqu: **205** — `native-hr` has no box in
  the v0 profile and sits on the reviewed absence record; a missing id
  without a reviewed record fails the gate.

Measured agreement in the recorded baseline capture (section-local
coordinates; page height velqu 5695px vs browser 5721px; per-session
sub-pixel re-rounding noted above):

| section | elements | max \|lx\| | max \|ly\| | max \|dw\| | max \|dh\| |
|---|---|---|---|---|---|
| sec-borders | 15 | 0.0 | 0.0 | 0.0 | 0.0 |
| sec-spacing | 10 | 0.0 | 0.0 | 0.0 | 0.0 |
| sec-sizing | 14 | 0.0 | 0.0 | 0.0 | 0.0 |
| sec-flex | 15 | 0.0 | 0.0 | 0.0 | 0.0 |
| sec-grid | 17 | 0.0 | 0.0 | 0.0 | 0.0 |
| sec-images | 5 | 0.0 | 0.0 | 0.0 | 0.0 |
| sec-overflow | 4 | 0.0 | 0.0 | 0.0 | 0.0 |
| sec-colors | 33 | 0.5 | 0.0 | 0.2 | 0.0 |
| sec-type | 42 | 1.1 | 0.0 | 0.5 | 0.0 |
| sec-controls | 12 | 6.3 | 0.0 | 12.6 | 0.0 |

Per-capture measured maxima for the conformance sections (from the gate's
derived output): six to seven sections at max |delta| 0.0 depending on
capture session (borders, spacing, sizing, flex, images, overflow exact;
grid 0.0 to 0.01), sec-colors ≈ 0.5, sec-type ≈ 1.1, sec-controls ≈ 12.6.
Cross-capture variation of up to 0.01 px was observed and remains within
the declared 0.51 px numerical tolerance. The cause is not established
(browser-side re-rounding is a plausible hypothesis, not a diagnosis), and
the tolerance is an acceptance policy, not proof that every difference it
admits is rounding. sec-colors/sec-type residuals are sub-pixel text-layout
rounding; sec-controls' four deviations are the **auto-width buttons and
link**, whose width comes from glyph advance widths
(`ctl-btn-primary` dw +2.44, `-secondary` +3.84, `-disabled` +6.34,
`ctl-link` +12.61 px; all heights and fixed-size controls exact). Every
accepted difference is a reviewed entry in `baselines.json` (49 deviation
entries total).

Scope attachment: these are results for the selected classes, fixtures,
font setup, browser build (Chromium 153.0.8010.12), fixed 1280x4096 facts
viewport, and 1.0x scale — a measured comparison at that scope, not an
unrestricted Tailwind or browser-compatibility claim.

## Differences by category

### A. Real semantic deviation (renderer profile)

**Inline children of flex containers are not blockified.** An in-flow
`<span>` child of a flex container is a flex item in CSS (display
blockified); velqu flows it as inline text in the container's line box.
Measured on the tiny fixture (`sec-inline-flex`): two spans in
`flex flex-col` — browser: stacked items w=1100 each; velqu: one text line,
span w≈136. Using `block` on such children is a **velqu authoring
workaround, not a CSS requirement**; conformance sections use it as the
documented idiom.

**Large inline text (separate fixture, separate facts — root cause with A
is NOT established):** a `text-4xl` span inline in a block: widths agree to
0.1px (164.9 vs 164.9), but the browser box honors the span's own paired
line-height (40px box; computed line-height 40px) while the velqu box stays
at the container line box (24px). Recorded as its own observation with
computed line-heights: browser `inflex-big` 36px/40px, `inflex-small`
12px/16px, both velqu boxes 24px.

### B. Native/default-style comparison (not profile defects)

- **Default line-height on unstyled text**: velqu 24px (its
  `text-base`-equivalent pairing) vs Chromium's `normal` for DejaVu Sans
  ≈ 18.4px (computed `line-height: normal`, font-metric dependent — the
  browser value is a measurement of this pinned font/browser setup, not a
  universal). Visible as dh deviations on every `native-*` text element.
  With explicit utilities (`text-base` etc.) both engines agree exactly
  (24px) — sec-type proves it.
- **Intrinsic control sizes**: input 200x32 vs 185x21; textarea 240x96 vs
  182x36; button 109.7x24 vs 94.5x21; select 144x24 vs 74x19. Explicit
  utility sizing removes the difference (sec-controls matches).

### C. Unsupported / limited behavior (profile-classified; boxes existing does not imply working semantics)

- `select`: renders as an empty box — no options text, no dropdown.
- List markers (bullets/numbers): not rendered; `li` boxes exist but the
  marker glyph and indentation semantics do not.
- Italic: no italic face bundled (regular + bold DejaVu Sans only).
- Monospace: `code`/`pre` are not monospaced (single face).
- `<pre>`: falls back to inline (no UA block default).
- `<hr>`: produces no box at all (reviewed absence record).

### D. Comparison-setup notes (recorded so they are never re-confused with engine behavior)

- Parity CSS is real Tailwind v3.4.17 utilities plus exactly two base
  rules: `box-sizing: border-box` (velqu's preflight-lite) and the
  preflight border base (`border-width:0; border-style:solid`) that
  Tailwind border utilities presuppose. Nothing else is normalized.
- Layout facts are taken at the same fixed 1280x4096 logical viewport in
  both engines at deviceScaleFactor 1; full-page PNGs are capture mechanics
  only. Geometry units are CSS px on both sides.
- The browser waits for `document.fonts.ready` before measuring; inputs are
  pinned by hash in both facts files, and the gate refuses to run against
  changed inputs.

## Holding future framework changes to this suite

The pins here reproduce the ACCEPTED revision (`velqu-view`/`velqu-tailwind`
git rev 1d0a562). To test a candidate framework revision, change the git
`rev` in `Cargo.toml` (both deps together), rerun the pipeline, and compare
against `baselines.json` — the gate will fail on every rendering change,
and `out/report.html` explains them. Record both revisions and the
disposition in this file; re-baselining requires reviewing each change. The
operator application's pins are never touched by this procedure.

Two distinctions govern candidate review:

- **An unexpected improvement is a baseline mismatch, not necessarily a
  correctness regression.** The gate rejects it pending review; the
  response is to inspect the candidate, update the expectation, and retire
  the corresponding deviation entry when justified — never to restore
  previous behavior merely to keep the old baseline green.
- **A deferred class beginning to compile is a profile revision, not a
  violation.** The `profile-check` deferred table records the *currently
  accepted* profile, not a permanent prohibition: a class starting to
  compile should trigger an explicit profile revision with verification of
  its exact declarations, updating both tables.

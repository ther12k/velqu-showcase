# velqu-showcase

A permanent external differential-test suite: a Tailwind page implementing
every HTML element and utility the VelquView renderer supports, checked
three ways — against velqu's own declared utility profile, against a real
browser's geometry, and against velqu's own pinned rasters.

Standalone on purpose: the framework repo (3c1ee34) and the operator
application (bd09924, pins 1d0a562) stay untouched. This crate consumes
`velqu-view` + `velqu-tailwind` at git rev `1d0a562` exactly like any
external developer would.

## The three checks (all must pass)

| check | command | what it pins |
|---|---|---|
| Profile contract | `cargo run --release --bin profile-check` | 202 page classes compile to exact declarations; 22 valid-upstream/deferred classes (e.g. `max-w-64`, variants, positioning) are diagnosed — never silently omitted; every page class classified |
| Structural comparison | `python3 tools/compare.py` | completeness (206 ids, none missing/duplicated; reviewed absences only), section-local geometry vs reviewed `baselines.json`, global section structure, input pinning |
| Velqu raster regression | inside `compare.py` | whole-page + per-section pixel digests and a double-render determinism digest of the velqu render |

Gate semantics: regressions fail, NEW differences fail, **unexpected
improvements also fail** (a delta shrinking requires reviewing the
deviation record — the reference is not silently rewritten), and
`baselines.json` records every accepted difference with measured values
and a reason. Cross-engine raster equality is deliberately NOT a gate;
`out/report.html` carries the side-by-side montages for human review of
paint-level properties.

## Pipeline

    ./tools/build-browser-css.sh            # real Tailwind v3.4.17 utilities
                                            # + the two documented parity rules
    cargo run --release --bin velqu-showcase    # velqu PNG + facts (fixed 1280x4096 facts viewport,
                                                # double-render determinism, input hashes)
    (cd tools && npm i && node browser-capture.mjs)  # Chromium facts at the SAME fixed viewport,
                                                     # document.fonts.ready, env pinned in the facts
    cargo run --release --bin profile-check  # utility/profile contract
    python3 tools/compare.py                 # the gate (first ever run: --init, then REVIEW baselines.json)

First-time baseline creation is explicit and review-gated:

    python3 tools/compare.py --init   # writes baselines.json — review before treating as accepted

## Files

- `showcase/index.html` — the page; 13 sections; ids via `data-vv-test`.
  Sections 2/10/13 are *expected-difference* sections (native defaults,
  native controls, inline-in-flex) kept deliberately so the differences
  stay measured instead of hidden.
- `baselines.json` — the reviewed acceptance record (per-element accepted
  deltas, global structure, raster digests, input hashes).
- `FINDINGS.md` — corrected measured results, differences classified by
  what they actually test (semantic deviation vs native defaults vs
  unsupported), comparison-setup notes.
- `tools/build-browser-css.sh` — reproducible browser CSS (pin: Tailwind
  v3.4.17 via npx; parity rules documented inline).
- `tools/browser-capture.mjs` — Chromium reference; records browser build,
  UA, device scale, CSS/HTML hashes into the facts.
- `tools/compare.py` — the gate + report generator.
- `out/report.html` — human-readable evidence (montages, delta tables,
  accepted deviations).

## Testing a candidate framework revision (external; never repins the operator app)

The pins here reproduce the accepted framework revision. To evaluate a
candidate:

1. Change the git `rev` in `Cargo.toml` (both velqu deps together) to the
   candidate.
2. Run the full pipeline above. The gate fails on every rendering change;
   `out/report.html` and the failure list explain each one.
3. Record candidate rev, accepted rev, and the disposition (accept /
   revise / re-baseline with review) in `FINDINGS.md`.
4. Re-baselining is a reviewed act: update `baselines.json` only after
   examining each change — improvements included.

Revert `Cargo.toml` to the accepted rev when done; the operator
application and its pins are untouched by this procedure.


## Live view (browse the sample in the app itself)

`live/` holds the same component coverage restructured for the app's
real UX constraints: the document itself does not scroll in the v0
profile (only overflow containers do), so the page is a two-pane shell —
a fixed sidebar menu (reactive: pick one component or view all) and a
right content pane that scrolls as an `overflow-y-auto` container. Wide
fixture rows use `flex-wrap` so nothing clips at narrower window sizes.
The page is loaded WITHOUT the browser-only Tailwind CSS, so VelquView
compiles its own utilities:

    ~/Workspace/Learning/velqu-view-starter/target/release/velqu-view-starter \
        --app-dir ~/Workspace/Learning/velqu-showcase/live \
        --data-dir /tmp/sample-data

`live/app.css` carries the full-viewport shell reset (the preflight-lite
sets only box-sizing, so the UA body margin would frame the page in
white). The comparison pipeline still uses `showcase/` only; baselines
are unaffected.

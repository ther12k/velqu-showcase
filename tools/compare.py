#!/usr/bin/env python3
"""Structural comparison gate — VelquView facts vs browser facts.

Three evidence layers, per FINDINGS.md:
1. COMPLETENESS — every data-vv-test id in the page must appear in BOTH
   engines' facts, exactly once (missing ids fail; they never vanish
   through an intersection join).
2. GEOMETRY — per-element deltas in section-local coordinates, gated
   against `baselines.json`: a property may differ from the browser only
   where a reviewed deviation entry exists (id + property + expected
   value + reason). Regressions beyond the recorded deviation, NEW
   deviations, and UNEXPECTED IMPROVEMENTS (delta shrank) all fail:
   improvements require reviewing the deviation record, not silently
   rewriting the reference.
3. GLOBAL STRUCTURE — section origins, sizes, inter-section gaps and page
   heights, reported (and baselined) independently: section-local deltas
   must not hide displaced sections.

Usage:
  python3 tools/compare.py            # gate against baselines.json
  python3 tools/compare.py --init     # first run: write baselines for review

Exit non-zero on any gate failure.
"""
import hashlib
import json
import re
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "out"
BASELINES = ROOT / "baselines.json"
GEOM_TOL = 0.51  # rounding slack around recorded values; real tolerance is the reviewed deviation itself

PROPS = ["lx", "ly", "dw", "dh"]

def fail(msg):
    print(f"GATE FAIL: {msg}")
    sys.exit(1)

def load(path):
    data = json.loads(path.read_text())
    nodes = {n["id"]: n for n in data["nodes"]}
    if len(nodes) != len(data["nodes"]):
        fail(f"{path.name}: duplicate ids in facts")
    return data, nodes

velqu_data, velqu = load(OUT / "velqu-facts.json")
browser_data, browser = load(OUT / "browser-facts.json")

# ---- input pinning: both facts must come from the page/css on disk ----
def fnv1a64(data: bytes) -> int:
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h

html_bytes = (ROOT / "showcase" / "index.html").read_bytes()
page_hash = f"{fnv1a64(html_bytes):016x}"
if velqu_data["meta"].get("htmlFnv1a64") != page_hash:
    fail("velqu facts were built from a different showcase/index.html — rerun the velqu render")
css_sha = hashlib.sha256((ROOT / "showcase" / "tw.css").read_bytes()).hexdigest()
if browser_data["meta"].get("cssSha256") != css_sha:
    fail("browser facts were built from a different tw.css — rerun the browser capture")
if browser_data["meta"].get("htmlSha256") != hashlib.sha256(html_bytes).hexdigest():
    fail("browser facts were built from a different showcase/index.html — rerun the browser capture")
for name, meta in (("velqu", velqu_data["meta"]), ("browser", browser_data["meta"])):
    if tuple(meta.get("factsViewport", ())) != (1280, 4096):
        fail(f"{name} facts viewport is not the fixed 1280x4096 facts viewport")

# ---- section classification (single source of truth for counts) ----
SECTION_KIND = {
    "sec-type": "conformance", "sec-colors": "conformance",
    "sec-borders": "conformance", "sec-spacing": "conformance",
    "sec-sizing": "conformance", "sec-flex": "conformance",
    "sec-grid": "conformance", "sec-controls": "conformance",
    "sec-images": "conformance", "sec-overflow": "conformance",
    "sec-native": "difference", "sec-controls-native": "difference",
    "sec-inline-flex": "difference",
}

def derived_summary():
    """Counts derived from SECTION_KIND + measured rows, never hand-counted."""
    conformance = [s_ for s_ in SECTION_IDS if SECTION_KIND[s_] == "conformance"]
    difference = [s_ for s_ in SECTION_IDS if SECTION_KIND[s_] == "difference"]
    def max_delta(sec):
        vals = [abs(rows[f][p_]) for f in EXPECTED[sec] if f in rows for p_ in PROPS]
        return max(vals) if vals else 0.0
    exact = [s_ for s_ in conformance if max_delta(s_) <= 0.0]
    rest = [(s_, max_delta(s_)) for s_ in conformance if max_delta(s_) > 0.0]
    lines = [
        f"sections: {len(SECTION_IDS)} tested · {len(conformance)} conformance-designated · "
        f"{len(difference)} difference-designated ({', '.join(difference)})",
        f"conformance sections at max |delta| 0.0: {len(exact)} ({', '.join(exact)})",
    ]
    for s_, m in sorted(rest, key=lambda kv: kv[1]):
        lines.append(f"conformance section with recorded deviations: {s_} max |delta| {m:.2f}")
    return "\n".join(lines)

# ---- expected membership, from the page itself ----
SECTION_IDS = [
    "sec-type", "sec-native", "sec-colors", "sec-borders", "sec-spacing",
    "sec-sizing", "sec-flex", "sec-grid", "sec-controls",
    "sec-controls-native", "sec-images", "sec-overflow", "sec-inline-flex",
]
EXPECTED: dict[str, list[str]] = {sec: [sec] for sec in SECTION_IDS}
current = None
for m in re.finditer(r'data-vv-test="([^"]+)"', html_bytes.decode()):
    fid = m.group(1)
    if fid in SECTION_IDS:
        current = fid
    elif current:
        EXPECTED[current].append(fid)
    else:
        fail(f"page has data-vv-test id {fid!r} before any section")
TOTAL_EXPECTED = sum(len(v) for v in EXPECTED.values())

# ---- completeness ----
# Reviewed absent-element record: ids the v0 profile demonstrably does not
# box at all. An id appearing here UNEXPECTEDLY is a profile change and
# fails for review, exactly like an unexpected improvement in geometry.
EXPECTED_ABSENT_IN_VELQU = {
    "native-hr": "<hr> produces no layout box in the v0 profile (FINDINGS.md, UA-default coverage gaps)",
}
EXPECTED_ABSENT_IN_BROWSER = {}

missing_velqu, missing_browser = [], []
unexpectedly_present = []
for sec, ids in EXPECTED.items():
    for fid in ids:
        if fid not in velqu and fid not in EXPECTED_ABSENT_IN_VELQU:
            missing_velqu.append(fid)
        if fid not in browser and fid not in EXPECTED_ABSENT_IN_BROWSER:
            missing_browser.append(fid)
for fid in EXPECTED_ABSENT_IN_VELQU:
    if fid in velqu:
        unexpectedly_present.append(f"{fid} now boxes in velqu — reviewed absence no longer holds")
if missing_velqu:
    fail(f"missing in VELQU facts (never hide through a join): {missing_velqu}")
if missing_browser:
    fail(f"missing in BROWSER facts: {missing_browser}")
if unexpectedly_present:
    fail(f"{unexpectedly_present} — re-review the absence record")
page_id_count = len(re.findall(r'data-vv-test="', html_bytes.decode()))
if missing_velqu:
    fail(f"missing in VELQU facts (never hide through a join): {missing_velqu}")
if missing_browser:
    fail(f"missing in BROWSER facts: {missing_browser}")
if page_id_count != TOTAL_EXPECTED:
    fail(f"page declares {page_id_count} data-vv-test ids but section walk collected {TOTAL_EXPECTED}")
GEOM_IDS = TOTAL_EXPECTED - len(EXPECTED_ABSENT_IN_VELQU)

# ---- geometry deltas (section-local position + direct size) ----
rows = {}
for sec, ids in EXPECTED.items():
    sv, sb = velqu[sec], browser[sec]
    for fid in ids:
        if fid in EXPECTED_ABSENT_IN_VELQU:
            continue  # no velqu box: recorded as absent, not compared
        v, b = velqu[fid], browser[fid]
        rows[fid] = {
            "id": fid, "section": sec, "tag": b["tag"],
            "lx": (v["x"] - sv["x"]) - (b["x"] - sb["x"]),
            "ly": (v["y"] - sv["y"]) - (b["y"] - sb["y"]),
            "dw": v["w"] - b["w"],
            "dh": v["h"] - b["h"],
        }

def near(a, b, tol=GEOM_TOL):
    return abs(a - b) <= tol

def prop_of_interest(delta):
    return delta

# ---- global structure ----
global_rows = []
order = [s for s in EXPECTED if True]
section_order = list(EXPECTED.keys())
for sec in section_order:
    v, b = velqu[sec], browser[sec]
    global_rows.append({
        "section": sec,
        "dx": v["x"] - b["x"], "dy": v["y"] - b["y"],
        "dw": v["w"] - b["w"], "dh": v["h"] - b["h"],
    })
page_heights = {
    "velqu_png": Image.open(OUT / "velqu.png").height,
    "browser_png": Image.open(OUT / "browser.png").height,
}

# ---- gate vs baselines ----
init = "--init" in sys.argv
if init:
    baselines = {
        "note": "Reviewed acceptance record. Per-element entries are the ACCEPTED "
                "velqu-vs-browser deltas; deviations list every accepted difference "
                "with its reason. Regressions, new deviations, and unexpected "
                "improvements fail the gate.",
        "inputs": {
            "pageFnv1a64": page_hash,
            "cssSha256": css_sha,
            "velquDeterminism": velqu_data["meta"]["determinismDigest1"],
        },
        "sections": {
            sec: {fid: {p: round(rows[fid][p], 2) for p in PROPS}
                  for fid in ids if fid in rows}
            for sec, ids in EXPECTED.items()
        },
        "globalSections": global_rows,
        "pageHeights": page_heights,
        "velquRaster": {},
    }
    # raster baselines: velqu-vs-velqu pins (per-section crops + whole page)
    vp = Image.open(OUT / "velqu.png")
    raster = {"page": hashlib.sha256(vp.tobytes()).hexdigest()}
    for sec in section_order:
        s = velqu[sec]
        y0, y1 = max(0, int(s["y"])), min(vp.height, int(s["y"] + s["h"]))
        crop = vp.crop((0, y0, vp.width, y1))
        raster[sec] = hashlib.sha256(crop.tobytes()).hexdigest()
    baselines["velquRaster"] = raster
    BASELINES.write_text(json.dumps(baselines, indent=1))
    print(f"--init: wrote {BASELINES} — REVIEW the recorded deltas and deviations "
          f"before treating them as accepted")
    print(derived_summary())
    print(f"page heights: velqu {page_heights['velqu_png']}px, browser {page_heights['browser_png']}px")
    for sec in section_order:
        sec_rows = [rows[f] for f in EXPECTED[sec] if f in rows]
        worst = {p: max(abs(r[p]) for r in sec_rows) for p in PROPS}
        print(f"  {sec:22s} n={len(sec_rows):3d} max |lx|={worst['lx']:6.1f} |ly|={worst['ly']:6.1f} "
              f"|dw|={worst['dw']:6.1f} |dh|={worst['dh']:6.1f}")
    sys.exit(0)

if not BASELINES.exists():
    fail("baselines.json missing — run `python3 tools/compare.py --init` and review it first")
baselines = json.loads(BASELINES.read_text())

if baselines["inputs"]["pageFnv1a64"] != page_hash:
    fail("showcase/index.html changed since the baselines were accepted — re-review and re-record")
if baselines["inputs"]["cssSha256"] != css_sha:
    fail("tw.css changed since the baselines were accepted — re-review and re-record")
if baselines["inputs"]["velquDeterminism"] != velqu_data["meta"]["determinismDigest1"]:
    fail("velqu determinism digest changed — the pinned rendering changed")

failures = []

# 3. raster gate (velqu-to-velqu: did the pinned rendering change?)
vp = Image.open(OUT / "velqu.png")
raster_now = {"page": hashlib.sha256(vp.tobytes()).hexdigest()}
for sec in section_order:
    s = velqu[sec]
    y0, y1 = max(0, int(s["y"])), min(vp.height, int(s["y"] + s["h"]))
    raster_now[sec] = hashlib.sha256(vp.crop((0, y0, vp.width, y1)).tobytes()).hexdigest()
for key, want in baselines["velquRaster"].items():
    if raster_now[key] != want:
        failures.append(f"VELQU RASTER changed for {key} (pinned baseline differs)")

# 2. geometry gate: accepted deltas only
accepted = baselines["sections"]
for sec, ids in EXPECTED.items():
    if sec not in accepted:
        failures.append(f"section {sec} missing from baselines")
        continue
    for fid in ids:
        if fid in EXPECTED_ABSENT_IN_VELQU:
            continue
        if fid not in accepted[sec]:
            failures.append(f"element {fid} missing from baselines (new element?)")
            continue
        for p in PROPS:
            recorded = accepted[sec][fid][p]
            current_val = rows[fid][p]
            if near(current_val, recorded):
                continue
            direction = "less-negative/moved-toward-zero" if abs(current_val) < abs(recorded) else "regressed"
            failures.append(
                f"{fid}.{p}: recorded {recorded:+.2f}, now {current_val:+.2f} ({direction}) — "
                f"re-review the deviation record"
            )

# global structure gate (matched by section name, not position)
rec_by_name = {rec["section"]: rec for rec in baselines["globalSections"]}
for now in global_rows:
    rec = rec_by_name.get(now["section"])
    if rec is None:
        failures.append(f"global structure: section {now['section']} missing from baselines")
        continue
    for p in ("dx", "dy", "dw", "dh"):
        if not near(rec[p], now[p]):
            failures.append(f"GLOBAL {now['section']}.{p}: recorded {rec[p]:+.2f}, now {now[p]:+.2f}")

if failures:
    print(f"compare FAILED ({len(failures)}):")
    for f in failures:
        print(f"  - {f}")
    sys.exit(1)

# ---- report (always useful for humans; gate already passed) ----
vp_img, br_img = Image.open(OUT / "velqu.png"), Image.open(OUT / "browser.png")
(OUT / "sections").mkdir(exist_ok=True)
LABEL = 28
for sec in section_order:
    b = browser[sec]
    y0, y1 = max(0, int(b["y"]) - 8), min(br_img.height, int(b["y"] + b["h"]) + 8)
    half = 640
    pair = Image.new("RGB", (half * 2 + 8, (y1 - y0) + LABEL), (24, 24, 27))
    d = ImageDraw.Draw(pair)
    d.text((8, 7), f"BROWSER — {sec}", fill=(255, 255, 255))
    d.text((half + 16, 7), f"VELQUVIEW — {sec}", fill=(255, 255, 255))
    pair.paste(br_img.crop((0, y0, 1280, y1)).resize((half, y1 - y0)), (0, LABEL))
    vh = max(1, min(y1, vp_img.height) - y0)
    pair.paste(vp_img.crop((0, y0, 1280, y0 + vh)).resize((half, vh)), (half + 8, LABEL))
    pair.save(OUT / "sections" / f"{sec}.png")

def section_line(sec):
    sec_rows = [rows[f] for f in EXPECTED[sec] if f in rows]
    worst = {p: max(sec_rows, key=lambda r: abs(r[p])) for p in PROPS}
    nonconstant = [
        f"{p} for {fid}"
        for fid in EXPECTED[sec]
        if fid in rows and fid in accepted[sec]
        for p in PROPS
        if not near(rows[fid][p], accepted[sec][fid][p])
    ]
    return (f"<tr><td>{sec}</td><td>{len(sec_rows)}</td>"
            + "".join(f"<td>{worst[p][p]:+.1f} ({worst[p]['id']})</td>" for p in PROPS)
            + f"<td>{len(nonconstant) or '0'}</td></tr>")

summary = "\n".join(section_line(s) for s in section_order)
global_tbl = "\n".join(
    f"<tr><td>{g['section']}</td><td>{g['dx']:+.1f}</td><td>{g['dy']:+.1f}</td>"
    f"<td>{g['dw']:+.1f}</td><td>{g['dh']:+.1f}</td></tr>"
    for g in global_rows
)
gallery = "\n".join(f"<h3>{s}</h3><img src='sections/{s}.png' width='100%'>" for s in section_order)
deviations = sorted(
    ({"id": fid, **{p: accepted[sec][fid][p] for p in PROPS}}
     for sec, ids in accepted.items() for fid in ids
     if any(abs(accepted[sec][fid][p]) > GEOM_TOL for p in PROPS)),
    key=lambda d: -max(abs(d[p]) for p in PROPS),
)
dev_tbl = "\n".join(
    f"<tr><td>{d['id']}</td>" + "".join(f"<td>{d[p]:+.2f}</td>" for p in PROPS) + "</tr>"
    for d in deviations
)
summary_text = derived_summary()
html = f"""<!doctype html><html><head><meta charset="utf-8"><title>Showcase comparison</title>
<style>
body{{font-family:'DejaVu Sans',sans-serif;margin:24px;background:#0f172a;color:#e2e8f0;max-width:1400px}}
h1,h2,h3{{color:#fff}} table{{border-collapse:collapse;font-size:12px}}
td,th{{border:1px solid #334155;padding:3px 7px;text-align:right}} td:first-child,th:first-child{{text-align:left}}
.dim{{color:#94a3b8}} img{{border:1px solid #334155;border-radius:6px}} code{{color:#93c5fd}}
</style></head><body>
<h1>VelquView vs Browser — Tailwind showcase (gated)</h1>
<pre style="background:#1e293b;padding:10px;border-radius:6px">{summary_text}</pre>
<p>All gates passed: completeness ({TOTAL_EXPECTED} ids, none missing, none duplicated),
geometry within recorded accepted deltas, global structure unchanged, velqu raster
identical to pinned baselines.</p>
<p>Velqu: {velqu_data['meta'].get('frameworkRev','see baselines')} · determinism
<code>{velqu_data['meta']['determinismDigest1'][:16]}…</code> · facts viewport 1280x4096.
Browser: {browser_data['meta'].get('browserVersion','?')} · Tailwind v3.4.17 utilities
(+ documented parity base) · CSS <code>{css_sha[:12]}…</code>.</p>
<p>Page heights: velqu {page_heights['velqu_png']}px · browser {page_heights['browser_png']}px
(Δ {page_heights['velqu_png'] - page_heights['browser_png']:+d}px).</p>
<h2>Per-section max deltas (section-local)</h2>
<table><tr><th>section</th><th>n</th><th>max lx</th><th>max ly</th><th>max dw</th><th>max dh</th><th>non-matching props</th></tr>
{summary}</table>
<h2>Global section deltas (browser→velqu)</h2>
<table><tr><th>section</th><th>dx</th><th>dy</th><th>dw</th><th>dh</th></tr>
{global_tbl}</table>
<h2>Accepted deviations (|delta| &gt; {GEOM_TOL})</h2>
<table><tr><th>id</th><th>lx</th><th>ly</th><th>dw</th><th>dh</th></tr>
{dev_tbl}</table>
<h2>Side-by-side (left browser · right velqu)</h2>
{gallery}
</body></html>"""
(OUT / "report.html").write_text(html)
print(derived_summary())
print(f"compare OK: {TOTAL_EXPECTED} ids complete · geometry within accepted records · "
      f"{len(deviations)} accepted deviation entries · raster identical · report written")

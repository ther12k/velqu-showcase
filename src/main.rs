//! velqu-showcase — renders the Tailwind showcase page headless through
//! VelquView's public API (the same git-pinned crate any external consumer
//! uses) and dumps the frame plus per-element layout facts for comparison
//! against a real browser.
//!
//! The page is intentionally static: Tailwind enabled, reactive disabled.
//! This isolates the comparison to rendering, not reactivity.
//!
//! Layout inputs are the SAME fixed facts viewport as the browser capture
//! (1280x4096 @ 1.0); the content-height PNG rerender is capture mechanics
//! only. Layout facts are never taken from a viewport sized to the content.

use std::path::PathBuf;

use serde_json::json;
use velqu_view::{
    Asset, AssetRequest, AssetResolver, DocumentSource, SourceId, VelquView, Viewport,
};

const FACTS_VIEWPORT: (u32, u32) = (1280, 4096);

struct DirAssets {
    root: PathBuf,
}

impl AssetResolver for DirAssets {
    fn resolve(&self, request: AssetRequest<'_>) -> Option<Asset> {
        let rel = request.path.trim_start_matches("./");
        if rel.starts_with('/') || rel.split(['/', '\\']).any(|s| s == "..") {
            return None;
        }
        let bytes = std::fs::read(self.root.join(rel)).ok()?;
        Some(Asset {
            id: SourceId::new(rel),
            bytes,
        })
    }
}

fn facts_json(view: &mut VelquView, viewport: Viewport, digest_1: &str, digest_2: &str, page_hash: &str) -> serde_json::Value {
    let facts = view.layout_facts(viewport).expect("layout facts");
    let nodes: Vec<serde_json::Value> = facts
        .nodes
        .iter()
        .map(|n| {
            json!({
                "id": n.fixture_id,
                "tag": n.tag,
                "display": n.display,
                "x": n.x, "y": n.y, "w": n.width, "h": n.height,
                "padding": n.padding, "border": n.border, "margin": n.margin,
                "text": n.text_runs.join(""),
            })
        })
        .collect();
    json!({
        "meta": {
            "engine": "velqu-view",
            "factsViewport": [viewport.width(), viewport.height()],
            "deviceScaleFactor": viewport.scale_factor(),
            "font": "bundled DejaVu Sans regular + bold (fontdue)",
            "htmlFnv1a64": page_hash.to_owned(),
            "frameworkRev": option_env!("VELQU_FRAMEWORK_REV").unwrap_or("see Cargo.lock"),
            "determinismDigest1": digest_1,
            "determinismDigest2": digest_2,
            "capturedAt": chrono_placeholder(),
        },
        "nodes": nodes,
    })
}

// Change-detection digest (FNV-1a 64): pins the page's input identity for
// the comparison records. Not cryptographic — identity pinning only.
mod digest {
    pub fn fnv1a64(data: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in data {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    pub fn file_fnv1a64(path: &std::path::Path) -> std::io::Result<u64> {
        Ok(fnv1a64(&std::fs::read(path)?))
    }

    pub fn hex64(value: u64) -> String {
        format!("{value:016x}")
    }
}

/// ISO-8601 UTC timestamp without pulling a time crate: read it from the
/// environment at runtime (the runner exports SOURCE_DATE_EPOCH when it
/// wants reproducibility), else say unknown.
fn chrono_placeholder() -> String {
    std::process::Command::new("date")
        .args(["-Iseconds"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn main() {
    let dir = PathBuf::from("showcase");
    let html = std::fs::read_to_string(dir.join("index.html")).expect("showcase/index.html");

    let mut view = VelquView::new();
    view.enable_tailwind();
    let document =
        DocumentSource::new("showcase/index.html".to_owned(), html).with_base(dir.display().to_string());
    view.set_asset_resolver(std::rc::Rc::new(DirAssets {
        root: PathBuf::from("showcase"),
    }));
    view.load_document(document).expect("document loads");

    // Determinism: two renders of the same document must produce identical
    // frames (velqu-to-velqu; the browser comparison is separate evidence).
    let facts_vp = Viewport::try_new(FACTS_VIEWPORT.0, FACTS_VIEWPORT.1, 1.0).expect("facts viewport");
    let digest_1 = view.render(facts_vp).expect("render 1").frame.sha256_hex();
    let _ = view.take_events();
    let digest_2 = view.render(facts_vp).expect("render 2").frame.sha256_hex();
    if digest_1 != digest_2 {
        eprintln!("FAIL: renders are not deterministic:\n  {digest_1}\n  {digest_2}");
        std::process::exit(4);
    }

    let page_hash = digest::hex64(
        digest::file_fnv1a64(std::path::Path::new("showcase/index.html")).expect("hash page"),
    );
    let dump = facts_json(&mut view, facts_vp, &digest_1, &digest_2, &page_hash);
    let nodes = dump["nodes"].as_array().expect("nodes");
    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates = Vec::new();
    for node in nodes {
        let id = node["id"].as_str().expect("id");
        if !seen.insert(id.to_owned()) {
            duplicates.push(id.to_owned());
        }
    }
    if !duplicates.is_empty() {
        eprintln!("FAIL: duplicate data-vv-test ids in facts: {duplicates:?}");
        std::process::exit(2);
    }

    std::fs::create_dir_all("out").expect("out dir");
    std::fs::write(
        "out/velqu-facts.json",
        serde_json::to_string_pretty(&dump).expect("facts json"),
    )
    .expect("write facts");

    // PNG at content height: capture mechanics, not a layout input. Facts
    // above came from the fixed facts viewport.
    let max_y = nodes
        .iter()
        .map(|n| n["y"].as_f64().unwrap_or(0.0) + n["h"].as_f64().unwrap_or(0.0))
        .fold(0.0_f64, f64::max);
    let height = max_y.ceil() as u32;
    println!("content height: {height}px");
    let vp = Viewport::try_new(1280, height, 1.0).expect("png viewport");
    let result = view.render(vp).expect("png render");
    result
        .frame
        .save_png(std::path::Path::new("out/velqu.png"))
        .expect("save png");

    println!(
        "items: {}, glyphs: {} · determinism {digest_1}",
        result.stats.items, result.stats.glyphs
    );
    let diagnostics = view.tailwind_diagnostics();
    if diagnostics.is_empty() {
        println!("tailwind: all utility classes compiled");
    } else {
        for message in diagnostics {
            println!("tailwind diagnostic: {message}");
        }
        std::process::exit(3);
    }
    println!("wrote out/velqu.png and out/velqu-facts.json");
}

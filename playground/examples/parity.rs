//! Native parity harness: renders a document through the VENDORED
//! engine and prints the frame digest, for comparison against the real
//! frozen starter engine's `--headless` run of the same document.
//! Equal digests prove the vendored copy + wasm clock shim leave the
//! native pixel path untouched.

use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let set = std::env::args().nth(1).unwrap_or_else(|| "login".into());
    let html = std::fs::read_to_string(root.join(&set).join("index.html")).expect("html");
    let css = std::fs::read_to_string(root.join(&set).join("app.css")).expect("css");
    let mut view = velqu_view::VelquView::new();
    view.enable_tailwind();
    view.load_html(&html).expect("load");
    view.load_stylesheet(velqu_view::StylesheetSource::new(
        "app.css".to_owned(),
        css,
    ))
    .expect("css");
    // each set's A/B viewport: 1280xH @2x (H from the set's capture)
    let h: u32 = std::env::args().nth(2).unwrap_or_else(|| "800".into()).parse().expect("height");
    let vp = velqu_view::Viewport::try_new(2560, h * 2, 2.0).expect("viewport");
    let frame = view.render(vp).expect("render");
    println!(
        "set={} size={}x{} sha256={} diagnostics={}",
        set,
        frame.frame.width(),
        frame.frame.height(),
        frame.frame.sha256_hex(),
        view.tailwind_diagnostics().len()
    );
}

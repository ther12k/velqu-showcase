//! vv-playground — the VelquView engine in the browser tab.
//!
//! Thin wasm-bindgen wrapper over the vendored `velqu-view` crate
//! (framework rev 1d0a562; see `engine/velqu-view/Cargo.toml` for the
//! one wasm-only, pixel-neutral difference). The engine compiles
//! unmodified to native for the parity harness (`examples/parity.rs`).

use velqu_view::{StylesheetSource, VelquView, Viewport};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Playground {
    view: VelquView,
    error: String,
    digest: String,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

#[wasm_bindgen]
impl Playground {
    /// Tailwind enabled; the reactive engine stays OFF (the playground
    /// renders static documents, like the A/B sample files).
    #[wasm_bindgen(constructor)]
    pub fn new(html: &str, css: &str) -> Playground {
        let mut view = VelquView::new();
        view.enable_tailwind();
        let mut error = String::new();
        if let Err(e) = view.load_html(html) {
            error = e.to_string();
        }
        if !css.trim().is_empty() {
            let sheet = StylesheetSource::new("app.css".to_owned(), css.to_owned());
            if let Err(e) = view.load_stylesheet(sheet) {
                error = e.to_string();
            }
        }
        Playground { view, error, digest: String::new(), width: 0, height: 0, rgba: Vec::new() }
    }

    /// Renders at `logical_w` x `logical_h` with a DPI `scale`, exactly
    /// like the starter's `--size WxH --scale F` headless path (physical
    /// size = round(logical * scale)).
    pub fn render(&mut self, logical_w: u32, logical_h: u32, scale: f32) -> bool {
        let physical = |v: u32| (v as f32 * scale).round() as u32;
        let viewport = match Viewport::try_new(physical(logical_w), physical(logical_h), scale) {
            Ok(vp) => vp,
            Err(e) => {
                self.error = e.to_string();
                return false;
            }
        };
        match self.view.render(viewport) {
            Ok(result) => {
                self.width = result.frame.width();
                self.height = result.frame.height();
                self.digest = result.frame.sha256_hex();
                self.rgba = result.frame.pixels().to_vec();
                true
            }
            Err(e) => {
                self.error = e.to_string();
                false
            }
        }
    }

    /// Tailwind profile diagnostics — the compile-time gate, live.
    pub fn tailwind_diagnostics(&self) -> Vec<String> {
        self.view.tailwind_diagnostics()
    }

    pub fn css_diagnostics(&self) -> Vec<String> {
        self.view.css_diagnostics()
    }

    pub fn style_diagnostics(&self) -> Vec<String> {
        self.view.style_diagnostics()
    }

    #[wasm_bindgen(getter)]
    pub fn rgba(&self) -> Vec<u8> {
        self.rgba.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn digest(&self) -> String {
        self.digest.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> String {
        self.error.clone()
    }
}

//! Utility/profile check — the first of the suite's three complementary
//! gates.
//!
//! For every utility class used by the showcase page:
//! - SUPPORTED classes must compile to exactly the expected declarations
//!   (the profile contract: supported means *these* declarations);
//! - DEFERRED classes (valid upstream Tailwind outside velqu's v0 subset,
//!   or outside the renderer profile) must produce a diagnostic and emit
//!   NO CSS rule — silent omissions fail;
//! - any page class missing from both tables fails (the tables must be
//!   maintained when the page changes).
//!
//! Exit 0 = profile classification complete and verified.

use std::collections::BTreeMap;

use velqu_tailwind::compile_utilities;

/// Classes the page uses, with the exact declarations velqu-tailwind must
/// emit. Values mirror the v0 implementation's documented scales (integer
/// 4px spacing steps, Tailwind v3 named max-w breakpoints, indigo-600 =
/// #4f46e5, …).
fn expected_table() -> BTreeMap<&'static str, Vec<(&'static str, &'static str)>> {
    let mut t = BTreeMap::new();
    let mut add = |class: &'static str, decls: Vec<(&'static str, &'static str)>| {
        t.insert(class, decls);
    };

    // display / flex / grid
    add("block", vec![("display", "block")]);
    add("flex", vec![("display", "flex")]);
    add("flex-col", vec![("flex-direction", "column")]);
    add("flex-wrap", vec![("flex-wrap", "wrap")]);
    add("flex-1", vec![("flex-grow", "1"), ("flex-shrink", "1"), ("flex-basis", "0%")]);
    add("grid", vec![("display", "grid")]);
    add("grid-cols-2", vec![("grid-template-columns", "repeat(2, minmax(0, 1fr))")]);
    add("grid-cols-3", vec![("grid-template-columns", "repeat(3, minmax(0, 1fr))")]);
    add("grid-cols-6", vec![("grid-template-columns", "repeat(6, minmax(0, 1fr))")]);
    add("grid-rows-2", vec![("grid-template-rows", "repeat(2, minmax(0, 1fr))")]);
    add("col-span-2", vec![("grid-column", "span 2")]);
    add("col-span-3", vec![("grid-column", "span 3")]);

    // alignment
    add("justify-start", vec![("justify-content", "flex-start")]);
    add("justify-center", vec![("justify-content", "center")]);
    add("justify-end", vec![("justify-content", "flex-end")]);
    add("justify-between", vec![("justify-content", "space-between")]);
    add("items-start", vec![("align-items", "flex-start")]);
    add("items-center", vec![("align-items", "center")]);
    add("items-end", vec![("align-items", "flex-end")]);
    add("items-stretch", vec![("align-items", "stretch")]);

    // gap / spacing
    add("gap-1", vec![("gap", "4px")]);
    add("gap-2", vec![("gap", "8px")]);
    add("gap-4", vec![("gap", "16px")]);
    add("gap-6", vec![("gap", "24px")]);
    add("p-0", vec![("padding", "0px")]);
    add("p-1", vec![("padding", "4px")]);
    add("p-2", vec![("padding", "8px")]);
    add("p-4", vec![("padding", "16px")]);
    add("p-6", vec![("padding", "24px")]);
    add("p-8", vec![("padding", "32px")]);
    add("p-12", vec![("padding", "48px")]);
    add("p-px", vec![("padding", "1px")]);
    add("px-2", vec![("padding-left", "8px"), ("padding-right", "8px")]);
    add("px-3", vec![("padding-left", "12px"), ("padding-right", "12px")]);
    add("px-4", vec![("padding-left", "16px"), ("padding-right", "16px")]);
    add("py-1", vec![("padding-top", "4px"), ("padding-bottom", "4px")]);
    add("py-2", vec![("padding-top", "8px"), ("padding-bottom", "8px")]);
    add("m-0", vec![("margin", "0px")]);
    add("mt-1", vec![("margin-top", "4px")]);
    add("mt-2", vec![("margin-top", "8px")]);
    add("mt-4", vec![("margin-top", "16px")]);
    add("mt-6", vec![("margin-top", "24px")]);
    add("mt-8", vec![("margin-top", "32px")]);
    add("ml-16", vec![("margin-left", "64px")]);
    add("mb-2", vec![("margin-bottom", "8px")]);
    add("mx-0", vec![("margin-left", "0px"), ("margin-right", "0px")]);

    // sizing
    add("w-8", vec![("width", "32px")]);
    add("w-12", vec![("width", "48px")]);
    add("w-16", vec![("width", "64px")]);
    add("w-20", vec![("width", "80px")]);
    add("w-24", vec![("width", "96px")]);
    add("w-32", vec![("width", "128px")]);
    add("w-40", vec![("width", "160px")]);
    add("w-48", vec![("width", "192px")]);
    add("w-64", vec![("width", "256px")]);
    add("w-72", vec![("width", "288px")]);
    add("w-96", vec![("width", "384px")]);
    add("w-full", vec![("width", "100%")]);
    add("w-auto", vec![("width", "auto")]);
    add("h-6", vec![("height", "24px")]);
    add("h-8", vec![("height", "32px")]);
    add("h-10", vec![("height", "40px")]);
    add("h-12", vec![("height", "48px")]);
    add("h-16", vec![("height", "64px")]);
    add("h-20", vec![("height", "80px")]);
    add("h-24", vec![("height", "96px")]);
    add("h-40", vec![("height", "160px")]);
    add("min-w-48", vec![("min-width", "192px")]);
    add("min-w-64", vec![("min-width", "256px")]);
    add("max-w-lg", vec![("max-width", "512px")]);

    // typography
    add("text-left", vec![("text-align", "left")]);
    add("text-center", vec![("text-align", "center")]);
    add("text-right", vec![("text-align", "right")]);
    add("text-xs", vec![("font-size", "12px"), ("line-height", "16px")]);
    add("text-sm", vec![("font-size", "14px"), ("line-height", "20px")]);
    add("text-base", vec![("font-size", "16px"), ("line-height", "24px")]);
    add("text-lg", vec![("font-size", "18px"), ("line-height", "28px")]);
    add("text-xl", vec![("font-size", "20px"), ("line-height", "28px")]);
    add("text-2xl", vec![("font-size", "24px"), ("line-height", "32px")]);
    add("text-3xl", vec![("font-size", "30px"), ("line-height", "36px")]);
    add("text-4xl", vec![("font-size", "36px"), ("line-height", "40px")]);
    add("font-medium", vec![("font-weight", "500")]);
    add("font-semibold", vec![("font-weight", "600")]);
    add("font-bold", vec![("font-weight", "700")]);
    add("font-thin", vec![("font-weight", "100")]);
    add("font-light", vec![("font-weight", "300")]);
    add("font-normal", vec![("font-weight", "400")]);
    add("font-black", vec![("font-weight", "900")]);
    add("leading-none", vec![("line-height", "1")]);
    add("leading-tight", vec![("line-height", "1.25")]);
    add("leading-normal", vec![("line-height", "1.5")]);
    add("leading-loose", vec![("line-height", "2")]);

    // text colors (one per family used on the page)
    add("text-white", vec![("color", "#ffffff")]);
    add("text-slate-400", vec![("color", "#94a3b8")]);
    add("text-slate-500", vec![("color", "#64748b")]);
    add("text-slate-600", vec![("color", "#475569")]);
    add("text-slate-700", vec![("color", "#334155")]);
    add("text-slate-800", vec![("color", "#1e293b")]);
    add("text-slate-900", vec![("color", "#0f172a")]);
    add("text-amber-600", vec![("color", "#d97706")]);
    add("text-red-600", vec![("color", "#dc2626")]);
    add("text-orange-600", vec![("color", "#ea580c")]);
    add("text-green-600", vec![("color", "#16a34a")]);
    add("text-emerald-600", vec![("color", "#059669")]);
    add("text-teal-600", vec![("color", "#0d9488")]);
    add("text-sky-600", vec![("color", "#0284c7")]);
    add("text-blue-600", vec![("color", "#2563eb")]);
    add("text-indigo-600", vec![("color", "#4f46e5")]);
    add("text-violet-600", vec![("color", "#7c3aed")]);
    add("text-pink-600", vec![("color", "#db2777")]);
    add("text-green-800", vec![("color", "#166534")]);
    add("text-amber-800", vec![("color", "#92400e")]);
    add("text-red-800", vec![("color", "#991b1b")]);
    add("text-indigo-500", vec![("color", "#6366f1")]);

    // backgrounds
    add("bg-white", vec![("background-color", "#ffffff")]);
    add("bg-black", vec![("background-color", "#000000")]);
    add("bg-slate-50", vec![("background-color", "#f8fafc")]);
    add("bg-slate-100", vec![("background-color", "#f1f5f9")]);
    add("bg-slate-200", vec![("background-color", "#e2e8f0")]);
    add("bg-slate-300", vec![("background-color", "#cbd5e1")]);
    add("bg-slate-800", vec![("background-color", "#1e293b")]);
    add("bg-slate-900", vec![("background-color", "#0f172a")]);
    add("bg-slate-400", vec![("background-color", "#94a3b8")]);
    add("bg-slate-500", vec![("background-color", "#64748b")]);
    add("bg-slate-600", vec![("background-color", "#475569")]);
    add("bg-slate-700", vec![("background-color", "#334155")]);
    add("bg-slate-950", vec![("background-color", "#020617")]);
    add("bg-red-500", vec![("background-color", "#ef4444")]);
    add("bg-red-100", vec![("background-color", "#fee2e2")]);
    add("bg-orange-400", vec![("background-color", "#fb923c")]);
    add("bg-orange-500", vec![("background-color", "#f97316")]);
    add("bg-orange-600", vec![("background-color", "#ea580c")]);
    add("bg-amber-300", vec![("background-color", "#fcd34d")]);
    add("bg-amber-400", vec![("background-color", "#fbbf24")]);
    add("bg-amber-500", vec![("background-color", "#f59e0b")]);
    add("bg-amber-100", vec![("background-color", "#fef3c7")]);
    add("bg-yellow-500", vec![("background-color", "#eab308")]);
    add("bg-lime-500", vec![("background-color", "#84cc16")]);
    add("bg-green-100", vec![("background-color", "#dcfce7")]);
    add("bg-green-500", vec![("background-color", "#22c55e")]);
    add("bg-emerald-300", vec![("background-color", "#6ee7b7")]);
    add("bg-emerald-400", vec![("background-color", "#34d399")]);
    add("bg-emerald-500", vec![("background-color", "#10b981")]);
    add("bg-teal-200", vec![("background-color", "#99f6e4")]);
    add("bg-teal-400", vec![("background-color", "#2dd4bf")]);
    add("bg-teal-500", vec![("background-color", "#14b8a6")]);
    add("bg-cyan-400", vec![("background-color", "#22d3ee")]);
    add("bg-cyan-500", vec![("background-color", "#06b6d4")]);
    add("bg-sky-300", vec![("background-color", "#7dd3fc")]);
    add("bg-sky-500", vec![("background-color", "#0ea5e9")]);
    add("bg-blue-500", vec![("background-color", "#3b82f6")]);
    add("bg-indigo-400", vec![("background-color", "#818cf8")]);
    add("bg-indigo-500", vec![("background-color", "#6366f1")]);
    add("bg-indigo-50", vec![("background-color", "#eef2ff")]);
    add("bg-indigo-600", vec![("background-color", "#4f46e5")]);
    add("bg-violet-400", vec![("background-color", "#a78bfa")]);
    add("bg-violet-500", vec![("background-color", "#8b5cf6")]);
    add("bg-violet-600", vec![("background-color", "#7c3aed")]);
    add("bg-purple-500", vec![("background-color", "#a855f7")]);
    add("bg-fuchsia-500", vec![("background-color", "#d946ef")]);
    add("bg-pink-400", vec![("background-color", "#f472b6")]);
    add("bg-pink-500", vec![("background-color", "#ec4899")]);
    add("bg-rose-300", vec![("background-color", "#fda4af")]);
    add("bg-rose-400", vec![("background-color", "#fb7185")]);
    add("bg-rose-500", vec![("background-color", "#f43f5e")]);
    add("bg-rose-600", vec![("background-color", "#e11d48")]);

    // borders & radius
    add("border", vec![
        ("border-width", "1px"),
        ("border-style", "solid"),
        ("border-color", "#e5e7eb"),
    ]);
    add("border-2", vec![
        ("border-width", "2px"),
        ("border-style", "solid"),
        ("border-color", "#e5e7eb"),
    ]);
    add("border-4", vec![
        ("border-width", "4px"),
        ("border-style", "solid"),
        ("border-color", "#e5e7eb"),
    ]);
    add("border-8", vec![
        ("border-width", "8px"),
        ("border-style", "solid"),
        ("border-color", "#e5e7eb"),
    ]);
    add("border-slate-200", vec![("border-color", "#e2e8f0")]);
    add("border-slate-300", vec![("border-color", "#cbd5e1")]);
    add("border-slate-400", vec![("border-color", "#94a3b8")]);
    add("border-indigo-400", vec![("border-color", "#818cf8")]);
    add("border-indigo-500", vec![("border-color", "#6366f1")]);
    add("border-indigo-600", vec![("border-color", "#4f46e5")]);
    add("border-cyan-600", vec![("border-color", "#0891b2")]);
    add("border-cyan-700", vec![("border-color", "#0e7490")]);
    add("border-violet-600", vec![("border-color", "#7c3aed")]);
    add("border-violet-700", vec![("border-color", "#6d28d9")]);
    add("border-violet-800", vec![("border-color", "#5b21b6")]);
    add("border-emerald-500", vec![("border-color", "#10b981")]);
    add("border-emerald-600", vec![("border-color", "#059669")]);
    add("border-emerald-700", vec![("border-color", "#047857")]);
    add("border-teal-500", vec![("border-color", "#14b8a6")]);
    add("border-rose-500", vec![("border-color", "#f43f5e")]);
    add("border-sky-500", vec![("border-color", "#0ea5e9")]);
    add("border-amber-500", vec![("border-color", "#f59e0b")]);
    add("border-none", vec![("border-style", "none")]);
    add("rounded", vec![("border-radius", "4px")]);
    add("rounded-sm", vec![("border-radius", "2px")]);
    add("rounded-md", vec![("border-radius", "6px")]);
    add("rounded-lg", vec![("border-radius", "8px")]);
    add("rounded-xl", vec![("border-radius", "12px")]);
    add("rounded-2xl", vec![("border-radius", "16px")]);
    add("rounded-3xl", vec![("border-radius", "24px")]);
    add("rounded-full", vec![("border-radius", "9999px")]);
    add("rounded-none", vec![("border-radius", "0px")]);

    // overflow / whitespace
    add("overflow-y-auto", vec![("overflow-y", "auto")]);
    add("overflow-hidden", vec![("overflow", "hidden")]);
    add("overflow-visible", vec![("overflow", "visible")]);
    add("whitespace-nowrap", vec![("white-space", "nowrap")]);

    t
}

/// Valid upstream Tailwind classes that are OUTSIDE velqu's declared v0
/// subset (or the renderer profile): each must produce a diagnostic and
/// emit no rule. Classified per the corrected FINDINGS.md — e.g.
/// `max-w-64` is a legitimate Tailwind v3.4 utility (16rem) that velqu's
/// v0 profile does not implement (named breakpoints only).
fn deferred_table() -> Vec<(&'static str, &'static str)> {
    vec![
        ("max-w-64", "valid upstream Tailwind v3.4 utility (spacing-scale max-width, 16rem — v3.4 added the spacing scale to max-width); velqu's v0 implements named breakpoints only"),
        ("hover:bg-indigo-700", "variant classes are not applied in the v0 static profile"),
        ("md:flex", "variant classes are not applied in the v0 static profile"),
        ("absolute", "positioning is outside the renderer profile v0"),
        ("shadow-md", "shadows are deferred in profile v0"),
        ("opacity-50", "opacity is deferred in profile v0"),
        ("italic", "text decoration is outside profile v0"),
        ("underline", "text decoration is outside profile v0"),
        ("tracking-wide", "letter-spacing is em-based and deferred in profile v0"),
        ("border-dashed", "border-style is outside the renderer profile (solid/none only)"),
        ("inline-block", "display value outside the renderer profile"),
        ("w-screen", "viewport units are deferred in profile v0"),
        ("m-auto", "margin auto is deferred in profile v0"),
        ("space-x-2", "the space-* family is outside profile v0"),
        ("list-disc", "the list-* family is outside profile v0"),
        ("aspect-video", "the aspect-* family is outside profile v0"),
        ("transition-colors", "transitions are deferred in profile v0"),
        ("truncate", "deferred utility (profile v0)"),
        ("uppercase", "deferred utility (profile v0)"),
        ("font-sans", "font families are deferred (single bundled face)"),
        ("cursor-pointer", "cursor is deferred in profile v0"),
        ("z-10", "z-index is deferred in profile v0"),
    ]
}

fn main() {
    let html = std::fs::read_to_string("showcase/index.html").expect("showcase/index.html");
    let mut page_classes: Vec<String> = Vec::new();
    for part in html.split("class=\"") {
        let Some(end) = part.find('"') else { continue };
        let candidate = &part[..end];
        // Skip non-attribute matches (text inside comments, prose).
        if candidate.contains('<') || candidate.contains('>') {
            continue;
        }
        for class in candidate.split_whitespace() {
            if !class.is_empty() && !page_classes.contains(&class.to_owned()) {
                page_classes.push(class.to_owned());
            }
        }
    }
    // Tag names used as styling hooks (vv-*) and plain HTML-classed strings
    // that are not utilities are the page's own hooks; the vv- prefix is the
    // documented author namespace velqu-tailwind skips.
    let page_utility_classes: Vec<&str> = page_classes
        .iter()
        .map(|s| s.as_str())
        .filter(|c| !c.starts_with("vv-"))
        .collect();

    let expected = expected_table();
    let deferred = deferred_table();
    let deferred: BTreeMap<&str, &str> = deferred.into_iter().collect();

    let mut unsupported = Vec::new();
    let mut failures = Vec::new();

    for class in &page_utility_classes {
        if expected.contains_key(class) {
            continue; // checked below
        }
        if deferred.contains_key(class) {
            failures.push(format!("page uses deferred class {class:?} — the page must stay inside the supported subset (profile probes live in this check's tables, not the page)"));
            continue;
        }
        failures.push(format!("page class {class:?} is in NEITHER the supported nor the deferred table — classify it"));
    }

    // Compile everything at once; the compiler reports diagnostics per class.
    let all: Vec<&str> = expected
        .keys()
        .chain(deferred.keys())
        .copied()
        .collect();
    let build = compile_utilities(&all);
    let css = build.css.clone();
    let diag_classes: Vec<&str> = build.diagnostics.iter().map(|d| d.class.as_str()).collect();

    // 1. Supported classes: exact declarations, no diagnostics.
    for (class, want) in &expected {
        if diag_classes.contains(class) {
            failures.push(format!("supported class {class:?} produced a diagnostic — profile regression"));
            continue;
        }
        let rule = format!(".{class} {{ }}");
        let want_css: Vec<String> = want
            .iter()
            .map(|(p, v)| format!("{p}: {v};"))
            .collect();
        let expected_rule = format!(".{class} {{ {} }}\n", want_css.join(" "));
        if !css.contains(&expected_rule) {
            failures.push(format!(
                "supported class {class:?} does not compile to the pinned declarations\n  want: {expected_rule:?}"
            ));
        }
        let _ = rule;
    }

    // 2. Deferred classes: must have a diagnostic and must NOT emit a rule.
    for class in deferred.keys() {
        if !diag_classes.contains(class) {
            failures.push(format!(
                "deferred class {class:?} compiled WITHOUT a diagnostic — silent omission"
            ));
        }
        if css.contains(&format!(".{class} {{")) {
            failures.push(format!("deferred class {class:?} emitted a CSS rule"));
        }
    }

    // 3. Unsupported classes the page uses must be diagnosed at render time
    // too — assert the compiler flags each of them.
    for class in &page_utility_classes {
        if expected.contains_key(class) {
            continue;
        }
        unsupported.push(*class);
    }

    if !failures.is_empty() {
        eprintln!("profile-check FAILED ({}):", failures.len());
        for failure in &failures {
            eprintln!("  - {failure}");
        }
        if !unsupported.is_empty() {
            eprintln!("  (unclassified page classes: {unsupported:?})");
        }
        std::process::exit(5);
    }
    println!(
        "profile-check OK: {} supported classes pinned to exact declarations, \
         {} deferred classes confirmed diagnosed (no silent omissions), \
         page classes fully classified",
        expected.len(),
        deferred.len(),
    );
}

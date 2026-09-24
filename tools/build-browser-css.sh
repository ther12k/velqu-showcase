#!/bin/sh
# Builds the browser reference CSS: REAL Tailwind v3.4 utilities from the
# page's classes, plus exactly the two base rules Tailwind's utilities
# presuppose in every real project (box-sizing, border base) and the
# renderer's single font face. Nothing else: native defaults stay visible.
set -e
cd "$(dirname "$0")/.."
printf '@tailwind utilities;\n' > /tmp/tw-input.css
npx -y tailwindcss@3.4.17 -i /tmp/tw-input.css -o /tmp/tw-core.css \
    --content showcase/index.html --minify
{
  cat /tmp/tw-core.css
  cat << 'PARITY'
/* parity base (documented in FINDINGS.md): (1) velqu-tailwind's
   preflight-lite emits * { box-sizing: border-box }; (2) Tailwind border
   utilities presuppose the preflight border base — without it Chromium
   computes border-width to 0 (border-style: none) and the border inputs
   are unequal, which misattributed a +2px delta to the engine. */
*,::before,::after{box-sizing:border-box;border-width:0;border-style:solid;border-color:#e5e7eb}
body{font-family:'DejaVu Sans',sans-serif}
PARITY
} > showcase/tw.css
echo "showcase/tw.css rebuilt ($(wc -c < showcase/tw.css) bytes)"

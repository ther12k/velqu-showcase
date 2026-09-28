#!/usr/bin/env bash
# Atomic deploy for velqu.rizeva.my.id (VelquView landing).
# build -> upload timestamped release -> flip symlinks -> restart ->
# health check. Rollback = flip symlinks back to a previous release.
#
# Requires: SSH key for opc@100.124.71.117 (pass via RIZEVA_KEY or
# default ~/Downloads/ssh-key-2026-07-03.key) and, first run only,
# the NPM proxy host + certificate for velqu.rizeva.my.id.
set -euo pipefail
cd "$(dirname "$0")/.."   # repo root (velqu-showcase)

KEY="${RIZEVA_KEY:-$HOME/Downloads/ssh-key-2026-07-03.key}"
HOST="opc@100.124.71.117"
APP=velqu-landing
REMOTE_ROOT="/home/opc/$APP"
PORT=8121
STAMP=$(date -u +%Y%m%d-%H%M%S)

echo "── building tw.css (always: cookbook.html + config are inputs too)"
printf '@tailwind base;\n@tailwind components;\n@tailwind utilities;\n' > /tmp/tw-input.css
# tailwind.config.js is REQUIRED: darkMode:'class' powers the theme toggle
(cd site && npx -y tailwindcss@3.4.17 -i /tmp/tw-input.css -o tw.css \
   -c tailwind.config.js --minify)

echo "── uploading release $STAMP"
ssh -i "$KEY" -o IdentitiesOnly=yes "$HOST" "mkdir -p $REMOTE_ROOT/releases/$STAMP/dist"
rsync -az -e "ssh -i $KEY -o IdentitiesOnly=yes" \
  site/index.html site/samples.html site/cookbook.html site/404.html site/robots.txt site/sitemap.xml site/tw.css site/app.css site/assets \
  "$HOST:$REMOTE_ROOT/releases/$STAMP/dist/"
rsync -az -e "ssh -i $KEY -o IdentitiesOnly=yes" \
  site/server.cjs "$HOST:$REMOTE_ROOT/releases/$STAMP/server/"

# Cache-bust compiled assets: HTML is no-cache but CSS is cached 1h —
# returning visitors must not keep the previous release's stylesheet.
ssh -i "$KEY" -o IdentitiesOnly=yes "$HOST" "
set -e
cd $REMOTE_ROOT/releases/$STAMP/dist
sed -i 's/app\.css\"/app.css?v=$STAMP\"/g; s/tw\.css\"/tw.css?v=$STAMP\"/g' index.html samples.html cookbook.html

cd $REMOTE_ROOT
mkdir -p data   # none today; convention kept for future state
ln -sfn releases/$STAMP/dist dist.new
ln -sfn releases/$STAMP/server server.new
mv -Tf dist.new dist && mv -Tf server.new server
if [ ! -f /etc/systemd/system/$APP.service ]; then
  sudo tee /etc/systemd/system/$APP.service >/dev/null <<UNIT
[Unit]
Description=VelquView landing (velqu.rizeva.my.id)
After=network-online.target

[Service]
Environment=VELQU_LANDING_PORT=$PORT
Environment=VELQU_LANDING_DIST=$REMOTE_ROOT/dist
WorkingDirectory=$REMOTE_ROOT
ExecStart=/usr/bin/node $REMOTE_ROOT/server/server.cjs
Restart=on-failure
User=opc

[Install]
WantedBy=multi-user.target
UNIT
  sudo systemctl daemon-reload && sudo systemctl enable --now $APP
else
  sudo systemctl restart $APP
fi
ls -1dt releases/*/ | tail -n +4 | xargs -r rm -rf
sleep 1
systemctl is-active $APP
curl -sfS http://127.0.0.1:$PORT/ >/dev/null && echo ' local: OK'
"

echo "── verifying public URL"
curl -sfSI "https://velqu.rizeva.my.id" | head -3 && echo " ✓ live"

#!/usr/bin/env bash
# Build xguitar for WebAssembly and prepare for web serving.
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli
#
# Usage:
#   ./build-wasm.sh
#   cd web-dist && python3 serve.py
#   Open http://localhost:8080

set -euo pipefail

PROFILE="${1:-release}"
OUT_DIR="web-dist"
TARGET="wasm32-unknown-unknown"

echo "==> Building for $TARGET ($PROFILE)..."
if [ "$PROFILE" = "release" ]; then
    cargo build --target "$TARGET" --release
    BIN="target/$TARGET/release/m_guitar.wasm"
else
    cargo build --target "$TARGET"
    BIN="target/$TARGET/debug/m_guitar.wasm"
fi

echo "==> Running wasm-bindgen..."
wasm-bindgen --no-typescript --target web \
    --out-dir "$OUT_DIR" \
    --out-name m_guitar \
    "$BIN"

echo "==> Copying static assets..."
cp index.html "$OUT_DIR/"

# Create a proper HTTP server script with WASM MIME type
cat > "$OUT_DIR/serve.py" << 'PYEOF'
import http.server, socketserver

class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        '.wasm': 'application/wasm',
    }

print('xguitar — http://localhost:8080')
socketserver.ThreadingTCPServer(('127.0.0.1', 8080), Handler).serve_forever()
PYEOF

echo ""
echo "==> Done! Output in $OUT_DIR/"
echo "    Size: $(du -sh "$OUT_DIR/m_guitar_bg.wasm" | cut -f1)"
echo ""
echo "    Serve:  cd $OUT_DIR && python3 serve.py"
echo "    Open:   http://localhost:8080"

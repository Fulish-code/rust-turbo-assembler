#!/usr/bin/env bash
# Crea el paquete para publicar en GitHub Releases:
#   dist/rust-tasm-linux-x86_64.tar.gz
# Compila con cargo-zigbuild contra glibc 2.31 para que funcione en
# Ubuntu 20.04+, Debian 11+, Mint, Fedora, Arch, etc.
# Requiere: zig y cargo-zigbuild (cargo install cargo-zigbuild).
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/rust-tasm-target}"
TARGET=x86_64-unknown-linux-gnu

cargo zigbuild --release --target "$TARGET.2.31"
rm -rf dist && mkdir -p dist/rust-tasm
install -m755 "$CARGO_TARGET_DIR/$TARGET/release/rust-tasm" dist/rust-tasm/rust-tasm
cp -r BIN docs locales examples assets README.md LICENSE instalar.sh dist/rust-tasm/
cat > dist/rust-tasm/LEEME.txt <<'EOF'
Rust Turbo Assembler
====================
Para instalar: abre una terminal en esta carpeta y escribe

    ./instalar.sh

(o haz clic derecho sobre "instalar.sh" > "Ejecutar como programa").
Después búscalo en el menú de aplicaciones como "Rust Turbo Assembler".
EOF
tar -C dist -czf dist/rust-tasm-linux-x86_64.tar.gz rust-tasm
(cd dist && sha256sum rust-tasm-linux-x86_64.tar.gz > rust-tasm-linux-x86_64.tar.gz.sha256)
ls -la dist

#!/bin/sh
# Builds the Linux package (static musl binaries) for tools/release.ps1, or on
# its own on any Linux:  sh tools/release-linux.sh v0.2.0 dist [notest]
set -e
tag="$1"
out="${2:-dist}"
target=x86_64-unknown-linux-musl
[ -n "$tag" ] || { echo "usage: $0 <tag> [out-dir] [notest]" >&2; exit 2; }
cd "$(dirname "$0")/.."

# cargo may not be on the PATH of a non-login shell (wsl bash ...).
command -v cargo >/dev/null 2>&1 || PATH="$HOME/.cargo/bin:$PATH"
command -v cargo >/dev/null 2>&1 || { echo "error: cargo not found (install Rust: https://rustup.rs)" >&2; exit 1; }
if ! rustup target list --installed | grep -qx "$target"; then
    echo "Adding the Rust target $target..."
    rustup target add "$target"
fi
command -v musl-gcc >/dev/null 2>&1 || {
    echo "error: musl-gcc not found. Install it once with:  sudo apt install musl-tools" >&2
    exit 1
}

# Outside /mnt/... the build is faster and does not mix with the Windows one.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/fun-tools-target}"
[ "$3" = notest ] || cargo test --release --locked --target "$target"
cargo build --release --locked --target "$target"

name="fun_tools-$tag-linux-x86_64"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir "$stage/$name"
find "$CARGO_TARGET_DIR/$target/release" -maxdepth 1 -type f -perm -u+x -name 'r*' -exec cp {} "$stage/$name/" \;
cp README.md README.pt.md LICENSE "$stage/$name/"
mkdir -p "$out"
tar -czf "$out/$name.tar.gz" -C "$stage" "$name"
echo "$out/$name.tar.gz: $(ls "$stage/$name" | grep -c '^r') tools"

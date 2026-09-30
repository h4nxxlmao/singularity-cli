#!/usr/bin/env sh
# install.sh — sgl installer
# Usage: curl -fsSL https://getsingularity.lol/cli/install.sh | sh

set -eu

REPO="singularity-cli/singularity-cli"
BIN="sgl"
INSTALL_DIR="${SGL_INSTALL_DIR:-$HOME/.local/bin}"

# ── helpers ──────────────────────────────────────────────────────────────────

step() {
  label="$1"
  value="$2"
  printf "  → %-26s %s\n" "$label" "$value"
}

die() {
  echo "error: $1" >&2
  exit 1
}

# ── detect platform ──────────────────────────────────────────────────────────

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)  os="linux" ;;
  Darwin) os="darwin" ;;
  *)      die "unsupported OS: $OS" ;;
esac

case "$ARCH" in
  x86_64)          arch="x86_64" ;;
  amd64)           arch="x86_64" ;;
  aarch64|arm64)   arch="aarch64" ;;
  *)               die "unsupported architecture: $ARCH" ;;
esac

if [ "$os" = "linux" ]; then
  target="${arch}-unknown-linux-musl"
else
  target="${arch}-apple-darwin"
fi

step "detecting platform ........." "$os-$arch"

# ── latest version ───────────────────────────────────────────────────────────

VERSION="${SGL_VERSION:-}"
if [ -z "$VERSION" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')"
fi

[ -z "$VERSION" ] && die "could not determine latest version"

ARCHIVE="sgl-${VERSION}-${target}.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$ARCHIVE"

step "downloading sgl $VERSION .." "done"
curl -fsSL "$URL" -o "/tmp/$ARCHIVE" || die "download failed"

# ── verify checksum ──────────────────────────────────────────────────────────

SUMS_URL="https://github.com/$REPO/releases/download/$VERSION/SHA256SUMS"
curl -fsSL "$SUMS_URL" -o /tmp/SHA256SUMS 2>/dev/null || true

if [ -f /tmp/SHA256SUMS ]; then
  expected="$(grep "$ARCHIVE" /tmp/SHA256SUMS | awk '{print $1}')"
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "/tmp/$ARCHIVE" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "/tmp/$ARCHIVE" | awk '{print $1}')"
  else
    actual=""
  fi
  if [ -n "$actual" ] && [ "$actual" != "$expected" ]; then
    die "checksum mismatch for $ARCHIVE"
  fi
  step "verifying checksum ........." "ok"
else
  step "verifying checksum ........." "skipped (no SHA256SUMS)"
fi

# ── install ──────────────────────────────────────────────────────────────────

mkdir -p "$INSTALL_DIR"
tar -xzf "/tmp/$ARCHIVE" -C "$INSTALL_DIR"
chmod +x "$INSTALL_DIR/$BIN"
rm -f "/tmp/$ARCHIVE" /tmp/SHA256SUMS

step "installing to $INSTALL_DIR" "done"

echo ""
echo "sgl installed. Run \`sgl\` in a project directory."
echo ""

# ── PATH warning ─────────────────────────────────────────────────────────────

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "warning: $INSTALL_DIR is not on your PATH."
    echo "  Add this to your shell profile:"
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac

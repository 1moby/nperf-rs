BINARY   := nperf-rs
VERSION  := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
DIST     := dist

# Targets
LINUX_AMD64  := x86_64-unknown-linux-musl
LINUX_ARM64  := aarch64-unknown-linux-musl
MACOS_INTEL  := x86_64-apple-darwin
MACOS_ARM    := aarch64-apple-darwin
WIN_X86      := i686-pc-windows-msvc
WIN_AMD64    := x86_64-pc-windows-msvc
WIN_ARM64    := aarch64-pc-windows-msvc

.PHONY: all clean linux macos windows

all: linux macos windows

clean:
	rm -rf $(DIST)
	cargo clean

$(DIST):
	mkdir -p $(DIST)

# ── Linux (musl, static) ──────────────────────────
linux: linux-amd64 linux-arm64

linux-amd64: $(DIST)
	cargo build --release --target $(LINUX_AMD64)
	cp target/$(LINUX_AMD64)/release/$(BINARY) $(DIST)/$(BINARY)-linux-amd64
	-upx --best --lzma $(DIST)/$(BINARY)-linux-amd64 2>/dev/null || true

linux-arm64: $(DIST)
	cross build --release --target $(LINUX_ARM64)
	cp target/$(LINUX_ARM64)/release/$(BINARY) $(DIST)/$(BINARY)-linux-arm64
	-upx --best --lzma $(DIST)/$(BINARY)-linux-arm64 2>/dev/null || true

# ── macOS ──────────────────────────────────────────
macos: macos-intel macos-arm

macos-intel: $(DIST)
	cargo build --release --target $(MACOS_INTEL)
	cp target/$(MACOS_INTEL)/release/$(BINARY) $(DIST)/$(BINARY)-macos-intel

macos-arm: $(DIST)
	cargo build --release --target $(MACOS_ARM)
	cp target/$(MACOS_ARM)/release/$(BINARY) $(DIST)/$(BINARY)-macos-arm64

# ── Windows ────────────────────────────────────────
windows: windows-x86 windows-amd64 windows-arm64

windows-x86: $(DIST)
	cargo build --release --target $(WIN_X86)
	cp target/$(WIN_X86)/release/$(BINARY).exe $(DIST)/$(BINARY)-windows-x86.exe

windows-amd64: $(DIST)
	cargo build --release --target $(WIN_AMD64)
	cp target/$(WIN_AMD64)/release/$(BINARY).exe $(DIST)/$(BINARY)-windows-amd64.exe

windows-arm64: $(DIST)
	cargo build --release --target $(WIN_ARM64)
	cp target/$(WIN_ARM64)/release/$(BINARY).exe $(DIST)/$(BINARY)-windows-arm64.exe

# ── Convenience ────────────────────────────────────
release: all
	@echo ""
	@echo "Built binaries:"
	@ls -lh $(DIST)/
	@echo ""
	@echo "SHA256:"
	@cd $(DIST) && sha256sum * 2>/dev/null || shasum -a 256 *

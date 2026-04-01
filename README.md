# nperf-rs

Network speed benchmark CLI built in Rust. Multi-server parallel testing with 87 Thai nperf servers. Single binary, zero dependencies.

เครื่องมือทดสอบความเร็วเครือข่ายแบบ CLI สร้างด้วย Rust ทดสอบหลายเซิร์ฟเวอร์พร้อมกัน รองรับ 87 เซิร์ฟเวอร์ nperf ในไทย ไฟล์เดียว ไม่ต้องติดตั้งอะไรเพิ่ม

## Features / คุณสมบัติ

- **Multi-server parallel testing** — Test against multiple servers simultaneously with real-time per-server throughput bars
  ทดสอบหลายเซิร์ฟเวอร์พร้อมกัน แสดงผลแบบ real-time พร้อม bar chart
- **Latency + Download + Upload** — Full test suite with jitter measurement, multi-threaded throughput, slow-start exclusion
  ชุดทดสอบครบ วัด Latency, Download, Upload พร้อม jitter และ multi-thread
- **87 Thai servers built-in** — Filter by provider (True, AIS, 3BB, DTAC, NT) or city. Random server selection
  87 เซิร์ฟเวอร์ในไทย กรองตามผู้ให้บริการหรือจังหวัด สุ่มเลือกได้
- **Pure Rust, single binary** — No C dependencies. Static binary with rustls TLS. Under 2MB (compressible to ~900KB with UPX)
  Rust ล้วน ไม่มี dependency ภายนอก ไฟล์เดียวพร้อมใช้
- **Colorful TUI output** — Live progress bars, color-coded results, JSON output mode
  แสดงผลสีสันสวยงาม รองรับ JSON output
- **Cross-platform** — Linux (amd64/arm64), macOS (Intel/Apple Silicon), Windows (amd64/x86/arm64)
  รองรับทุกแพลตฟอร์ม 7 รุ่น

## Quick Start / เริ่มต้น

### Download / ดาวน์โหลด

**macOS (Apple Silicon):**
```bash
curl -Lo nperf-rs https://github.com/1moby/nperf-rs/releases/latest/download/nperf-rs-macos-arm64
xattr -d com.apple.quarantine nperf-rs
chmod +x nperf-rs
./nperf-rs
```

**macOS (Intel):**
```bash
curl -Lo nperf-rs https://github.com/1moby/nperf-rs/releases/latest/download/nperf-rs-macos-intel
xattr -d com.apple.quarantine nperf-rs
chmod +x nperf-rs
./nperf-rs
```

> **Note:** The `xattr` command removes the macOS quarantine flag that causes the "Apple could not verify" warning.
> **หมายเหตุ:** คำสั่ง `xattr` ลบ quarantine flag ที่ทำให้ macOS แจ้งเตือน "Apple could not verify"

**Linux (x86_64):**
```bash
curl -Lo nperf-rs https://github.com/1moby/nperf-rs/releases/latest/download/nperf-rs-linux-amd64
chmod +x nperf-rs
./nperf-rs
```

**Linux (ARM64):**
```bash
curl -Lo nperf-rs https://github.com/1moby/nperf-rs/releases/latest/download/nperf-rs-linux-arm64
chmod +x nperf-rs
./nperf-rs
```

**Windows (PowerShell):**
```powershell
Invoke-WebRequest -Uri https://github.com/1moby/nperf-rs/releases/latest/download/nperf-rs-windows-amd64.exe -OutFile nperf-rs.exe
.\nperf-rs.exe
```

> **Note:** If Windows SmartScreen warns "unrecognized app", click **More info** → **Run anyway**.
> **หมายเหตุ:** ถ้า Windows SmartScreen แจ้งเตือน กด **More info** → **Run anyway**

For other platforms (Windows x86, Windows ARM64), see [all releases](https://github.com/1moby/nperf-rs/releases).

### Build from Source / Build จาก Source

```bash
cargo install --git https://github.com/1moby/nperf-rs
```

Or clone and build:
```bash
git clone https://github.com/1moby/nperf-rs.git
cd nperf-rs
cargo build --release
./target/release/nperf-rs
```

## Usage / วิธีใช้

```bash
# Default: test 3 Bangkok servers
# ค่าเริ่มต้น: ทดสอบ 3 เซิร์ฟเวอร์กรุงเทพ
nperf-rs

# List all servers / แสดงรายชื่อเซิร์ฟเวอร์
nperf-rs --list

# Filter by provider / กรองตามผู้ให้บริการ
nperf-rs -f AIS

# 5 random servers, 10 threads
# สุ่ม 5 เซิร์ฟเวอร์, 10 threads
nperf-rs --random 5 -t 10

# Specific server + JSON output
# เจาะจงเซิร์ฟเวอร์ + JSON
nperf-rs -u wss://th-ais-bangkok-01-10g.nperf.net/wsock --json

# Download only, custom duration
# ทดสอบเฉพาะดาวน์โหลด กำหนดเวลาเอง
nperf-rs --no-latency --no-upload --download-duration 30

# Latency only / ทดสอบเฉพาะ Latency
nperf-rs --no-download --no-upload
```

## Platforms / แพลตฟอร์ม

| Platform | Binary | Architecture |
|----------|--------|-------------|
| Linux | `nperf-rs-linux-amd64` | x86_64 (musl static) |
| Linux | `nperf-rs-linux-arm64` | AArch64 (musl static) |
| macOS | `nperf-rs-macos-intel` | x86_64 (Intel) |
| macOS | `nperf-rs-macos-arm64` | AArch64 (Apple Silicon) |
| Windows | `nperf-rs-windows-amd64.exe` | x86_64 |
| Windows | `nperf-rs-windows-x86.exe` | x86 (32-bit) |
| Windows | `nperf-rs-windows-arm64.exe` | ARM64 (Snapdragon) |

## Website

[1moby.github.io/nperf-rs](https://1moby.github.io/nperf-rs) — Landing page with server list and download links.

## License

MIT

# MyBuds

Desktop manager for Huawei FreeBuds and HONOR Earbuds headphones on Linux. Built with Rust.

Based on [OpenFreebuds](https://github.com/melianmiko/OpenFreebuds) by melianmiko — rewritten from Python to Rust for better performance and native system integration.

## Features

- **GUI** (Iced) and **TUI** (Ratatui) interfaces
- **System tray** integration with battery indicator
- ANC mode switching (Normal / Cancellation / Awareness)
- Battery monitoring (global + per-earbud + case)
- Equalizer presets
- Gesture customization (double tap, triple tap, long tap, swipe)
- Dual-device connection management
- Auto-pause, low latency, sound quality preferences
- Auto-reconnect with exponential backoff
- Desktop notifications

## Supported Devices

| Feature | Pro 3 / Pro 4 / FreeClip | Pro 2 / Pro | 5i | 6i | 4i / HONOR Earbuds 2 | SE 2 | 5 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Battery | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| ANC | Full (dynamic) | Full (dynamic) | Cancel + Aware | Cancel + Aware | Basic | :x: | Cancel only |
| Equalizer | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :x: | :white_check_mark: | :white_check_mark: |
| Double Tap | :white_check_mark: | :white_check_mark: | :white_check_mark: (L/R) | :white_check_mark: (L/R) | :white_check_mark: | :white_check_mark: (L/R) | :white_check_mark: (L/R) |
| Triple Tap | :x: | :x: | :white_check_mark: | :white_check_mark: | :x: | :white_check_mark: | :white_check_mark: |
| Long Tap | Split (ANC) | Split (ANC) | Split (ANC) | Split (ANC) | Basic | Hold | Split (ANC) |
| Swipe | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :x: | :x: | :white_check_mark: |
| Auto Pause | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :x: | :white_check_mark: |
| Low Latency | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :x: | :white_check_mark: | :white_check_mark: |
| Sound Quality | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :x: | :x: | :white_check_mark: |
| Dual Connect | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :x: | :x: | :x: |

Devices not listed above will use a **generic probe** profile that attempts to detect available features automatically.

## Requirements

- Linux with BlueZ (Bluetooth stack)
- Paired Huawei FreeBuds or HONOR Earbuds device
- D-Bus (for system tray / StatusNotifierItem)

## Building

```bash
cargo build --release
```

The binary will be at `target/release/mybuds`.

## Usage

```bash
# GUI mode (default)
mybuds

# Terminal UI mode
mybuds --tui
```

### Configuration

Config file: `~/.config/mybuds/config.toml`

```toml
device_address = "AA:BB:CC:DD:EE:FF"  # optional, auto-detected if omitted
device_name = "HUAWEI FreeBuds Pro 3"  # optional
auto_connect = true                     # reconnect automatically (default: true)
start_minimized = false                 # start in system tray (default: false)
```

### Logging

```bash
RUST_LOG=mybuds=debug mybuds
```

In TUI mode, logs are written to `/tmp/mybuds.log`.

## Architecture

```
src/
├── bluetooth/     # BlueZ RFCOMM/SPP connection and device scanning
├── protocol/      # Huawei SPP wire protocol (packet framing, CRC16, command IDs)
├── device/        # Feature handlers (ANC, battery, EQ, gestures, etc.)
│   └── models/    # Per-device profiles defining supported features
├── ui/            # Iced GUI (pages + widgets)
├── tui/           # Ratatui terminal UI
├── tray/          # System tray (ksni / StatusNotifierItem)
└── config/        # TOML app config
```

## Acknowledgments

This project is based on the protocol reverse-engineering and implementation work done in [OpenFreebuds](https://github.com/melianmiko/OpenFreebuds) by [melianmiko](https://github.com/melianmiko).

## License

See [OpenFreebuds](https://github.com/melianmiko/OpenFreebuds) for the original license terms.

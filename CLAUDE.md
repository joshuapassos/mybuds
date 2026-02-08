# CLAUDE.md

## Project Overview

Rust rewrite of [OpenFreebuds](https://github.com/melianmiko/OpenFreebuds) — a desktop manager for Huawei FreeBuds / HONOR Earbuds headphones on Linux.

## Build & Run

```bash
cargo build                  # debug build
cargo build --release        # release build
cargo run                    # GUI mode
cargo run -- --tui           # TUI mode
RUST_LOG=mybuds=debug cargo run  # with debug logging
```

## Architecture

- **`src/protocol/`** — Huawei SPP wire protocol. Packets are `[0x5A][len:2BE][0x00][cmd:2B][TLV params][CRC16]`. Do not change the packet format without testing on real hardware.
- **`src/device/`** — Feature handlers implementing `DeviceHandler` trait. Each feature (ANC, battery, EQ, gestures) is a separate handler. Device profiles in `models/mod.rs` wire handlers to specific device models.
- **`src/bluetooth/`** — BlueZ RFCOMM/SPP via `bluer`. Uses raw libc sockets for RFCOMM (bluer doesn't expose RFCOMM connect directly). Connection runs in `spawn_blocking`.
- **`src/ui/`** — Iced 0.13 GUI. Pages under `pages/`, custom widgets under `widgets/`.
- **`src/tui/`** — Ratatui terminal UI. Same page structure as GUI.
- **`src/tray/`** — System tray via `ksni` (D-Bus StatusNotifierItem).
- **`src/config/`** — TOML config at `~/.config/mybuds/config.toml`.

## Key Patterns

- **PropertyStore** (`Arc<Mutex<HashMap<String, HashMap<String, String>>>>`) is the shared state between Bluetooth manager, UI, and tray. Properties are grouped by category (battery, anc, info, etc.).
- **Handler trait** — `DeviceHandler` has `handle_packet()` for incoming data and `init()` to request initial state. Handlers read/write to PropertyStore.
- **Dual UI** — GUI and TUI share the same PropertyStore and device layer. Only one runs at a time (`--tui` flag).
- **Reconnect loop** — `BluetoothManager::run_with_reconnect()` handles connection drops with exponential backoff.

## Adding a New Device

1. Create a profile function in `src/device/models/mod.rs`
2. Select which handlers apply (check hardware capabilities)
3. Add the Bluetooth device name to `profile_for_device()` match
4. Set the correct SPP port (typically 1 or 16)

## Adding a New Feature Handler

1. Create a new file in `src/device/` implementing `DeviceHandler`
2. Define command IDs in `src/protocol/commands.rs`
3. Add the handler to relevant device profiles in `models/mod.rs`
4. Add UI controls in both `src/ui/pages/` and `src/tui/pages/`

## Conventions

- Use `anyhow::Result` for fallible functions
- Use `tracing` macros (`info!`, `debug!`, `error!`) for logging
- Property keys are lowercase snake_case strings
- Command IDs are `(service_id, command_id)` tuples of `u8`

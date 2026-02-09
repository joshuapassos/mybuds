# CLAUDE.md

## Project Overview

Rust rewrite of [OpenFreebuds](https://github.com/melianmiko/OpenFreebuds) — a desktop manager for Huawei FreeBuds / HONOR Earbuds and Apple AirPods headphones on Linux.

## Build & Run

```bash
cargo build                  # debug build
cargo build --release        # release build
cargo run                    # GUI mode
cargo run -- --tui           # TUI mode
RUST_LOG=mybuds=debug cargo run  # with debug logging
```

## Architecture

- **`src/protocol/`** — Wire protocols for both Huawei SPP and Apple AACP.
  - `packet.rs` — Huawei SPP: `[0x5A][len:2BE][0x00][cmd:2B][TLV params][CRC16]`
  - `aap.rs` — Apple AACP: `[04 00 04 00][opcode][00][payload]` over L2CAP PSM 0x1001. AAP packets are mapped to `HuaweiSppPacket` for handler compatibility using prefix bytes: `[0xAA, opcode]` for general opcodes, `[0xA9, identifier]` for control command subtypes.
  - `commands.rs` — Huawei command ID constants
  - `crc.rs` — CRC-16 XModem for Huawei packets
- **`src/device/`** — Feature handlers implementing `DeviceHandler` trait. Each feature (ANC, battery, EQ, gestures) is a separate handler. Device profiles in `models/mod.rs` wire handlers to specific device models.
  - `airpods.rs` — AirPods-specific handlers (battery, ear detection, ANC/listening modes, conversational awareness, personalized volume, device info)
- **`src/bluetooth/`** — BlueZ connections via raw libc sockets (bluer doesn't expose RFCOMM/L2CAP connect directly). Connection runs in `spawn_blocking`.
  - `connection.rs` — RFCOMM/SPP for Huawei devices (SOCK_STREAM)
  - `l2cap.rs` — L2CAP for AirPods (SOCK_SEQPACKET, PSM 0x1001). Handles AAP handshake, feature flags, notification subscription, and translates AAP ↔ HuaweiSppPacket in recv/send loops.
  - `scanner.rs` — Device discovery via BlueZ D-Bus
- **`src/ui/`** — Iced 0.13 GUI. Pages under `pages/`, custom widgets under `widgets/`.
- **`src/tui/`** — Ratatui terminal UI. Same page structure as GUI.
- **`src/tray/`** — System tray via `ksni` (D-Bus StatusNotifierItem).
- **`src/config/`** — TOML config at `~/.config/mybuds/config.toml`.

## Key Patterns

- **PropertyStore** (`Arc<Mutex<HashMap<String, HashMap<String, String>>>>`) is the shared state between Bluetooth manager, UI, and tray. Properties are grouped by category (battery, anc, info, ear_detection, conversation_awareness, personalized_volume, etc.).
- **Handler trait** — `DeviceHandler` has `on_packet()` for incoming data and `on_init()` to request initial state. Handlers read/write to PropertyStore.
- **Transport abstraction** — `Transport` enum (`Rfcomm(u16)` / `L2cap(u16)`) in DeviceProfile selects connection type. `BluetoothManager::run()` dispatches to `run_rfcomm()` or `run_l2cap()`, both feeding into a shared `run_packet_loop()`.
- **AAP ↔ Handler mapping** — AAP packets are converted to/from `HuaweiSppPacket` at the L2CAP transport boundary. Handlers never see raw AAP bytes. Command ID prefixes `0xAA` (general opcode) and `0xA9` (control command subtype) distinguish AAP from Huawei commands.
- **Dual UI** — GUI and TUI share the same PropertyStore and device layer. Only one runs at a time (`--tui` flag).
- **Reconnect loop** — `BluetoothManager::run_with_reconnect()` handles connection drops with exponential backoff.

## Adding a New Huawei Device

1. Create a profile function in `src/device/models/mod.rs`
2. Select which handlers apply (check hardware capabilities)
3. Add the Bluetooth device name to `profile_for_device()` match
4. Set the correct transport: `Transport::Rfcomm(1)` or `Transport::Rfcomm(16)`

## Adding a New AirPods Model

1. Create a profile function in `src/device/models/mod.rs` using `Transport::L2cap(0x1001)`
2. Select which AirPods handlers to include (from `src/device/airpods.rs`)
3. Add the Bluetooth name pattern to `profile_for_device()` match (use `contains()` for AirPods names)

## Adding a New Feature Handler

1. Create a new file in `src/device/` implementing `DeviceHandler`
2. For Huawei: define command IDs in `src/protocol/commands.rs`
3. For AirPods: use `[0xAA, opcode]` or `[0xA9, subtype]` as command IDs, define constants in `src/protocol/aap.rs`
4. Add the handler to relevant device profiles in `models/mod.rs`
5. Add UI controls in both `src/ui/pages/` and `src/tui/pages/`

## Conventions

- Use `anyhow::Result` for fallible functions
- Use `tracing` macros (`info!`, `debug!`, `error!`) for logging
- Property keys are lowercase snake_case strings
- Huawei command IDs are `[service_id, command_id]` arrays of `u8`
- AirPods command IDs use prefix `0xAA` (general) or `0xA9` (control command subtype)

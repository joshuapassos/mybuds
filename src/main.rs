mod bluetooth;
mod config;
mod device;
mod protocol;
mod tray;
mod tui;
mod ui;

use std::sync::Arc;

use anyhow::Result;
use bluer::Address;
use clap::Parser;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info};

use bluetooth::scanner;
use config::AppConfig;
use device::handler::PropertyStore;
use device::models::profile_for_device;
use tray::TrayFlags;

#[derive(Parser)]
#[command(name = "mybuds", about = "Desktop manager for Huawei FreeBuds headphones")]
struct Cli {
    /// Run in terminal UI mode instead of GUI
    #[arg(long)]
    tui: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging — in TUI mode, write to a log file to avoid corrupting the terminal
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("mybuds=debug".parse().unwrap())
        .add_directive("bluer=info".parse().unwrap());

    if cli.tui {
        let log_file = std::fs::File::create("/tmp/mybuds.log")?;
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(log_file)
            .with_ansi(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    }

    info!("MyBuds starting");

    // Load config
    let config = AppConfig::load();

    // Property change channel (UI -> device manager)
    let (prop_tx, prop_rx) = mpsc::channel::<(String, String, String)>(32);

    // Shared property store
    let props: PropertyStore = Arc::new(Mutex::new(std::collections::HashMap::new()));

    if cli.tui {
        run_tui_mode(config, props, prop_tx, prop_rx)
    } else {
        run_gui_mode(config, props, prop_tx, prop_rx)
    }
}

fn run_gui_mode(
    config: AppConfig,
    props: PropertyStore,
    prop_tx: mpsc::Sender<(String, String, String)>,
    prop_rx: mpsc::Receiver<(String, String, String)>,
) -> Result<()> {
    use std::sync::atomic::Ordering;

    let props_clone = props.clone();

    // Shared tray flags for tray <-> iced communication
    let tray_flags = TrayFlags::new();
    let tray_flags_clone = tray_flags.clone();

    // Spawn Bluetooth manager in background
    let config_clone = config.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            // Spawn tray
            let tray_handle = tray::spawn_tray(tray_flags_clone);

            if let Err(e) =
                run_bluetooth_with_tray(config_clone, props_clone.clone(), prop_rx, tray_handle)
                    .await
            {
                error!("Bluetooth manager error: {}", e);
            }
        });
    });

    // Run iced daemon on main thread.
    // Unlike iced::application, the daemon does NOT exit when the last window
    // is closed — it keeps running so we can reopen from the system tray.
    iced::daemon("MyBuds", MyBudsApp::update, MyBudsApp::view)
        .theme(MyBudsApp::theme)
        .subscription(MyBudsApp::subscription)
        .run_with(move || MyBudsApp::new(props.clone(), Some(prop_tx), Some(tray_flags)))?;

    Ok(())
}

fn run_tui_mode(
    config: AppConfig,
    props: PropertyStore,
    prop_tx: mpsc::Sender<(String, String, String)>,
    prop_rx: mpsc::Receiver<(String, String, String)>,
) -> Result<()> {
    let props_clone = props.clone();

    // Spawn Bluetooth manager in background (no tray for TUI mode)
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            if let Err(e) = run_bluetooth_headless(config, props_clone, prop_rx).await {
                error!("Bluetooth manager error: {}", e);
            }
        });
    });

    // Run TUI on main thread
    tui::run(props, prop_tx)
}

// Re-export for iced
use ui::MyBudsApp;

async fn run_bluetooth_with_tray(
    config: AppConfig,
    props: PropertyStore,
    prop_rx: mpsc::Receiver<(String, String, String)>,
    tray_handle: ksni::Handle<tray::MyBudsTray>,
) -> Result<()> {
    // Find device
    let (address, device_name) = match find_device(&config).await {
        Some(dev) => dev,
        None => {
            info!("No device found. Waiting for device...");
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                if let Some(dev) = find_device(&config).await {
                    break dev;
                }
            }
        }
    };

    info!("Using device: {} ({})", device_name, address);

    let profile = profile_for_device(&device_name);
    info!(
        "Device profile: {}, transport: {:?}",
        profile.name, profile.transport
    );

    let mut bt_manager = bluetooth::BluetoothManager::new(address, profile, props.clone(), prop_rx);

    // Update tray with device name
    let name = device_name.clone();
    tray_handle.update(move |tray| {
        tray.device_name = Some(name.clone());
    });

    // Spawn tray update loop
    let dm_props = props.clone();
    let tray_handle_clone = tray_handle.clone();
    let device_name_clone = device_name.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            tray::update_tray_from_props(
                &tray_handle_clone,
                &dm_props,
                Some(&device_name_clone),
            )
            .await;
        }
    });

    bt_manager.run_with_reconnect().await;

    Ok(())
}

async fn run_bluetooth_headless(
    config: AppConfig,
    props: PropertyStore,
    prop_rx: mpsc::Receiver<(String, String, String)>,
) -> Result<()> {
    let (address, device_name) = match find_device(&config).await {
        Some(dev) => dev,
        None => {
            info!("No device found. Waiting for device...");
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                if let Some(dev) = find_device(&config).await {
                    break dev;
                }
            }
        }
    };

    info!("Using device: {} ({})", device_name, address);

    let profile = profile_for_device(&device_name);
    info!(
        "Device profile: {}, transport: {:?}",
        profile.name, profile.transport
    );

    let mut bt_manager = bluetooth::BluetoothManager::new(address, profile, props.clone(), prop_rx);
    bt_manager.run_with_reconnect().await;

    Ok(())
}

async fn find_device(config: &AppConfig) -> Option<(Address, String)> {
    // Try configured device first
    if let (Some(addr_str), Some(name)) = (&config.device_address, &config.device_name) {
        if let Ok(addr) = addr_str.parse::<Address>() {
            return Some((addr, name.clone()));
        }
    }

    // Scan for paired devices
    match scanner::list_paired_devices(true).await {
        Ok(devices) => {
            if let Some(dev) = devices.first() {
                Some((dev.address, dev.name.clone()))
            } else {
                None
            }
        }
        Err(e) => {
            error!("Failed to scan devices: {}", e);
            None
        }
    }
}

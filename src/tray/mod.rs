pub mod icon;
pub mod menu;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::device::handler::PropertyStore;

/// Shared flags for tray <-> iced communication.
#[derive(Clone)]
pub struct TrayFlags {
    pub show_window: Arc<AtomicBool>,
    pub quit_app: Arc<AtomicBool>,
    /// Pending ANC mode change from tray menu (consumed by bluetooth loop).
    pub pending_anc_mode: Arc<std::sync::Mutex<Option<String>>>,
}

impl TrayFlags {
    pub fn new() -> Self {
        Self {
            show_window: Arc::new(AtomicBool::new(false)),
            quit_app: Arc::new(AtomicBool::new(false)),
            pending_anc_mode: Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

/// Tray application state.
pub struct MyBudsTray {
    pub connected: bool,
    pub device_name: Option<String>,
    pub battery: HashMap<String, String>,
    pub anc_mode: Option<String>,
    pub anc_options: Vec<String>,
    pub flags: TrayFlags,
}

impl MyBudsTray {
    pub fn new(flags: TrayFlags) -> Self {
        Self {
            connected: false,
            device_name: None,
            battery: HashMap::new(),
            anc_mode: None,
            anc_options: Vec::new(),
            flags,
        }
    }
}

impl ksni::Tray for MyBudsTray {
    fn id(&self) -> String {
        "mybuds".into()
    }

    fn title(&self) -> String {
        if let Some(ref name) = self.device_name {
            if let Some(global) = self.battery.get("global") {
                format!("{} - {}%", name, global)
            } else {
                name.clone()
            }
        } else {
            "MyBuds".into()
        }
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let (width, height, data) = icon::tray_icon();
        vec![ksni::Icon {
            width,
            height,
            data,
        }]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.flags.show_window.store(true, Ordering::Relaxed);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        let anc_refs: Vec<&str> = self.anc_options.iter().map(|s| s.as_str()).collect();
        menu::build_menu(
            self.device_name.as_deref(),
            &self.battery,
            self.anc_mode.as_deref(),
            &anc_refs,
            self.connected,
        )
    }
}

/// Spawn the tray service. Returns a handle to update tray state.
pub fn spawn_tray(flags: TrayFlags) -> ksni::Handle<MyBudsTray> {
    let service = ksni::TrayService::new(MyBudsTray::new(flags));
    let handle = service.handle();
    service.spawn();
    handle
}

/// Update tray state from the property store.
pub async fn update_tray_from_props(
    handle: &ksni::Handle<MyBudsTray>,
    props: &PropertyStore,
    device_name: Option<&str>,
) {
    let store = props.lock().await;

    let battery = store.get("battery").cloned().unwrap_or_default();
    let anc_mode = store
        .get("anc")
        .and_then(|m| m.get("mode"))
        .cloned();
    let anc_options: Vec<String> = store
        .get("anc")
        .and_then(|m| m.get("mode_options"))
        .map(|s| s.split(',').map(String::from).collect())
        .unwrap_or_default();
    let connected = !battery.is_empty();

    let name = device_name.map(String::from);

    handle.update(move |tray| {
        tray.connected = connected;
        tray.device_name = name.clone();
        tray.battery = battery.clone();
        tray.anc_mode = anc_mode.clone();
        tray.anc_options = anc_options.clone();
    });
}

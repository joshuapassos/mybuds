pub mod pages;
pub mod theme;
pub mod widgets;

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use iced::widget::{button, column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length, Task, Theme};

use crate::device::handler::PropertyStore;
use crate::tray::TrayFlags;

/// Tab pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Sound,
    Gestures,
    DualConnect,
    DeviceInfo,
    Settings,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Sound => "Sound",
            Tab::Gestures => "Gestures",
            Tab::DualConnect => "Dual Connect",
            Tab::DeviceInfo => "Device Info",
            Tab::Settings => "Settings",
        }
    }

    fn all() -> &'static [Tab] {
        &[
            Tab::Home,
            Tab::Sound,
            Tab::Gestures,
            Tab::DualConnect,
            Tab::DeviceInfo,
            Tab::Settings,
        ]
    }
}

/// Application messages.
#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(Tab),
    SetAncMode(String),
    SetAncLevel(String),
    SetEqPreset(String),
    SetSoundQuality(String),
    SetLowLatency(bool),
    SetAutoPause(bool),
    SetGesture(String, String),
    SetDualConnect(bool),
    /// Property store snapshot received from async task.
    PropsRefreshed(HashMap<String, HashMap<String, String>>),
    Tick,
}

/// Application state.
pub struct MyBudsApp {
    current_tab: Tab,
    props: PropertyStore,
    // Cached property snapshots
    battery: HashMap<String, String>,
    anc: HashMap<String, String>,
    info: HashMap<String, String>,
    sound: HashMap<String, String>,
    actions: HashMap<String, String>,
    config: HashMap<String, String>,
    dual_connect: HashMap<String, String>,
    connected: bool,
    /// Channel to send property change requests
    property_tx: Option<tokio::sync::mpsc::Sender<(String, String, String)>>,
    /// Tray communication flags
    tray_flags: Option<TrayFlags>,
}

impl MyBudsApp {
    pub fn new(
        props: PropertyStore,
        property_tx: Option<tokio::sync::mpsc::Sender<(String, String, String)>>,
        tray_flags: Option<TrayFlags>,
    ) -> (Self, Task<Message>) {
        (
            Self {
                current_tab: Tab::Home,
                props,
                battery: HashMap::new(),
                anc: HashMap::new(),
                info: HashMap::new(),
                sound: HashMap::new(),
                actions: HashMap::new(),
                config: HashMap::new(),
                dual_connect: HashMap::new(),
                connected: false,
                property_tx,
                tray_flags,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.current_tab = tab;
            }
            Message::SetAncMode(mode) => {
                self.send_property("anc", "mode", &mode);
            }
            Message::SetAncLevel(level) => {
                self.send_property("anc", "level", &level);
            }
            Message::SetEqPreset(preset) => {
                self.send_property("config_eq", "equalizer_preset", &preset);
            }
            Message::SetSoundQuality(quality) => {
                self.send_property("config_sound_quality", "quality_preference", &quality);
            }
            Message::SetLowLatency(enabled) => {
                self.send_property("low_latency", "low_latency", if enabled { "true" } else { "false" });
            }
            Message::SetAutoPause(enabled) => {
                self.send_property("tws_auto_pause", "auto_pause", if enabled { "true" } else { "false" });
            }
            Message::SetGesture(prop, value) => {
                let group = if prop.starts_with("double_tap") {
                    "gesture_double"
                } else if prop.starts_with("triple_tap") {
                    "gesture_triple"
                } else if prop.starts_with("long_tap") || prop.starts_with("noise_control") {
                    "gesture_long_split"
                } else if prop.starts_with("swipe") {
                    "gesture_swipe"
                } else {
                    "action"
                };
                self.send_property(group, &prop, &value);
            }
            Message::SetDualConnect(enabled) => {
                self.send_property("dual_connect", "enabled", if enabled { "true" } else { "false" });
            }
            Message::Tick => {
                // Check tray quit signal
                if let Some(ref flags) = self.tray_flags {
                    if flags.quit_app.swap(false, Ordering::Relaxed) {
                        return iced::exit();
                    }
                }

                // Fetch latest props from the shared store
                let props = self.props.clone();
                return Task::perform(
                    async move {
                        let store = props.lock().await;
                        store.clone()
                    },
                    Message::PropsRefreshed,
                );
            }
            Message::PropsRefreshed(store) => {
                self.battery = store.get("battery").cloned().unwrap_or_default();
                self.anc = store.get("anc").cloned().unwrap_or_default();
                self.info = store.get("info").cloned().unwrap_or_default();
                self.sound = store.get("sound").cloned().unwrap_or_default();
                self.actions = store.get("action").cloned().unwrap_or_default();
                self.config = store.get("config").cloned().unwrap_or_default();
                self.dual_connect = store.get("dual_connect").cloned().unwrap_or_default();
                self.connected = !self.battery.is_empty();
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Tab bar
        let tab_bar = row(
            Tab::all().iter().map(|&tab| {
                let is_active = tab == self.current_tab;
                let style = if is_active {
                    button::primary
                } else {
                    button::secondary
                };
                button(text(tab.label()).size(13))
                    .on_press(Message::SwitchTab(tab))
                    .style(style)
                    .into()
            }),
        )
        .spacing(4)
        .padding(8);

        // Page content
        let page_content: Element<'_, Message> = match self.current_tab {
            Tab::Home => pages::home::view(&self.battery, &self.anc, &self.info, self.connected),
            Tab::Sound => pages::sound::view(&self.sound, &self.config),
            Tab::Gestures => pages::gestures::view(&self.actions),
            Tab::DualConnect => pages::dual_connect::view(&self.dual_connect),
            Tab::DeviceInfo => pages::device_info::view(&self.info),
            Tab::Settings => pages::settings::view(&self.config),
        };

        let content = column![
            tab_bar,
            horizontal_rule(1),
            scrollable(page_content).height(Length::Fill),
        ]
        .spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn theme(&self) -> Theme {
        theme::app_theme()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
    }

    fn send_property(&self, group: &str, prop: &str, value: &str) {
        if let Some(ref tx) = self.property_tx {
            let _ = tx.try_send((group.to_string(), prop.to_string(), value.to_string()));
        }
    }
}

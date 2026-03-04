pub mod pages;
pub mod widgets;

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use gtk4::prelude::*;
use gtk4::gdk;
use libadwaita as adw;
use relm4::prelude::*;

use crate::device::handler::PropertyStore;
use crate::tray::TrayFlags;

/// Embedded Lucide icons (SVG) — loaded directly into memory, no disk writes.
const ICONS: &[(&str, &[u8])] = &[
    ("mybuds-home-symbolic", include_bytes!("../../assets/icons/mybuds-home-symbolic.svg")),
    ("mybuds-sound-symbolic", include_bytes!("../../assets/icons/mybuds-sound-symbolic.svg")),
    ("mybuds-gestures-symbolic", include_bytes!("../../assets/icons/mybuds-gestures-symbolic.svg")),
    ("mybuds-bluetooth-symbolic", include_bytes!("../../assets/icons/mybuds-bluetooth-symbolic.svg")),
    ("mybuds-info-symbolic", include_bytes!("../../assets/icons/mybuds-info-symbolic.svg")),
    ("mybuds-settings-symbolic", include_bytes!("../../assets/icons/mybuds-settings-symbolic.svg")),
];

/// Write embedded icons to a temp dir so GTK can find them via search path.
/// Must be called before any widgets are created.
pub fn install_icons() {
    let icon_dir = std::env::temp_dir()
        .join("mybuds-icons")
        .join("hicolor")
        .join("scalable")
        .join("actions");
    std::fs::create_dir_all(&icon_dir).ok();

    for (name, data) in ICONS {
        std::fs::write(icon_dir.join(format!("{}.svg", name)), data).ok();
    }
}

/// Add the icon temp dir to the display's icon theme search path.
/// Called from init() after GTK display is available.
fn register_icon_theme(display: &gdk::Display) {
    let base = std::env::temp_dir().join("mybuds-icons");
    let theme = gtk4::IconTheme::for_display(display);
    theme.add_search_path(base.to_str().unwrap());
}

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

    fn icon(&self) -> &'static str {
        match self {
            Tab::Home => "mybuds-home-symbolic",
            Tab::Sound => "mybuds-sound-symbolic",
            Tab::Gestures => "mybuds-gestures-symbolic",
            Tab::DualConnect => "mybuds-bluetooth-symbolic",
            Tab::DeviceInfo => "mybuds-info-symbolic",
            Tab::Settings => "mybuds-settings-symbolic",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Tab::Home => "home",
            Tab::Sound => "sound",
            Tab::Gestures => "gestures",
            Tab::DualConnect => "dual_connect",
            Tab::DeviceInfo => "device_info",
            Tab::Settings => "settings",
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
#[allow(dead_code)]
pub enum Message {
    SwitchTab(String),
    SetAncMode(String),
    SetAncLevel(String),
    SetEqPreset(String),
    SetSoundQuality(String),
    SetLowLatency(bool),
    SetAutoPause(bool),
    SetGesture(String, String),
    SetDualConnect(bool),
    SetConversationAwareness(bool),
    SetPersonalizedVolume(bool),
    Tick,
}

/// Init data passed from main.
pub struct AppInit {
    pub props: PropertyStore,
    pub property_tx: Option<tokio::sync::mpsc::Sender<(String, String, String)>>,
    pub tray_flags: Option<TrayFlags>,
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
    ear_detection: HashMap<String, String>,
    conversation_awareness: HashMap<String, String>,
    personalized_volume: HashMap<String, String>,
    connected: bool,
    property_tx: Option<tokio::sync::mpsc::Sender<(String, String, String)>>,
    tray_flags: Option<TrayFlags>,
    // Page content containers (stored in model so update_view can access them)
    home_box: gtk4::Box,
    sound_box: gtk4::Box,
    gestures_box: gtk4::Box,
    dual_connect_box: gtk4::Box,
    device_info_box: gtk4::Box,
    settings_box: gtk4::Box,
}

impl MyBudsApp {
    fn refresh_props(&mut self) {
        let store = self.props.lock().unwrap();
        self.battery = store.get("battery").cloned().unwrap_or_default();
        self.anc = store.get("anc").cloned().unwrap_or_default();
        self.info = store.get("info").cloned().unwrap_or_default();
        self.sound = store.get("sound").cloned().unwrap_or_default();
        self.actions = store.get("action").cloned().unwrap_or_default();
        self.config = store.get("config").cloned().unwrap_or_default();
        self.dual_connect = store.get("dual_connect").cloned().unwrap_or_default();
        self.ear_detection = store.get("ear_detection").cloned().unwrap_or_default();
        self.conversation_awareness = store
            .get("conversation_awareness")
            .cloned()
            .unwrap_or_default();
        self.personalized_volume = store
            .get("personalized_volume")
            .cloned()
            .unwrap_or_default();
        self.connected = !self.battery.is_empty();
    }

    fn send_property(&self, group: &str, prop: &str, value: &str) {
        if let Some(ref tx) = self.property_tx {
            let _ = tx.try_send((group.to_string(), prop.to_string(), value.to_string()));
        }
    }

    fn rebuild_pages(&self, sender: &ComponentSender<Self>) {
        pages::home::build(
            &self.home_box,
            &self.battery,
            &self.anc,
            &self.info,
            &self.ear_detection,
            &self.conversation_awareness,
            &self.personalized_volume,
            self.connected,
            sender,
        );
        pages::sound::build(&self.sound_box, &self.sound, &self.config, sender);
        pages::gestures::build(&self.gestures_box, &self.actions, sender);
        pages::dual_connect::build(&self.dual_connect_box, &self.dual_connect, sender);
        pages::device_info::build(&self.device_info_box, &self.info);
        pages::settings::build(&self.settings_box, &self.config, sender);
    }
}

#[allow(unused_variables, unused_assignments)]
#[relm4::component(pub)]
impl SimpleComponent for MyBudsApp {
    type Init = AppInit;
    type Input = Message;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_width: 480,
            set_default_height: 600,
            set_title: Some("MyBuds"),

            connect_close_request[sender] => move |window| {
                window.set_visible(false);
                gtk4::glib::Propagation::Stop
            },

            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,

                adw::HeaderBar {
                    set_centering_policy: adw::CenteringPolicy::Strict,
                    #[wrap(Some)]
                    #[name = "switcher_title"]
                    set_title_widget = &adw::ViewSwitcherTitle {
                        set_stack: Some(&stack),
                        set_title: "MyBuds",
                    },
                },

                #[local_ref]
                stack -> adw::ViewStack {},

                #[name = "switcher_bar"]
                adw::ViewSwitcherBar {
                    set_stack: Some(&stack),
                    set_reveal: false,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Register icon search path now that we have a display
        if let Some(display) = gdk::Display::default() {
            register_icon_theme(&display);
        }

        // Build page content boxes
        let home_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let sound_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let gestures_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let dual_connect_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let device_info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let settings_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        // Build the ViewStack
        let stack = adw::ViewStack::new();
        stack.set_vexpand(true);

        let page_defs: &[(&gtk4::Box, Tab)] = &[
            (&home_box, Tab::Home),
            (&sound_box, Tab::Sound),
            (&gestures_box, Tab::Gestures),
            (&dual_connect_box, Tab::DualConnect),
            (&device_info_box, Tab::DeviceInfo),
            (&settings_box, Tab::Settings),
        ];

        for (page_box, tab) in page_defs {
            let scrolled = gtk4::ScrolledWindow::builder()
                .hscrollbar_policy(gtk4::PolicyType::Never)
                .vexpand(true)
                .child(*page_box)
                .build();
            let page = stack.add_titled(&scrolled, Some(tab.name()), tab.label());
            page.set_icon_name(Some(tab.icon()));
        }

        let model = MyBudsApp {
            current_tab: Tab::Home,
            props: init.props,
            battery: HashMap::new(),
            anc: HashMap::new(),
            info: HashMap::new(),
            sound: HashMap::new(),
            actions: HashMap::new(),
            config: HashMap::new(),
            dual_connect: HashMap::new(),
            ear_detection: HashMap::new(),
            conversation_awareness: HashMap::new(),
            personalized_volume: HashMap::new(),
            connected: false,
            property_tx: init.property_tx,
            tray_flags: init.tray_flags,
            home_box,
            sound_box,
            gestures_box,
            dual_connect_box,
            device_info_box,
            settings_box,
        };

        let widgets = view_output!();

        // Bind: when title switcher can't fit tabs, reveal bottom bar instead
        widgets
            .switcher_title
            .bind_property("title-visible", &widgets.switcher_bar, "reveal")
            .sync_create()
            .build();

        // GLib timeout for periodic property refresh
        let tick_sender = sender.clone();
        gtk4::glib::timeout_add_seconds_local(1, move || {
            tick_sender.input(Message::Tick);
            gtk4::glib::ControlFlow::Continue
        });

        // Handle tray show_window / quit signals
        if let Some(ref flags) = model.tray_flags {
            let show_flag = flags.show_window.clone();
            let quit_flag = flags.quit_app.clone();
            let win = root.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
                if quit_flag.load(Ordering::Relaxed) {
                    if let Some(app) = win.application() {
                        app.quit();
                    }
                    return gtk4::glib::ControlFlow::Break;
                }
                if show_flag.swap(false, Ordering::Relaxed) {
                    win.set_visible(true);
                    win.present();
                }
                gtk4::glib::ControlFlow::Continue
            });
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Message::SwitchTab(name) => {
                for tab in Tab::all() {
                    if tab.name() == name {
                        self.current_tab = *tab;
                        break;
                    }
                }
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
                self.send_property(
                    "low_latency",
                    "low_latency",
                    if enabled { "true" } else { "false" },
                );
            }
            Message::SetAutoPause(enabled) => {
                self.send_property(
                    "tws_auto_pause",
                    "auto_pause",
                    if enabled { "true" } else { "false" },
                );
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
                self.send_property(
                    "dual_connect",
                    "enabled",
                    if enabled { "true" } else { "false" },
                );
            }
            Message::SetConversationAwareness(enabled) => {
                self.send_property(
                    "conversation_awareness",
                    "enabled",
                    if enabled { "true" } else { "false" },
                );
            }
            Message::SetPersonalizedVolume(enabled) => {
                self.send_property(
                    "personalized_volume",
                    "enabled",
                    if enabled { "true" } else { "false" },
                );
            }
            Message::Tick => {
                self.refresh_props();
                self.rebuild_pages(&sender);
            }
        }
    }
}

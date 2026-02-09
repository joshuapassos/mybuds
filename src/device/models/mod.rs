
use super::airpods::{
    AirPodsAncHandler, AirPodsBatteryHandler, AirPodsConversationAwarenessHandler,
    AirPodsEarDetectionHandler, AirPodsInfoHandler, AirPodsPersonalizedVolumeHandler,
};
use super::anc::{AncHandler, AncLegacyChangeHandler};
use super::battery::BatteryHandler;
use super::config::{AutoPauseHandler, LowLatencyHandler, SoundQualityHandler};
use super::dual_connect::DualConnectHandler;
use super::equalizer::EqualizerHandler;
use super::gestures::{LongTapSplitHandler, SwipeGestureHandler, TapActionHandler};
use super::handler::DeviceHandler;
use super::info::InfoHandler;

/// Bluetooth transport type.
#[derive(Debug, Clone, Copy)]
pub enum Transport {
    /// RFCOMM/SPP (Huawei devices). Value is the channel number.
    Rfcomm(u16),
    /// L2CAP (AirPods). Value is the PSM.
    L2cap(u16),
}

/// Device profile configuration.
pub struct DeviceProfile {
    pub name: &'static str,
    pub transport: Transport,
    pub handlers: Vec<Box<dyn DeviceHandler>>,
}

// ============================================================
// Huawei / HONOR profiles
// ============================================================

/// Build handlers for FreeBuds Pro 3 / Pro 4 / FreeClip.
pub fn freebuds_pro3() -> DeviceProfile {
    DeviceProfile {
        name: "FreeBuds Pro 3",
        transport: Transport::Rfcomm(1),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(AncHandler::new(true, true, true)),
            Box::new(AncLegacyChangeHandler),
            Box::new(BatteryHandler::default()),
            Box::new(SoundQualityHandler),
            Box::new(EqualizerHandler::with_presets(vec![
                (5, "default"),
                (1, "hardbass"),
                (2, "treble"),
                (9, "voice"),
            ])),
            Box::new(AutoPauseHandler),
            Box::new(DualConnectHandler::default()),
            Box::new(TapActionHandler::double_tap(false)),
            Box::new(LongTapSplitHandler::new(true, true, false, true)),
            Box::new(SwipeGestureHandler),
            Box::new(LowLatencyHandler),
        ],
    }
}

/// Build handlers for FreeBuds Pro 2.
pub fn freebuds_pro2() -> DeviceProfile {
    DeviceProfile {
        name: "FreeBuds Pro 2",
        transport: Transport::Rfcomm(16),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(AncHandler::new(true, true, true)),
            Box::new(AncLegacyChangeHandler),
            Box::new(BatteryHandler::default()),
            Box::new(SoundQualityHandler),
            Box::new(EqualizerHandler::with_presets(vec![
                (5, "default"),
                (1, "hardbass"),
                (2, "treble"),
                (9, "voice"),
            ])),
            Box::new(AutoPauseHandler),
            Box::new(DualConnectHandler::default()),
            Box::new(TapActionHandler::double_tap(false)),
            Box::new(LongTapSplitHandler::new(true, true, false, true)),
            Box::new(SwipeGestureHandler),
            Box::new(LowLatencyHandler),
        ],
    }
}

/// Build handlers for FreeBuds 5i.
pub fn freebuds_5i() -> DeviceProfile {
    DeviceProfile {
        name: "FreeBuds 5i",
        transport: Transport::Rfcomm(16),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(BatteryHandler::default()),
            Box::new(AncHandler::new(true, true, false)),
            Box::new(AncLegacyChangeHandler),
            Box::new(TapActionHandler::double_tap(true)),
            Box::new(TapActionHandler::triple_tap()),
            Box::new(LongTapSplitHandler::new(true, true, false, true)),
            Box::new(SwipeGestureHandler),
            Box::new(AutoPauseHandler),
            Box::new(SoundQualityHandler),
            Box::new(LowLatencyHandler),
            Box::new(EqualizerHandler::with_presets(vec![
                (1, "default"),
                (2, "hardbass"),
                (3, "treble"),
                (9, "voices"),
            ])),
            Box::new(DualConnectHandler::default()),
        ],
    }
}

/// Build handlers for FreeBuds 6i.
pub fn freebuds_6i() -> DeviceProfile {
    DeviceProfile {
        name: "FreeBuds 6i",
        transport: Transport::Rfcomm(16),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(BatteryHandler::default()),
            Box::new(AncHandler::new(true, true, false)),
            Box::new(AncLegacyChangeHandler),
            Box::new(TapActionHandler::double_tap(true)),
            Box::new(TapActionHandler::triple_tap()),
            Box::new(LongTapSplitHandler::new(true, true, false, true)),
            Box::new(SwipeGestureHandler),
            Box::new(AutoPauseHandler),
            Box::new(SoundQualityHandler),
            Box::new(LowLatencyHandler),
            Box::new(EqualizerHandler::with_presets(vec![
                (1, "default"),
                (2, "hardbass"),
                (3, "treble"),
                (9, "voices"),
            ])),
            Box::new(DualConnectHandler::default()),
        ],
    }
}

/// Build handlers for FreeBuds 4i / HONOR Earbuds 2.
pub fn freebuds_4i() -> DeviceProfile {
    DeviceProfile {
        name: "FreeBuds 4i",
        transport: Transport::Rfcomm(16),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(AncHandler::default()),
            Box::new(AncLegacyChangeHandler),
            Box::new(BatteryHandler::default()),
            Box::new(TapActionHandler::double_tap(false)),
            Box::new(LongTapSplitHandler::default()),
            Box::new(AutoPauseHandler),
        ],
    }
}

/// Build handlers for FreeBuds SE 2.
pub fn freebuds_se2() -> DeviceProfile {
    DeviceProfile {
        name: "FreeBuds SE 2",
        transport: Transport::Rfcomm(1),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(BatteryHandler::default()),
            Box::new(TapActionHandler::double_tap(true)),
            Box::new(TapActionHandler::triple_tap()),
            Box::new(LongTapSplitHandler::new(false, false, true, false)),
            Box::new(EqualizerHandler::with_presets(vec![
                (1, "default"),
                (2, "hardbass"),
                (3, "treble"),
                (9, "voices"),
            ])),
            Box::new(LowLatencyHandler),
        ],
    }
}

/// Build a generic profile that probes for all features.
/// Used for unknown devices (like FreeBuds 5 open-fit).
pub fn generic_probe() -> DeviceProfile {
    DeviceProfile {
        name: "Generic Huawei",
        transport: Transport::Rfcomm(16),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(BatteryHandler::default()),
            Box::new(AncHandler::new(true, true, true)),
            Box::new(AncLegacyChangeHandler),
            Box::new(AutoPauseHandler),
            Box::new(TapActionHandler::double_tap(true)),
            Box::new(LongTapSplitHandler::new(true, true, true, true)),
            Box::new(SwipeGestureHandler),
            Box::new(LowLatencyHandler),
            Box::new(SoundQualityHandler),
            Box::new(DualConnectHandler::default()),
        ],
    }
}

/// Build handlers for FreeBuds 5 (open-fit).
pub fn freebuds_5() -> DeviceProfile {
    DeviceProfile {
        name: "FreeBuds 5",
        transport: Transport::Rfcomm(1),
        handlers: vec![
            Box::new(InfoHandler),
            Box::new(BatteryHandler::default()),
            Box::new(AncHandler::new(true, false, false)),
            Box::new(AncLegacyChangeHandler),
            Box::new(AutoPauseHandler),
            Box::new(TapActionHandler::double_tap(true)),
            Box::new(TapActionHandler::triple_tap()),
            Box::new(LongTapSplitHandler::new(true, true, false, true)),
            Box::new(SwipeGestureHandler),
            Box::new(LowLatencyHandler),
            Box::new(SoundQualityHandler),
            Box::new(EqualizerHandler::with_presets(vec![
                (1, "default"),
                (2, "hardbass"),
                (3, "treble"),
                (9, "voices"),
            ])),
        ],
    }
}

// ============================================================
// AirPods profiles
// ============================================================

/// AirPods Pro 2nd Gen / AirPods Pro 3rd Gen (full features).
pub fn airpods_pro() -> DeviceProfile {
    DeviceProfile {
        name: "AirPods Pro",
        transport: Transport::L2cap(0x1001),
        handlers: vec![
            Box::new(AirPodsInfoHandler),
            Box::new(AirPodsBatteryHandler),
            Box::new(AirPodsEarDetectionHandler),
            Box::new(AirPodsAncHandler::new(true)),
            Box::new(AirPodsConversationAwarenessHandler),
            Box::new(AirPodsPersonalizedVolumeHandler),
        ],
    }
}

/// AirPods Max (full features, no ear detection differences).
pub fn airpods_max() -> DeviceProfile {
    DeviceProfile {
        name: "AirPods Max",
        transport: Transport::L2cap(0x1001),
        handlers: vec![
            Box::new(AirPodsInfoHandler),
            Box::new(AirPodsBatteryHandler),
            Box::new(AirPodsEarDetectionHandler),
            Box::new(AirPodsAncHandler::new(true)),
            Box::new(AirPodsConversationAwarenessHandler),
            Box::new(AirPodsPersonalizedVolumeHandler),
        ],
    }
}

/// Generic AirPods (basic features: battery + ear detection).
pub fn airpods_generic() -> DeviceProfile {
    DeviceProfile {
        name: "AirPods",
        transport: Transport::L2cap(0x1001),
        handlers: vec![
            Box::new(AirPodsInfoHandler),
            Box::new(AirPodsBatteryHandler),
            Box::new(AirPodsEarDetectionHandler),
        ],
    }
}

// ============================================================
// Device lookup
// ============================================================

/// Get device profile by Bluetooth device name.
pub fn profile_for_device(name: &str) -> DeviceProfile {
    match name {
        // Huawei / HONOR
        "HUAWEI FreeBuds Pro 3" | "HUAWEI FreeBuds Pro 4" | "HUAWEI FreeClip" => freebuds_pro3(),
        "HUAWEI FreeBuds Pro 2" | "HUAWEI FreeBuds Pro" => freebuds_pro2(),
        "HUAWEI FreeBuds 5" => freebuds_5(),
        "HUAWEI FreeBuds 5i" => freebuds_5i(),
        "HUAWEI FreeBuds 6i" => freebuds_6i(),
        "HUAWEI FreeBuds 4i" | "HONOR Earbuds 2" | "HONOR Earbuds 2 SE"
        | "HONOR Earbuds 2 Lite" => freebuds_4i(),
        "HUAWEI FreeBuds SE 2" => freebuds_se2(),

        // AirPods
        n if n.contains("AirPods Pro") => airpods_pro(),
        n if n.contains("AirPods Max") => airpods_max(),
        n if n.contains("AirPods") => airpods_generic(),

        _ => generic_probe(),
    }
}

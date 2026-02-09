use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::protocol::commands::CommandId;
use crate::protocol::HuaweiSppPacket;

/// Shared property store: group -> (key -> value)
pub type PropertyStore = Arc<Mutex<HashMap<String, HashMap<String, String>>>>;

/// Sender for outgoing packets.
pub type PacketSender = tokio::sync::mpsc::Sender<HuaweiSppPacket>;

/// A device handler processes specific command IDs and manages a subset of device properties.
#[async_trait]
pub trait DeviceHandler: Send + Sync {
    /// Unique identifier for this handler.
    fn handler_id(&self) -> &'static str;

    /// Which command IDs this handler responds to.
    fn commands(&self) -> &[CommandId];

    /// Command IDs to silently ignore (e.g. write-ack responses).
    fn ignore_commands(&self) -> &[CommandId] {
        &[]
    }

    /// Called once after connection to fetch initial state.
    async fn on_init(&mut self, sender: &PacketSender, props: &PropertyStore) -> Result<()>;

    /// Handle an incoming packet matching one of our command IDs.
    async fn on_packet(&mut self, packet: &HuaweiSppPacket, props: &PropertyStore) -> Result<()>;

    /// Set a property value (triggered by UI or tray action).
    async fn set_property(
        &mut self,
        _sender: &PacketSender,
        _props: &PropertyStore,
        _group: &str,
        _prop: &str,
        _value: &str,
    ) -> Result<()> {
        Ok(())
    }
}

/// Helper to update multiple properties in a group at once.
pub async fn put_properties(
    props: &PropertyStore,
    group: &str,
    values: HashMap<String, String>,
) {
    let mut store = props.lock().await;
    let entry = store.entry(group.to_string()).or_default();
    for (k, v) in values {
        entry.insert(k, v);
    }
}


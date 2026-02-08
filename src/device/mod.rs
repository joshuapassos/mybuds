pub mod anc;
pub mod battery;
pub mod config;
pub mod dual_connect;
pub mod equalizer;
pub mod gestures;
pub mod handler;
pub mod info;
pub mod models;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::{debug, error, info, warn};

use crate::protocol::commands::CommandId;
use crate::protocol::HuaweiSppPacket;
use handler::{DeviceHandler, PacketSender, PropertyStore};

/// Events emitted by the device manager.
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    /// Connection state changed.
    StateChanged(ConnectionState),
    /// A property group was updated.
    PropertyChanged { group: String },
}

/// Connection state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// Manages device handlers and coordinates packet routing.
pub struct DeviceManager {
    handlers: Vec<Box<dyn DeviceHandler>>,
    command_map: HashMap<CommandId, usize>,
    ignore_set: HashMap<CommandId, ()>,
    props: PropertyStore,
    event_tx: broadcast::Sender<DeviceEvent>,
    packet_tx: PacketSender,
    packet_rx: Option<mpsc::Receiver<HuaweiSppPacket>>,
    state: ConnectionState,
}

impl DeviceManager {
    pub fn new(handlers: Vec<Box<dyn DeviceHandler>>, props: PropertyStore) -> Self {
        let (event_tx, _) = broadcast::channel(64);
        let (packet_tx, packet_rx) = mpsc::channel(32);

        let mut command_map = HashMap::new();
        let mut ignore_set = HashMap::new();

        for (idx, handler) in handlers.iter().enumerate() {
            for cmd in handler.commands() {
                command_map.insert(*cmd, idx);
            }
            for cmd in handler.ignore_commands() {
                ignore_set.insert(*cmd, ());
            }
        }

        Self {
            handlers,
            command_map,
            ignore_set,
            props,
            event_tx,
            packet_tx,
            packet_rx: Some(packet_rx),
            state: ConnectionState::Disconnected,
        }
    }

    /// Get a clone of the property store.
    pub fn props(&self) -> PropertyStore {
        self.props.clone()
    }

    /// Subscribe to device events.
    pub fn subscribe(&self) -> broadcast::Receiver<DeviceEvent> {
        self.event_tx.subscribe()
    }

    /// Get the packet sender for outgoing packets.
    pub fn packet_sender(&self) -> PacketSender {
        self.packet_tx.clone()
    }

    /// Take the packet receiver (can only be called once).
    pub fn take_packet_rx(&mut self) -> Option<mpsc::Receiver<HuaweiSppPacket>> {
        self.packet_rx.take()
    }

    /// Reset internal channels (call before each new connection).
    pub fn reset_channels(&mut self) {
        let (packet_tx, packet_rx) = mpsc::channel(32);
        self.packet_tx = packet_tx;
        self.packet_rx = Some(packet_rx);
    }

    /// Initialize all handlers (call after connection is established).
    /// Returns Err if the connection dies during init.
    pub async fn init_handlers(&mut self) -> Result<()> {
        self.set_state(ConnectionState::Connecting);

        for handler in &mut self.handlers {
            // Check if the outgoing channel is still alive
            if self.packet_tx.is_closed() {
                error!("Connection lost during handler init");
                self.set_state(ConnectionState::Failed);
                anyhow::bail!("Connection lost during handler initialization");
            }

            let id = handler.handler_id();
            let mut success = false;
            for attempt in 0..3 {
                debug!("Init handler '{}', attempt {}", id, attempt);
                match tokio::time::timeout(
                    std::time::Duration::from_secs(3),
                    handler.on_init(&self.packet_tx, &self.props),
                )
                .await
                {
                    Ok(Ok(())) => {
                        debug!("Handler '{}' initialized", id);
                        success = true;
                        break;
                    }
                    Ok(Err(e)) => {
                        // If the channel is closed, abort immediately
                        if self.packet_tx.is_closed() {
                            error!("Connection lost while initializing handler '{}'", id);
                            self.set_state(ConnectionState::Failed);
                            anyhow::bail!("Connection lost during handler initialization");
                        }
                        warn!("Handler '{}' init error: {}", id, e);
                    }
                    Err(_) => {
                        warn!("Handler '{}' init timeout", id);
                    }
                }
            }
            if !success {
                warn!("Skipping handler '{}' after failed init attempts", id);
            }

            // Small yield to let write errors propagate before next handler
            tokio::task::yield_now().await;

            // Re-check after yield in case write error propagated
            if self.packet_tx.is_closed() {
                error!("Connection lost during handler init (post-yield check)");
                self.set_state(ConnectionState::Failed);
                anyhow::bail!("Connection lost during handler initialization");
            }
        }

        self.set_state(ConnectionState::Connected);
        Ok(())
    }

    /// Route an incoming packet to the appropriate handler.
    pub async fn handle_packet(&mut self, packet: &HuaweiSppPacket) {
        if self.ignore_set.contains_key(&packet.command_id) {
            return;
        }

        if let Some(&idx) = self.command_map.get(&packet.command_id) {
            if let Err(e) = self.handlers[idx].on_packet(packet, &self.props).await {
                warn!(
                    "Handler error for cmd {:02X}{:02X}: {}",
                    packet.command_id[0], packet.command_id[1], e
                );
            }
            let _ = self.event_tx.send(DeviceEvent::PropertyChanged {
                group: self.handlers[idx].handler_id().to_string(),
            });
        } else {
            debug!(
                "Unhandled command: {:02X}{:02X}",
                packet.command_id[0], packet.command_id[1]
            );
        }
    }

    /// Set a property value, routing to the correct handler.
    pub async fn set_property(&mut self, group: &str, prop: &str, value: &str) -> Result<()> {
        for handler in &mut self.handlers {
            if handler.handler_id() == group
                || handler
                    .commands()
                    .iter()
                    .any(|_| handler.handler_id() == group)
            {
                handler
                    .set_property(&self.packet_tx, &self.props, group, prop, value)
                    .await?;
                let _ = self.event_tx.send(DeviceEvent::PropertyChanged {
                    group: group.to_string(),
                });
                return Ok(());
            }
        }
        anyhow::bail!("No handler found for group '{}'", group)
    }

    /// Clear all properties (call on disconnect so UI shows disconnected state).
    pub async fn clear_props(&self) {
        let mut store = self.props.lock().await;
        store.clear();
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
        let _ = self.event_tx.send(DeviceEvent::StateChanged(state));
        info!("Connection state: {:?}", state);
    }
}

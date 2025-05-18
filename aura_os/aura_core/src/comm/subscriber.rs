// aura_os/aura_core/src/comm/subscriber.rs

use crate::{AuraMessage, MESSAGE_BUS, aura_log}; // Import from lib.rs
use crate::error::{AuraError, Result};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

/// Represents a subscriber that can receive messages from a specific topic.
///
/// In this simplified sketch, `Subscriber` receives `AuraMessage` data.
/// A full implementation would be generic over a message type `T: AuraMessageTrait`,
/// handle deserialization, and interact with the AuraComm layer for network transport
/// and Quality of Service (QoS) management.
#[derive(Debug)] // Allow easy printing for debugging
pub struct Subscriber {
    topic_name: String,
    receiver: mpsc::Receiver<AuraMessage>, // Channel receiver for incoming messages
    // In a real system with generic message types:
    // _message_type: PhantomData<M>, // Ensures type M is known at compile time
}

impl Subscriber {
    /// Creates a new `Subscriber` for the given topic name.
    ///
    /// In a real AuraOS system, this would:
    /// - Register the subscriber with the AuraComm system.
    /// - Subscribe to the topic and its message type.
    /// - Negotiate QoS settings.
    /// - Set up underlying communication resources (e.g., network sockets, shared memory segments).
    ///
    /// # Arguments
    /// * `topic_name`: The fully qualified name of the topic to subscribe to (e.g., "/chatter", "/robot/odom").
    ///
    /// # Returns
    /// A `Result` containing the new `Subscriber` or an `AuraError` if creation fails.
    pub fn new(topic_name: &str) -> Result<Self> {
        aura_log!(info, "Creating subscriber for topic: '{}'", topic_name);
        let (sender, receiver) = mpsc::channel();

        // Lock the global message bus to register this subscriber
        let mut bus = MESSAGE_BUS.lock().map_err(|_| AuraError::CommunicationError("Failed to lock message bus".into()))?;
        bus.entry(topic_name.to_string())
            .or_insert_with(Vec::new)
            .push(sender); // Register the sender channel for this topic

        Ok(Self {
            topic_name: topic_name.to_string(),
            receiver,
            // _message_type: PhantomData, // For generic version
        })
    }

    /// Receives a message from the topic associated with this subscriber.
    ///
    /// # Arguments
    /// * `timeout`: Duration to wait for a message before timing out.
    ///
    /// # Returns
    /// A `Result` containing the received `AuraMessage` if successful, or an `AuraError` if receiving fails.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<AuraMessage> {
        self.receiver.recv_timeout(timeout).map_err(|e| {
            match e {
                mpsc::RecvTimeoutError::Timeout => AuraError::CommunicationError("Receive timed out".into()),
                mpsc::RecvTimeoutError::Disconnected => AuraError::CommunicationError("Channel disconnected".into()),
            }
        })
    }

    /// Returns the topic name this subscriber is associated with.
    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    // --- Future Enhancements ---
    // - `fn get_num_subscribers(&self) -> Result<usize>`
    // - Methods related to QoS settings.
    // - Lifecycle methods if the subscriber itself has a state.
}

// `Drop` trait is not strictly necessary for this simplified Subscriber,
// as it doesn't hold resources that need explicit cleanup (like network sockets
// or threads). If it did, a `Drop` impl would be important.
// For example, it might unregister itself from AuraComm.
// impl Drop for Subscriber {
//     fn drop(&mut self) {
//         aura_log!(info, "Dropping subscriber for topic: '{}'. Unregistering...", self.topic_name);
//         // AuraComm::unregister_subscriber(&self.topic_name, self.id);
//     }
// }

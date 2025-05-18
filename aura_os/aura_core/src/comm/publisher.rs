// aura_os/aura_core/src/comm/publisher.rs

use crate::{AuraMessage, MESSAGE_BUS, aura_log}; // Import from lib.rs
use crate::error::{AuraError, Result};
// In a real system with typed messages, you'd import the AuraMessageTrait
// use super::AuraMessageTrait; // Assuming AuraMessageTrait is in comm/mod.rs
// use std::marker::PhantomData; // For generic typed publishers

/// Represents a publisher that can send messages on a specific topic.
///
/// In this simplified sketch, `Publisher` sends `String` data.
/// A full implementation would be generic over a message type `T: AuraMessageTrait`,
/// handle serialization, and interact with the AuraComm layer for network transport
/// and Quality of Service (QoS) management.
///
/// # Type Parameters (Conceptual for Future)
/// // `M`: The type of the message to be published. Must implement `AuraMessageTrait`.
#[derive(Debug)] // Allow easy printing for debugging
pub struct Publisher {
    topic_name: String,
    // In a real system with generic message types:
    // _message_type: PhantomData<M>, // Ensures type M is known at compile time
    //
    // This sketch directly interacts with the global MESSAGE_BUS.
    // A real publisher would hold a client handle to the AuraComm system,
    // or a specific communication channel object.
}

impl Publisher {
    /// Creates a new `Publisher` for the given topic name.
    ///
    /// In a real AuraOS system, this would:
    /// - Register the publisher with the AuraComm system.
    /// - Advertise the topic and its message type.
    /// - Negotiate QoS settings.
    /// - Set up underlying communication resources (e.g., network sockets, shared memory segments).
    ///
    /// # Arguments
    /// * `topic_name`: The fully qualified name of the topic to publish to (e.g., "/chatter", "/robot/odom").
    ///
    /// # Returns
    /// A `Result` containing the new `Publisher` or an `AuraError` if creation fails.
    pub fn new(topic_name: &str) -> Result<Self> {
        // Basic validation for topic name (could be more extensive)
        if topic_name.is_empty() || !topic_name.starts_with('/') {
            return Err(AuraError::CommunicationError(format!(
                "Invalid topic name '{}': Must be absolute (start with '/') and non-empty.",
                topic_name
            )));
        }

        aura_log!(info, "Creating publisher for topic: '{}'", topic_name);

        // In this sketch, there's no complex registration with a bus here,
        // as the MESSAGE_BUS is global and subscribers register themselves.
        // A real publisher would "advertise" itself.

        Ok(Self {
            topic_name: topic_name.to_string(),
            // _message_type: PhantomData, // For generic version
        })
    }

    /// Publishes a message to the topic associated with this publisher.
    ///
    /// # Arguments
    /// * `data`: The data to publish. For this sketch, it's a `String`.
    ///           In a real system, this would be `message: M` where `M: AuraMessageTrait`.
    ///
    /// # Returns
    /// `Ok(())` if the message was successfully handed off for publication,
    /// or an `AuraError` if publication fails (e.g., communication error, serialization error).
    pub fn publish(&self, data: String /* In future: message: M */) -> Result<()> {
        aura_log!(trace, "Attempting to publish to topic '{}': \"{}\"", self.topic_name, data);

        // Construct the AuraMessage (in future, this would involve serialization of M)
        let aura_message = AuraMessage {
            topic: self.topic_name.clone(),
            data, // If M was generic: data: serialize(message)?
        };

        // Lock the global message bus to get access to the subscriber list.
        // This is a major simplification and a performance bottleneck.
        let bus_guard = MESSAGE_BUS.lock().map_err(|e| {
            AuraError::CommunicationError(format!("Failed to lock message bus for publishing: {}", e))
        })?;

        if let Some(subscribers_senders) = bus_guard.get(&self.topic_name) {
            if subscribers_senders.is_empty() {
                aura_log!(trace, "No active subscribers for topic '{}' at the moment.", self.topic_name);
                return Ok(()); // Not an error, just no one listening right now
            }

            let mut disconnected_subscriber_indices = Vec::new();
            for (index, sender_channel) in subscribers_senders.iter().enumerate() {
                // Clone the message for each subscriber (mpsc::Sender sends by value).
                // In a real system with zero-copy or shared memory, this would be different.
                match sender_channel.send(aura_message.clone()) {
                    Ok(_) => {
                        aura_log!(trace, "Successfully sent message to a subscriber for topic '{}'", self.topic_name);
                    }
                    Err(mpsc::SendError(sent_message)) => {
                        // This error means the receiving end of the channel (subscriber) has been dropped.
                        aura_log!(warn,
                            "Failed to send message to a subscriber for topic '{}' (receiver disconnected). Message: {:?}",
                            self.topic_name, sent_message
                        );
                        // Mark this subscriber for potential cleanup.
                        // Actual cleanup is complex with just `Vec<mpsc::Sender>`
                        // as senders are not easily identifiable or removable without more info.
                        // For this sketch, we'll just log. A real system would prune disconnected subscribers.
                        disconnected_subscriber_indices.push(index);
                    }
                }
            }
            // Conceptual: if disconnected_subscriber_indices is not empty, the bus manager
            // would later try to clean those up.
        } else {
            aura_log!(trace, "No subscriber list found for topic '{}' (no one has ever subscribed).", self.topic_name);
        }

        Ok(())
    }

    /// Returns the topic name this publisher is associated with.
    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    // --- Future Enhancements ---
    // - `fn wait_for_subscribers(&self, num_subscribers: usize, timeout: Duration) -> Result<()>`
    // - `fn get_num_subscribers(&self) -> Result<usize>`
    // - Methods related to QoS settings.
    // - Lifecycle methods if the publisher itself has a state.
}

// `Drop` trait is not strictly necessary for this simplified Publisher,
// as it doesn't hold resources that need explicit cleanup (like network sockets
// or threads). If it did, a `Drop` impl would be important.
// For example, it might unregister itself from AuraComm.
// impl Drop for Publisher {
//     fn drop(&mut self) {
//         aura_log!(info, "Dropping publisher for topic: '{}'. Unregistering...", self.topic_name);
//         // AuraComm::unregister_publisher(&self.topic_name, self.id);
//     }
// }
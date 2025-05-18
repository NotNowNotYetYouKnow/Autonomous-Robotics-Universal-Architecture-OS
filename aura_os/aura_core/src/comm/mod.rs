// aura_os/aura_core/src/comm/mod.rs

//! # AuraOS Communication (`comm`) Module
//!
//! This module provides the core communication primitives for AuraOS, enabling
//! nodes to exchange data through topics (publish-subscribe), services (request-response),
//! and actions (long-running tasks with feedback).
//!
//! For this initial sketch, we are only implementing a very simplified version
//! of publish-subscribe. Services and actions are future enhancements.
//!
//! The design aims for:
//! - **Decoupling:** Nodes do not need direct knowledge of each other.
//! - **Type Safety:** Leveraging Rust's type system for message definitions.
//! - **Configurable Quality of Service (QoS):** (Conceptual for now) Allowing control
//!   over reliability, durability, etc.
//! - **Introspection:** (Future) Tools to inspect topics, message rates, etc.

// Declare the sub-modules within the `comm` module.
// These correspond to `publisher.rs` and `subscriber.rs` files.
pub mod publisher;
pub mod subscriber;
// Future sub-modules:
// pub mod service_server;
// pub mod service_client;
// pub mod action_server;
// pub mod action_client;
// pub mod qos; // For Quality of Service profile definitions

// Re-export the primary types from the sub-modules to make them directly
// accessible via `aura_core::comm::Publisher` etc.
// This defines the public API of the `comm` module.
pub use publisher::Publisher;
pub use subscriber::Subscriber;
// Future re-exports:
// pub use service_server::ServiceServer;
// pub use service_client::ServiceClient;
// pub use qos::QosProfile; // Example

// --- Constants related to communication ---

/// Default queue size for publishers and subscribers if not otherwise specified
/// by a QoS profile. This is a conceptual constant for now.
pub const DEFAULT_MESSAGE_QUEUE_SIZE: usize = 10;

// --- Traits for Message Types (Conceptual for Future Use) ---

/// A conceptual marker trait for types that can be used as AuraOS messages.
///
/// In a real implementation, this trait might have bounds like:
/// `trait AuraMessageTrait: Clone + Send + Sync + 'static + serde::Serialize + serde::DeserializeOwned {`
/// `    const ROS_MSG_NAME: &'static str; // e.g., "std_msgs/String"`
/// `}`
/// This would allow for type-safe communication and potentially for message
/// introspection or generation of message definitions.
pub trait AuraMessageTrait: Clone + Send + Sync + 'static {
    // For this sketch, it's just a marker trait.
    // In the future, it could hold metadata about the message type.
    // For example, a static method to get the message type name:
    // fn type_name() -> &'static str;
}

// Implement the marker trait for String for our simple AuraMessage.data
impl AuraMessageTrait for String {}

// We could also implement it for our `crate::AuraMessage` struct if we wanted
// to pass the whole struct around with this trait, but our current pub/sub
// uses `String` directly for the data payload for simplicity.
// impl AuraMessageTrait for crate::AuraMessage {}
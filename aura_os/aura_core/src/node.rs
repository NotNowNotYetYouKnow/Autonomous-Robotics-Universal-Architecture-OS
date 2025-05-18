// aura_os/aura_core/src/node.rs

use crate::param::{ParameterManager, ParamValue}; // Assuming ParamValue might be used directly by node for defaults
use crate::comm::{Publisher, Subscriber}; // For future methods to create these
use crate::error::{AuraError, Result};
use crate::aura_log; // Internal logging macro
use std::sync::Arc;
use std::thread; // For a conceptual unique ID, will be replaced
use std::time::SystemTime; // For a conceptual unique ID

/// Represents a fundamental unit of computation within the AuraOS framework.
///
/// A `Node` encapsulates a specific piece of functionality in a robotic system,
/// such as controlling a sensor, running a perception algorithm, planning motion,
/// or executing a high-level behavior. Nodes communicate with each other using
/// AuraOS communication primitives (publishers, subscribers, services, actions).
///
/// In a complete AuraOS system, nodes would:
/// - Be managed by an `AuraScheduler` or `Executor`.
/// - Adhere to a defined lifecycle (e.g., Unconfigured, Inactive, Active, Finalized).
/// - Have their own isolated parameter scope, potentially inheriting from global parameters.
/// - Provide methods to easily create communication primitives tied to the node's context.
#[derive(Debug)] // Allow easy printing for debugging
pub struct Node {
    name: String,
    namespace: String, // Nodes typically operate within a namespace
    unique_id: String, // A unique identifier for this node instance
    params: Arc<ParameterManager>, // Each node has its own parameter manager instance

    // In a more complete implementation, a Node would hold:
    // - `Arc<Context>`: A shared context providing access to core AuraOS services
    //   (e.g., clock, graph manager, executor handle).
    // - `Vec<Arc<dyn Stoppable>>`: A list of owned resources like publishers, subscribers,
    //   timers that need to be managed during the node's lifecycle.
    // - `LifecycleState`: Current state of the node.
}

impl Node {
    /// Creates a new AuraOS node with the given name.
    ///
    /// The node name should be unique within its namespace.
    /// In a real system, this would involve registration with a central discovery
    /// mechanism or node manager.
    ///
    /// # Arguments
    /// * `name`: The desired name for the node (e.g., "lidar_driver", "path_planner").
    /// * `namespace`: The namespace for the node (e.g., "/robot1", "/perception").
    ///                An empty string typically means the global namespace.
    ///
    /// # Returns
    /// A `Result` containing the new `Node` or an `AuraError` if creation fails
    /// (e.g., name collision, initialization error).
    pub fn new(name: &str, namespace: &str) -> Result<Self> {
        // Basic name validation (in reality, more robust validation is needed)
        if name.is_empty() {
            return Err(AuraError::NodeError(
                "Node name cannot be empty.".to_string(),
            ));
        }
        // Sanitize or validate namespace (e.g., ensure it starts with '/')
        let clean_namespace = if namespace.is_empty() || namespace == "/" {
            "".to_string() // Represent global namespace as empty for internal logic
        } else if namespace.starts_with('/') {
            namespace.trim_end_matches('/').to_string()
        } else {
            format!("/{}", namespace.trim_end_matches('/'))
        };

        let fully_qualified_name = if clean_namespace.is_empty() {
            format!("/{}", name)
        } else {
            format!("{}/{}", clean_namespace, name)
        };

        aura_log!(info, "Creating node: '{}'", fully_qualified_name);

        // Generate a conceptual unique ID. In a real system, this would be more robust,
        // possibly involving a UUID or a scheme from a central node manager.
        let unique_id = format!(
            "{}-{:?}",
            fully_qualified_name,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() // Using nanos for more uniqueness in quick succession
        );

        Ok(Self {
            name: name.to_string(),
            namespace: clean_namespace,
            unique_id,
            params: Arc::new(ParameterManager::new(&fully_qualified_name)),
        })
    }

    /// Returns the base name of the node.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the namespace of the node.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the fully qualified name of the node (e.g., "/namespace/name").
    pub fn fully_qualified_name(&self) -> String {
        if self.namespace.is_empty() {
            format!("/{}", self.name)
        } else {
            format!("{}/{}", self.namespace, self.name)
        }
    }

    /// Returns the unique ID of this node instance.
    pub fn unique_id(&self) -> &str {
        &self.unique_id
    }

    /// Gets a thread-safe, reference-counted pointer to the node's `ParameterManager`.
    /// This allows the node and other parts of the system to safely access and modify
    /// the node's parameters.
    pub fn params(&self) -> Arc<ParameterManager> {
        Arc::clone(&self.params)
    }

    /// A conceptual "spin_once" method.
    ///
    /// In a real AuraOS system with an executor model, nodes wouldn't typically
    /// have a manual `spin_once` method. Instead, they would register callbacks
    /// (for message subscriptions, timers, service requests) with an executor,
    /// and the executor would invoke these callbacks when appropriate.
    ///
    /// This method is included in the sketch to allow the simple examples to simulate
    /// periodic work or event processing.
    pub fn spin_once(&self) {
        // aura_log!(trace, "Node '{}' spinning once (conceptual)", self.fully_qualified_name());
        // In a real system, this might involve:
        // - Checking for incoming messages on subscribed topics and invoking callbacks.
        // - Checking for timer expirations and invoking callbacks.
        // - Processing service requests.
        // For this sketch, it does nothing. The examples will call it in their loops.
    }

    // --- Methods for Creating Communication Primitives (Conceptual) ---
    // These would be the primary way nodes interact with AuraComm.

    /// Creates a publisher for a given topic.
    /// The topic name will be resolved relative to the node's namespace.
    ///
    /// # Arguments
    /// * `topic_name`: The name of the topic (e.g., "scan", "cmd_vel").
    /// * `_qos_profile`: (Placeholder) Quality of Service profile for the publisher.
    ///
    /// # Returns
    /// A `Result` containing the new `Publisher` or an `AuraError`.
    pub fn create_publisher(&self, topic_name: &str /*, qos_profile: QosProfile */) -> Result<Publisher> {
        let resolved_topic = self.resolve_topic_name(topic_name);
        aura_log!(info, "[{}] Creating publisher for topic '{}'", self.fully_qualified_name(), resolved_topic);
        Publisher::new(&resolved_topic)
        // In a real system, the publisher would be associated with this node's context
        // and potentially managed by the node for cleanup.
    }

    /// Creates a subscriber for a given topic.
    /// The topic name will be resolved relative to the node's namespace.
    ///
    /// # Arguments
    /// * `topic_name`: The name of the topic (e.g., "scan", "odom").
    /// * `_qos_profile`: (Placeholder) Quality of Service profile for the subscriber.
    /// * `_callback`: (Placeholder) A function or closure to be called when a message is received.
    ///
    /// # Returns
    /// A `Result` containing the new `Subscriber` or an `AuraError`.
    pub fn create_subscriber(&self, topic_name: &str /*, qos_profile: QosProfile, callback: F */) -> Result<Subscriber> {
        let resolved_topic = self.resolve_topic_name(topic_name);
        aura_log!(info, "[{}] Creating subscriber for topic '{}'", self.fully_qualified_name(), resolved_topic);
        Subscriber::new(&resolved_topic)
        // In a real system, the subscriber would be associated with this node's context,
        // manage the callback, and be cleaned up by the node.
    }

    /// Helper to resolve a topic name relative to the node's namespace.
    fn resolve_topic_name(&self, topic_name: &str) -> String {
        if topic_name.starts_with('/') {
            // Absolute topic name
            topic_name.to_string()
        } else if self.namespace.is_empty() {
            // Relative topic in global namespace
            format!("/{}", topic_name)
        } else {
            // Relative topic in node's namespace
            format!("{}/{}", self.namespace, topic_name)
        }
    }
}

/// Ensures that node resources are cleaned up when the `Node` instance goes out of scope.
///
/// In a real system, this `Drop` implementation would be more involved, signaling
/// the node's shutdown to the AuraOS system, unregistering from discovery services,
/// and ensuring all owned resources (publishers, subscribers, timers) are properly
/// stopped and released.
impl Drop for Node {
    fn drop(&mut self) {
        aura_log!(info, "Node '{}' (ID: {}) is being dropped. Performing cleanup.", self.fully_qualified_name(), self.unique_id);
        // Conceptual cleanup:
        // - Unregister from any central node manager.
        // - Signal all owned publishers/subscribers/timers to stop.
        // - Wait for graceful shutdown of owned resources.
    }
}
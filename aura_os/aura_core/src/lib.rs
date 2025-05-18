// aura_os/aura_core/src/lib.rs

//! # AuraOS Core (`aura_core`)
//!
//! This crate provides the foundational building blocks for the AuraOS
//! robotics application framework. It includes core concepts for communication,
//! node management, parameter handling, error types, and more.
//!
//! The design emphasizes developer experience, performance (with Rust's safety guarantees),
//! and a modular architecture to support the growth of the AuraOS ecosystem.

// Publicly re-export modules to make their contents accessible to users of `aura_core`.
// This defines the public API of the `aura_core` crate.
pub mod comm;
pub mod error;
pub mod node;
pub mod param;
// As AuraOS grows, more modules will be added here:
// pub mod lifecycle;
// pub mod executor;
// pub mod tf; // Transform system
// pub mod time; // Time and duration utilities

// --- Shared Structures & Global State (Simplified for this Sketch) ---

// For this initial sketch, we'll use a very simplified in-memory message bus.
// In a real AuraOS, `AuraComm` would be a sophisticated, distributed system.
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex}; // mpsc for simple multi-producer, single-consumer channels

/// Represents a message passed within the AuraOS system.
///
/// In a real implementation, `data` would be a generic type `T` constrained by
/// serialization traits (e.g., `serde::Serialize + serde::DeserializeOwned`),
/// or simply `Vec<u8>` representing serialized bytes. The `topic` would also
/// be more structured.
#[derive(Debug, Clone)]
pub struct AuraMessage {
    pub topic: String,
    pub data: String, // Simplified to String for this sketch
}

// Simulate a central message bus or topic registry using lazy_static for global initialization.
// - `Arc<Mutex<...>>` is a common pattern for thread-safe shared mutable state in Rust.
// - The `HashMap` maps a topic name (String) to a list of `mpsc::Sender` channels.
//   Each sender corresponds to an active subscriber on that topic.
//
// CAVEAT: This global, mutex-protected HashMap is a major simplification and would be a
// bottleneck in a real, high-performance system. It serves only to demonstrate the
// basic pub/sub interaction within a single process for this sketch.
lazy_static::lazy_static! {
    pub(crate) static ref MESSAGE_BUS: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<AuraMessage>>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// --- Core Initialization & Shutdown ---

/// Initializes the AuraOS core environment.
///
/// This function should be called once at the beginning of an AuraOS application.
/// In a real system, this would perform tasks such as:
/// - Setting up global logging and tracing.
/// - Initializing the parameter server client.
/// - Starting core scheduler threads or executors.
/// - Initializing the AuraComm layer (e.g., network discovery, transport setup).
pub fn init() {
    // For now, just a print statement.
    // In a real system, use a proper logging facade like `log` or `tracing`.
    println!("[AuraCore] Initializing AuraOS environment (v{})...", env!("CARGO_PKG_VERSION"));

    // Example: Initialize a global logger (if not handled by an external crate)
    // setup_global_logger();

    // Example: Connect to a (hypothetical) global parameter service
    // global_parameter_service::connect();
}

/// Shuts down the AuraOS core environment gracefully.
///
/// This function should be called once at the end of an AuraOS application.
/// It handles the orderly termination of AuraOS services and resource cleanup.
/// In a real system, this would:
/// - Signal all active nodes to begin their shutdown sequence (lifecycle management).
/// - Wait for nodes to terminate or force termination after a timeout.
/// - Flush any buffered logs.
/// - Disconnect from network services.
/// - Release any globally held resources.
pub fn shutdown() {
    println!("[AuraCore] Shutting down AuraOS environment...");

    // Example: Signal all nodes to shutdown
    // node_manager::signal_all_nodes_to_shutdown();
    // node_manager::wait_for_all_nodes_shutdown(Duration::from_secs(5));

    // Example: Flush logs
    // global_logger::flush();
}

// --- Utility Macros (Internal to aura_core) ---

/// A simple internal logging macro for `aura_core`.
///
/// In a production system, `aura_core` would use a standard logging facade
/// like the `log` crate or `tracing` crate, allowing users to configure
/// log levels and outputs. This macro is just for illustrative purposes
/// within this sketch.
///
/// Example: `aura_log!(info, "Node {} created successfully.", node_name);`
macro_rules! aura_log {
    ($level:ident, $($arg:tt)*) => {
        // Simple println-based logging.
        // `stringify!($level)` converts the identifier `info`, `warn`, etc., to a string.
        // `file!()` and `line!()` give the source location of the log call.
        println!(
            "[AuraCore::{}] [{}:{}] {}",
            stringify!($level).to_uppercase(),
            file!(),
            line!(),
            format!($($arg)*)
        );
    };
}
// Make the macro available for use *within* the `aura_core` crate (e.g., in submodules).
// It's not part of the public API unless explicitly re-exported with `pub use`.
pub(crate) use aura_log;
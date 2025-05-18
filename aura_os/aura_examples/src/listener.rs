
// aura_os/aura_examples/src/listener.rs

// Import necessary items from our aura_core crate
use aura_core::node::Node;
use aura_core::comm::Subscriber; // We'll use the simplified Subscriber
use aura_core::Result as AuraResult; // Use the AuraError Result type alias

// Standard library imports
use std::time::Duration;
// No `thread::sleep` needed here if we are just blocking on recv,
// but `Duration` is used for `recv_timeout`.

// The main function for the listener example.
// It returns `AuraResult<()>` to allow using the `?` operator for AuraErrors.
fn main() -> AuraResult<()> {
    println!("[AuraListenerExample] Starting up...");

    // 1. Initialize the AuraOS core environment.
    aura_core::init();

    // 2. Create an AuraOS Node.
    let listener_node = Node::new("listener_node", "/examples")?;
    println!(
        "[AuraListenerExample] Node '{}' created with ID '{}'.",
        listener_node.fully_qualified_name(),
        listener_node.unique_id()
    );

    // 3. Create a Subscriber.
    // Nodes use subscribers to receive messages from topics.
    // We subscribe to the same topic the talker is publishing to.
    // The topic name "/examples/chatter" is an absolute topic name.
    // We could also use `listener_node.create_subscriber("chatter")?`
    let chatter_subscriber: Subscriber = listener_node.create_subscriber("chatter")?;
    // Or, for an absolute topic name:
    // let chatter_subscriber = Subscriber::new("/examples/chatter")?;
    println!(
        "[AuraListenerExample] Subscriber created for topic: '{}'. Waiting for messages...",
        chatter_subscriber.topic_name()
    );

    // 4. Main loop: Receive and process messages.
    // In a real AuraOS application with an executor, this would typically be a callback
    // function registered with the subscriber. For this sketch, we poll manually.

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n[AuraListenerExample] CTRL-C received, signaling shutdown...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut messages_received_count = 0;

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Conceptually, allow the node to perform any internal processing.
        listener_node.spin_once();

        // Try to receive a message with a timeout.
        // This prevents the loop from blocking indefinitely if no messages are coming
        // and allows the `running` flag to be checked periodically.
        match chatter_subscriber.recv_timeout(Duration::from_millis(100)) {
            Ok(aura_message) => {
                // A message was successfully received.
                aura_core::aura_log!(info,
                    "[{}] Received on topic '{}': \"{}\"",
                    listener_node.fully_qualified_name(),
                    aura_message.topic, // AuraMessage struct contains the topic
                    aura_message.data   // And the actual data
                );
                messages_received_count += 1;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // No message received within the timeout. This is normal.
                // Continue the loop to check the `running` flag and try again.
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // The channel is disconnected. This likely means all publishers
                // (our talker, in this case) have been dropped.
                aura_core::aura_log!(warn,
                    "[{}] Subscription channel for topic '{}' disconnected. Assuming no more messages.",
                    listener_node.fully_qualified_name(),
                    chatter_subscriber.topic_name()
                );
                break; // Exit the loop.
            }
        }

        // For this example, let's add a condition to stop if we've received a certain number
        // of messages and we're not in a CI environment (where we want it to run until talker stops or CTRL-C).
        // This is just to make the example terminate naturally if not interrupted by CTRL-C.
        // if messages_received_count >= 10 && std::env::var("CI").is_err() {
        //     aura_core::aura_log!(info, "Listener example reached message limit. Signaling shutdown.");
        //     running.store(false, std::sync::atomic::Ordering::SeqCst);
        // }
    }

    // 5. Shutdown AuraOS gracefully.
    println!("[AuraListenerExample] Loop finished. Shutting down AuraOS...");
    aura_core::shutdown();

    println!("[AuraListenerExample] Exited cleanly.");
    Ok(())
}
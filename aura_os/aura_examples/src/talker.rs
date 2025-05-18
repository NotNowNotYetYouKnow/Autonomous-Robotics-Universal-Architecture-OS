
// aura_os/aura_examples/src/talker.rs

// Import necessary items from our aura_core crate
use aura_core::node::Node;
use aura_core::comm::Publisher; // We'll use the simplified Publisher
use aura_core::param::ParamValue;
use aura_core::Result as AuraResult; // Use the AuraError Result type alias

// Standard library imports
use std::thread;
use std::time::Duration;

// The main function for the talker example.
// It returns `AuraResult<()>` to allow using the `?` operator for AuraErrors.
fn main() -> AuraResult<()> {
    println!("[AuraTalkerExample] Starting up...");

    // 1. Initialize the AuraOS core environment.
    // This should be the first AuraOS-related call in your application.
    aura_core::init();

    // 2. Create an AuraOS Node.
    // Nodes are the fundamental building blocks of an AuraOS application.
    // We give it a name ("talker_node") and a namespace ("/examples").
    // The `?` operator will propagate any errors from `Node::new`.
    let talker_node = Node::new("talker_node", "/examples")?;
    println!(
        "[AuraTalkerExample] Node '{}' created with ID '{}'.",
        talker_node.fully_qualified_name(),
        talker_node.unique_id()
    );

    // 3. Declare and manage parameters for this node.
    // Parameters allow configuring node behavior without recompiling code.
    let node_params = talker_node.params(); // Get a reference to the node's parameter manager

    // Declare a parameter for the publishing rate (in Hz).
    // If this parameter is set externally (e.g., via a config file in a real system),
    // that value would be used. Otherwise, the default is 1.0 Hz.
    node_params.declare_parameter("publish_rate_hz", ParamValue::Float(1.0))?;

    // Declare a parameter for the greeting message.
    node_params.declare_parameter(
        "greeting_message",
        ParamValue::String("Hello from AuraOS Talker!".to_string()),
    )?;

    // Get the current values of the parameters.
    // The `unwrap_or` provides a fallback if the parameter somehow isn't a float
    // (though `declare_parameter` sets it, robust code might handle type errors better).
    let publish_rate_hz = node_params
        .get_parameter("publish_rate_hz")?
        .as_f64() // Attempt to get as f64
        .unwrap_or_else(|| {
            aura_core::aura_log!(warn, "publish_rate_hz parameter is not a float, using default 1.0");
            1.0
        });

    let greeting_message = node_params
        .get_parameter("greeting_message")?
        .as_string() // Attempt to get as String
        .cloned() // Clone the String out of the Option<&String>
        .unwrap_or_else(|| {
            aura_core::aura_log!(warn, "greeting_message parameter is not a string, using default.");
            "Default Greeting (param error)".to_string()
        });

    println!(
        "[AuraTalkerExample] Configured to publish at {:.2} Hz with message prefix: '{}'",
        publish_rate_hz, greeting_message
    );

    // Calculate the sleep duration based on the desired publishing rate.
    let sleep_duration = if publish_rate_hz > 0.0 {
        Duration::from_secs_f64(1.0 / publish_rate_hz)
    } else {
        Duration::from_secs(1) // Default to 1 second if rate is invalid
    };

    // 4. Create a Publisher.
    // Nodes use publishers to send messages on topics.
    // The topic name "/examples/chatter" is an absolute topic name.
    // We could also use `talker_node.create_publisher("chatter")?` which would resolve
    // to "/examples/chatter" based on the node's namespace.
    let chatter_publisher: Publisher = talker_node.create_publisher("chatter")?;
    // Or, for an absolute topic name:
    // let chatter_publisher = Publisher::new("/examples/chatter")?;
    println!(
        "[AuraTalkerExample] Publisher created for topic: '{}'",
        chatter_publisher.topic_name()
    );

    // 5. Main loop: Publish messages periodically.
    let mut count: u32 = 0;
    println!("[AuraTalkerExample] Starting to publish messages...");

    // In a real AuraOS application, this loop would likely be driven by an executor
    // managing a timer callback. For this sketch, we use a simple `loop` and `thread::sleep`.
    // We'll use a `ctrlc` handler to allow graceful shutdown.
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n[AuraTalkerExample] CTRL-C received, signaling shutdown...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");


    while running.load(std::sync::atomic::Ordering::SeqCst) {
        // Conceptually, allow the node to perform any internal processing.
        // In a real system, this isn't how nodes are typically "spun".
        talker_node.spin_once();

        let message_data = format!("{} Count: {}", greeting_message, count);

        aura_core::aura_log!(info, "[{}] Publishing: '{}'", talker_node.fully_qualified_name(), message_data);

        // Publish the message. The `?` operator handles potential communication errors.
        match chatter_publisher.publish(message_data) {
            Ok(_) => {} // Message sent successfully
            Err(e) => {
                // Log the error but continue trying, as subscribers might come and go.
                aura_core::aura_log!(error, "[{}] Failed to publish: {}", talker_node.fully_qualified_name(), e);
            }
        }

        count = count.wrapping_add(1); // Increment count, wrapping around on overflow

        // Wait for the next publishing interval.
        thread::sleep(sleep_duration);

        // For this example, let's limit the number of publishes if no CTRL-C
        // This is just to make the example terminate if not interrupted.
        // A real node would run until explicitly shut down.
        // if count > 20 && std::env::var("CI").is_err() { // Don't auto-exit in CI
        //     aura_core::aura_log!(info, "Talker example reached message limit. Signaling shutdown.");
        //     running.store(false, std::sync::atomic::Ordering::SeqCst);
        // }
    }

    // 6. Shutdown AuraOS gracefully.
    // This should be called before the application exits to ensure proper cleanup.
    println!("[AuraTalkerExample] Loop finished. Shutting down AuraOS...");
    aura_core::shutdown();

    println!("[AuraTalkerExample] Exited cleanly.");
    Ok(())
}
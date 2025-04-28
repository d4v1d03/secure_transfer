// Removed #[macro_use] and extern crate lazy_static

use std::thread;
use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;

// Declare modules
mod model;
mod constants;
mod crypto;
mod network;

fn main() {
    println!("Pseudo Secure Channel Simulation");

    // Generate some sample data to send
    let sample_data = model::TransferData {
        transaction_id: Uuid::new_v4().to_string(),
        payload: b"This is some important data being sent securely (not really).".to_vec(),
        created_at: Utc::now(),
    };

    // Clone data for the sender thread
    let sender_data = sample_data.clone();

    // Start receiver (server) in a separate thread
    let receiver_handle = thread::spawn(move || {
        network::run_receiver();
    });

    // Wait briefly for the receiver to start up
    thread::sleep(Duration::from_secs(1));

    // Start sender (client) in a separate thread
    let sender_handle = thread::spawn(move || {
        network::run_sender(sender_data);
    });

    // Wait for threads to complete (or add better handling)
    sender_handle.join().expect("Sender thread panicked");
    receiver_handle.join().expect("Receiver thread panicked");

    println!("Simulation finished.");
}

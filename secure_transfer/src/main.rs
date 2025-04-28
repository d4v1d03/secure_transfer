use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

// Simple generic data structure
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GenericData {
    id: u32,
    message: String,
    sent_at: DateTime<Utc>,
}

// --- Encryption Logic: Rotating Character Shift ---

// Generates a simple base key from timestamp nanoseconds
// NOTE: Still NOT cryptographically secure.
fn generate_base_key(timestamp: &DateTime<Utc>) -> u8 {
    (timestamp.timestamp_nanos_opt().unwrap_or(0) % 256) as u8
}

// Encrypts data using Rotating Character Shift
fn encrypt_data_rcs(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().enumerate().map(|(i, byte)| {
        let shift = key.wrapping_add(i as u8); // Calculate shift based on key + index (modulo is implicit with u8 wrapping)
        byte.wrapping_add(shift) // Add shift with wrapping
    }).collect()
}

// Decrypts data using Rotating Character Shift
fn decrypt_data_rcs(encrypted_data: &[u8], key: u8) -> Vec<u8> {
    encrypted_data.iter().enumerate().map(|(i, byte)| {
        let shift = key.wrapping_add(i as u8); // Calculate the *same* shift (modulo is implicit with u8 wrapping)
        byte.wrapping_sub(shift) // Subtract shift with wrapping
    }).collect()
}

// Wrapper to send timestamp with payload
#[derive(Serialize, Deserialize, Debug)]
struct SecurePacket {
    timestamp: DateTime<Utc>,
    payload: Vec<u8>,
}

// --- Simulation Logic ---

fn main() {
    println!("Secure Transfer Simulation (Rotating Character Shift)");

    // Start receiver (server) in a separate thread
    thread::spawn(move || {
        receiver_logic_rcs();
    });

    // Wait a bit for the server to start
    thread::sleep(Duration::from_secs(1));

    // Start sender (client)
    sender_logic_rcs();

    // Keep main thread alive briefly
    thread::sleep(Duration::from_secs(2));
    println!("Simulation finished.");
}

// Simulates the receiving peer
fn receiver_logic_rcs() {
    let listener = TcpListener::bind("127.0.0.1:8081").expect("Failed to bind receiver socket"); // Use a different port
    println!("Receiver RCS listening on 127.0.0.1:8081...");

    match listener.accept() {
        Ok((mut stream, addr)) => {
            println!("Receiver RCS connected to sender: {}", addr);

            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer).expect("Failed to read from stream");

            if buffer.is_empty() {
                println!("Receiver RCS received empty data.");
                return;
            }

            // Deserialize the wrapper packet
            let packet: SecurePacket = serde_json::from_slice(&buffer)
                .expect("Failed to deserialize packet");
            println!("Receiver RCS got packet with timestamp: {}", packet.timestamp);

            // Generate the key using the received timestamp
            let key = generate_base_key(&packet.timestamp);
            println!("Receiver RCS generated key: {}", key);

            // Decrypt the payload
            let decrypted_payload = decrypt_data_rcs(&packet.payload, key);

            // Deserialize the original GenericData
            match serde_json::from_slice::<GenericData>(&decrypted_payload) {
                 Ok(data) => {
                    println!("Receiver RCS decrypted data: {:?}", data);
                 },
                 Err(e) => {
                    eprintln!("Receiver RCS failed to deserialize decrypted data: {}", e);
                    // println!("Raw decrypted bytes: {:?}", decrypted_payload);
                 }
            }
        }
        Err(e) => {
            eprintln!("Receiver RCS failed to accept connection: {}", e);
        }
    }
}

// Simulates the sending peer
fn sender_logic_rcs() {
    match TcpStream::connect("127.0.0.1:8081") { // Connect to the new port
        Ok(mut stream) => {
            println!("Sender RCS connected to receiver.");

            // Create sample data
            let sample_data = GenericData {
                id: 123,
                message: "This is a secret message.".to_string(),
                sent_at: Utc::now(),
            };
            println!("Sender RCS original data: {:?}", sample_data);

            // Serialize the original data
            let serialized_data = serde_json::to_vec(&sample_data).expect("Failed to serialize data");

            // Use a consistent timestamp for key generation and packet wrapping
            let encryption_timestamp = Utc::now();
            println!("Sender RCS using timestamp for encryption: {}", encryption_timestamp);

            // Generate the encryption key
            let key = generate_base_key(&encryption_timestamp);
            println!("Sender RCS generated key: {}", key);

            // Encrypt the serialized data
            let encrypted_payload = encrypt_data_rcs(&serialized_data, key);

            // Create the wrapper packet
            let packet = SecurePacket {
                timestamp: encryption_timestamp,
                payload: encrypted_payload,
            };

            // Serialize the wrapper packet
            let serialized_packet = serde_json::to_vec(&packet).expect("Failed to serialize packet");

            // Send the encrypted packet
            stream.write_all(&serialized_packet).expect("Failed to write to stream");
            println!("Sender RCS sent encrypted packet.");
        }
        Err(e) => {
            eprintln!("Sender RCS failed to connect: {}", e);
        }
    }
}

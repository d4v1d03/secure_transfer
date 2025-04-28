use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, BufReader};
use std::thread;
use std::time::Duration;
use rand::Rng;
use std::fs::File;
use std::path::Path;

// Placeholder for SCADA data - adapt based on excelData.xlsx
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScadaData {
    timestamp: DateTime<Utc>,
    source_id: String,
    sensor_name: String,
    value: f64,
    status: String,
}

// --- Encryption Logic: Rolling XOR with Timestamp ---

// Generates a simple XOR key based on the timestamp's nanoseconds
// NOTE: This is NOT cryptographically secure, for demonstration only.
fn generate_xor_key(timestamp: &DateTime<Utc>) -> u8 {
    // Use nanoseconds for variability, wrap around 256 for a byte key
    (timestamp.timestamp_nanos_opt().unwrap_or(0) % 256) as u8
}

// Encrypts data using the generated XOR key
fn encrypt_data(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|byte| byte ^ key).collect()
}

// Decrypts data using the same XOR key (XOR is symmetric)
fn decrypt_data(encrypted_data: &[u8], key: u8) -> Vec<u8> {
    encrypt_data(encrypted_data, key) // XORing again decrypts
}

// Wrapper structure to send timestamp along with encrypted data
#[derive(Serialize, Deserialize, Debug)]
struct EncryptedPacket {
    timestamp: DateTime<Utc>,
    payload: Vec<u8>,
}

// --- Simulation Logic ---

fn main() {
    println!("SCADA Shim Simulation");

    // Start receiver (server) in a separate thread
    thread::spawn(move || {
        receiver_logic();
    });

    // Wait a bit for the server to start
    thread::sleep(Duration::from_secs(1));

    // Start sender (client)
    sender_logic();

    // Keep main thread alive briefly to see output (in a real app, handle threads properly)
    thread::sleep(Duration::from_secs(2));
    println!("Simulation finished.");

}

// Simulates the receiving peer
fn receiver_logic() {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind receiver socket");
    println!("Receiver listening on 127.0.0.1:8080...");

    match listener.accept() {
        Ok((mut stream, addr)) => {
            println!("Receiver connected to sender: {}", addr);

            let mut buffer = Vec::new();
            stream.read_to_end(&mut buffer).expect("Failed to read from stream");

            if buffer.is_empty() {
                println!("Receiver received empty data.");
                return;
            }

            // Deserialize the wrapper packet
            let packet: EncryptedPacket = serde_json::from_slice(&buffer)
                .expect("Failed to deserialize packet");

            println!("Receiver got packet with timestamp: {}", packet.timestamp);

            // Generate the key using the *received* timestamp
            let key = generate_xor_key(&packet.timestamp);
            println!("Receiver generated key: {}", key);

            // Decrypt the payload
            let decrypted_payload = decrypt_data(&packet.payload, key);

            // Deserialize the original ScadaData
            match serde_json::from_slice::<ScadaData>(&decrypted_payload) {
                 Ok(data) => {
                    println!("Receiver decrypted data: {:?}", data);
                 },
                 Err(e) => {
                    eprintln!("Receiver failed to deserialize decrypted data: {}", e);
                    // Optionally print the raw decrypted bytes for debugging
                    // println!("Raw decrypted bytes: {:?}", decrypted_payload);
                 }
            }


        }
        Err(e) => {
            eprintln!("Receiver failed to accept connection: {}", e);
        }
    }
}

// Simulates the sending peer
fn sender_logic() {
    // Read data from JSON file
    let data_path = Path::new("data.json");
    let file = match File::open(&data_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Sender Error: Failed to open data.json: {}", e);
            return;
        }
    };
    let reader = BufReader::new(file);

    let scada_records: Vec<ScadaData> = match serde_json::from_reader(reader) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Sender Error: Failed to parse data.json: {}", e);
            return;
        }
    };

    // Get the first record (if any)
    if let Some(data_to_send) = scada_records.first().cloned() {
        println!("Sender reading data from JSON: {:?}", data_to_send);

        match TcpStream::connect("127.0.0.1:8080") {
            Ok(mut stream) => {
                println!("Sender connected to receiver.");

                // Serialize the original data
                let serialized_data = serde_json::to_vec(&data_to_send).expect("Failed to serialize data");

                // Use a consistent timestamp for key generation and packet wrapping
                let encryption_timestamp = Utc::now();
                println!("Sender using timestamp for encryption: {}", encryption_timestamp);

                // Generate the encryption key
                let key = generate_xor_key(&encryption_timestamp);
                println!("Sender generated key: {}", key);

                // Encrypt the serialized data
                let encrypted_payload = encrypt_data(&serialized_data, key);

                // Create the wrapper packet
                let packet = EncryptedPacket {
                    timestamp: encryption_timestamp, // Send the timestamp used for key generation
                    payload: encrypted_payload,
                };

                // Serialize the wrapper packet
                let serialized_packet = serde_json::to_vec(&packet).expect("Failed to serialize packet");

                // Send the encrypted packet
                stream.write_all(&serialized_packet).expect("Failed to write to stream");
                println!("Sender sent encrypted packet.");

                // Explicitly close the stream (optional, happens on drop anyway)
                // stream.shutdown(std::net::Shutdown::Both).expect("shutdown call failed");

            }
            Err(e) => {
                eprintln!("Sender failed to connect: {}", e);
            }
        }
    } else {
        println!("Sender Info: No data records found in data.json.");
    }
}

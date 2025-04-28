use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

use crate::model::{TransferData, EncryptedPacket};
use crate::crypto::{encrypt, decrypt};
use crate::constants::NETWORK_PORT;

// Simulates the receiving peer
pub fn run_receiver() {
    let address = format!("127.0.0.1:{}", NETWORK_PORT);
    let listener = match TcpListener::bind(&address) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[Receiver] Failed to bind to {}: {}", address, e);
            return;
        }
    };
    println!("[Receiver] Listening on {}...", address);

    match listener.accept() {
        Ok((mut stream, addr)) => {
            println!("[Receiver] Accepted connection from: {}", addr);

            let mut buffer = Vec::new();
            if let Err(e) = stream.read_to_end(&mut buffer) {
                eprintln!("[Receiver] Failed to read from stream: {}", e);
                return;
            }

            if buffer.is_empty() {
                println!("[Receiver] Received empty data.");
                return;
            }

            // Deserialize the wrapper packet
            let packet: EncryptedPacket = match serde_json::from_slice(&buffer) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[Receiver] Failed to deserialize packet: {}", e);
                    // Also log raw bytes for debugging if needed
                    // eprintln!("[Receiver] Raw buffer: {:?}", buffer);
                    return;
                }
            };
            println!("[Receiver] Received packet with nonce: ({} bytes)", packet.nonce.len());

            // Decrypt the payload using the received nonce (and the known PSS)
            match decrypt(&packet.encrypted_payload, &packet.nonce) {
                Ok(decrypted_payload) => {
                    // Deserialize the original TransferData
                    match serde_json::from_slice::<TransferData>(&decrypted_payload) {
                        Ok(data) => {
                            println!("[Receiver] Successfully decrypted data:");
                            println!("  Transaction ID: {}", data.transaction_id);
                            println!("  Created At: {}", data.created_at); // Still present in TransferData
                            println!("  Payload Length: {}", data.payload.len());
                            if let Ok(payload_str) = String::from_utf8(data.payload) {
                                println!("  Payload Preview: {:?}...", payload_str.chars().take(50).collect::<String>());
                            } else {
                                println!("  Payload: (non-UTF8 data)");
                            }
                        },
                        Err(e) => {
                            eprintln!("[Receiver] Failed to deserialize decrypted data: {}", e);
                        }
                   }
                },
                Err(e) => {
                    eprintln!("[Receiver] Failed to decrypt payload: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[Receiver] Failed to accept connection: {}", e);
        }
    }
}

// Simulates the sending peer
pub fn run_sender(data_to_send: TransferData) {
    let address = format!("127.0.0.1:{}", NETWORK_PORT);
    println!("[Sender] Attempting to connect to {}...", address);

    match TcpStream::connect(&address) {
        Ok(mut stream) => {
            println!("[Sender] Connected to receiver.");

            println!("[Sender] Original data:");
            println!("  Transaction ID: {}", data_to_send.transaction_id);
            println!("  Payload Length: {}", data_to_send.payload.len());

            // Serialize the original data
            let serialized_data = match serde_json::to_vec(&data_to_send) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[Sender] Failed to serialize data: {}", e);
                    return;
                }
            };

            // Encrypt the serialized data - returns payload AND nonce
            let (encrypted_payload, nonce) = encrypt(&serialized_data);
            println!("[Sender] Encrypted payload length: {}", encrypted_payload.len());
            println!("[Sender] Generated nonce: ({} bytes)", nonce.len());

            // Create the wrapper packet with the nonce
            let packet = EncryptedPacket {
                nonce, // Use the generated nonce
                encrypted_payload,
            };

            // Serialize the wrapper packet
            let serialized_packet = match serde_json::to_vec(&packet) {
                Ok(sp) => sp,
                Err(e) => {
                    eprintln!("[Sender] Failed to serialize packet: {}", e);
                    return;
                }
            };

            // Send the encrypted packet
            if let Err(e) = stream.write_all(&serialized_packet) {
                eprintln!("[Sender] Failed to write to stream: {}", e);
            } else {
                println!("[Sender] Sent encrypted packet ({} bytes).", serialized_packet.len());
            }
            // Shutdown gracefully
             let _ = stream.shutdown(std::net::Shutdown::Write);
        }
        Err(e) => {
            eprintln!("[Sender] Failed to connect: {}", e);
        }
    }
}

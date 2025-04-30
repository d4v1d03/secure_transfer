import json
import socket
import threading
import time
from dataclasses import dataclass
from datetime import datetime
import random
import os
from typing import List


@dataclass
class ScadaData:
    timestamp: datetime
    source_id: str
    sensor_name: str
    value: float
    status: str


def generate_xor_key(timestamp: datetime) -> int:
    """Generate an encryption key based on timestamp."""
    # Extract more entropy from the timestamp to avoid always getting 0
    seconds = int(timestamp.timestamp())
    microseconds = timestamp.microsecond
    # Combine seconds and microseconds to get a more varied key
    combined = (seconds + microseconds) % 256
    if combined == 0:  # Ensure key is never 0
        combined = 1
    return combined


def encrypt_data(data: bytes, key: int) -> bytes:
    """Encrypt data using XOR with the given key."""
    return bytes(byte ^ key for byte in data)


def decrypt_data(encrypted_data: bytes, key: int) -> bytes:
    """Decrypt data using XOR with the given key."""
    return encrypt_data(encrypted_data, key)


@dataclass
class EncryptedPacket:
    timestamp: datetime
    payload: bytes


def main():
    print("SCADA Shim Simulation")
    
    # Start receiver in a separate thread
    receiver_thread = threading.Thread(target=receiver_logic)
    receiver_thread.start()
    
    # Give the receiver time to start
    time.sleep(1)
    
    # Run sender
    sender_logic()
    
    # Give time for simulation to complete
    time.sleep(2)
    print("Simulation finished.")


def receiver_logic():
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.bind(("127.0.0.1", 8080))
        sock.listen(1)
        print("Receiver listening on 127.0.0.1:8080...")
        
        conn, addr = sock.accept()
        print(f"Receiver connected to sender: {addr}")
        
        buffer = b""
        while True:
            data = conn.recv(4096)
            if not data:
                break
            buffer += data
        
        if not buffer:
            print("Receiver received empty data.")
            return
        
        # Deserialize the packet
        packet_data = json.loads(buffer.decode())
        packet = EncryptedPacket(
            timestamp=datetime.fromisoformat(packet_data["timestamp"]),
            payload=bytes(packet_data["payload"])
        )
        
        print(f"Receiver got packet with timestamp: {packet.timestamp}")
        
        key = generate_xor_key(packet.timestamp)
        print(f"Receiver generated key: {key}")
        
        decrypted_payload = decrypt_data(packet.payload, key)
        
        try:
            data_json = json.loads(decrypted_payload.decode())
            data = ScadaData(
                timestamp=datetime.fromisoformat(data_json["timestamp"]),
                source_id=data_json["source_id"],
                sensor_name=data_json["sensor_name"],
                value=data_json["value"],
                status=data_json["status"]
            )
            print(f"Receiver decrypted data: {data}")
        except Exception as e:
            print(f"Receiver failed to deserialize decrypted data: {e}")
    
    except Exception as e:
        print(f"Receiver error: {e}")
    finally:
        try:
            sock.close()
        except:
            pass


def sender_logic():
    try:
        # Read data from JSON file
        try:
            with open("data.json", "r") as file:
                scada_records_json = json.load(file)
                
            scada_records = []
            for record in scada_records_json:
                scada_records.append(ScadaData(
                    timestamp=datetime.fromisoformat(record["timestamp"]),
                    source_id=record["source_id"],
                    sensor_name=record["sensor_name"],
                    value=record["value"],
                    status=record["status"]
                ))
                
        except Exception as e:
            print(f"Sender Error: Failed to read data.json: {e}")
            return
        
        if not scada_records:
            print("Sender Info: No data records found in data.json.")
            return
        
        data_to_send = scada_records[0]
        print(f"Sender reading data from JSON: {data_to_send}")
        
        # Connect to receiver
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(("127.0.0.1", 8080))
        print("Sender connected to receiver.")
        
        # Serialize the data
        serialized_data = json.dumps({
            "timestamp": data_to_send.timestamp.isoformat(),
            "source_id": data_to_send.source_id,
            "sensor_name": data_to_send.sensor_name,
            "value": data_to_send.value,
            "status": data_to_send.status
        }).encode()
        
        # Encrypt the data
        encryption_timestamp = datetime.now()
        print(f"Sender using timestamp for encryption: {encryption_timestamp}")
        
        key = generate_xor_key(encryption_timestamp)
        print(f"Sender generated key: {key}")
        
        encrypted_payload = encrypt_data(serialized_data, key)
        
        # Create and send the packet
        packet = {
            "timestamp": encryption_timestamp.isoformat(),
            "payload": list(encrypted_payload)
        }
        
        serialized_packet = json.dumps(packet).encode()
        sock.sendall(serialized_packet)
        print("Sender sent encrypted packet.")
        
        sock.close()
    
    except Exception as e:
        print(f"Sender error: {e}")


if __name__ == "__main__":
    main() 
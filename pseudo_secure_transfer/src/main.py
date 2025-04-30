import json
import socket
import threading
import time
from dataclasses import dataclass
from datetime import datetime
from typing import List, Optional


@dataclass
class GenericData:
    id: int
    message: str
    sent_at: datetime


def generate_base_key(timestamp: datetime) -> int:
    """Generate an encryption key based on timestamp."""
    # Extract more entropy from the timestamp to avoid always getting 0
    seconds = int(timestamp.timestamp())
    microseconds = timestamp.microsecond
    # Combine seconds and microseconds to get a more varied key
    combined = (seconds + microseconds) % 256
    if combined == 0:  # Ensure key is never 0
        combined = 1
    return combined


def encrypt_data_rcs(data: bytes, key: int) -> bytes:
    """Encrypt data using Rotating Character Shift."""
    result = bytearray()
    for i, byte in enumerate(data):
        shift = (key + i) % 256
        result.append((byte + shift) % 256)  # Adding with wrapping
    return bytes(result)


def decrypt_data_rcs(encrypted_data: bytes, key: int) -> bytes:
    """Decrypt data using Rotating Character Shift."""
    result = bytearray()
    for i, byte in enumerate(encrypted_data):
        shift = (key + i) % 256
        result.append((byte - shift) % 256)  # Subtracting with wrapping
    return bytes(result)


@dataclass
class SecurePacket:
    timestamp: datetime
    payload: bytes


def main():
    print("Secure Transfer Simulation (Rotating Character Shift)")
    
    # Start receiver in a separate thread
    receiver_thread = threading.Thread(target=receiver_logic_rcs)
    receiver_thread.start()
    
    # Give the receiver time to start
    time.sleep(1)
    
    # Run sender
    sender_logic_rcs()
    
    # Give time for simulation to complete
    time.sleep(2)
    print("Simulation finished.")


def receiver_logic_rcs():
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.bind(("127.0.0.1", 8081))
        sock.listen(1)
        print("Receiver RCS listening on 127.0.0.1:8081...")
        
        conn, addr = sock.accept()
        print(f"Receiver RCS connected to sender: {addr}")
        
        buffer = b""
        while True:
            data = conn.recv(4096)
            if not data:
                break
            buffer += data
        
        if not buffer:
            print("Receiver RCS received empty data.")
            return
        
        # Deserialize the packet
        packet_data = json.loads(buffer.decode())
        packet = SecurePacket(
            timestamp=datetime.fromisoformat(packet_data["timestamp"]),
            payload=bytes(packet_data["payload"])
        )
        
        print(f"Receiver RCS got packet with timestamp: {packet.timestamp}")
        
        key = generate_base_key(packet.timestamp)
        print(f"Receiver RCS generated key: {key}")
        
        decrypted_payload = decrypt_data_rcs(packet.payload, key)
        
        try:
            data_json = json.loads(decrypted_payload.decode())
            data = GenericData(
                id=data_json["id"],
                message=data_json["message"],
                sent_at=datetime.fromisoformat(data_json["sent_at"])
            )
            print(f"Receiver RCS decrypted data: {data}")
        except Exception as e:
            print(f"Receiver RCS failed to deserialize decrypted data: {e}")
    
    except Exception as e:
        print(f"Receiver RCS error: {e}")
    finally:
        try:
            sock.close()
        except:
            pass


def sender_logic_rcs():
    try:
        # Connect to receiver
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(("127.0.0.1", 8081))
        print("Sender RCS connected to receiver.")
        
        # Create sample data
        sample_data = GenericData(
            id=123,
            message="This is a secret message.",
            sent_at=datetime.now()
        )
        print(f"Sender RCS original data: {sample_data}")
        
        # Serialize the data
        serialized_data = json.dumps({
            "id": sample_data.id,
            "message": sample_data.message,
            "sent_at": sample_data.sent_at.isoformat()
        }).encode()
        
        # Encrypt the data
        encryption_timestamp = datetime.now()
        print(f"Sender RCS using timestamp for encryption: {encryption_timestamp}")
        
        key = generate_base_key(encryption_timestamp)
        print(f"Sender RCS generated key: {key}")
        
        encrypted_payload = encrypt_data_rcs(serialized_data, key)
        
        # Create and send the packet
        packet = {
            "timestamp": encryption_timestamp.isoformat(),
            "payload": list(encrypted_payload)
        }
        
        serialized_packet = json.dumps(packet).encode()
        sock.sendall(serialized_packet)
        print("Sender RCS sent encrypted packet.")
        
        sock.close()
    
    except Exception as e:
        print(f"Sender RCS error: {e}")


if __name__ == "__main__":
    main() 
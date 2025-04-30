import socket
import json
from datetime import datetime
import struct

from constants import NETWORK_PORT
from crypto import encrypt, decrypt
from model import EncryptedPacket, TransferData


class DateTimeEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, datetime):
            return obj.isoformat()
        elif isinstance(obj, bytes):
            return list(obj)  # Convert bytes to list for JSON serialization
        return super().default(obj)


def run_receiver():
    address = f"127.0.0.1:{NETWORK_PORT}"
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.bind(("127.0.0.1", NETWORK_PORT))
        sock.listen(1)
        print(f"[Receiver] Listening on {address}...")

        conn, addr = sock.accept()
        print(f"[Receiver] Accepted connection from: {addr}")

        buffer = b""
        while True:
            data = conn.recv(4096)
            if not data:
                break
            buffer += data

        if not buffer:
            print("[Receiver] Received empty data.")
            return

        # Deserialize the received packet
        packet_data = json.loads(buffer.decode())
        packet = EncryptedPacket(
            nonce=bytes(packet_data["nonce"]),
            encrypted_payload=bytes(packet_data["encrypted_payload"])
        )
        print(f"[Receiver] Received packet with nonce: ({len(packet.nonce)} bytes)")

        try:
            # Decrypt the payload
            decrypted_payload = decrypt(packet.encrypted_payload, packet.nonce)
            
            # Deserialize the decrypted data
            data_json = json.loads(decrypted_payload.decode())
            data = TransferData(
                transaction_id=data_json["transaction_id"],
                payload=bytes(data_json["payload"]),
                created_at=datetime.fromisoformat(data_json["created_at"])
            )
            
            print("[Receiver] Successfully decrypted data:")
            print(f"  Transaction ID: {data.transaction_id}")
            print(f"  Created At: {data.created_at}")
            print(f"  Payload Length: {len(data.payload)}")
            
            try:
                payload_str = data.payload.decode('utf-8')
                print(f"  Payload Preview: {payload_str[:50]}...")
            except UnicodeDecodeError:
                print("  Payload: (non-UTF8 data)")
        
        except Exception as e:
            print(f"[Receiver] Failed to decrypt payload: {e}")
    
    except Exception as e:
        print(f"[Receiver] Error: {e}")
    finally:
        try:
            sock.close()
        except:
            pass


def run_sender(data_to_send: TransferData):
    address = f"127.0.0.1:{NETWORK_PORT}"
    print(f"[Sender] Attempting to connect to {address}...")
    
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.connect(("127.0.0.1", NETWORK_PORT))
        print("[Sender] Connected to receiver.")
        
        print("[Sender] Original data:")
        print(f"  Transaction ID: {data_to_send.transaction_id}")
        print(f"  Payload Length: {len(data_to_send.payload)}")
        
        # Serialize the data
        serialized_data = json.dumps({
            "transaction_id": data_to_send.transaction_id,
            "payload": list(data_to_send.payload),
            "created_at": data_to_send.created_at.isoformat()
        }).encode()
        
        # Encrypt the data
        encrypted_payload, nonce = encrypt(serialized_data)
        print(f"[Sender] Encrypted payload length: {len(encrypted_payload)}")
        print(f"[Sender] Generated nonce: ({len(nonce)} bytes)")
        
        # Create and serialize the packet
        packet = {
            "nonce": list(nonce),
            "encrypted_payload": list(encrypted_payload)
        }
        
        serialized_packet = json.dumps(packet).encode()
        
        # Send the packet
        sock.sendall(serialized_packet)
        print(f"[Sender] Sent encrypted packet ({len(serialized_packet)} bytes).")
        
        sock.close()
    
    except Exception as e:
        print(f"[Sender] Error: {e}") 
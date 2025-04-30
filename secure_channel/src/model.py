from dataclasses import dataclass
from datetime import datetime
from typing import List, Any


@dataclass
class TransferData:
    transaction_id: str
    payload: bytes
    created_at: datetime


@dataclass
class EncryptedPacket:
    nonce: bytes
    encrypted_payload: bytes 
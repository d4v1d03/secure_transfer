import random
import os
import struct
from typing import Tuple, List, Optional
import functools

from constants import (
    BLOCK_SIZE,
    KEY_DERIVATION_CONSTANT_1,
    KEY_DERIVATION_CONSTANT_2,
    LCG_A,
    LCG_C,
    NONCE_SIZE,
    PRE_SHARED_SECRET,
)


def derive_keys(pss: bytes, nonce: bytes) -> Tuple[int, int]:
    derived_key_material = 0
    max_len = max(len(pss), len(nonce))
    
    for i in range(max_len):
        pss_byte = pss[i % len(pss)] if i < len(pss) else 0
        nonce_byte = nonce[i % len(nonce)] if i < len(nonce) else 0
        xored_byte = pss_byte ^ nonce_byte
        
        # Simulate Rust's rotate_left
        derived_key_material = ((derived_key_material << 8) | (derived_key_material >> 56) & 0xFF) & 0xFFFFFFFFFFFFFFFF
        derived_key_material ^= xored_byte
    
    k1 = (derived_key_material & 0xFF)
    k2 = ((derived_key_material >> 8) & 0xFF)
    
    final_k1 = k1 ^ KEY_DERIVATION_CONSTANT_1
    final_k2 = (k2 + KEY_DERIVATION_CONSTANT_2) & 0xFF  # Simulate wrapping_add
    
    return final_k1, final_k2


class Lcg:
    def __init__(self, seed: int):
        self.state = seed
    
    def next(self) -> int:
        # Simulate Rust's wrapping operations
        self.state = (self.state * LCG_A + LCG_C) & 0xFFFFFFFFFFFFFFFF
        return (self.state >> 16) & 0xFF


def add_padding(data: bytes, k2: int) -> bytes:
    original_len = len(data)
    len_bytes = struct.pack(">Q", original_len)  # 8 bytes for 64-bit size
    total_len_with_len = original_len + 8
    padding_len = (BLOCK_SIZE - (total_len_with_len % BLOCK_SIZE)) % BLOCK_SIZE
    
    padded_data = bytearray(len_bytes)
    padded_data.extend(data)
    
    lcg = Lcg(k2)
    for _ in range(padding_len):
        padded_data.append(lcg.next())
    
    return bytes(padded_data)


def remove_padding(padded_data: bytes) -> bytes:
    if len(padded_data) < 8:
        raise ValueError("Padded data too short to contain length")
    if len(padded_data) % BLOCK_SIZE != 0:
        raise ValueError("Padded data length not multiple of block size")
    
    original_len = struct.unpack(">Q", padded_data[:8])[0]
    expected_end = 8 + original_len
    
    if expected_end > len(padded_data):
        raise ValueError("Stored length exceeds padded data length")
    
    return padded_data[8:expected_end]


# Initialize S_BOX and INV_S_BOX
random.seed(LCG_A ^ LCG_C)
S_BOX = list(range(256))
random.shuffle(S_BOX)
INV_S_BOX = [0] * 256
for i, val in enumerate(S_BOX):
    INV_S_BOX[val] = i


def substitute(data: bytearray) -> None:
    for i in range(len(data)):
        data[i] = S_BOX[data[i]]


def inverse_substitute(data: bytearray) -> None:
    for i in range(len(data)):
        data[i] = INV_S_BOX[data[i]]


def permute_blocks(data: bytearray) -> None:
    for i in range(0, len(data), BLOCK_SIZE):
        block = data[i:i+BLOCK_SIZE]
        if len(block) == BLOCK_SIZE:
            data[i], data[i+BLOCK_SIZE-1] = data[i+BLOCK_SIZE-1], data[i]
            data[i+1], data[i+BLOCK_SIZE-2] = data[i+BLOCK_SIZE-2], data[i+1]


def inverse_permute_blocks(data: bytearray) -> None:
    for i in range(0, len(data), BLOCK_SIZE):
        block = data[i:i+BLOCK_SIZE]
        if len(block) == BLOCK_SIZE:
            data[i+1], data[i+BLOCK_SIZE-2] = data[i+BLOCK_SIZE-2], data[i+1]
            data[i], data[i+BLOCK_SIZE-1] = data[i+BLOCK_SIZE-1], data[i]


def xor_with_key(data: bytearray, k1: int) -> None:
    for i in range(len(data)):
        data[i] ^= k1


def encrypt(data: bytes) -> Tuple[bytes, bytes]:
    nonce = os.urandom(NONCE_SIZE)
    k1, k2 = derive_keys(PRE_SHARED_SECRET, nonce)
    
    padded_data = bytearray(add_padding(data, k2))
    
    xor_with_key(padded_data, k1)
    substitute(padded_data)
    permute_blocks(padded_data)
    
    return bytes(padded_data), nonce


def decrypt(encrypted_data: bytes, nonce: bytes) -> bytes:
    if len(encrypted_data) % BLOCK_SIZE != 0:
        raise ValueError("Encrypted data length not multiple of block size")
    if len(nonce) != NONCE_SIZE:
        raise ValueError("Invalid nonce length")
    
    k1, _ = derive_keys(PRE_SHARED_SECRET, nonce)
    
    data = bytearray(encrypted_data)
    
    inverse_permute_blocks(data)
    inverse_substitute(data)
    xor_with_key(data, k1)
    
    return remove_padding(bytes(data)) 
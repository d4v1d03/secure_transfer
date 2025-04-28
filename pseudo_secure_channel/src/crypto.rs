use rand::RngCore;

use crate::constants::*;

// --- Key Derivation (Using PSS and Nonce) ---

pub fn derive_keys(pss: &[u8], nonce: &[u8]) -> (u8, u8) {
    // Simple derivation: XOR PSS bytes with nonce bytes repeatedly
    // In a real system, use a proper KDF (Key Derivation Function) like HKDF.
    let mut derived_key_material: u64 = 0;
    for i in 0..pss.len().max(nonce.len()) {
        let pss_byte = pss.get(i % pss.len()).unwrap_or(&0);
        let nonce_byte = nonce.get(i % nonce.len()).unwrap_or(&0);
        let xored_byte = pss_byte ^ nonce_byte;
        // Fold the XORed byte into the 64-bit state
        derived_key_material = derived_key_material.rotate_left(8) ^ (xored_byte as u64);
    }

    // Extract k1 and k2 from the derived material
    let k1 = (derived_key_material & 0xFF) as u8; // Lower byte
    let k2 = ((derived_key_material >> 8) & 0xFF) as u8; // Next byte

    // Apply constants for some extra mixing (optional)
    let final_k1 = k1 ^ KEY_DERIVATION_CONSTANT_1;
    let final_k2 = k2.wrapping_add(KEY_DERIVATION_CONSTANT_2);

    (final_k1, final_k2)
}

// --- Pseudo-Random Padding (using LCG) ---

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u8) -> Self {
        Lcg { state: seed as u64 }
    }

    fn next(&mut self) -> u8 {
        self.state = self.state.wrapping_mul(LCG_A).wrapping_add(LCG_C);
        (self.state >> 16) as u8 // Take some bits from the middle
    }
}

fn add_padding(data: &[u8], k2: u8) -> Vec<u8> {
    let original_len = data.len();
    let len_bytes = original_len.to_be_bytes(); // Store length as big-endian usize (typically 8 bytes)
    let total_len_with_len = original_len + std::mem::size_of::<usize>();
    let padding_len = (BLOCK_SIZE - (total_len_with_len % BLOCK_SIZE)) % BLOCK_SIZE;

    let mut padded_data = Vec::with_capacity(total_len_with_len + padding_len);
    padded_data.extend_from_slice(&len_bytes);
    padded_data.extend_from_slice(data);

    let mut lcg = Lcg::new(k2);
    for _ in 0..padding_len {
        padded_data.push(lcg.next());
    }

    padded_data
}

fn remove_padding(padded_data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if padded_data.len() < std::mem::size_of::<usize>() {
        return Err("Padded data too short to contain length");
    }
    if padded_data.len() % BLOCK_SIZE != 0 {
        // This shouldn't happen if padding was added correctly
        return Err("Padded data length not multiple of block size");
    }

    let len_bytes: [u8; std::mem::size_of::<usize>()] = padded_data[0..std::mem::size_of::<usize>()]
        .try_into()
        .map_err(|_| "Failed to extract length bytes")?;
    let original_len = usize::from_be_bytes(len_bytes);

    let expected_end = std::mem::size_of::<usize>() + original_len;
    if expected_end > padded_data.len() {
        return Err("Stored length exceeds padded data length");
    }

    Ok(padded_data[std::mem::size_of::<usize>()..expected_end].to_vec())
}

// --- Substitution Layer ---

// Generate a fixed, shuffled substitution box (S-box) and its inverse
// In a real scenario, this might be derived from the key
lazy_static::lazy_static! {
    static ref S_BOX: [u8; 256] = {
        let mut s_box: [u8; 256] = [0; 256];
        for i in 0..256 { s_box[i] = i as u8; }
        // Simple deterministic shuffle based on constants (replace with rand::thread_rng().shuffle if desired)
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(LCG_A ^ LCG_C);
        use rand::seq::SliceRandom;
        s_box.shuffle(&mut rng);
        s_box
    };
    static ref INV_S_BOX: [u8; 256] = {
        let mut inv_s_box = [0u8; 256];
        for (i, &val) in S_BOX.iter().enumerate() {
            inv_s_box[val as usize] = i as u8;
        }
        inv_s_box
    };
}

fn substitute(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte = S_BOX[*byte as usize];
    }
}

fn inverse_substitute(data: &mut [u8]) {
    for byte in data.iter_mut() {
        *byte = INV_S_BOX[*byte as usize];
    }
}

// --- Permutation Layer ---

// Simple fixed permutation within blocks (e.g., swap first and last byte)
fn permute_blocks(data: &mut [u8]) {
    for block in data.chunks_mut(BLOCK_SIZE) {
        if block.len() == BLOCK_SIZE { // Only permute full blocks
            block.swap(0, BLOCK_SIZE - 1);
            // Add more complex swaps if desired
            block.swap(1, BLOCK_SIZE - 2);
        }
    }
}

// Inverse permutation is the same swaps in reverse order
fn inverse_permute_blocks(data: &mut [u8]) {
    for block in data.chunks_mut(BLOCK_SIZE) {
        if block.len() == BLOCK_SIZE { // Only permute full blocks
             block.swap(1, BLOCK_SIZE - 2);
             block.swap(0, BLOCK_SIZE - 1);
        }
    }
}

// --- XOR Layer ---

fn xor_with_key(data: &mut [u8], k1: u8) {
    for byte in data.iter_mut() {
        *byte ^= k1;
    }
}

// --- Main Encryption/Decryption Functions ---

// Encrypt now returns (encrypted_data, nonce)
// It no longer takes timestamp
pub fn encrypt(data: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // 1. Generate Nonce
    let mut nonce = vec![0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce);

    // 2. Derive keys using PSS and Nonce
    let (k1, k2) = derive_keys(PRE_SHARED_SECRET, &nonce);

    // 3. Add padding using k2 for LCG seed
    let mut padded_data = add_padding(data, k2);

    // 4. Apply layers using k1 for XOR
    xor_with_key(&mut padded_data, k1);
    substitute(&mut padded_data);
    permute_blocks(&mut padded_data);

    // 5. Return encrypted data and the nonce used
    (padded_data, nonce)
}

// Decrypt now takes nonce instead of timestamp
pub fn decrypt(encrypted_data: &[u8], nonce: &[u8]) -> Result<Vec<u8>, &'static str> {
    if encrypted_data.len() % BLOCK_SIZE != 0 {
        return Err("Encrypted data length not multiple of block size");
    }
    if nonce.len() != NONCE_SIZE {
         return Err("Invalid nonce length");
    }

    // 1. Derive keys using PSS and the provided Nonce
    let (k1, _k2) = derive_keys(PRE_SHARED_SECRET, nonce);

    // 2. Copy encrypted data to modify
    let mut data = encrypted_data.to_vec();

    // 3. Apply inverse layers (using derived k1)
    inverse_permute_blocks(&mut data);
    inverse_substitute(&mut data);
    xor_with_key(&mut data, k1);

    // 4. Remove padding
    remove_padding(&data)
}

// Needed for lazy_static

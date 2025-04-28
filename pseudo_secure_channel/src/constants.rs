pub const BLOCK_SIZE: usize = 16; // Padded data will be multiple of this size
pub const NETWORK_PORT: u16 = 8082; // Port for this simulation
pub const KEY_DERIVATION_CONSTANT_1: u8 = 0xAB; // Arbitrary constant for key derivation
pub const KEY_DERIVATION_CONSTANT_2: u8 = 0xCD; // Arbitrary constant for key derivation

// LCG parameters (example values, can be anything)
pub const LCG_A: u64 = 1664525;
pub const LCG_C: u64 = 1013904223;

// Nonce size
pub const NONCE_SIZE: usize = 16;

// !!! VERY INSECURE - DO NOT HARDCODE SECRETS IN REAL APPLICATIONS !!!
pub const PRE_SHARED_SECRET: &[u8; 32] = b"this_is_a_very_secret_key_1234\0\0"; // Padded to 32 bytes

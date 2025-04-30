# Pseudo Secure Channel (Python)

This is a Python implementation of the Pseudo Secure Channel simulation originally written in Rust.

## Overview

This project simulates a secure communication channel between a sender and a receiver, implementing:
- Custom encryption/decryption with a pre-shared secret
- Nonce generation for each transmission
- S-Box substitution for confusion
- Block permutation for diffusion
- Key derivation
- Padding scheme

## Requirements

- Python 3.6+
- uuid package (see requirements.txt)

## Running the Simulation

1. Install the dependencies:
```
pip install -r requirements.txt
```

2. Run the simulation:
```
python -m src.main
```

The simulation will start both a sender and receiver thread. The sender will encrypt some sample data and send it to the receiver, which will decrypt it and display the results. 
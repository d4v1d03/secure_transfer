# Secure Transfer (Python)

This is a Python implementation of the Secure Transfer simulation originally written in Rust.

## Overview

This project simulates a secure communication mechanism using a Rotating Character Shift (RCS) algorithm:
- Time-based key generation
- Position-dependent character shifting for encryption
- TCP/IP-based communication
- Serialization and deserialization of data structures

## Requirements

- Python 3.6+

## Running the Simulation

Run the simulation:
```
python -m src.main
```

The simulation will start both a sender and receiver. The sender creates sample data, encrypts it using the RCS algorithm, and sends it to the receiver, which will decrypt it and display the results. 
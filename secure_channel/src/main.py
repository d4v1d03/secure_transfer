import threading
import time
import uuid
from datetime import datetime

import constants, crypto, model, network


def main():
    print("Pseudo Secure Channel Simulation")

    sample_data = model.TransferData(
        transaction_id=str(uuid.uuid4()),
        payload=b"This is some important data being sent securely (not really).",
        created_at=datetime.now()
    )

    sender_data = sample_data

    # Start receiver in a separate thread
    receiver_thread = threading.Thread(target=network.run_receiver)
    receiver_thread.start()

    # Give the receiver time to start
    time.sleep(1)

    # Start sender in a separate thread
    sender_thread = threading.Thread(target=network.run_sender, args=(sender_data,))
    sender_thread.start()

    # Wait for both threads to complete
    sender_thread.join()
    receiver_thread.join()

    print("Simulation finished.")


if __name__ == "__main__":
    main() 
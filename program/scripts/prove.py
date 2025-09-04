#!/usr/bin/env python3

from generate_data import generate_data
from pathlib import Path
import logging
import json
import argparse
from prove_pow import run_prover

INITIAL_HEIGHT = 913139
BATCH_SIZE = 1
# Block 913140
TARGET_UTXO_TXID = "92902693c34c80da75da19f97bfb3719013883d8d307e88011a648363dd2f334"
TARGET_UTXO_VOUT = 1

def generate_args(output_path):
    batch_data = generate_data(
        mode="full",
        initial_height=INITIAL_HEIGHT,
        num_blocks=BATCH_SIZE,
        fast=False,
        mmr_roots=False,
    )
    batch_args = {
        "chain_state": batch_data["chain_state"],
        "blocks": batch_data["blocks"],
        "target_utxo": {
            "txid": TARGET_UTXO_TXID,
            "vout": TARGET_UTXO_VOUT,
        },
    }
    batch_file = Path(output_path) / "batch.json"
    batch_file.write_text(json.dumps(batch_args, indent=2))

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output-path",
        type=str,
        required=True,
        help="Output path",
    )
    parser.add_argument(
        "--executable-path",
        type=str,
        required=False,
        help="Executable path",
    )
    parser.add_argument(
        "--generate-args",
        type=str,
        required=False,
        help="Generate args",
    )

    args = parser.parse_args()

    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.DEBUG)
    console_handler.setFormatter(
        logging.Formatter("%(asctime)s - %(levelname)4.4s - %(message)s")
    )
    root_logger = logging.getLogger()
    root_logger.addHandler(console_handler)
    root_logger.setLevel(logging.DEBUG)

    logging.getLogger("urllib3").setLevel(logging.WARNING)

    if args.generate_args:
        generate_args(args.proof_path, args.output_path)

    proof_file = Path(args.output_path) / "result_proof.json"
    arguments_file = Path(args.output_path) / "args.json"

    steps_info = run_prover(
        f"Job(height='{INITIAL_HEIGHT}', blocks={BATCH_SIZE})",
        args.executable_path,
        str(proof_file),
        str(arguments_file),
    )

    print(steps_info)

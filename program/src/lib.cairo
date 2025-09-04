use utils::double_sha256::double_sha256_word_array;
use consensus::codec::Encode;
use consensus::types::transaction::Transaction;
use consensus::types::block::{Block, TransactionData};
use consensus::types::chain_state::{ChainState, ChainStateHashTrait};
use consensus::validation::header::validate_block_header;
use core::box::BoxImpl;
use stwo_cairo_air::{CairoProof, VerificationOutput, get_verification_output, verify_cairo};
use utils::blake2s_hasher::{Blake2sDigestFromU256, Blake2sDigestIntoU256};
use utils::hash::Digest;
use utils::merkle_tree::merkle_root;

#[derive(Drop, Serde)]
struct Args {
    /// Current (initial) chain state.
    chain_state: ChainState,
    /// Batch of blocks that have to be applied to the current chain state.
    blocks: Array<Block>,
    /// UTXO that is not allowed to be spent.
    target_utxo: Utxo,
    /// Proof of the previous chain state transition.
    /// If set to None, the chain state is assumed to be the genesis state.
    chain_state_proof: CairoProof,
}

#[derive(Drop, Serde)]
struct Utxo {
    /// The hash of the referenced transaction.
    txid: Digest,
    /// The index of the specific output in the transaction.
    vout: u32,
}

#[derive(Drop, Serde)]
struct Result {
    /// Hash of the chain state after the blocks have been applied.
    chain_state_hash: u256,
    /// Hash of the roots of the Merkle Mountain Range of the block hashes.
    block_mmr_hash: u256,
    /// Hash of the bootloader program that was recursively verified.
    bootloader_hash: felt252,
    /// Hash of the program that was recursively verified.
    /// We cannot know the hash of the program from within the program, so we have to carry it over.
    /// This also allows composing multiple programs (e.g. if we'd need to upgrade at a certain
    /// block height).
    program_hash: felt252,
}

#[derive(Drop, Serde)]
struct BootloaderOutput {
    /// Number of tasks (must be always 1)
    n_tasks: usize,
    /// Size of the task output in felts (including the size field)
    task_output_size: usize,
    /// Hash of the payload program.
    task_program_hash: felt252,
    /// Output of the payload program.
    task_result: Result,
}

#[executable]
fn main(args: Args) -> Result {
    let Args { chain_state, blocks, chain_state_proof, target_utxo } = args;

    // Verify the proof and extract the previous result
    let prev_result = get_prev_result(chain_state_proof);

    // Check that the provided chain state matches the final state hash of the previous run.
    assert(
        prev_result.chain_state_hash == chain_state.blake2s_digest().into(), 'Invalid initial state',
    );

    let mut current_chain_state = chain_state;
    let mut target_utxo_created = false;

    // Validate the blocks and update the current chain state
    for block in blocks {
        let merkle_root = match block.data {
            TransactionData::MerkleRoot(_) => panic!("Expected list of transactions"),
            TransactionData::Transactions(txs) => compute_txid_root_checked(txs, @target_utxo, ref target_utxo_created)
        };

        let mut block = block;
        block.data = TransactionData::MerkleRoot(merkle_root);

        // Validate the block header
        match validate_block_header(current_chain_state, block) {
            Ok(new_chain_state) => { current_chain_state = new_chain_state; },
            Err(err) => panic!("FAIL: error='{}'", err),
        };
    }

    assert(target_utxo_created, 'Target UTXO is not created');

    println!("OK");

    Result {
        chain_state_hash: current_chain_state.blake2s_digest().into(),
        block_mmr_hash: prev_result.block_mmr_hash,
        bootloader_hash: prev_result.bootloader_hash,
        program_hash: prev_result.program_hash,
    }
}

/// Verify Cairo proof, extract and validate the task output.
fn get_prev_result(proof: CairoProof) -> Result {
    let VerificationOutput { program_hash, output } = get_verification_output(proof: @proof);

    // Verify the proof
    match verify_cairo(proof) {
        Ok(_) => {},
        Err(e) => panic!("Invalid proof: {:?}", e),
    }

    // Deserialize the bootloader output
    let mut serialized_bootloader_output = output.span();
    let BootloaderOutput {
        n_tasks, task_output_size, task_program_hash, task_result,
    }: BootloaderOutput =
        Serde::deserialize(ref serialized_bootloader_output).expect('Invalid bootloader output');

    // Check that the bootloader output contains exactly one task
    assert(serialized_bootloader_output.is_empty(), 'Output too long');
    assert(n_tasks == 1, 'Unexpected number of tasks');
    assert(
        task_output_size == 8, 'Unexpected task output size',
    ); // 1 felt for program hash, 6 for output, 1 for the size

    // Check that the task bootloader hash and program hash is the same as
    // the previous bootloader hash and program hash. In case of the genesis state,
    // the previous hash is 0

    if task_result.bootloader_hash != 0 {
        assert(task_result.bootloader_hash == program_hash, 'Bootloader hash mismatch')
    }
    if task_result.program_hash != 0 {
        assert(task_result.program_hash == task_program_hash, 'Program hash mismatch');
    }

    Result {
        chain_state_hash: task_result.chain_state_hash,
        block_mmr_hash: task_result.block_mmr_hash,
        bootloader_hash: program_hash,
        program_hash: task_program_hash,
    }
}

/// Compute the txid merkle root of a list of transactions.
/// Make sure the target UTXO is either not created yet, or unspent if it exists.
fn compute_txid_root_checked(txs: Span<Transaction>, target_utxo: @Utxo, ref target_utxo_created: bool) -> Digest {
    let mut txids: Array<Digest> = array![];

    for tx in txs {
        let tx_words = tx.encode();
        let txid = double_sha256_word_array(tx_words);

        if !target_utxo_created {
            if txid == *target_utxo.txid && tx.outputs.len() > *target_utxo.vout {
                target_utxo_created = true;
            }
        } else {
            for input in tx.inputs {
                let prev_output = input.previous_output;
                if prev_output.txid == target_utxo.txid && prev_output.vout == target_utxo.vout {
                    panic!("Target UTXO is spent");
                }
            }
        }

        txids.append(txid);
    }

    let txid_root = merkle_root(txids.span());
    txid_root
}

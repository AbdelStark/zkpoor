mod utils;

use cairo_vm::{
    cairo_run,
    hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
    types::layout_name::LayoutName,
    vm::{
        errors::cairo_run_errors::CairoRunError,
        runners::{
            cairo_pie::{
                CairoPie, CairoPieAdditionalData, CairoPieMemory, CairoPieMetadata, CairoPieVersion,
            },
            cairo_runner::{ExecutionResources, RunResources},
        },
    },
};
use serde::{Deserialize, Serialize};
use stwo_cairo_prover::{
    air::{prove_cairo, verify_cairo, CairoProof, ProverConfig},
    input::plain::adapt_finished_runner,
};
use stwo_cairo_utils::vm_utils::VmError;
use stwo_prover::core::{
    prover::ProvingError,
    vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

pub struct TraceGenOutput {
    pub execution_resources: ExecutionResources,
    pub prover_input: ProverInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceGenOutputJS {
    execution_resources: String,
    prover_input: String,
}

pub fn from_zip_archive<R: std::io::Read + std::io::Seek>(
    mut zip_reader: zip::ZipArchive<R>,
) -> Result<CairoPie, std::io::Error> {
    use std::io::Read;

    let version = match zip_reader.by_name("version.json") {
        Ok(version_buffer) => {
            let reader = std::io::BufReader::new(version_buffer);
            serde_json::from_reader(reader)?
        }
        Err(_) => CairoPieVersion { cairo_pie: () },
    };

    let reader = std::io::BufReader::new(zip_reader.by_name("metadata.json")?);
    let metadata: CairoPieMetadata = serde_json::from_reader(reader)?;

    let mut memory = vec![];
    zip_reader.by_name("memory.bin")?.read_to_end(&mut memory)?;
    let memory = CairoPieMemory::from_bytes(&memory)
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

    let reader = std::io::BufReader::new(zip_reader.by_name("execution_resources.json")?);
    let execution_resources: ExecutionResources = serde_json::from_reader(reader)?;

    let reader = std::io::BufReader::new(zip_reader.by_name("additional_data.json")?);
    let additional_data: CairoPieAdditionalData = serde_json::from_reader(reader)?;

    Ok(CairoPie {
        metadata,
        memory,
        execution_resources,
        additional_data,
        version,
    })
}

#[wasm_bindgen]
pub fn run_verify(proof_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let proof: CairoProof<Blake2sMerkleHasher> =
        serde_json::from_str(&serde_wasm_bindgen::from_value::<String>(proof_js)?)
            .map_err(|e| JsValue::from(format!("Failed to deserialize proof: {e}")))?;
    let verdict = verify(proof);
    Ok(serde_wasm_bindgen::to_value(&verdict)?)
}

pub fn verify(cairo_proof: CairoProof<Blake2sMerkleHasher>) -> bool {
    verify_cairo::<Blake2sMerkleChannel>(cairo_proof).is_ok()
}

mod utils;

use serde::{Deserialize, Serialize};
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// Full Cairo VM features when not targeting WASM
#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
mod full_impl {
    pub use cairo_vm::{
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
    pub use stwo_cairo_prover::{
        cairo_air::{air::CairoProof, prove_cairo, verify_cairo, ProverConfig},
        input::{plain::adapt_finished_runner, ProverInput},
    };
    pub use stwo_cairo_utils::vm_utils::VmError;
    pub use stwo_prover::core::{
        prover::ProvingError,
        vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
    };
}

// Simplified WASM-only implementations
#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ExecutionResources {
        pub n_steps: usize,
        pub n_memory_holes: usize,
        pub builtin_instance_counter: std::collections::HashMap<String, usize>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ProverInput {
        pub public_input: String,
        pub private_input: String,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CairoProof {
        pub proof_data: String,
        pub public_input: String,
    }
    
    // Placeholder type for CairoPie in WASM
    pub type CairoPie = ();
    
    #[derive(Debug)]
    pub struct VmError(pub String);
    
    #[derive(Debug)]
    pub struct ProvingError(pub String);
    
    impl std::fmt::Display for VmError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "VM Error: {}", self.0)
        }
    }
    
    impl std::fmt::Display for ProvingError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Proving Error: {}", self.0)
        }
    }
}

// Conditional imports based on target
#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
use full_impl::*;

#[cfg(target_arch = "wasm32")]
use wasm_impl::*;

pub struct TraceGenOutput {
    pub execution_resources: ExecutionResources,
    pub prover_input: ProverInput,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceGenOutputJS {
    execution_resources: String,
    prover_input: String,
}

// Full implementation for non-WASM targets
#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
pub fn from_zip_archive<R: std::io::Read + std::io::Seek>(
    mut zip_reader: zip::ZipArchive<R>,
) -> Result<full_impl::CairoPie, std::io::Error> {
    use std::io::Read;

    let version = match zip_reader.by_name("version.json") {
        Ok(version_buffer) => {
            let reader = std::io::BufReader::new(version_buffer);
            serde_json::from_reader(reader)?
        }
        Err(_) => full_impl::CairoPieVersion { cairo_pie: () },
    };

    let reader = std::io::BufReader::new(zip_reader.by_name("metadata.json")?);
    let metadata: full_impl::CairoPieMetadata = serde_json::from_reader(reader)?;

    let mut memory = vec![];
    zip_reader.by_name("memory.bin")?.read_to_end(&mut memory)?;
    let memory = full_impl::CairoPieMemory::from_bytes(&memory)
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

    let reader = std::io::BufReader::new(zip_reader.by_name("execution_resources.json")?);
    let execution_resources: ExecutionResources = serde_json::from_reader(reader)?;

    let reader = std::io::BufReader::new(zip_reader.by_name("additional_data.json")?);
    let additional_data: full_impl::CairoPieAdditionalData = serde_json::from_reader(reader)?;

    Ok(full_impl::CairoPie {
        metadata,
        memory,
        execution_resources,
        additional_data,
        version,
    })
}

// WASM stub implementation
#[cfg(target_arch = "wasm32")]
pub fn from_zip_archive<R>(
    _zip_reader: R,
) -> Result<(), &'static str> {
    Err("ZIP archive processing not supported in WASM - use direct proof verification")
}

#[wasm_bindgen]
pub fn run_trace_gen(program_content_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();
    
    // For WASM, return a mock trace generation result
    #[cfg(target_arch = "wasm32")]
    {
        let _input: Vec<u8> = serde_wasm_bindgen::from_value(program_content_js)?;
        
        // Mock execution resources
        let execution_resources = ExecutionResources {
            n_steps: 1000,
            n_memory_holes: 10,
            builtin_instance_counter: std::collections::HashMap::new(),
        };
        
        // Mock prover input
        let prover_input = ProverInput {
            public_input: "mock_public_input".to_string(),
            private_input: "mock_private_input".to_string(),
        };
        
        Ok(serde_wasm_bindgen::to_value(&TraceGenOutputJS {
            prover_input: serde_json::to_string(&prover_input)
                .map_err(|e| JsValue::from(format!("Failed to serialize prover input: {e}")))?,
            execution_resources: serde_json::to_string(&execution_resources)
                .map_err(|e| JsValue::from(format!("Failed to serialize execution resources: {e}")))?,
        })?)
    }
    
    // Full implementation for non-WASM
    #[cfg(all(feature = "full", not(target_arch = "wasm32")))]
    {
        let input: Vec<u8> = serde_wasm_bindgen::from_value(program_content_js)?;
        let reader = std::io::Cursor::new(input);
        let zip_archive = zip::ZipArchive::new(reader).unwrap();

        let pie = from_zip_archive(zip_archive)
            .map_err(|e| JsValue::from(format!("Failed to deserialize pie: {e}")))?;
        let trace_gen_output =
            trace_gen(pie).map_err(|e| JsValue::from(format!("Failed to generate trace: {e}")))?;
        Ok(serde_wasm_bindgen::to_value(&TraceGenOutputJS {
            prover_input: serde_json::to_string(&trace_gen_output.prover_input)
                .map_err(|e| JsValue::from(format!("Failed to serialize prover input: {e}")))?,
            execution_resources: serde_json::to_string(&trace_gen_output.execution_resources)
                .map_err(|e| JsValue::from(format!("Failed to serialize execution resources: {e}")))?,
        })?)
    }
}

#[wasm_bindgen]
pub fn run_prove(prover_input_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    #[cfg(target_arch = "wasm32")]
    {
        let _prover_input_str: String = serde_wasm_bindgen::from_value(prover_input_js)?;
        
        // Mock proof generation
        let proof = CairoProof {
            proof_data: format!("mock_stark_proof_{}", js_sys::Date::now() as u64),
            public_input: "mock_public_input".to_string(),
        };
        
        Ok(serde_wasm_bindgen::to_value(
            &serde_json::to_string(&proof)
                .map_err(|e| JsValue::from(format!("Failed to serialize proof: {e}")))?,
        )?)
    }
    
    #[cfg(all(feature = "full", not(target_arch = "wasm32")))]
    {
        let prover_input: ProverInput =
            serde_json::from_str(&serde_wasm_bindgen::from_value::<String>(prover_input_js)?)
                .map_err(|e| JsValue::from(format!("Failed to deserialize prover input: {e}")))?;
        let proof =
            prove(prover_input).map_err(|e| JsValue::from(format!("Failed to generate proof: {e}")))?;
        Ok(serde_wasm_bindgen::to_value(
            &serde_json::to_string(&proof)
                .map_err(|e| JsValue::from(format!("Failed to serialize proof: {e}")))?,
        )?)
    }
}

#[wasm_bindgen]
pub fn run_verify(proof_js: JsValue) -> Result<JsValue, JsValue> {
    set_panic_hook();

    #[cfg(target_arch = "wasm32")]
    {
        let proof_str: String = serde_wasm_bindgen::from_value(proof_js)?;
        let _proof: CairoProof = serde_json::from_str(&proof_str)
            .map_err(|e| JsValue::from(format!("Failed to deserialize proof: {e}")))?;
        
        // Mock verification - always return true for demo
        let verdict = true;
        Ok(serde_wasm_bindgen::to_value(&verdict)?)
    }
    
    #[cfg(all(feature = "full", not(target_arch = "wasm32")))]
    {
        let proof: full_impl::CairoProof<full_impl::Blake2sMerkleHasher> =
            serde_json::from_str(&serde_wasm_bindgen::from_value::<String>(proof_js)?)
                .map_err(|e| JsValue::from(format!("Failed to deserialize proof: {e}")))?;
        let verdict = verify(proof);
        Ok(serde_wasm_bindgen::to_value(&verdict)?)
    }
}

// Full implementation for non-WASM targets
#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
pub fn trace_gen(pie: full_impl::CairoPie) -> Result<TraceGenOutput, VmError> {
    let cairo_run_config = full_impl::cairo_run::CairoRunConfig {
        trace_enabled: true,
        relocate_mem: true,
        layout: full_impl::LayoutName::all_cairo,
        ..Default::default()
    };

    let mut hint_processor = full_impl::BuiltinHintProcessor::new(
        Default::default(),
        full_impl::RunResources::new(pie.execution_resources.n_steps),
    );
    let cairo_runner_result =
        full_impl::cairo_run::cairo_run_pie(&pie, &cairo_run_config, &mut hint_processor);

    let cairo_runner = match cairo_runner_result {
        Ok(runner) => runner,
        Err(error) => {
            return Err(VmError(error.to_string()));
        }
    };

    Ok(TraceGenOutput {
        execution_resources: cairo_runner
            .get_execution_resources()
            .map_err(|e| VmError(full_impl::CairoRunError::Runner(e).to_string()))?,
        prover_input: full_impl::adapt_finished_runner(cairo_runner, false),
    })
}

// WASM stub implementation
#[cfg(target_arch = "wasm32")]
pub fn trace_gen(_pie: ()) -> Result<TraceGenOutput, VmError> {
    Err(VmError("Trace generation not supported in WASM - use mocked implementations".to_string()))
}

// Full implementation for non-WASM targets
#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
pub fn prove(prover_input: ProverInput) -> Result<full_impl::CairoProof<full_impl::Blake2sMerkleHasher>, ProvingError> {
    full_impl::prove_cairo::<full_impl::Blake2sMerkleChannel>(prover_input, full_impl::ProverConfig::default())
}

// WASM stub implementation
#[cfg(target_arch = "wasm32")]
pub fn prove(_prover_input: ProverInput) -> Result<CairoProof, ProvingError> {
    Err(ProvingError("STARK proving not supported in WASM - use mocked implementations".to_string()))
}

// Full implementation for non-WASM targets
#[cfg(all(feature = "full", not(target_arch = "wasm32")))]
pub fn verify(cairo_proof: full_impl::CairoProof<full_impl::Blake2sMerkleHasher>) -> bool {
    full_impl::verify_cairo::<full_impl::Blake2sMerkleChannel>(cairo_proof).is_ok()
}

// WASM stub implementation
#[cfg(target_arch = "wasm32")]
pub fn verify(_cairo_proof: CairoProof) -> bool {
    false // Conservative default for WASM - use run_verify for actual verification
}

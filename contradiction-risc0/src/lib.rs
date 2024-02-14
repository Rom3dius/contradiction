pub mod models;
include!(concat!(env!("OUT_DIR"), "/methods.rs"));

use methods::{
    HYPOTENUSE_ELF, HYPOTENUSE_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "circuit", content = "inputs")]
pub enum CircuitInputs {
    hypotenuse(models::Hypotenuse),
    // Add other variants here
}

pub fn execute_circuit<T: IntoExecutorEnv>(input: T, elf: &[u8], id: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let mut env_builder = ExecutorEnv::builder();
    
    // Use the trait method to write struct fields into the ExecutorEnv
    input.write_to_env(&mut env_builder)?;
    
    let env = env_builder.build()?;
    let prover = default_prover();
    
    // Run the circuit with the prepared environment
    let receipt = prover.prove(env, elf)?;
    
    // Verify the receipt
    receipt.verify(id)?;

    Ok(())
}
use risc0_zkvm::{default_prover, ExecutorEnv};
use risc0_zkvm::Receipt;
use anyhow::{Result, Error};
use crate::risc0::models;

pub fn execute_circuit<T: models::IntoExecutorEnv>(input: &T) -> Result<Receipt> {
    let mut env_builder = ExecutorEnv::builder();

    let (elf, id) = models::fetch_circuit(input);
    
    match input.write_to_env(&mut env_builder) {
        Ok(_) => {},
        Err(e) => return Err(Error::msg(e.to_string())),
    }
    
    let env = env_builder.build()?;
    let prover = default_prover();
    
    let receipt = prover.prove(env, elf)?;
    receipt.verify(id)?;
    Ok(receipt)
}
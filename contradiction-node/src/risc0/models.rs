use risc0_zkvm::ExecutorEnvBuilder;
use serde::{Serialize, Deserialize};
use contradiction_risc0_methods as methods;
use anyhow::Result;

pub trait IntoExecutorEnv {
    fn write_to_env(&self, builder: &mut ExecutorEnvBuilder) -> Result<()>;
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CircuitInputs {
    hypotenuse(Hypotenuse)
}

impl IntoExecutorEnv for CircuitInputs {
    fn write_to_env(&self, builder: &mut ExecutorEnvBuilder) -> Result<()> {
        match self {
            CircuitInputs::hypotenuse(h) => h.write_to_env(builder),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hypotenuse {
    x: u32,
    y: u32,
}

impl IntoExecutorEnv for Hypotenuse {
    fn write_to_env(&self, builder: &mut ExecutorEnvBuilder) -> Result<()> {
        builder.write(&self.x)?;
        builder.write(&self.y)?;
        Ok(())
    }
}

pub fn fetch_circuit<T: IntoExecutorEnv>(input: &T) -> (&'static [u8], [u32; 8]) {
    match input {
        Hypotenuse => {
            (methods::HYPOTENUSE_ELF, methods::HYPOTENUSE_ID)
        }
    }
}
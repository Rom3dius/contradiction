use anyhow::Result;
use risc0_zkvm::Executor;

pub struct Hypotenuse {
    x: u32,
    y: u32,
}

impl IntoExecutorEnv for Hypotenuse {
    fn write_to_env(&self, &mut ExecutorEnvBuilder: Executor) -> Result<()> {
        builder.write(&self.x)?;
        builder.write(&self.y)?;
        Ok(())
    }
}
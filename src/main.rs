use anyhow::Result;
use rustychains::sandbox::DockerSandbox;

#[tokio::main]
async fn main() -> Result<()> {
    let _sandbox = DockerSandbox::new("/home/can/workspace/rustychains/docker", "sandbox").await?;
    Ok(())
}

use anyhow::Result;
use rustychains::sandbox::DockerSandbox;

#[tokio::main]
async fn main() -> Result<()> {
    let _sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    Ok(())
}

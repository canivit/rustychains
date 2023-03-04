use anyhow::Result;
use rustychains::sandbox::DockerSandbox;

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let container_id = sandbox.create_container().await?;
    println!("{}", &container_id);
    Ok(())
}

use anyhow::Result;
use rustychains::sandbox::DockerSandbox;
use rustychains::sandbox::Language;

#[tokio::main]
async fn main() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code("./example_code/hello.py", Language::Python)
        .await?;
    print!("{}", &output.stdout);
    Ok(())
}

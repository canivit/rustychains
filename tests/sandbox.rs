use anyhow::Result;
use rustychains::sandbox::DockerSandbox;
use rustychains::sandbox::Language;

#[tokio::test]
async fn test_python_successful_run() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code("./example_code/hello.py", Language::Python)
        .await?;
    assert_eq!("Hello World\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

use anyhow::Result;
use rustychains::sandbox::DockerSandbox;
use rustychains::sandbox::Language;

#[tokio::test]
async fn test_python_hello_world() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code("./example_code/hello.py", Language::Python, None)
        .await?;
    assert_eq!("Hello World\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_java_hello_world() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code("./example_code/Hello.java", Language::Java, None)
        .await?;
    assert_eq!("Hello World\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_python_echo() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code(
            "./example_code/hello.py",
            Language::Python,
            Some("Hello World\n"),
        )
        .await?;
    assert_eq!("Hello World\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_java_echo() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code(
            "./example_code/Echo.java",
            Language::Java,
            Some("Hello World\n"),
        )
        .await?;
    assert_eq!("Hello World\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_python_sum() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code("./example_code/sum.py", Language::Python, Some("3\n5\n8\n"))
        .await?;
    assert_eq!("16\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_java_sum() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code("./example_code/Sum.java", Language::Java, Some("3 5 8 "))
        .await?;
    assert_eq!("16", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

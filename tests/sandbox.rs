use std::time::Duration;

use anyhow::Result;
use rustychains::sandbox::DockerSandbox;
use rustychains::sandbox::Language;
use rustychains::sandbox::SandboxError;

#[tokio::test]
async fn test_python_hello_world() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code(
            "./example_code/hello.py",
            Language::Python,
            Duration::from_secs(3),
            None,
        )
        .await?;
    assert_eq!("Hello World\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_java_hello_world() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code(
            "./example_code/Hello.java",
            Language::Java,
            Duration::from_secs(3),
            None,
        )
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
            Duration::from_secs(3),
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
            Duration::from_secs(3),
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
        .run_code(
            "./example_code/sum.py",
            Language::Python,
            Duration::from_secs(3),
            Some("3\n5\n8\n"),
        )
        .await?;
    assert_eq!("16\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_javascript_sum() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code(
            "./example_code/sum.js",
            Language::JavaScript,
            Duration::from_secs(3),
            Some("3\n5\n8\n"),
        )
        .await?;
    assert_eq!("16\n", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_java_sum() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;
    let output = sandbox
        .run_code(
            "./example_code/Sum.java",
            Language::Java,
            Duration::from_secs(3),
            Some("3 5 8 "),
        )
        .await?;
    assert_eq!("16", &output.stdout);
    assert!(&output.stderr.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_python_timeout() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;

    let result = sandbox
        .run_code(
            "./example_code/slow_echo.py",
            Language::Python,
            Duration::from_secs(3),
            Some("Hello\n"),
        )
        .await;
    assert!(match result {
        Ok(_) => false,
        Err(err) => matches!(err, SandboxError::Timeout { .. }),
    });

    let output = sandbox
        .run_code(
            "./example_code/slow_echo.py",
            Language::Python,
            Duration::from_secs(5),
            Some("Hello\n"),
        )
        .await?;
    assert_eq!("Hello\n", output.stdout);

    Ok(())
}

#[tokio::test]
async fn test_java_timeout() -> Result<()> {
    let sandbox = DockerSandbox::new("./docker", "sandbox").await?;

    let result = sandbox
        .run_code(
            "./example_code/SlowEcho.java",
            Language::Java,
            Duration::from_secs(3),
            Some("Hello\n"),
        )
        .await;
    assert!(match result {
        Ok(_) => false,
        Err(err) => matches!(err, SandboxError::Timeout { .. }),
    });

    let output = sandbox
        .run_code(
            "./example_code/SlowEcho.java",
            Language::Java,
            Duration::from_secs(6),
            Some("Hello\n"),
        )
        .await?;
    assert_eq!("Hello\n", output.stdout);

    Ok(())
}

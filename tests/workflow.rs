use std::time::Duration;

use anyhow::Error;
use anyhow::Result;
use rustychains::workflow::Language;
use rustychains::workflow::Step;
use rustychains::workflow::Workflow;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Point {
    x: usize,
    y: usize,
}

#[tokio::test]
async fn test_workflow_success() -> Result<()> {
    let point = Point { x: 2, y: 5 };
    let point = format!("{}\n", serde_json::to_string(&point)?);
    let workflow = Workflow::builder("./docker", "sandbox")
        .input(Some(&point))
        .add_step(Step::new(
            Language::Python,
            "./example_code/move_point.py",
            Duration::from_secs(3),
            "python script to move a point",
        ))
        .add_step(Step::new(
            Language::JavaScript,
            "./example_code/move_point.js",
            Duration::from_secs(3),
            "JS script to move a point",
        ))
        .add_step(Step::new(
            Language::Python,
            "./example_code/move_point.py",
            Duration::from_secs(3),
            "python script to move a point",
        ))
        .add_step(Step::new(
            Language::JavaScript,
            "./example_code/move_point.js",
            Duration::from_secs(3),
            "JS script to move a point",
        ))
        .build()
        .await?;

    let result = workflow.execute().await?;
    let output = result
        .output()
        .ok_or_else(|| Error::msg("Workflow did not produce any output"))?;
    let actual = serde_json::from_str::<Point>(output)?;
    let expected = Point { x: 30, y: 21 };
    assert_eq!(expected, actual);
    assert!(result.exec_time() <= Duration::from_secs(12));
    Ok(())
}

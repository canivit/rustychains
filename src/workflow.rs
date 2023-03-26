use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;
use tokio::time::Instant;

pub use crate::sandbox::Language;
use crate::sandbox::{DockerSandbox, SandboxError};

pub struct Workflow {
    sandbox: DockerSandbox,
    input: Option<String>,
    steps: Vec<Step>,
    exports: Vec<Export>,
}

#[derive(Clone)]
pub struct Step {
    pub lang: Language,
    pub code_file: PathBuf,
    pub timeout: Duration,
    pub desc: String,
}

#[derive(Clone)]
pub enum Export {
    SaveFile {
        desc: String,
        path: PathBuf,
    },
    SendEmail {
        desc: String,
        to: String,
        subject: String,
    },
}

pub struct WorkflowBuilder {
    directory: PathBuf,
    image_tag: String,
    input: Option<String>,
    steps: Vec<Step>,
    exports: Vec<Export>,
}

#[derive(Debug)]
pub struct StepResult {
    pub step_idx: usize,
    pub stdout: String,
    pub stderr: String,
    pub exec_time: Duration,
}

#[derive(Debug)]
pub struct ExportResult {
    pub export_idx: usize,
    pub exec_time: Duration,
}

pub struct WorkflowResult {
    step_results: Vec<StepResult>,
    export_results: Vec<ExportResult>,
}

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("failed to init docker sandbox")]
    SandboxInit(#[source] SandboxError),

    #[error("failed to execute step at index {}", .prev_steps_results.len())]
    StepError {
        #[source]
        source: SandboxError,
        prev_steps_results: Vec<StepResult>,
    },

    #[error("failed to execute export")]
    ExportError {
        #[source]
        source: SandboxError,
        prev_step_results: Vec<StepResult>,
        prev_export_results: Vec<ExportResult>,
    },
}

impl Workflow {
    pub fn builder<T>(directory: T, image_tag: &str) -> WorkflowBuilder
    where
        T: AsRef<Path>,
    {
        WorkflowBuilder {
            directory: directory.as_ref().to_owned(),
            image_tag: image_tag.to_owned(),
            input: None,
            steps: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn input(&self) -> Option<&str> {
        self.input.as_deref()
    }

    pub fn steps(&self) -> impl Iterator<Item = &Step> {
        self.steps.iter()
    }

    pub fn exports(&self) -> impl Iterator<Item = &Export> {
        self.exports.iter()
    }

    pub async fn execute(&self) -> Result<WorkflowResult, WorkflowError> {
        let step_results = self.execute_steps().await?;
        let export_results = self.execute_exports().await?;
        Ok(WorkflowResult {
            step_results,
            export_results,
        })
    }

    async fn execute_steps(&self) -> Result<Vec<StepResult>, WorkflowError> {
        let mut step_results = Vec::<StepResult>::new();
        for (idx, step) in self.steps().enumerate() {
            let input = step_results
                .last()
                .map_or(self.input(), |last_result| Some(&last_result.stdout));
            match step.execute(input, idx, &self.sandbox).await {
                Ok(r) => step_results.push(r),
                Err(err) => {
                    return Err(WorkflowError::StepError {
                        source: err,
                        prev_steps_results: step_results,
                    })
                }
            };
        }
        Ok(step_results)
    }

    async fn execute_exports(&self) -> Result<Vec<ExportResult>, WorkflowError> {
        Ok(Vec::new())
    }
}

impl Step {
    pub fn new<T>(lang: Language, code_file: T, timeout: Duration, desc: &str) -> Self
    where
        T: AsRef<Path>,
    {
        Self {
            lang,
            code_file: code_file.as_ref().to_owned(),
            timeout,
            desc: desc.to_owned(),
        }
    }

    async fn execute(
        &self,
        input: Option<&str>,
        step_idx: usize,
        sandbox: &DockerSandbox,
    ) -> Result<StepResult, SandboxError> {
        let start = Instant::now();
        let output = sandbox
            .run_code(&self.code_file, self.lang, self.timeout, input)
            .await?;
        let exec_time = start.elapsed();
        Ok(StepResult {
            step_idx,
            stdout: output.stdout,
            stderr: output.stderr,
            exec_time,
        })
    }
}

impl WorkflowBuilder {
    pub fn input(mut self, value: Option<&str>) -> Self {
        self.input = value.map(|i| i.to_owned());
        self
    }

    pub fn add_step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    pub fn add_export(mut self, export: Export) -> Self {
        self.exports.push(export);
        self
    }

    pub async fn build(self) -> Result<Workflow, WorkflowError> {
        let sandbox = DockerSandbox::new(&self.directory, &self.image_tag)
            .await
            .map_err(WorkflowError::SandboxInit)?;
        Ok(Workflow {
            sandbox,
            input: self.input,
            steps: self.steps,
            exports: self.exports,
        })
    }
}

impl WorkflowResult {
    pub fn step_results(&self) -> impl Iterator<Item = &StepResult> {
        self.step_results.iter()
    }

    pub fn export_results(&self) -> impl Iterator<Item = &ExportResult> {
        self.export_results.iter()
    }

    pub fn exec_time(&self) -> Duration {
        let step_time = self.step_results().map(|sr| sr.exec_time).sum::<Duration>();
        let export_time = self
            .export_results()
            .map(|er| er.exec_time)
            .sum::<Duration>();
        step_time + export_time
    }

    pub fn output(&self) -> Option<&str> {
        self.step_results.last().map(|r| r.stdout.as_str())
    }
}

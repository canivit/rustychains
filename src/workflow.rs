use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;
use tokio::time::Instant;

pub use crate::sandbox::Language;
use crate::sandbox::{DockerSandbox, SandboxError};

pub struct Workflow {
    sandbox: DockerSandbox,
    input: String,
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
    input: String,
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

    #[error("failed to execute step")]
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
            input: "".to_owned(),
            steps: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn steps(&self) -> impl Iterator<Item = &Step> {
        self.steps.iter()
    }

    pub fn exports(&self) -> impl Iterator<Item = &Export> {
        self.exports.iter()
    }

    pub async fn execute(&self) -> Result<WorkflowResult, WorkflowError> {
        let mut step_results = Vec::<StepResult>::new();
        for (idx, step) in self.steps.iter().enumerate() {
            let input = step_results
                .last()
                .map_or(&self.input, |last_result| &last_result.stdout);
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
        Ok(WorkflowResult {
            step_results,
            export_results: Vec::new(),
        })
    }
}

impl Step {
    async fn execute(
        &self,
        input: &str,
        step_idx: usize,
        sandbox: &DockerSandbox,
    ) -> Result<StepResult, SandboxError> {
        let start = Instant::now();
        let output = sandbox
            .run_code(&self.code_file, self.lang, self.timeout, Some(input))
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
    pub fn input(&mut self, value: &str) -> &mut Self {
        self.input = value.to_owned();
        self
    }

    pub fn add_step(&mut self, step: &Step) -> &mut Self {
        self.steps.push(step.clone());
        self
    }

    pub fn add_export(&mut self, export: &Export) -> &mut Self {
        self.exports.push(export.clone());
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

use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;

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
    pub output: String,
    pub step_results: Vec<StepResult>,
    pub export_results: Vec<ExportResult>,
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

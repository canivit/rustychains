use futures::AsyncWriteExt;
use futures::StreamExt;
use shiplift::tty::TtyChunk::{StdErr, StdIn, StdOut};
use shiplift::{BuildOptions, Container, ContainerOptions, Docker, RmContainerOptions};
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use std::{fs, vec};
use tempdir::TempDir;
use thiserror::Error;

pub struct DockerSandbox {
    docker: Docker,
    image_tag: String,
}

#[derive(Clone, Copy)]
pub enum Language {
    Python,
    Java,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("directory {0:?} does not contain a dockerfile named 'Dockerfile'")]
    MissingDockerfile(PathBuf),

    #[error("failed to retrieve the absolute path of directory {directory:?}")]
    FailedToRetrieveAbsolutePath {
        directory: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("failed to create temp directory")]
    FailedToCreateTempDirectory(#[source] std::io::Error),

    #[error("path {0:?} does not point to an existing file")]
    InvalidCodeFile(PathBuf),

    #[error("failed to copy the code file at {src:?} to {dest:?}")]
    FailedToCopyCodeFile {
        src: PathBuf,
        dest: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("failed to build docker image")]
    FailedToBuildImage(#[source] shiplift::Error),

    #[error("failed to create docker container from image with tag {image_tag:?}")]
    FailedToCreateContainer {
        image_tag: String,

        #[source]
        source: shiplift::Error,
    },

    #[error("failed to attach to docker container with id {container_id:?}")]
    FailedToAtachToContainer {
        container_id: String,

        #[source]
        source: shiplift::Error,
    },

    #[error("failed to start docker container with id {container_id:?}")]
    FailedToStartContainer {
        container_id: String,

        #[source]
        source: shiplift::Error,
    },

    #[error("failed to execute {cmd:?} inside docker container")]
    FailedToExecute {
        cmd: String,

        #[source]
        source: shiplift::Error,
    },

    #[error("failed to remove docker container with id {container_id:?}")]
    FailedToRemoveContainer {
        container_id: String,

        #[source]
        source: shiplift::Error,
    },

    #[error("failed to write to stdin of container")]
    WriteToStdin(#[source] std::io::Error),

    #[error("failed to close the stdin of container")]
    CloseStdin(#[source] std::io::Error),

    #[error("docker container outputted non utf-8 bytes to stdout")]
    InvalidBytesStdOut {
        #[source]
        source: std::str::Utf8Error,
    },

    #[error("docker container outputted non utf-8 bytes to stderr")]
    InvalidBytesStdErr {
        #[source]
        source: std::str::Utf8Error,
    },
}

pub struct RunOutput {
    pub stdout: String,
    pub stderr: String,
}

impl DockerSandbox {
    pub async fn new<T>(directory: T, image_tag: &str) -> Result<Self, Error>
    where
        T: AsRef<Path>,
    {
        let absolute_path = validate_directory(directory.as_ref())?;
        let docker = Docker::new();
        build_image(&docker, &absolute_path, image_tag).await?;
        Ok(DockerSandbox {
            docker,
            image_tag: image_tag.to_owned(),
        })
    }

    pub async fn run_code<T>(
        &self,
        code_file: T,
        lang: Language,
        stdin: Option<&str>,
    ) -> Result<RunOutput, Error>
    where
        T: AsRef<Path>,
    {
        let temp_dir = TempDir::new("").map_err(Error::FailedToCreateTempDirectory)?;
        let sandbox_files = get_sandbox_files(code_file.as_ref(), lang, temp_dir.as_ref())?;
        let commands = get_commands(&sandbox_files, lang);
        copy_code_file(code_file.as_ref(), &sandbox_files.host_src)?;
        if !&commands.build_cmd.is_empty() {
            exec_container(
                &self.docker,
                temp_dir.as_ref(),
                &self.image_tag,
                &commands.build_cmd,
                None,
            )
            .await?;
        }
        let output = exec_container(
            &self.docker,
            temp_dir.as_ref(),
            &self.image_tag,
            &commands.run_cmd,
            stdin,
        )
        .await?;
        Ok(output)
    }
}

fn validate_directory(dir: &Path) -> Result<PathBuf, Error> {
    let docker_file = dir.join("Dockerfile");
    let exist = docker_file
        .try_exists()
        .map_err(|_err| Error::MissingDockerfile(dir.to_path_buf()))?;
    if !exist || !docker_file.is_file() {
        return Err(Error::MissingDockerfile(dir.to_path_buf()));
    }

    dir.canonicalize()
        .map_err(|err| Error::FailedToRetrieveAbsolutePath {
            directory: dir.to_path_buf(),
            source: err,
        })
}

async fn build_image(docker: &Docker, path: &Path, tag: &str) -> Result<(), Error> {
    let options = BuildOptions::builder(path.display().to_string())
        .tag(tag)
        .build();
    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        build_result.map_err(Error::FailedToBuildImage)?;
    }
    Ok(())
}

async fn exec_container(
    docker: &Docker,
    temp_dir: &Path,
    image_tag: &str,
    cmd: &[String],
    stdin: Option<&str>,
) -> Result<RunOutput, Error> {
    let container_id = create_container(docker, temp_dir, image_tag, cmd).await?;
    let container = docker.containers().get(&container_id);

    container
        .start()
        .await
        .map_err(|err| Error::FailedToStartContainer {
            container_id: container_id.to_owned(),
            source: err,
        })?;

    let (mut reader, mut writer) = container
        .attach()
        .await
        .map_err(|err| Error::FailedToAtachToContainer {
            container_id: container_id.to_owned(),
            source: err,
        })?
        .split();

    if let Some(s) = stdin {
        writer
            .write_all(s.as_bytes())
            .await
            .map_err(Error::WriteToStdin)?;
        writer.flush().await.map_err(Error::WriteToStdin)?;
    }
    //writer.close().await.map_err(Error::CloseStdin)?;

    let mut stdout: Vec<u8> = Vec::new();
    let mut stderr: Vec<u8> = Vec::new();
    while let Some(result) = reader.next().await {
        let chunk = result.map_err(|err| Error::FailedToExecute {
            cmd: cmd.join(" "),
            source: err,
        })?;

        match chunk {
            StdOut(mut bytes) => stdout.append(&mut bytes),
            StdErr(mut bytes) => stderr.append(&mut bytes),
            StdIn(_) => (),
        }
    }

    remove_container(&container).await?;

    let stdout = from_utf8(&stdout)
        .map_err(|err| Error::InvalidBytesStdOut { source: err })?
        .to_owned();

    let stderr = from_utf8(&stderr)
        .map_err(|err| Error::InvalidBytesStdErr { source: err })?
        .to_owned();

    Ok(RunOutput { stdout, stderr })
}

async fn create_container(
    docker: &Docker,
    temp_dir: &Path,
    image_tag: &str,
    cmd: &[String],
) -> Result<String, Error> {
    let mount = format!("{}:/home/sandbox", temp_dir.display());
    let slice_cmd: Vec<&str> = cmd.iter().map(String::as_str).collect();
    let options = ContainerOptions::builder(image_tag)
        .volumes(vec![&mount])
        .working_dir("/home/sandbox")
        .attach_stdin(true)
        .attach_stdout(true)
        .attach_stderr(true)
        .cmd(slice_cmd)
        .build();
    docker.containers().create(&options).await.map_or_else(
        |err| {
            Err(Error::FailedToCreateContainer {
                image_tag: image_tag.to_owned(),
                source: err,
            })
        },
        |result| Ok(result.id),
    )
}

async fn remove_container(container: &Container<'_>) -> Result<(), Error> {
    let options = RmContainerOptions::builder()
        .volumes(true)
        .force(true)
        .build();
    container
        .remove(options)
        .await
        .map_err(|err| Error::FailedToRemoveContainer {
            container_id: container.id().to_owned(),
            source: err,
        })?;
    Ok(())
}

struct Commands {
    build_cmd: Vec<String>,
    run_cmd: Vec<String>,
}

struct SandboxFiles {
    host_src: PathBuf,
    container_src: PathBuf,
    container_bin: PathBuf,
}

fn get_source_extension(lang: Language) -> &'static str {
    match lang {
        Language::Python => "py",
        Language::Java => "java",
    }
}

fn get_compiled_extension(lang: Language) -> &'static str {
    match lang {
        Language::Python => "py",
        Language::Java => "",
    }
}

fn get_compiler(lang: Language) -> Option<&'static str> {
    match lang {
        Language::Python => None,
        Language::Java => Some("javac"),
    }
}

fn get_runner(lang: Language) -> &'static str {
    match lang {
        Language::Python => "python",
        Language::Java => "java",
    }
}

fn copy_code_file(src: &Path, dest: &Path) -> Result<(), Error> {
    fs::copy(src, dest).map_err(|err| Error::FailedToCopyCodeFile {
        src: src.to_path_buf(),
        dest: dest.to_path_buf(),
        source: err,
    })?;
    Ok(())
}

fn get_sandbox_files(
    code_file: &Path,
    lang: Language,
    temp_dir: &Path,
) -> Result<SandboxFiles, Error> {
    let base_file_name: PathBuf = code_file
        .file_stem()
        .ok_or_else(|| Error::InvalidCodeFile(code_file.to_path_buf()))?
        .into();
    let source_ext = get_source_extension(lang);
    let compiled_ext = get_compiled_extension(lang);
    let host_src = temp_dir.join(&base_file_name).with_extension(source_ext);
    let container_src = base_file_name.with_extension(source_ext);
    let container_bin = base_file_name.with_extension(compiled_ext);
    Ok(SandboxFiles {
        host_src,
        container_src,
        container_bin,
    })
}

fn get_commands(sandbox_files: &SandboxFiles, lang: Language) -> Commands {
    Commands {
        build_cmd: get_build_cmd(&sandbox_files.container_src, lang),
        run_cmd: get_run_cmd(&sandbox_files.container_bin, lang),
    }
}

fn get_build_cmd(source_file: &Path, lang: Language) -> Vec<String> {
    get_compiler(lang).map_or_else(Vec::new, |compiler| {
        vec![compiler.to_owned(), source_file.display().to_string()]
    })
}

fn get_run_cmd(compiled_file: &Path, lang: Language) -> Vec<String> {
    let runner = get_runner(lang).to_owned();
    vec![runner, compiled_file.display().to_string()]
}

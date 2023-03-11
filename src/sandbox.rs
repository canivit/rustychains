use futures::StreamExt;
use shiplift::{BuildOptions, ContainerOptions, Docker, ExecContainerOptions};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub struct DockerSandbox {
    docker: Docker,
    docker_dir: PathBuf,
    image_tag: String,
}

#[derive(Clone, Copy)]
pub enum Language {
    Python,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("path '{0}' does not point to an existing directory")]
    InvalidDirectory(PathBuf),

    #[error("directory '{0}' does not contain a dockerfile named 'Dockerfile'")]
    MissingDockerfile(PathBuf),

    #[error("failed to retrieve the absolute path of directory '{directory:?}'")]
    FailedToRetrieveAbsolutePath {
        directory: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("failed to build docker image")]
    FailedToBuildImage(#[source] shiplift::Error),

    #[error("failed to create docker container from image with tag '{image_tag:?}'")]
    FailedToCreateContainer {
        image_tag: String,

        #[source]
        source: shiplift::Error,
    },

    #[error("failed to create directory at '{directory:?}'")]
    FailedToCreateDirectory {
        directory: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("failed to remove directory at '{directory:?}'")]
    FailedToRemoveDirectory {
        directory: PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("path '{0}' does not point to an existing file")]
    InvalidCodeFile(PathBuf),

    #[error("failed to copy the code file at '{src:?}' to '{dest:?}'")]
    FailedToCopyCodeFile {
        src: PathBuf,
        dest: PathBuf,

        #[source]
        source: std::io::Error,
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
            docker_dir: absolute_path,
            image_tag: image_tag.to_owned(),
        })
    }

    pub async fn run_code<T>(&self, code_file: T, lang: Language) -> Result<String, Error>
    where
        T: AsRef<Path>,
    {
        // get sandbox dir
        let sandbox_dir = get_sandbox_dir(&self.docker_dir);

        // cleanup sandbox folder
        clean_sandbox_dir(&sandbox_dir)?;

        // create sandbox folder
        create_sandbox_dir(&sandbox_dir)?;

        // copy code file to sandbox folder
        let sandbox_files = get_sandbox_files(code_file.as_ref(), lang, &sandbox_dir)?;
        let commands = get_commands(&sandbox_files, lang);
        copy_code_file(code_file.as_ref(), &sandbox_files.source_file)?;
        let container_id =
            create_container(&self.docker, &self.docker_dir, &self.image_tag).await?;
        exec_container(&self.docker, &container_id, &commands.run_cmd).await?;

        // run code
        // delete container
        // cleanup sandbox folder
        todo!()
    }
}

fn validate_directory(dir: &Path) -> Result<PathBuf, Error> {
    let exist = dir
        .try_exists()
        .map_err(|_err| Error::InvalidDirectory(dir.to_path_buf()))?;
    if !exist || !dir.is_dir() {
        return Err(Error::InvalidDirectory(dir.to_path_buf()));
    }

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

async fn create_container(
    docker: &Docker,
    docker_dir: &Path,
    image_tag: &str,
) -> Result<String, Error> {
    let mount = format!("{}:/home/sanbox", docker_dir.display());
    let options = ContainerOptions::builder(image_tag)
        .volumes(vec![&mount])
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

async fn exec_container(
    docker: &Docker,
    container_id: &str,
    cmd: &[String],
) -> Result<RunOutput, Error> {
    let cmd: Vec<&str> = cmd.iter().map(String::as_str).collect();
    let options = ExecContainerOptions::builder()
        .cmd(cmd)
        .attach_stdout(true)
        .attach_stderr(true)
        .build();

    docker.containers().get(container_id).exec(&options);
    todo!()
}

struct Commands {
    build_cmd: Vec<String>,
    run_cmd: Vec<String>,
}

struct SandboxFiles {
    source_file: PathBuf,
    compiled_file: PathBuf,
}

fn get_sandbox_dir(docker_dir: &Path) -> PathBuf {
    docker_dir.join("sandbox")
}

fn get_source_extension(lang: Language) -> &'static str {
    match lang {
        Language::Python => "py",
    }
}

fn get_compiled_extension(lang: Language) -> &'static str {
    match lang {
        Language::Python => "py",
    }
}

fn get_compiler(lang: Language) -> Option<&'static str> {
    match lang {
        Language::Python => None,
    }
}

fn get_runner(lang: Language) -> &'static str {
    match lang {
        Language::Python => "python",
    }
}

fn create_sandbox_dir(sandbox_dir: &Path) -> Result<(), Error> {
    fs::create_dir(sandbox_dir).map_err(|err| Error::FailedToCreateDirectory {
        directory: sandbox_dir.to_path_buf(),
        source: err,
    })
}

fn clean_sandbox_dir(sandbox_dir: &Path) -> Result<(), Error> {
    fs::remove_dir_all(sandbox_dir).map_err(|err| Error::FailedToRemoveDirectory {
        directory: sandbox_dir.to_path_buf(),
        source: err,
    })
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
    sandbox_dir: &Path,
) -> Result<SandboxFiles, Error> {
    let base_file_name = code_file
        .file_stem()
        .ok_or_else(|| Error::InvalidCodeFile(code_file.to_path_buf()))?;
    let source_ext = get_source_extension(lang);
    let compiled_ext = get_compiled_extension(lang);
    let source_file = sandbox_dir
        .with_file_name(base_file_name)
        .with_extension(source_ext);
    let compiled_file = sandbox_dir
        .with_file_name(base_file_name)
        .with_extension(compiled_ext);
    Ok(SandboxFiles {
        source_file,
        compiled_file,
    })
}

fn get_commands(copied_file: &SandboxFiles, lang: Language) -> Commands {
    Commands {
        build_cmd: get_build_cmd(&copied_file.source_file, lang),
        run_cmd: get_run_cmd(&copied_file.compiled_file, lang),
    }
}

fn get_build_cmd(source_file: &Path, lang: Language) -> Vec<String> {
    get_compiler(lang).map_or_else(
        Vec::new,
        |compiler| vec![compiler.to_owned(), source_file.display().to_string()],
    )
}

fn get_run_cmd(compiled_file: &Path, lang: Language) -> Vec<String> {
    let runner = get_runner(lang).to_owned();
    vec![runner, compiled_file.display().to_string()]
}

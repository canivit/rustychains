use futures::{StreamExt};
use shiplift::rep::ContainerCreateInfo;
use shiplift::{BuildOptions, ContainerOptions, Docker};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub struct DockerSandbox {
    docker: Docker,
    root_dir: PathBuf,
    image_tag: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("path '{0}' does not point to an existing directory")]
    InvalidDirectory(String),

    #[error("failed to read the contents of directory '{0}'")]
    FailedToReadDirectory(String),

    #[error("directory '{0}' does not contain a dockerfile named 'Dockerfile'")]
    MissingDockerfile(String),

    #[error("failed to create image")]
    FailedToCreateImage
}

impl DockerSandbox {
    pub async fn new(directory: &str, image_tag: &str) -> Result<Self, Error> {
        Self::validate_directory(directory)?;
        let docker = Docker::new();
        Self::build_image(&docker, directory, image_tag).await?;
        Ok(DockerSandbox {
            docker,
            root_dir: directory.into(),
            image_tag: image_tag.to_owned(),
        })
    }

    async fn build_image(docker: &Docker, path: &str, tag: &str) -> Result<(), Error> {
        let options = BuildOptions::builder(path).tag(tag).build();
        let mut stream = docker.images().build(&options);
        while let Some(build_result) = stream.next().await {
            build_result.map_err(|_err| Error::FailedToCreateImage)?;
        }
        Ok(())
    }

    fn validate_directory(directory: &str) -> Result<(), Error> {
        let path = Path::new(directory);
        let exist = path
            .try_exists()
            .map_err(|_err| Error::InvalidDirectory(directory.to_owned()))?;
        if !exist || !path.is_dir() {
            return Err(Error::InvalidDirectory(directory.to_owned()));
        }

        let docker_file = path.join("Dockerfile");
        let exist = docker_file
            .try_exists()
            .map_err(|_err| Error::MissingDockerfile(directory.to_owned()))?;
        if !exist || !docker_file.is_file() {
            return Err(Error::MissingDockerfile(directory.to_owned()));
        }

        Ok(())
    }
}

async fn build_container(image_name: &str) -> Result<ContainerCreateInfo, shiplift::Error> {
    let docker = Docker::new();
    let options = ContainerOptions::builder(image_name).build();
    docker.containers().create(&options).await
}

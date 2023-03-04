use futures::StreamExt;
use shiplift::{BuildOptions, ContainerOptions, Docker};
use std::path::Path;
use thiserror::Error;

pub struct DockerSandbox {
    docker: Docker,
    root_dir: String,
    image_tag: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("path '{0}' does not point to an existing directory")]
    InvalidDirectory(String),

    #[error("directory '{0}' does not contain a dockerfile named 'Dockerfile'")]
    MissingDockerfile(String),

    #[error("failed to retrieve the absolute path of directory '{directory:?}'")]
    FailedToRetrieveAbsolutePath {
        directory: String,
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
}

impl DockerSandbox {
    pub async fn new(directory: &str, image_tag: &str) -> Result<Self, Error> {
        let absolute_path = Self::validate_directory(directory)?;
        let docker = Docker::new();
        Self::build_image(&docker, &absolute_path, image_tag).await?;
        Ok(DockerSandbox {
            docker,
            root_dir: absolute_path,
            image_tag: image_tag.to_owned(),
        })
    }

    async fn build_image(docker: &Docker, path: &str, tag: &str) -> Result<(), Error> {
        let options = BuildOptions::builder(path).tag(tag).build();
        let mut stream = docker.images().build(&options);
        while let Some(build_result) = stream.next().await {
            build_result.map_err(Error::FailedToBuildImage)?;
        }
        Ok(())
    }

    pub async fn create_container(&self) -> Result<String, Error> {
        let mount = format!("{}:/home/code", &self.root_dir);
        let options = ContainerOptions::builder(&self.image_tag)
            .volumes(vec![&mount])
            .build();
        self.docker.containers().create(&options).await.map_or_else(
            |err| {
                Err(Error::FailedToCreateContainer {
                    image_tag: self.image_tag.to_owned(),
                    source: err,
                })
            },
            |result| Ok(result.id),
        )
    }

    fn validate_directory(directory: &str) -> Result<String, Error> {
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

        Path::new(directory).canonicalize().map_or_else(
            |err| {
                Err(Error::FailedToRetrieveAbsolutePath {
                    directory: directory.to_owned(),
                    source: err,
                })
            },
            |path_buf| Ok(path_buf.to_string_lossy().to_string()),
        )
    }
}

use std::env;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use failure::ResultExt;
use toml;

use errors::*;
use service::Service;

const TOML_TEMPLATE: &str = r#"# Example devbox project configuration
#
# Devbox can automatically create external docker volumes by specifying each
# volume as a name.
#
# volumes = [
#   "mysql",
#   "postgres"
# ]
#
# Service definitions allow devbox to know how to clone or find the source code.
#
# * `name` - The name of the service
# * `repo` - The source repository path on GitHub (i.e. "scrogson/devbox")
# * `path` - The path to the source on disk.
#
# [[services]]
# name = "service1"
# repo = "<github-org-or-user>/<repo>"
#
# [[services]]
# name = "service2"
# repo = "<github-org-or-user>/<repo>"
# path = "/path/to/source"

volumes = []

[[services]]
"#;

const COMPOSE_YAML_TEMPLATE: &str = r#"# This is an example docker-compose config file
#
# Replace the contents based on your project's requirements.
#
# See https://docs.docker.com/compose/compose-file for details.
version: "3"

networks:
  example:
    external: true

volumes:
  postgres:
    external: true
  mysql:
    external: true

services:

  redis:
    image: redis
    restart: "on-failure"
    ports:
      - "127.0.0.1:6379:6379"
    networks:
      - example

  mysql:
    image: mysql:5.6
    restart: "on-failure"
    environment:
      MYSQL_ALLOW_EMPTY_PASSWORD: "yes"
    volumes:
      - mysql:/var/lib/mysql
    ports:
      - "127.0.0.1:3306:3306"
    networks:
      - example

  postgres:
    image: postgres:9.6
    restart: "on-failure"
    environment:
      POSTGRES_USER: "postgres"
      POSTGRES_PASSWORD: "postgres"
      PGDATA: /var/lib/postgresql/data/pgdata
    volumes:
      - postgres:/var/lib/postgresql/data/pgdata
    ports:
      - "127.0.0.1:5432:5432"
    networks:
      - example
"#;

#[derive(Debug, Deserialize)]
pub struct Project {
    #[serde(skip)]
    pub docker_compose_file: PathBuf,

    #[serde(skip)]
    pub name: String,

    pub volumes: Vec<String>,
    pub services: Vec<Service>,
}

impl Project {
    pub fn new(name: &str) -> Result<Self> {
        let toml_config = toml_config_path(name)?;
        let yaml_config = yaml_config_path(name)?;

        let mut contents = String::new();
        File::open(toml_config)
            .context("Couldn't find devbox project config")?
            .read_to_string(&mut contents)
            .context("Unable to read config file")?;

        let mut project: Project =
            toml::from_str(&contents).context("Error deserializing toml config")?;

        project.docker_compose_file = yaml_config;
        project.name = name.to_owned();

        project
            .services
            .iter_mut()
            .for_each(|ref mut service| service.project_name = name.to_owned());

        env::set_var("COMPOSE_PROJECT_NAME", &name);
        env::set_var("COMPOSE_FILE", &project.docker_compose_file);

        Ok(project)
    }

    pub fn find_service(&mut self, name: &str) -> Result<&mut Service> {
        let service = self.services
            .iter_mut()
            .find(|ref mut service| service.name == name)
            .ok_or_else(|| ServiceNotFound(name.to_owned()))?;

        Ok(service)
    }
}

pub fn init(name: &str) -> Result<()> {
    let devbox_dir = devbox_dir(name)?;
    let toml_config = toml_config_path(name)?;
    let yaml_config = yaml_config_path(name)?;

    ensure_directory_exists(&devbox_dir);
    create_file_if_not_exists(&toml_config, TOML_TEMPLATE)?;
    create_file_if_not_exists(&yaml_config, COMPOSE_YAML_TEMPLATE)?;

    Ok(())
}

pub fn devbox_dir(name: &str) -> Result<PathBuf> {
    let home = env::home_dir().ok_or_else(|| format_err!("unable to determine home directory"))?;
    Ok(home.join(".config").join("devbox").join(name))
}

fn toml_config_path(name: &str) -> Result<PathBuf> {
    Ok(devbox_dir(name)?.join("config.toml"))
}

fn yaml_config_path(name: &str) -> Result<PathBuf> {
    Ok(devbox_dir(name)?.join("docker-compose.yml"))
}

fn ensure_directory_exists(path: &PathBuf) {
    if !path.exists() {
        println!("Creating {:?}", path);
        DirBuilder::new()
            .recursive(true)
            .create(path)
            .expect("Unable to create devbox project path");
    }
}

fn create_file_if_not_exists(path: &Path, content: &str) -> Result<()> {
    if !&path.exists() {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;

        println!("Creating file {:?}", &path);

        file.write_all(content.as_bytes())?;
    }

    Ok(())
}

use std::env;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use dirs::home_dir;
use failure::ResultExt;
use tempdir::TempDir;
use toml;

use crate::errors::*;
use crate::service::Service;

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
# * `git` - The source of a git repository
# * `path` - The path to the source on disk (optional)
#
# [services]
# service1 = { git = "https://github.com/scrogson/service1" }
# service2 = { path = "/path/to/service2" }
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

#[derive(Debug)]
pub struct Project {
    pub docker_compose_file: PathBuf,
    pub name: String,
    pub services: Vec<Service>,
    pub volumes: Vec<String>,
}

impl Project {
    pub fn new(project_name: &str) -> Result<Self> {
        let toml_config_path = toml_config_path(project_name)?;
        let yaml_config_path = yaml_config_path(project_name)?;
        let value = parse_toml_config(toml_config_path)?;

        let services = match value.get("services") {
            Some(services) => services
                .as_table()
                .expect("services must be in table format")
                .iter()
                .map(|(name, attributes)| {
                    let hooks = None;
                    let name = name.to_owned();
                    let path = attributes
                        .get("path")
                        .map(|s| PathBuf::from(s.as_str().unwrap()));
                    let project_name = project_name.to_owned();
                    let repo = attributes
                        .get("git")
                        .map(|s| s.as_str().unwrap().to_owned());
                    let tasks = None;

                    Service {
                        hooks,
                        name,
                        path,
                        project_name,
                        repo,
                        tasks,
                    }
                })
                .collect(),
            None => Vec::new(),
        };

        let volumes = match value.get("volumes") {
            Some(volumes) => volumes
                .as_array()
                .expect("volumes must be an array of strings")
                .to_vec()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect(),
            None => Vec::new(),
        };

        env::set_var("COMPOSE_PROJECT_NAME", &project_name);
        env::set_var("COMPOSE_FILE", &yaml_config_path);

        Ok(Project {
            docker_compose_file: yaml_config_path,
            name: project_name.to_owned(),
            services,
            volumes,
        })
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

    pub fn init_from_git(name: &str, git: &str) -> Result<()> {
        let dir = TempDir::new("devbox")?;

        let _ = Command::new("git")
            .arg("clone")
            .arg(git)
            .arg(&dir.path())
            .spawn()?
            .wait();

        let toml_path = &dir.path().join("config.toml");
        let yaml_path = &dir.path().join("docker-compose.yml");
        let toml_contents = read_file(&toml_path)?;
        let yaml_contents = read_file(&yaml_path)?;

        ensure_directory_exists(&devbox_dir(name)?);
        create_file_if_not_exists(&toml_config_path(name)?, &toml_contents)?;
        create_file_if_not_exists(&yaml_config_path(name)?, &yaml_contents)?;

        dir.close()?;

        Ok(())
    }

    pub fn find_service(&mut self, name: &str) -> Result<&mut Service> {
        let service = self.services
            .iter_mut()
            .find(|ref mut service| service.name == name)
            .ok_or_else(|| ServiceNotFound(name.to_owned()))?;

        let _ = service.rehydrate_from_devbox_toml();

        Ok(service)
    }
}

fn parse_toml_config(path: PathBuf) -> Result<toml::Value> {
    let mut contents = String::new();

    File::open(path)
        .context("Couldn't find devbox project config")?
        .read_to_string(&mut contents)
        .context("Unable to read config file")?;

    let value = toml::from_str::<toml::Value>(&contents)?;

    Ok(value)
}

pub fn devbox_dir(name: &str) -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| format_err!("unable to determine home directory"))?;
    Ok(home.join(".config").join("devbox").join(name))
}

fn read_file(path: &PathBuf) -> Result<String> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;

    Ok(contents)
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

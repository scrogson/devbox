use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

use colored::*;
use failure::ResultExt;
use prettytable::Table;
use prettytable::format;
use toml;

use errors::*;
use project;
use task::Task;

const COMPOSE_PATH: &str = ".devbox/docker-compose.yml";
const TOML_PATH: &str = ".devbox/config.toml";

pub fn cmd<S: AsRef<OsStr>>(program: S, service: &Service) -> Command {
    let devbox_compose_file = env::var("COMPOSE_FILE").unwrap();
    let service_compose_file = service.devbox_compose_file();

    let mut cmd = Command::new(program);

    cmd.arg("-f")
        .arg(&devbox_compose_file)
        .arg("-f")
        .arg(&service_compose_file)
        .arg("--project-directory")
        .arg(&service_compose_file.parent().unwrap());

    cmd
}

#[derive(Clone, Debug)]
pub struct Service {
    pub hooks: Option<BTreeMap<String, Vec<Task>>>,
    pub name: String,
    pub repo: Option<String>,
    pub path: Option<PathBuf>,
    pub project_name: String,
    pub tasks: Option<Vec<Task>>,
}

impl Service {
    pub fn rehydrate_from_devbox_toml(&mut self) -> Result<()> {
        let mut contents = String::new();

        if let Ok(mut file) = File::open(self.devbox_toml_file()) {
            file.read_to_string(&mut contents)
                .context("Unable to read config file")?;
        } else {
            println!(
                "{} Config file not found, no tasks or hooks are defined for {}",
                "WARN".yellow(),
                self.name
            );
            return Ok(());
        }

        if let Ok(values) = toml::from_str(&contents) {
            let _ = self.insert_tasks(&values);
            let _ = self.insert_hooks(&values);
        }

        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        if self.devbox_compose_file().exists() {
            let _ = cmd("docker-compose", self)
                .arg("up")
                .arg("-d")
                .arg(&self.name)
                .spawn()?
                .wait();

            return Ok(());
        }

        Err(format_err!(
            "Failed to start {} - missing docker-compose file",
            self.name
        ))?
    }

    pub fn stop(&self) -> Result<()> {
        if self.devbox_compose_file().exists() {
            let _ = cmd("docker-compose", self)
                .arg("stop")
                .arg(&self.name)
                .spawn()?
                .wait();

            return Ok(());
        }

        Err(format_err!(
            "Failed to stop {} - missing docker-compose file",
            self.name
        ))?
    }

    pub fn build(&mut self) -> Result<()> {
        if self.devbox_compose_file().exists() {
            self.run_lifecycle_hooks("before-build")?;

            let _ = cmd("docker-compose", self)
                .arg("build")
                .arg(&self.name)
                .spawn()?
                .wait();

            self.run_lifecycle_hooks("after-build")?;

            return Ok(());
        }

        Err(format_err!(
            "Failed to build {} - missing docker-compose file",
            self.name
        ))?
    }

    pub fn path_exists(&self) -> bool {
        self.source_path().exists()
    }

    pub fn clone_repo(&self) -> Result<()> {
        if self.path_exists() {
            eprintln!("{} already exists, fetching updates...", self.name);
            self.update_repo()
        } else {
            match self.repo {
                Some(ref repo) => {
                    let _ = Command::new("git")
                        .arg("clone")
                        .arg(repo)
                        .arg(self.source_path())
                        .spawn()?
                        .wait();
                    Ok(())
                }
                None => Err(format_err!("No repository configured for {}", self.name)),
            }
        }
    }

    pub fn update(&mut self) -> Result<()> {
        if self.path_exists() {
            self.run_lifecycle_hooks("before-update")?;
            self.update_repo()?;
            self.run_lifecycle_hooks("after-update")?;

            Ok(())
        } else {
            self.clone_repo()
        }
    }

    pub fn update_repo(&self) -> Result<()> {
        let _ = Command::new("git")
            .current_dir(self.source_path())
            .args(&["pull", "origin", "master"])
            .spawn()?
            .wait();
        Ok(())
    }

    pub fn find_task(&mut self, name: &str) -> Option<Task> {
        if let Ok(ref tasks) = self.tasks() {
            tasks.iter().cloned().find(|task| task.name == name)
        } else {
            None
        }
    }

    pub fn list_tasks(&mut self) -> Result<()> {
        let mut table = Table::new();

        table.set_format(*format::consts::FORMAT_CLEAN);
        table.add_row(row!["TASK", "DESCRIPTION"]);

        if let Ok(ref tasks) = self.tasks() {
            for task in tasks {
                table.add_row(row![task.name, task.description]);
            }
        }

        table.printstd();

        Ok(())
    }

    pub fn exec_tasks(&mut self, task_names: Vec<String>) -> Result<()> {
        for name in task_names {
            if let Some(ref task) = self.find_task(&name) {
                self.exec_task(task)?;
            } else {
                eprintln!(
                    "Task '{}' could not be found for service {}",
                    name, self.name
                );
            }
        }

        Ok(())
    }

    fn exec_task(&self, task: &Task) -> Result<()> {
        if self.devbox_compose_file().exists() {
            let _ = cmd("docker-compose", self)
                .arg("exec")
                .arg(&self.name)
                .args(&task.exec)
                .spawn()?
                .wait();

            return Ok(());
        }

        Err(format_err!(
            "Failed to execute {} - missing docker-compose file",
            self.name
        ))
    }

    fn run_task(&self, task: &Task) -> Result<()> {
        if self.devbox_compose_file().exists() {
            let _ = cmd("docker-compose", self)
                .arg("run")
                .arg("--rm")
                .arg(&self.name)
                .args(&task.exec)
                .spawn()?
                .wait();

            return Ok(());
        }

        Err(format_err!(
            "Failed to execute {} - missing docker-compose file",
            self.name
        ))
    }

    pub fn tasks(&mut self) -> Result<Vec<Task>> {
        match self.tasks {
            Some(ref v) => Ok(v.clone()),
            None => Ok(Vec::new()),
        }
    }

    fn hooks(&mut self) -> Result<BTreeMap<String, Vec<Task>>> {
        match self.hooks {
            Some(ref v) => Ok(v.clone()),
            None => Ok(BTreeMap::new()),
        }
    }

    fn run_lifecycle_hooks(&mut self, lifecycle: &str) -> Result<()> {
        println!("{} Running {} hooks", "INFO".green(), lifecycle);
        let tasks = self.tasks_for_hook(lifecycle);
        if !tasks.is_empty() {
            for task in tasks {
                let _ = self.run_task(&task);
            }
        }

        Ok(())
    }

    fn tasks_for_hook(&mut self, name: &str) -> Vec<Task> {
        self.hooks()
            .unwrap_or_default()
            .get(name)
            .unwrap_or(&vec![])
            .to_vec()
    }

    fn insert_tasks(&mut self, values: &toml::Value) -> Result<()> {
        match values.get("tasks") {
            Some(v) => self.tasks = v.clone().try_into::<Vec<Task>>().ok(),
            None => println!("No tasks found for service '{}'", self.name),
        };

        Ok(())
    }

    fn insert_hooks(&mut self, values: &toml::Value) -> Result<()> {
        let hooks = match values.get("hooks") {
            Some(v) => {
                let mut hooks = BTreeMap::new();
                let map = v.clone().try_into::<BTreeMap<String, Vec<String>>>()?;

                for (key, task_names) in &map {
                    let mut tasks: Vec<Task> = Vec::new();
                    for name in task_names {
                        match self.find_task(name.as_str()) {
                            Some(task) => tasks.push(task),
                            None => println!(
                                "{} Task `{}` was not found in the available tasks",
                                "WARN".yellow(),
                                name.as_str()
                            ),
                        }
                    }
                    hooks.insert(key.clone(), tasks);
                }

                Some(hooks)
            }
            None => None,
        };

        self.hooks = hooks;

        Ok(())
    }

    pub fn source_path(&self) -> PathBuf {
        match self.path {
            Some(ref path) => path.into(),
            None => project::devbox_dir(&self.project_name)
                .expect("unable to determine devbox project directory")
                .join("src")
                .join(&self.name),
        }
    }

    pub fn devbox_compose_file(&self) -> PathBuf {
        match self.path {
            Some(ref path) => path.join(COMPOSE_PATH),
            None => project::devbox_dir(&self.project_name)
                .expect("unable to determine devbox project directory")
                .join("src")
                .join(&self.name)
                .join(COMPOSE_PATH),
        }
    }

    pub fn devbox_toml_file(&self) -> PathBuf {
        match self.path {
            Some(ref path) => path.join(TOML_PATH),
            None => project::devbox_dir(&self.project_name)
                .expect("unable to determine devbox project directory")
                .join("src")
                .join(&self.name)
                .join(TOML_PATH),
        }
    }
}

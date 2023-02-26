use std::path::PathBuf;
use std::fs;
use std::process::Command;

use url::Url;
use tempfile::TempDir;
use serde::Deserialize;

pub struct Controller {
    task_id: String,
    host: Url,
    tempdir: TempDir,
}

impl Controller {
    pub fn new(host: Url, task_id: &str) -> ::anyhow::Result<Self> {
        let task_id = task_id.to_owned();
        let tempdir = TempDir::new()?;

        Ok(Self { task_id, host, tempdir })
    }

    pub fn download(&self) -> ::anyhow::Result<Task> {
        let path = format!("/tasks/{}.json", self.task_id);
        let endpoint = self.host.join(&path)?;
        let response = reqwest::blocking::get(endpoint)?;
        let exercise = response.json()?;

        Ok(exercise)
    }

    pub fn create_file(&self, name: &str, contents: &str) -> ::anyhow::Result<PathBuf> {
        let path = self.tempdir.path().join(name);
        fs::write(&path, contents)?;
        Ok(path)
    }
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub input: String,
    pub output: String,
    pub version: String,
}

pub struct Vim {
    path: PathBuf,
}

impl Vim {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn run(&self) -> ::anyhow::Result<String> {
        Command::new("gvim").
            arg("--nofork").
            arg(&self.path).
            output()?;

        let result = fs::read_to_string(&self.path)?;
        Ok(result)
    }
}

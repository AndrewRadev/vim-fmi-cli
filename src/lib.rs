use std::path::PathBuf;
use std::fs;
use std::process::Command;

use url::Url;
use tempfile::TempDir;
use serde::Deserialize;

const VIMRC_CONTENTS: &'static str = include_str!("vimrc");

pub struct Controller {
    task_id: String,
    host: Url,
    tempdir: TempDir,
}

impl Controller {
    pub fn new(host: Url, task_id: &str) -> ::anyhow::Result<Self> {
        let task_id = task_id.to_owned();
        let tempdir = TempDir::new()?;

        fs::write(tempdir.path().join("vimrc"), VIMRC_CONTENTS)?;

        Ok(Self { task_id, host, tempdir })
    }

    pub fn vimrc_path(&self) -> PathBuf {
        self.tempdir.path().join("vimrc").to_owned()
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
    input_path: PathBuf,
    log_path: PathBuf,
    vimrc_path: PathBuf,
}

impl Vim {
    pub fn new(input_path: PathBuf, log_path: PathBuf, vimrc_path: PathBuf) -> Self {
        Self { input_path, log_path, vimrc_path }
    }

    pub fn run(&self) -> ::anyhow::Result<(String, Vec<u8>)> {
        // -Z         - restricted mode, utilities not allowed
        // -n         - no swap file, memory only editing
        // --noplugin - don't load any plugins, lets be fair!
        // --nofork   - otherwise GOLFVIM=gvim forks and returns immediately
        // -i NONE    - don't load .viminfo (for saved macros and the like)
        // +0         - always start on line 0
        // -u vimrc   - load vimgolf .vimrc to level the playing field
        // -U NONE    - don't load .gvimrc
        // -W logfile - keylog file (overwrites if already exists)
        Command::new("gvim").
            args(["--nofork", "-Z", "-n", "--noplugin", "-i", "NONE", "+0", "-U", "NONE"]).
            args(["-u", self.vimrc_path.to_str().unwrap()]).
            args(["-W", self.log_path.to_str().unwrap()]).
            arg(self.input_path.to_str().unwrap()).
            output()?;

        let result = fs::read_to_string(&self.input_path)?;
        let log = fs::read(&self.log_path)?;

        Ok((result, log))
    }
}

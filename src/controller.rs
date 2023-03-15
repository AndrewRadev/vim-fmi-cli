use std::path::PathBuf;
use std::fs::{self, File};

use anyhow::anyhow;
use url::Url;
use tempfile::TempDir;
use serde::{Serialize, Deserialize};
use base64::{Engine as _};
use directories::ProjectDirs;

const VIMRC_CONTENTS: &'static str = include_str!("vimrc");

pub struct Controller {
    host: Url,
    tempdir: TempDir,
}

impl Controller {
    pub fn new(host: Url) -> ::anyhow::Result<Self> {
        let tempdir = TempDir::new()?;

        fs::write(tempdir.path().join("vimrc"), VIMRC_CONTENTS)?;

        Ok(Self { host, tempdir })
    }

    pub fn vimrc_path(&self) -> PathBuf {
        self.tempdir.path().join("vimrc").to_owned()
    }

    pub fn setup_user(&self, user_token: &str) -> ::anyhow::Result<User> {
        let endpoint = self.host.join("/api/setup.json")?;
        let client = reqwest::blocking::Client::new();
        let meta = get_meta();

        let body = serde_urlencoded::to_string(&[
            ("token", user_token.to_owned()),
            ("meta", meta.to_string()),
        ])?;

        let response = client.post(endpoint).body(body).send()?;

        if response.status() == 200 {
            let user = response.json()?;
            write_user(&user)?;
            Ok(user)
        } else {
            let error: JsonError = response.json()?;
            Err(anyhow!("{}", error.message))
        }
    }

    pub fn download_task(&self, task_id: &str) -> ::anyhow::Result<Task> {
        let path = format!("/api/task/{}.json", task_id);
        let endpoint = self.host.join(&path)?;
        let response = reqwest::blocking::get(endpoint)?;

        if response.status() == 200 {
            let exercise = response.json()?;
            Ok(exercise)
        } else {
            let error: JsonError = response.json()?;
            Err(anyhow!("{}", error.message))
        }
    }

    pub fn upload(&self, task_id: &str, bytes: Vec<u8>) -> ::anyhow::Result<bool> {
        let endpoint = self.host.join("/api/solution.json")?;
        let client = reqwest::blocking::Client::new();
        let meta = get_meta();
        // Unwrap: We should have checked for a user before
        let user = read_user()?.unwrap();

        let body = serde_urlencoded::to_string(&[
            ("entry", ::base64::engine::general_purpose::STANDARD.encode(&bytes)),
            ("challenge_id", task_id.to_owned()),
            ("user_token", user.token),
            ("meta", meta.to_string()),
        ])?;
        let response = client.post(endpoint).body(body).send()?;

        if response.status().is_success() {
            Ok(true)
        } else {
            let error: JsonError = response.json()?;
            Err(anyhow!("{}", error.message))
        }
    }

    pub fn create_file(&self, name: &str, contents: &str) -> ::anyhow::Result<PathBuf> {
        let path = self.tempdir.path().join(name);
        fs::write(&path, contents)?;
        Ok(path)
    }
}

fn get_meta() -> serde_json::Value {
    serde_json::json!({
        "username": ::whoami::username(),
        "devicename": ::whoami::devicename(),
        "platform": ::whoami::platform().to_string(),
    })
}

pub fn read_user() -> ::anyhow::Result<Option<User>> {
    let proj_dirs = ProjectDirs::from("bg", "fmi", "vim-fmi-cli").
        ok_or_else(|| anyhow!("Couldn't initialize project dir"))?;
    let path = proj_dirs.data_dir().join("user.json");
    if !path.exists() {
        return Ok(None);
    }

    let data_file = File::open(path)?;
    let user = serde_json::from_reader(data_file)?;

    Ok(Some(user))
}

fn write_user(user: &User) -> ::anyhow::Result<()> {
    let proj_dirs = ProjectDirs::from("bg", "fmi", "vim-fmi-cli").
        ok_or_else(|| anyhow!("Couldn't initialize project dir"))?;
    let data_dir = proj_dirs.data_dir();

    fs::create_dir_all(data_dir)?;
    fs::write(data_dir.join("user.json"), serde_json::to_string(user)?)?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct JsonError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub input: String,
    pub output: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: u32,
    pub faculty_number: String,
    pub token: String,
}

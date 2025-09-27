use std::{
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    sync::OnceLock,
};

use clap::{Parser, Subcommand, crate_version};
use directories::BaseDirs;
use figment::{
    Figment,
    providers::{Env, Format, Json, Serialized},
};
use serde::{Deserialize, Serialize};

const CONFIG_FOLDER_RELATIVE_PATH: &str = ".git_diff_sync/";
const CONFIG_FILE_NAME: &str = "config.json";

pub static CONFIG: OnceLock<Config> = OnceLock::new();
pub static ARGUMENTS: OnceLock<Arguments> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server_address: String,
    pub api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: String::from("http://127.0.0.1:8000"),
            api_key: String::from("GIT_DIFF_SYNC_SERVER_API_KEY_HERE"),
        }
    }
}

#[derive(Debug, Parser, Serialize)]
#[command(version = crate_version!(), about, long_about = None, arg_required_else_help = true)]
pub struct Arguments {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long, global = true)]
    server_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long, global = true)]
    api_key: Option<String>,
    #[serde(skip)]
    #[command(subcommand)]
    pub command: Commands,
    #[serde(skip)]
    #[arg(action, short, long, global = true)]
    pub force: bool,
    #[serde(skip)]
    #[arg(action, short, long, global = true)]
    pub reset_working_tree: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Pushes the current diff to the configured Git Diff Sync server.
    Push,
    /// Pulls the most recent diff from the configured Git Diff Sync server.
    Pull,
}

pub fn parse() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let config: Config = {
        let user_config_folder_path = BaseDirs::new()
            .expect("should be able to get base directories")
            .config_local_dir()
            .join(CONFIG_FOLDER_RELATIVE_PATH.trim_prefix("."));
        let user_config_file_path = user_config_folder_path.join(CONFIG_FILE_NAME);
        if !fs::exists(&user_config_file_path)? {
            fs::create_dir_all(&user_config_folder_path)?;
            serde_json::to_writer_pretty(
                BufWriter::new(File::create(&user_config_file_path)?),
                &Config::default(),
            )?;
        }

        Figment::new()
            .merge(Json::file_exact(user_config_file_path))
            .merge(Json::file(
                PathBuf::from(CONFIG_FOLDER_RELATIVE_PATH).join(CONFIG_FILE_NAME),
            ))
            .merge(Env::prefixed("GIT_DIFF_SYNC_"))
            .merge(Serialized::defaults(&arguments))
            .extract()?
    };

    match ARGUMENTS.set(arguments) {
        Ok(arguments) => Ok(arguments),
        Err(_) => Err(anyhow::Error::msg("should be able to set ARGUMENTS")),
    }?;
    match CONFIG.set(config) {
        Ok(config) => Ok(config),
        Err(_) => Err(anyhow::Error::msg("should be able to set CONFIG")),
    }?;
    Ok(())
}

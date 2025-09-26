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

#[derive(Debug, Default, Parser, Serialize, Deserialize)]
#[command(version = crate_version!(), about, long_about = None, arg_required_else_help = true)]
pub struct Config {
    #[serde(skip)]
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Default, Subcommand)]
pub enum Commands {
    /// Pushes the current diff to the configured Git Diff Sync server.
    #[default]
    Push,
    /// Pulls the most recent diff from the configured Git Diff Sync server.
    Pull,
}

pub fn parse() -> anyhow::Result<()> {
    let config = {
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
            .merge(Serialized::defaults(Config::parse()))
            .merge(Json::file_exact(user_config_file_path))
            .merge(Json::file(
                PathBuf::from(CONFIG_FOLDER_RELATIVE_PATH).join(CONFIG_FILE_NAME),
            ))
            .merge(Env::prefixed("GIT_DIFF_SYNC_"))
            .extract()?
    };

    match CONFIG.set(config) {
        Ok(config) => Ok(config),
        Err(_) => Err(anyhow::Error::msg("should be able to set CONFIG")),
    }
}

use std::{path::PathBuf, sync::OnceLock};

use clap::{Parser, crate_authors, crate_description, crate_name, crate_version};
use directories::BaseDirs;
use figment::{
    Figment,
    providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};

pub static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Parser, Serialize, Deserialize)]
#[command(version = crate_version!(), author = crate_authors!(), long_about = format!("{}  Copyright (C) 2025  {}\n{}", crate_name!(), crate_authors!(), crate_description!()))]
pub struct Config {
    #[arg(long, default_value = Config::default().git_diffs_folder_path.display().to_string())]
    pub git_diffs_folder_path: PathBuf,
    #[arg(long, default_values_t = Config::default().api_keys, value_delimiter = ',')]
    pub api_keys: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            git_diffs_folder_path: {
                let base_directories =
                    BaseDirs::new().expect("should be able to get base directories");
                base_directories
                    .data_dir()
                    .to_path_buf()
                    .join("git_diff_sync_server/data/diffs")
            },
            api_keys: Default::default(),
        }
    }
}

pub fn parse() -> anyhow::Result<()> {
    let config: Config = {
        Figment::new()
            .admerge(Env::prefixed("GIT_DIFF_SYNC_"))
            .adjoin(Serialized::defaults(Config::parse()))
            .extract()?
    };
    match CONFIG.set(config) {
        Ok(config) => Ok(config),
        Err(_) => Err(anyhow::Error::msg("should be able to set CONFIG")),
    }
}

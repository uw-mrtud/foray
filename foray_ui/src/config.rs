use std::{
    fs::{self, read_to_string},
    path::PathBuf,
    process::Command,
};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::style::theme::AppTheme;

/// User configuration data that is saved/loaded from a config.toml file
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    venv_dir: PathBuf,
}

impl Config {
    pub fn read_config() -> Self {
        let user_dirs = directories::UserDirs::new()
            .expect("Application configuration folder should be accessible");
        let config_dir = user_dirs.home_dir().join(".config/gpi");
        let config_file = config_dir.join("config.toml");

        match read_to_string(&config_file).map(|s| toml::from_str::<Config>(&s)) {
            Ok(Ok(c)) => {
                info!("Loaded config: {config_file:?}");
                c
            }
            Ok(Err(e)) => {
                panic!("Error reading config {config_file:?}:\n{e}");
            }
            _ => {
                //// Could not read config, creating a default...
                // TODO: Prompt for venv path
                // TEMP: create default node location
                let nodes_dir = user_dirs.home_dir().join("gpi_default");
                let venv_dir = nodes_dir.join(".venv");

                println!("No configuration file found at {config_file:?} creating default config");
                if fs::read_dir(&nodes_dir).is_err() {
                    println!("creating default node directory at {nodes_dir:?}");
                    fs::create_dir(&nodes_dir)
                        .unwrap_or_else(|_| panic!("couldn't create default folder{nodes_dir:?}"));

                    let output = Command::new("python3")
                        .arg("-m")
                        .arg("venv")
                        .arg(&venv_dir)
                        .output()
                        .unwrap_or_else(|e| panic!("failed to execute python process {e}"));
                    info!("{output:?}");
                }

                println!("Creating default config file");
                let config = Config { venv_dir };
                let _ = std::fs::create_dir(config_dir);
                std::fs::write(
                    &config_file,
                    toml::to_string_pretty(&config).unwrap_or_else(|e| {
                        panic!("Could not parse config file {config_file:?}\n{e}")
                    }),
                )
                .unwrap_or_else(|e| panic!("Could not write config file {config_file:?}\n{e}"));
                config
            }
        }
    }
}

impl Config {
    pub fn load_theme() -> AppTheme {
        let user_dirs =
            directories::UserDirs::new().expect("application configuration folder is accessible");
        let theme_file = user_dirs.home_dir().join(".config/gpi/theme.ron");

        match read_to_string(&theme_file).map(|s| ron::from_str::<AppTheme>(&s)) {
            Ok(Ok(network)) => network,
            Ok(Err(e)) => {
                error!("Could not parse theme file: {theme_file:?}, using default.\n{e}");
                AppTheme::default()
            }
            Err(_e) => AppTheme::default(),
        }
    }
}

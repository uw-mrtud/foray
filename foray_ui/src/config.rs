use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, read_to_string},
    iter,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use foray_py::discover;
use log::{debug, error, info, warn};
use pyo3::{py_run, types::PyAnyMethods, PyResult, Python};
use serde::{Deserialize, Serialize};

use crate::{project::python_project, style::theme::AppTheme};

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

    pub(crate) fn read_projects(&self) -> Vec<crate::project::Project> {
        let raw = discover::get_foray_py_packages();
        raw.into_iter().map(python_project).collect()
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
    pub fn setup_environment(&self) {
        self.setup_python();
    }

    fn setup_python(&self) {
        //// Shell ENV variables
        {
            env::set_var("VIRTUAL_ENV", &self.venv_dir);
            env::set_var(
                "Path",
                prepend_env("PATH", self.venv_dir.join("bin")).unwrap(),
            );
            env::set_var("PYO3_PYTHON", self.venv_dir.join("bin/python"));

            // Set PYTHONPATH to appropriate paths in the venv directory
            // needed to address open pyo3 issue https://github.com/PyO3/pyo3/issues/1741
            if let Ok(paths) = glob::glob(
                self.venv_dir
                    .join("lib/python3*")
                    .to_str()
                    .unwrap_or_else(|| panic!("Paths must be valid unicode {:?}", self.venv_dir)),
            ) {
                let paths: Vec<_> = paths.filter_map(|p| p.ok()).collect();
                if paths.len() > 1 {
                    warn!("Multiple python versions detected in venv {:?}, this has not been tested. Unexpected results may occur",self.venv_dir)
                }
                let paths_to_add: Vec<PathBuf> = paths
                    .into_iter()
                    .map(|path| path.join("site-packages"))
                    .flat_map(|path| {
                        // When a package is installed as "editable"
                        // a *.pth file is used to point to where the source code actually lives.
                        // we find all these *.pth files and add them to path
                        let editable_paths: Vec<PathBuf> =
                            glob::glob(path.join("*.pth").to_str().unwrap())
                                .unwrap()
                                .filter_map(|p| p.ok())
                                .filter(|path| {
                                    path.file_name() != Some(OsStr::new("_virtualenv.pth"))
                                })
                                .filter_map(|path| {
                                    let contents = fs::read_to_string(&path).unwrap();
                                    match PathBuf::from_str(&contents) {
                                        Ok(p) => Some(p),
                                        Err(_) => {
                                            warn!(
                                        "Unexpected `.pth` file contents in {path:?}: {contents}"
                                    );
                                            None
                                        }
                                    }
                                })
                                .collect();

                        [path].into_iter().chain(editable_paths).collect::<Vec<_>>()
                    })
                    .collect();
                debug!("Adding paths to PYTHONPATH {paths_to_add:#?}");

                paths_to_add.iter().for_each(|path| {
                    env::set_var(
                        "PYTHONPATH",
                        prepend_env("PYTHONPATH", path)
                            .unwrap()
                            .to_str()
                            .unwrap_or_else(|| panic!("Paths must be valid unicode {path:?}")),
                    );
                });
            }
        }

        //// PYO3 init
        pyo3::prepare_freethreaded_python();

        // Check python is working correctly, and display
        // venv location
        Python::with_gil(|py| {
            let list = 0u32;
            py_run!(
                py,
                list,
                r#"
import sys
print("Using python virtual environment:",sys.path)
"#
            );
        });

        // Configure python to close the program when
        // SIGINT (ctrl-c) is received. Otherwise ctrl-c won't work!
        let _ = Python::with_gil(|py| -> PyResult<()> {
            let signal = py.import("signal")?;
            signal
                .getattr("signal")?
                .call1((signal.getattr("SIGINT")?, signal.getattr("SIG_DFL")?))?;
            Ok(())
        });
    }
}

/// Create a new env string that has the given value prepended
fn prepend_env<P: AsRef<Path>>(env: &str, p: P) -> Result<OsString, env::JoinPathsError> {
    let new_path = p.as_ref();
    if let Some(path) = env::var_os(env) {
        let old = env::split_paths(&path);
        Ok(env::join_paths(iter::once(new_path.to_owned()).chain(old))?)
    } else {
        Ok(new_path.into())
    }
}

use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self},
    iter,
    path::{Path, PathBuf},
    str::FromStr,
};

use log::{debug, warn};
use pyo3::{types::PyAnyMethods, PyResult, Python};

pub fn setup_python(venv_dir: PathBuf) {
    let venv_dir = fs::canonicalize(&venv_dir)
        .unwrap_or_else(|_| panic!("virtual environment should exist at {venv_dir:?}"));

    println!("Using python environment: {venv_dir:?}");

    //// Shell ENV variables
    {
        env::set_var("VIRTUAL_ENV", &venv_dir);
        env::set_var("Path", prepend_env("PATH", venv_dir.join("bin")).unwrap());
        env::set_var("PYO3_PYTHON", venv_dir.join("bin/python"));

        // Set PYTHONPATH to appropriate paths in the venv directory
        // needed to address open pyo3 issue https://github.com/PyO3/pyo3/issues/1741
        if let Ok(paths) = glob::glob(
            venv_dir
                .join("lib/python3*")
                .to_str()
                .unwrap_or_else(|| panic!("Paths must be valid unicode {:?}", venv_dir)),
        ) {
            let paths: Vec<_> = paths.filter_map(|p| p.ok()).collect();
            if paths.len() > 1 {
                warn!("Multiple python versions detected in venv {:?}, this has not been tested. Unexpected results may occur",venv_dir)
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
                            .filter(|path| path.file_name() != Some(OsStr::new("_virtualenv.pth")))
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

            // **Override** PYTHONPATH
            env::set_var(
                "PYTHONPATH",
                paths_to_add
                    .iter()
                    .map(|p| {
                        p.to_str()
                            .unwrap_or_else(|| panic!("Paths must be valid unicode {:?}", &p))
                    })
                    .collect::<Vec<_>>()
                    .join(":"),
            );
        }
    }

    //// PYO3 init
    pyo3::prepare_freethreaded_python();

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

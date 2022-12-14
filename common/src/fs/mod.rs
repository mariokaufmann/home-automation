use std::path::PathBuf;

use anyhow::{anyhow, Context};

pub mod util;

const ORGANIZATION_SUBFOLDER_NAME: &str = "mariokaufmann";
const LOGS_SUBFOLDER_NAME: &str = "logs";

pub fn get_log_folder_path(application_name: &str) -> anyhow::Result<PathBuf> {
    let mut path = get_application_folder(application_name)?;
    path.push(LOGS_SUBFOLDER_NAME);

    if !path.exists() {
        match std::fs::create_dir_all(&path) {
            Ok(()) => Ok(path),
            Err(err) => Err(anyhow!("Could not prepare log subfolder: {}", err)),
        }
    } else {
        Ok(path)
    }
}

pub fn get_application_folder(application_name: &str) -> anyhow::Result<PathBuf> {
    let mut path = get_profile_folder()?;
    let organization_subfolder = ".".to_owned() + ORGANIZATION_SUBFOLDER_NAME;
    path.push(organization_subfolder);
    path.push(application_name);

    if !path.exists() {
        match std::fs::create_dir_all(&path) {
            Ok(()) => Ok(path),
            Err(err) => Err(anyhow!("Could not prepare application folder: {}.", err)),
        }
    } else {
        Ok(path)
    }
}

#[cfg(target_os = "windows")]
fn get_profile_folder() -> anyhow::Result<PathBuf> {
    const PROFILE_FOLDER_VAR: &str = "userprofile";
    let profile_folder_path = std::env::var(PROFILE_FOLDER_VAR)
        .context("User profile environment variable was not set.")?;
    Ok(PathBuf::from(profile_folder_path))
}

#[cfg(not(target_os = "windows"))]
fn get_profile_folder() -> anyhow::Result<PathBuf> {
    const USER_FOLDER_VAR: &str = "HOME";
    let profile_folder_path =
        std::env::var(USER_FOLDER_VAR).context("Home environment variable was not set.")?;
    Ok(PathBuf::from(profile_folder_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_profile_folder() {
        let path = get_profile_folder().unwrap();
        assert!(path.exists());
    }
}

use std::{env, error::Error, path::PathBuf};

use rocket::{
    form::Form,
    fs::{FileServer, TempFile},
};
use rocket_errors::anyhow;
use static_init::dynamic;
use thiserror::Error;

#[macro_use]
extern crate rocket;

const GIT_DIFFS_PATH_ENVIRONMENT_VARIABLE: &str = "GIT_DIFFS_PATH";

#[dynamic]
pub static GIT_DIFFS_PATH: PathBuf = PathBuf::from(
    env::var(GIT_DIFFS_PATH_ENVIRONMENT_VARIABLE)
        .unwrap_or(String::from("target/debug/data/diffs/")),
);

#[derive(Debug, Error)]
pub enum GitDiffError {
    #[error("Git diff file should have a name")]
    FileWithoutName,
}

#[launch]
fn rocket() -> _ {
    // TODO: Automatically create the required directories for the `FileServer` mount
    rocket::build()
        .mount("/", routes![status, push_git_diff])
        .mount(
            "/diffs",
            FileServer::from(
                env::current_dir()
                    .expect("should be able to get working directory")
                    .join(GIT_DIFFS_PATH.as_path()),
            ),
        )
}

#[get("/status")]
fn status() -> &'static str {
    "OK"
}

#[post(
    "/diffs/push",
    format = "multipart/form-data",
    data = "<git_diff_file>"
)]
async fn push_git_diff(mut git_diff_file: Form<TempFile<'_>>) -> anyhow::Result<()> {
    let git_diff_file_name = match git_diff_file.name() {
        Some(git_diff_file_name) => git_diff_file_name.to_owned(),
        None => return Err(GitDiffError::FileWithoutName)?,
    };
    let git_diff_file_path = {
        let mut git_diff_file_path = env::current_dir()?;
        git_diff_file_path.push(GIT_DIFFS_PATH.as_path());
        git_diff_file_path.push(git_diff_file_name);
        git_diff_file_path.with_extension("patch")
    };
    git_diff_file.copy_to(git_diff_file_path).await?;
    Ok(())
}

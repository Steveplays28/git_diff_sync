use std::env;

use clap::CommandFactory;
use git_diff_sync::config::{self, ARGUMENTS, Arguments, CONFIG, Commands};
use git2::{DiffFormat, Error, Repository, Tree};
use reqwest::{
    Client,
    multipart::{Form, Part},
};
use sha2::{Digest, Sha256};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::parse()?;

    let git_repository = match Repository::open_from_env() {
        Ok(git_repository) => git_repository,
        Err(_) => {
            println!("warning: Not a git repository.");
            let _ = Arguments::command().print_long_help();
            return Ok(());
        }
    };
    let head_oid = git_repository
        .head()
        .and_then(|head| {
            if let Some(head_target) = head.target() {
                return Ok(git_repository.find_tree(head_target)?);
            }

            Err(Error::from_str("git repository head did not have a target"))
        })
        .ok();
    let git_diff_file_name = get_git_diff_file_name(head_oid.as_ref());
    let client = Client::new();
    match ARGUMENTS
        .get()
        .expect("should be able to get ARGUMENTS")
        .command
    {
        Commands::Push => {
            push_git_diff(
                &git_repository,
                head_oid.as_ref(),
                git_diff_file_name,
                client,
            )
            .await?
        }
        Commands::Pull => pull_git_diff(&git_diff_file_name, client).await?,
    }

    Ok(())
}

fn get_git_diff_file_name(head_oid: Option<&Tree<'_>>) -> String {
    match head_oid {
        Some(head_oid) => {
            format!("{}.patch", head_oid.id())
        }
        None => {
            let working_directory_path_hash = {
                let mut hasher = Sha256::new();
                hasher.update(
                    env::current_dir()
                        .expect("should be able to get working directory")
                        .file_name()
                        .expect("should be able to get the working directory folder name")
                        .to_string_lossy()
                        .as_bytes(),
                );
                hasher.finalize()
            };
            format!("{}.patch", hex::encode(working_directory_path_hash))
        }
    }
}

async fn push_git_diff(
    git_repository: &Repository,
    head_oid: Option<&Tree<'_>>,
    git_diff_file_name: String,
    client: Client,
) -> anyhow::Result<()> {
    let git_diff = {
        let mut git_diff = git_repository.diff_tree_to_index(head_oid, None, None)?;
        git_diff.merge(&git_repository.diff_index_to_workdir(None, None)?)?;
        git_diff
    };
    let mut git_diff_file = String::new();
    git_diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        git_diff_file.push_str(&String::from_utf8_lossy(line.content()));
        true
    })?;

    let config = CONFIG.get().expect("should be able to get CONFIG");
    let response = client
        .post(format!("{}/diffs/push", &config.server_address))
        .bearer_auth(&config.api_key)
        .multipart(
            Form::new().part(
                "git_diff_file",
                Part::text(git_diff_file)
                    .file_name(git_diff_file_name.clone())
                    .mime_str("text/plain; charset=UTF-8")?,
            ),
        )
        .send()
        .await?;
    let response_text = {
        let mut response_text = response.text().await?;
        if !response_text.is_empty() {
            response_text.insert_str(0, "\nReceived response:\n");
        }
        response_text
    };
    println!(
        "Pushed diff {} to the configured Git Diff Sync server.{}",
        git_diff_file_name, response_text
    );
    Ok(())
}

async fn pull_git_diff(git_diff_file_name: &str, client: Client) -> anyhow::Result<()> {
    let config = CONFIG.get().expect("should be able to get CONFIG");
    let git_diff_file = client
        .get(format!(
            "{}/diffs/{}",
            &config.server_address, git_diff_file_name
        ))
        .bearer_auth(&config.api_key)
        .send()
        .await?;

    println!("{}", git_diff_file.text().await?);
    println!("Pulled diff from the configured Git Diff Sync server.");
    Ok(())
}

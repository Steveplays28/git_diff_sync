use std::{env, io::Read};

use anyhow::anyhow;
use clap::CommandFactory;
use git_diff_sync::config::{self, ARGUMENTS, Arguments, CONFIG, Commands};
use git2::{
    ApplyLocation, Commit, Diff, DiffFormat, DiffLineType, Error, Repository, ResetType, Tree,
    build::CheckoutBuilder,
};
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
            println!("Warning: not a Git repository.");
            let _ = Arguments::command().print_long_help();
            return Ok(());
        }
    };
    let git_head_commit = git_repository
        .head()
        .and_then(|head| {
            if let Some(head_target) = head.target() {
                return Ok(git_repository.find_commit(head_target)?);
            }

            Err(Error::from_str(
                "Git repository head did not have a target commit",
            ))
        })
        .ok();
    let git_head_tree = git_head_commit
        .as_ref()
        .and_then(|git_head_commit| git_head_commit.tree().ok());
    let git_diff_file_name = get_git_diff_file_name(git_head_tree.as_ref());
    let client = Client::new();
    match ARGUMENTS
        .get()
        .expect("should be able to get ARGUMENTS")
        .command
    {
        Commands::Push => {
            push_git_diff(
                &git_repository,
                git_head_tree.as_ref(),
                git_diff_file_name,
                client,
            )
            .await?
        }
        Commands::Pull => {
            pull_git_diff(
                &git_repository,
                git_head_tree.as_ref(),
                git_head_commit.as_ref(),
                &git_diff_file_name,
                client,
            )
            .await?
        }
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

fn get_git_diff_file(
    git_repository: &Repository,
    head_oid: Option<&Tree<'_>>,
) -> Result<String, Error> {
    let git_diff = {
        let mut git_diff = git_repository.diff_tree_to_index(head_oid, None, None)?;
        git_diff.merge(&git_repository.diff_index_to_workdir(None, None)?)?;
        git_diff
    };
    let mut git_diff_file = String::new();
    git_diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let line_origin = line.origin_value();
        if line_origin == DiffLineType::Addition
            || line_origin == DiffLineType::Deletion
            || line_origin == DiffLineType::Context
        {
            git_diff_file.push(line.origin());
        }
        line.content()
            .read_to_string(&mut git_diff_file)
            .expect("should be able to read Git diff content to a `String`");
        true
    })?;
    Ok(git_diff_file)
}

async fn push_git_diff(
    git_repository: &Repository,
    git_head_tree: Option<&Tree<'_>>,
    git_diff_file_name: String,
    client: Client,
) -> anyhow::Result<()> {
    let git_diff_file = get_git_diff_file(git_repository, git_head_tree)?;
    if git_diff_file.is_empty() {
        return Err(anyhow!(
            "There are no changes to push to the configured Git Diff Sync server."
        ));
    }

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

async fn pull_git_diff(
    git_repository: &Repository,
    git_head_tree: Option<&Tree<'_>>,
    git_head_commit: Option<&Commit<'_>>,
    git_diff_file_name: &str,
    client: Client,
) -> anyhow::Result<()> {
    let config = CONFIG.get().expect("should be able to get CONFIG");
    let arguments = ARGUMENTS.get().expect("should be able to get ARGUMENTS");
    let git_diff_file = client
        .get(format!(
            "{}/diffs/{}",
            &config.server_address, git_diff_file_name
        ))
        .bearer_auth(&config.api_key)
        .send()
        .await?;
    if !get_git_diff_file(git_repository, git_head_tree)?.is_empty() && !arguments.force {
        return Err(anyhow!(
            "Git working directory not clean.\nDid not apply diff from the configured Git Diff Sync server, use --force to override."
        ));
    }

    if arguments.reset_working_tree
        && let Some(git_head_commit) = git_head_commit
    {
        git_repository.reset(
            git_head_commit.as_object(),
            ResetType::Hard,
            Some(CheckoutBuilder::new().force()),
        )?;
    }

    git_repository.apply(
        &Diff::from_buffer(&git_diff_file.bytes().await?)?,
        ApplyLocation::Both,
        None,
    )?;
    println!(
        "Pulled and applied diff {} from the configured Git Diff Sync server.",
        git_diff_file_name
    );
    Ok(())
}

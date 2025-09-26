#![feature(trim_prefix_suffix)]

use std::fs::{self};

use git_diff_sync_server::config::{self, CONFIG};
use rocket::{
    Request,
    form::Form,
    fs::{FileServer, TempFile},
    http::Status,
    request::{self, FromRequest},
};
use thiserror::Error;

#[macro_use]
extern crate rocket;

struct ApiKey();

#[derive(Debug, Error)]
enum ApiKeyError {
    #[error("API key should be present")]
    Missing,
    #[error("API key should be valid")]
    Invalid,
}

impl<'r> ApiKey {
    pub fn check_api_key(request: &'r Request<'_>) -> request::Outcome<Self, ApiKeyError> {
        let authorization_header = request.headers().get_one("Authorization");
        match authorization_header {
            Some(bearer_token) => {
                if (CONFIG.get().expect("should be able to get CONFIG"))
                    .api_keys
                    .contains(&bearer_token.trim_prefix("Bearer ").to_string())
                {
                    return request::Outcome::Success(ApiKey());
                }

                return request::Outcome::Error((Status::Unauthorized, ApiKeyError::Invalid));
            }
            None => request::Outcome::Error((Status::Unauthorized, ApiKeyError::Missing)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        Self::check_api_key(request)
    }
}

#[launch]
fn rocket() -> _ {
    config::parse().expect("should be able to parse config");

    let config = CONFIG.get().expect("should be able to get CONFIG");
    fs::create_dir_all(&config.git_diffs_folder_path).expect(&format!(
        "should be able to create data folder at {}",
        &config.git_diffs_folder_path.display()
    ));
    rocket::build()
        .mount("/", routes![status, push_git_diff])
        .mount(
            "/diffs",
            FileServer::new(&config.git_diffs_folder_path)
                .filter(|_, request| ApiKey::check_api_key(request).is_success()),
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
async fn push_git_diff(
    _api_key: ApiKey,
    mut git_diff_file: Form<TempFile<'_>>,
) -> Result<(), Status> {
    let git_diff_file_name = match git_diff_file.name() {
        Some(git_diff_file_name) => git_diff_file_name.to_owned(),
        None => {
            return Err(Status::BadRequest);
        }
    };
    let git_diff_file_path = CONFIG
        .get()
        .expect("should be able to get CONFIG")
        .git_diffs_folder_path
        .join(git_diff_file_name)
        .with_extension("patch");
    match git_diff_file.copy_to(git_diff_file_path).await {
        Ok(_) => return Ok(()),
        Err(_) => return Err(Status::InternalServerError),
    }
}

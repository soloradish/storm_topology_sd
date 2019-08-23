use reqwest::Error as ReqwestError;
use snafu::Snafu;
use std::io;
use std::path::{Path, PathBuf};
use std::result;

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Enconter an error when request to Storm REST API. source {}", source))]
    HttpError { url: String, source: ReqwestError },
    #[snafu(display("Can't parse json response. srouce {}", source))]
    ParseError { source: ReqwestError },
    #[snafu(display("Unable to write result to {}: {}", path.display(), source))]
    WriteResult { source: io::Error, path: PathBuf },
}

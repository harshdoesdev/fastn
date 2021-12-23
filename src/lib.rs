extern crate self as fpm;

mod commands;
mod config;
mod dependency;
mod document;
mod font;
mod library;
mod snapshot;
mod tracker;
mod translation;
mod utils;

pub(crate) use commands::build::process_file;
pub use commands::{
    build::build, diff::diff, mark_upto_date::mark_upto_date, start_tracking::start_tracking,
    status::status, stop_tracking::stop_tracking, sync::sync,
    translation_status::translation_status,
};
pub use config::Config;
pub(crate) use config::Package;
pub(crate) use dependency::Dependency;
pub(crate) use document::{get_documents, get_file, paths_to_files, Document, File, Static};
pub(crate) use font::Font;
pub(crate) use library::{FPMLibrary, Library};
pub(crate) use snapshot::Snapshot;
pub(crate) use tracker::Track;
pub(crate) use translation::{TranslatedDocument, TranslationData};
pub(crate) use utils::{copy_dir_all, get_timestamp_nanosecond};

fn fpm_ftd() -> &'static str {
    include_str!("../ftd/fpm.ftd")
}

fn with_fallback() -> &'static str {
    include_str!("../with-fallback.html")
}

fn with_message() -> &'static str {
    include_str!("../with-message.html")
}

fn get_messages(status: &fpm::TranslatedDocument) -> &'static str {
    match status {
        TranslatedDocument::Missing { .. } => include_str!("../ftd/translation/missing.ftd"),
        TranslatedDocument::NeverMarked { .. } => {
            include_str!("../ftd/translation/never-marked.ftd")
        }
        TranslatedDocument::Outdated { .. } => include_str!("../ftd/translation/out-of-date.ftd"),
        TranslatedDocument::UptoDate { .. } => include_str!("../ftd/translation/upto-date.ftd"),
    }
}

// fn default_markdown() -> &'static str {
//     include_str!("../ftd/markdown.ftd")
// }

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HttpError: {}", _0)]
    HttpError(#[from] reqwest::Error),

    #[error("IoError: {}", _0)]
    IoError(#[from] std::io::Error),

    #[error("IoError: {}", _0)]
    ZipError(#[from] zip::result::ZipError),

    #[error("FTDError: {}", _0)]
    FTDError(#[from] ftd::p1::Error),

    #[error("PackageError: {message}")]
    PackageError { message: String },

    #[error("UsageError: {message}")]
    UsageError { message: String },

    #[error("IgnoreError: {}", _0)]
    IgnoreError(#[from] ignore::Error),

    #[error("FromPathBufError: {}", _0)]
    FromPathBufError(#[from] camino::FromPathBufError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn usage_error<T>(message: String) -> Result<T> {
    Err(Error::UsageError { message })
}

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}

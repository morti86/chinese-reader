use std::string::FromUtf8Error;

use image::ImageError;
use rig::completion::CompletionError;
#[cfg(feature = "scraper")]
use scraper::error::SelectorErrorKind;
use tokio::task::JoinError;

pub type ReaderResult<T> = Result<T, ReaderError>;

#[derive(Debug, Clone)]
pub enum ReaderError {
    Sqlite(String),
    Ai(String),
    Image(String),
    Io(String),
    Ocr(String),
    Other(String),
    Scraper(String),
}

impl ReaderError {
    pub fn other(s: &str) -> Self {
        Self::Other(s.to_string())
    }

    pub fn ocr(s: &str) -> Self {
        Self::Ocr(s.to_string())
    }
}

impl std::error::Error for ReaderError {}

impl std::fmt::Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Sqlite(s) => f.write_str(format!("SqliteError: {}", s).as_str()),
            Self::Ai(s) => f.write_str(format!("Ai Error: {}", s).as_str()),
            Self::Io(s) => f.write_str(format!("IO Error: {}", s).as_str()),
            Self::Image(s) => f.write_str(format!("Image Error: {}", s).as_str()),
            Self::Ocr(s) => f.write_str(format!("OCR Error: {}", s).as_str()),
            Self::Scraper(s) => f.write_str(format!("Scraper Error: {}", s).as_str()),
            Self::Other(s) => f.write_str(format!("Error: {}", s).as_str()),
        }
    }
}

impl From<rusqlite::Error> for ReaderError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e.to_string())
    }
}

impl From<JoinError> for ReaderError {
    fn from(e: JoinError) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<ImageError> for ReaderError {
    fn from(e: ImageError) -> Self {
        Self::Image(e.to_string())
    }
}

impl From<std::io::Error> for ReaderError { 
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<ort::error::Error> for ReaderError {
    fn from(e: ort::error::Error) -> Self {
        let msg = format!("{:?}: {}", e.code(), e);
        Self::Ocr(msg)
    }
}

impl From<rig::providers::mira::MiraError> for ReaderError {
    fn from(e: rig::providers::mira::MiraError) -> Self {
        Self::Ai(e.to_string())
    }
}

impl From<CompletionError> for ReaderError {
    fn from(e: CompletionError) -> Self {
        Self::Ai(e.to_string())
    }
}

impl From<deepl::Error> for ReaderError {
    fn from(e: deepl::Error) -> Self {
        Self::Other(e.to_string())
    }
}

impl From<FromUtf8Error> for ReaderError {
    fn from(e: FromUtf8Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<reqwest::Error> for ReaderError {
    fn from(e: reqwest::Error) -> Self {
        Self::Scraper(e.to_string())
    }
}

#[cfg(feature = "scraper")]
impl From<SelectorErrorKind<'_>> for ReaderError {
    fn from(e: SelectorErrorKind<'_>) -> Self {
        Self::Scraper(e.to_string())
    }
}

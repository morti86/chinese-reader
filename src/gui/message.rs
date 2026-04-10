use std::fmt::Debug;

use iced::widget::{markdown::Uri, text_editor};

use crate::ai::ChatEvent;
#[cfg(feature = "scraper")]
use crate::scraper::{LinkExtractorType, TextExtractorType};

#[derive(Clone,Debug)]
pub enum FileToDl {
    Det,
    Rec,
    Index,
}

#[derive(Clone,Debug)]
pub enum Message {
    ShowModal(String),
    Close,
    AiSettings,
    AiSettingsSave,
    AiSaveDone,
    Settings,
    SettingsSave,
    SaveDone,

    EditAction(text_editor::Action),
    TextAction(super::TextOption),
    NotesAction(text_editor::Action),

    ClearResult,
    AppendResult(String),
    SetResult(String),

    ClearAnswer,
    AppendAnswer(String),
    SetAnswer(String),

    SetTextValue(String),

    DictionaryCopy,
    DictionaryToNotes,
    DictionaryToNotesAppend(bool),

    Ocr,
    OcrFile,

    Simplified,

    LinkClicked(Uri),

    AiChatSelected(crate::config::AiChatConfig),
    AskAi,
    AiNameChange(String),
    AiUrlChange(String),
    AiModelChange(String),
    AiKeyChange(String),
    AiRoleChanged(String),
    AiProviderChange(crate::config::Provider),
    NewAi,
    AiStop,

    SidebarModeChanged(super::SidebarMode),

    ThemeSelected(iced::Theme),

    Void,

    AnswerCopy,
    AnswerToNotes,
    RemoveNote,
    SaveNote,
    PrevNote,
    NextNote,

    DbChange,

    LoadText,
    TextSelected(crate::textbase::Document),
    SaveText,
    RemoveText,
    TextTitleChange(String),
    CancelLoad,
    NewTextCheck(bool),
    NewText,

    AiChatEvent(ChatEvent),
    PromptMeaning,
    PromptUsage,
    PromptExplain,
    PromptGrammar,
    PromptSummary,

    FontSizeChange(f32),
    DeeplKeyChange(String),

    Deepl,

    LoadImage,
    LoadImageFile,
    SetImage(Vec<u8>),
    ClearImage,

    UpdateProgress,

    DownloadFile(FileToDl),
    DlEvent(crate::ocr::dl::DlEvent),

    #[cfg(feature = "scraper")]
    Scraper,
    #[cfg(feature = "scraper")]
    StartScraper,
    #[cfg(feature = "scraper")]
    StopScraper,
    #[cfg(feature = "scraper")]
    ScraperEvent(crate::scraper::ScraperEvent),
    #[cfg(feature = "scraper")]
    ScraperSleepChanged(String),
    #[cfg(feature = "scraper")]
    ScraperUrlChanged(String),
    #[cfg(feature = "scraper")]
    ScraperFileChanged(String),
    #[cfg(feature = "scraper")]
    ScraperSingle(bool),
    #[cfg(feature = "scraper")]
    ScraperCurrentFile,

    Language(crate::config::Language),

    AppDataChanged(String),
    AnkiResUpdate(Vec<String>),
    AnkiChanged(String),

    MarkDown(bool),

    TextMode(super::TextMode),

    #[cfg(feature = "scraper")]
    LP(String),
    #[cfg(feature = "scraper")]
    TP(String),
    #[cfg(feature = "scraper")]
    LX(String),
    #[cfg(feature = "scraper")]
    LN(String),
    #[cfg(feature = "scraper")]
    TN(String),
    #[cfg(feature = "scraper")]
    TT(String),
    #[cfg(feature = "scraper")]
    LU(String),

    #[cfg(feature = "scraper")]
    LinkExtractorChanged(LinkExtractorType),
    #[cfg(feature = "scraper")]
    ScNew,
    #[cfg(feature = "scraper")]
    TextExtractorChanged(TextExtractorType),

    Notes,
    NotesExport,
    NotesCopy(String),
    NotesDelete{ document: u32, line: i64, character: i64 },
}

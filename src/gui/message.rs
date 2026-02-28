use std::fmt::Debug;

use iced::widget::{markdown::Uri, text_editor};

use crate::ai::ChatEvent;
#[cfg(feature = "scraper")]
use crate::scraper::{LinkExtractorType, TextExtractorType};

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
    AiNdimsChanged(String),
    AiMaxDocsChanged(String),
    AiProviderChange(crate::config::Provider),
    NewAi,

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

    Scraper,
    StartScraper,
    StopScraper,
    #[cfg(feature = "scraper")]
    ScraperEvent(crate::scraper::ScraperEvent),
    ScraperSleepChanged(String),
    ScraperUrlChanged(String),
    ScraperFileChanged(String),
    ScraperSingle(bool),
    ScraperCurrentFile,

    Language(crate::config::Language),

    AppDataChanged(String),
    AnkiResUpdate(Vec<String>),
    AnkiChanged(String),

    MarkDown(bool),

    TextMode(super::TextMode),

    LP(String),
    TP(String),
    LX(String),
    LN(String),
    TN(String),
    TT(String),
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
    NotesDelete{ document: u32, line: usize, character: usize },
}

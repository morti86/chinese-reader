use std::fmt::Debug;

use iced::widget::{markdown::Uri, text_editor};

use crate::{ai::ChatEvent, config::LlamaType};

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
    Settings,
    SettingsSave,
    AnkiStats,

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
    AiNameChange(String),
    AiUrlChange(String),
    AiModelChange(String),
    AiKeyChange(String),
    AiRoleChanged(String),
    AiTemperatureChanged(String),
    AiProviderChange(crate::config::Provider),
    AiNoStreamResult(String),
    NewAi,
    AiOllamaKeepAlive,
    AiOllamaKeepAliveRes(bool),
    AiOllamaKeepAliveModal{ msg: String, ka: bool },
    AiLlamaTypeSelected(LlamaType),
    AiLlamaModelPathPick,
    AiLlamaCtxSizeChanged(String),
    AiCacheTypeKChanged(crate::config::CacheType),
    AiCacheTypeVChanged(crate::config::CacheType),
    AiFlashAttnChanged(bool),
    AiNCpuMoeChanged(String),
    AiReasoningBudgetChanged(String),
    AiPortChanged(String),
    AiLlamaEvent(crate::ai::llama::LlamaEvent),
    AiLlamaToggle,
    AiPresencePenaltyChanged(String),
    AiLlamaMmprojPicked,
    AiCancelPrompt,

    AiPromptSelected(crate::gui::Prompt),
    AiCustomPromptChanged(text_editor::Action),
    AiTextCtxToggle(bool),
    AiImageIncludeToggle(bool),
    Prompt,
    AiCurrentTextToEditor,
    AiHistoryClear,

    SidebarModeChanged(super::SidebarMode),

    ThemeSelected(iced::Theme),

    Void,
    Exit,
    BeforeExit,

    AnswerCopy,
    AnswerToNotes,
    RemoveNote,
    SaveNote,

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
    PromptTranslate,

    FontSizeChange(f32),
    LoadImage,
    LoadImageFile,
    SetImage(Vec<u8>),
    ClearImage,

    UpdateProgress,

    DlEvent(crate::ocr::dl::DlEvent),

    Language(String),

    AppDataChanged(String),
    //AnkiResUpdate(Vec<String>),
    AnkiChanged(String),

    //MarkDown(bool),

    TextMode(super::TextMode),

    Notes,
    NotesExport,
    NotesCopy(String),
    NotesDelete{ document: u32, line: i64, character: i64 },

    DeeplAsk,
    DeeplLangSel(deepl::Lang),
    DeeplKeyChanged(String),
}

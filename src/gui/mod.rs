mod settings;
pub mod message;

use crate::ai::{ask_deepl, ChatCommand, ChatEvent};
use crate::cedict::Cedict;
use crate::error::ReaderError;
#[cfg(feature = "scraper")]
use crate::scraper::{LinkExtractorType, ScraperCommand, ScraperEvent, TextExtractorType};
use crate::{ai, make_enum, ocr, AGENT};
use crate::config::{AiChatConfig, Config};
use iced::widget::text_editor::{Content, Position};
use iced::{clipboard, Element, Subscription, Theme};
use iced::widget::{text_editor,markdown};
use rig::client::builder::DynClientBuilder;
use rig::completion::Message as Rmsg;
use rig::message::{ContentFormat, Document, DocumentMediaType, UserContent};
use rig::streaming::StreamedAssistantContent;
use rig::OneOrMany;
use rusqlite::Connection;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender;
use crate::textbase::{*, Document as Doc};
use crate::utils::{get_image, str_to_op, url_for_provider};
use tracing::{debug, error, info, warn};
use std::sync::Arc;
use message::Message;

make_enum!(SidebarMode, [AI, Notes, Dictionary, Anki]);
make_enum!(TextOption, [Load, Save, Add, New, Delete]);

#[derive(Clone, Debug, PartialEq)]
pub enum TextMode {
    Raw,
    Md,
}

const CONF: &str = "./app.toml";

#[derive(Clone, Debug)]
pub enum AppState {
    Default,
    Modal(String),
    Settings,
    AiSettings,
    TextManage(TextOption),
    #[cfg(feature = "scraper")]
    Scraper,
    Notes,
}

pub struct App {
    conf: Config,
    state: AppState,
    doc_conn: Connection,
    // Sidebar
    sidebar_mode: SidebarMode,
    sidebar_notes: text_editor::Content,
    answer_text: markdown::Content,
    answer_raw: String,
    result_text: markdown::Content,
    result_raw: String,
    text: text_editor::Content,
    text_md: markdown::Content,

    image_data: Arc<RwLock<Vec<u8>>>,
    loaded_text: Doc,
    cedict: Option<Cedict>,

    new_ai: Option<AiChatConfig>,
    chat_history: Vec<Rmsg>,
    ai_changed: bool,

    sender: Option<Sender<ChatCommand>>,
    #[cfg(feature = "scraper")]
    sc_sender: Option<Sender<ScraperCommand>>,

    temp_document: Option<Document>,
    documents: Vec<Doc>,
    notes: Vec<Note>,
    note_idx: usize,

    note_edited: bool,
    new_text: bool,

    #[cfg(feature = "scraper")]
    scraper_progress: f32,
    #[cfg(feature = "scraper")]
    scraper_to: f32,
    #[cfg(feature = "scraper")]
    scraper_url: String,
    #[cfg(feature = "scraper")]
    scraper_file: String,
    #[cfg(feature = "scraper")]
    scraper_single: bool,

    anki_result: markdown::Content,
    text_mode: TextMode,
    sc_new: bool,

    position: Position,
    dtn_append: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let conf: Config = toml::from_str( std::fs::read_to_string( CONF ).unwrap().as_str() ).unwrap();
        let key = conf.ai_chat.clone();
        debug!("Ai Chat key={}", key);
        let chat = conf.ai_chats.get(key.as_str()).unwrap();
        let key = chat.key.clone().unwrap_or_default();
        let agent = DynClientBuilder::new()
            .agent_with_api_key_val(chat.provider.as_str().to_lowercase().as_str(), chat.model.as_str(), key)
            .unwrap()
            .preamble(conf.ai_preamble.as_str())
            .build();

        if let Err(e) = AGENT.set(agent) {
            error!("Error initializing agent: {}", e);
        }

        let appdata = conf.db.clone().unwrap_or("appdata.db".to_string());

        let doc_conn = crate::scraper::db::init_db(appdata).unwrap();
        let documents = match get_documents(&doc_conn) {
            Ok(documents) => documents,
            Err(e) => {
                error!("Error loading documents, {e}");
                vec![]
            }
        };

        Self {
            conf,
            state: AppState::Default,
            doc_conn,

            sidebar_mode: SidebarMode::Dictionary,
            sidebar_notes: text_editor::Content::new(),
            answer_text: markdown::Content::new(),
            answer_raw: String::new(),
            result_text: markdown::Content::new(),
            result_raw: String::new(),
            text: text_editor::Content::new(),

            image_data: Arc::new(RwLock::new(vec![])),
            loaded_text: crate::textbase::Document::default(),
            cedict: Cedict::new("dict.db").ok(),

            new_ai: None,
            chat_history: vec![],
            ai_changed: false,
            sender: None,
            #[cfg(feature = "scraper")]
            sc_sender: None,

            temp_document: None,
            documents,
            notes: vec![],
            note_idx: 0,

            note_edited: false,
            new_text: false,

            #[cfg(feature = "scraper")]
            scraper_progress: 0.0,
            #[cfg(feature = "scraper")]
            scraper_to: 1.0,
            #[cfg(feature = "scraper")]
            scraper_url: String::new(),
            #[cfg(feature = "scraper")]
            scraper_file: String::new(),
            #[cfg(feature = "scraper")]
            scraper_single: false,

            anki_result: markdown::Content::new(),
            text_mode: TextMode::Raw,
            text_md: markdown::Content::new(),
            sc_new: false,

            position: Position { line: 0, column: 0 },
            dtn_append: false,
        }
    }

    pub fn theme(&self) -> Theme {
        self.conf.window.theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {

        let ai_chat_sub = Subscription::run(ai::connect).map(Message::AiChatEvent);
        #[cfg(feature = "scraper")]
        let scr_sub = Subscription::run(crate::scraper::connect).map(Message::ScraperEvent);
        #[cfg(feature = "scraper")]
        let subs = vec![ai_chat_sub, scr_sub];
        #[cfg(not(feature = "scraper"))]
        let subs = vec![ai_chat_sub];
        Subscription::batch(subs)
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.state {
            AppState::Default => {
                settings::default(self).into()
            }
            AppState::Modal(msg) => {
                settings::display_av(&self.conf.window, msg.as_str())
            }
            AppState::Settings => {
                settings::settings(self, self.conf.get_labels())
            }
            AppState::AiSettings => {
                settings::ai_settings(self).into()
            }
            AppState::TextManage(opt) => {
                settings::text_action(self, *opt).into()
            }
            #[cfg(feature = "scraper")]
            AppState::Scraper => {
                settings::scrapper(self).into()
            }
            AppState::Notes => {
                settings::notes(self).into()
            }
        }
    }

    fn image_empty(&self) -> bool {
        let r = self.image_data.blocking_read();
        r.is_empty()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Close => {
                self.state = AppState::Default;
                self.new_ai = None;
                self.new_text = false;
                self.sc_new = false;
                match std::fs::read_to_string( CONF ) {
                    Ok(r) => {
                        match toml::from_str(r.as_str()) {
                            Ok(e) => { self.conf = e; }
                            Err(e) => {
                                error!("Error parsing toml {e}");
                                self.state = AppState::Modal(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error parsing toml {e}");
                        self.state = AppState::Modal(e.to_string());
                    }
                }
            }
            Message::ShowModal(msg) => {
                self.state = AppState::Modal(msg);
            }
            Message::TextAction(ta) => {
                self.state = AppState::TextManage(ta);
            }
            Message::SidebarModeChanged(sm) => {
                self.sidebar_mode = sm;
            }
            Message::AiSettings => {
                self.state = AppState::AiSettings;
            }
            Message::Settings => {
                self.state = AppState::Settings;
            }
            Message::SetTextValue(s) => {
                self.text = text_editor::Content::new();
                self.loaded_text = Doc::default();
                self.text.perform(
                    text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(s)))
                );
            }
            Message::FontSizeChange(fs) => {
                self.conf.window.font_size = Some(fs);
            }
            Message::ClearResult => {
                self.result_text = markdown::Content::new();
                self.result_raw = String::new();
            }
            Message::AppendResult(s) => {
                self.result_text.push_str(s.as_str());
                self.result_raw.push_str(s.as_str());
            }
            Message::SetResult(s) => {
                self.result_text = markdown::Content::new();
                self.result_text.push_str(s.as_str());
                self.result_raw = s;
            }
            Message::ClearAnswer => {
                self.answer_text = markdown::Content::new();
            }
            Message::AppendAnswer(s) => {
                self.answer_text.push_str(s.as_str());
            }
            Message::SetAnswer(s) => {
                self.answer_text = markdown::Content::new();
                self.answer_text.push_str(s.as_str());
            }

            Message::Ocr => {
                let model_path = self.conf.ocr_models.clone();
                let image_data = self.image_data.clone();
                return iced::Task::perform(async move {
                    ocr::ocr_i(&model_path, image_data).await
                }, |r| {
                    match r {
                        Ok(r) => Message::SetTextValue(r),
                        Err(e) => Message::ShowModal(e.to_string()),
                    }
                });
            }
            Message::LoadImage => {
                let mut rw = self.image_data.blocking_write();
                *rw = get_image();
            }
            Message::LoadImageFile => {
                let file_name = rfd::FileDialog::new()
                    .add_filter("image", &["png", "bmp", "jpg", "webp"])
                    .pick_file();
                if let Some(file_name) = file_name {
                    return iced::Task::perform(async move {
                        tokio::fs::read(file_name).await
                    }, |r| {
                        match r {
                            Err(e) => Message::ShowModal(e.to_string()),
                            Ok(r) => Message::SetImage(r),
                        }
                    })
                }

            }
            Message::SetImage(r) => {
                let mut rw = self.image_data.blocking_write();
                *rw = r;
            }
            Message::ClearImage => {
                let mut rw = self.image_data.blocking_write();
                *rw = vec![];
            }
            Message::OcrFile => {
                let file_name = rfd::FileDialog::new()
                    .add_filter("image", &["png", "bmp", "jpg", "webp"])
                    .pick_file();
                let model_path = self.conf.ocr_models.clone();
                if let Some(file_name) = file_name {
                    return iced::Task::perform(async move {
                        ocr::ocr_file(&model_path, file_name).await
                    }, |r| {
                        match r {
                            Ok(r) => Message::SetTextValue(r),
                            Err(e) => Message::ShowModal(e.to_string()),
                        }
                    })
                }

            }
            Message::EditAction(a) => {
                match a {
                    text_editor::Action::Click(_) => {
                        self.text.perform(a);
                        self.position = self.text.cursor().position;
                        let c = self.position;
                        if self.loaded_text.id > 0 {

                            debug!("{:?}",c);

                                                    }

                        //self.text.perform(a)
                    }
                    text_editor::Action::Select(_) | text_editor::Action::Drag(_) => {
                        self.text.perform(a);
                        self.position = self.text.cursor().position;

                        self.result_text = markdown::Content::new();
                        self.result_raw = String::new();
                        if let Some(s) = self.text.selection() 
                            && s.len() <= 15 { // is word
                            if let Some(cedict) = &self.cedict {
                                let res = cedict.find(s.trim());
                                debug!("Result {:?}", res);
                                for e in res {
                                    debug!("Entry: {:?}", e);
                                    self.result_text.push_str(e.to_md().as_str());
                                    self.result_raw.push_str( format!("{}{}", self.result_raw, e.to_string()).as_str() );
                                }
                                if let Some(anki) = self.conf.anki.clone() {
                                    return iced::Task::perform(async move {
                                        tokio::task::spawn_blocking(move || {
                                            if let Ok(anki_conn) = rusqlite::Connection::open(anki) {
                                                debug!("Anki: {}", s);
                                                crate::anki::search_anki(&anki_conn, s.as_str())
                                            } else {
                                                Err(ReaderError::Io("Error opening Anki database".to_string()))
                                            }
                                        }).await?
                                    }, |res| {
                                        match res {
                                            Ok(res) => {
                                                Message::AnkiResUpdate(res)
                                            }
                                            Err(e) => Message::ShowModal(e.to_string()),
                                        }
                                    });
                                }

                            } else {
                                warn!("Dictionary not available!");
                            }
                        }
                    }
                    _ => {
                        self.text.perform(a);
                        self.position = self.text.cursor().position;
                    }
                }

                let c = self.position;
                if self.notes.is_empty() && !self.loaded_text.is_empty() 
                    && let Ok(notes) = get_notes(&self.doc_conn, self.loaded_text.id) {
                    self.notes = notes;
                }
                if let Some(note) = self.notes.iter().find(|n| n.line == c.line && n.char == c.column ) {
                    debug!("Found note");
                    self.sidebar_notes = text_editor::Content::with_text(note.text.as_str());
                } else {
                    debug!("No note");
                    //debug!("Notes: {:?}", self.notes);
                    //self.sidebar_notes = text_editor::Content::new();
                }

            }
            Message::AnkiResUpdate(v) => {
                self.anki_result = markdown::Content::new();
                v.iter().for_each(|x| {
                    let x = x.trim();
                    let item = format!("\n- [{}](https://cedict-link-clicked/?v={})", x, x);
                    self.anki_result.push_str(item.as_str());
                });
            }
            Message::AiUrlChange(url) => {
                if let Some(c) = self.new_ai.as_mut() {
                    c.url = str_to_op(url);
                } else if let Some(c) = self.conf.get_ai_config_mut() {
                    c.url = str_to_op(url);
                }
                self.ai_changed = true;
            }
            Message::AiNameChange(name) => {
                if let Some(c) = self.new_ai.as_mut() {
                    c.name = name;
                } else if let Some(c) = self.conf.get_ai_config_mut() {
                    c.name = name;
                }
                self.ai_changed = true;
            }
            Message::AiModelChange(m) => {
                if let Some(c) = self.new_ai.as_mut() {
                    c.model = m;
                } else if let Some(c) = self.conf.get_ai_config_mut() {
                    c.model = m;
                }
                self.ai_changed = true;
            }
            Message::AiKeyChange(k) => {
                if let Some(c) = self.new_ai.as_mut() {
                    c.key = str_to_op(k);
                } else if let Some(c) = self.conf.get_ai_config_mut() {
                    c.key = str_to_op(k);
                }
                self.ai_changed = true;
            }
            Message::AiProviderChange(p) => {
                if let Some(c) = self.new_ai.as_mut() {
                    c.provider = p;
                    c.model = String::new();
                    c.url = Some(url_for_provider(&p));
                    c.key = None;
                    c.ndims = match p {
                        crate::config::Provider::Gemini => Some(768),
                        _ => None,
                    };
                    c.max_docs = match p {
                        crate::config::Provider::Gemini => Some(1024),
                        crate::config::Provider::Mistral => Some(1024),
                        crate::config::Provider::Openai => Some(1024),
                        _ => None,  
                    };
                    c.embedding_model = match p {
                        crate::config::Provider::Mistral => Some(rig::providers::mistral::MISTRAL_EMBED.to_string()),
                        crate::config::Provider::Gemini => Some(rig::providers::gemini::embedding::EMBEDDING_001.to_string()),
                        crate::config::Provider::Openai => Some(rig::providers::openai::TEXT_EMBEDDING_3_LARGE.to_string()),
                        _ => None,
                    }
                } else if let Some(c) = self.conf.get_ai_config_mut() {
                    c.provider = p;
                    c.model = String::new();
                    c.url = Some(url_for_provider(&p));
                    c.key = None;
                }
                self.ai_changed = true;
            }
            Message::AiRoleChanged(r) => {
                self.conf.ai_role = r;
                self.ai_changed = true;
            }
            Message::AiNdimsChanged(x) => {
                if let Ok(x) = x.parse::<usize>() {
                    if let Some(r) = self.new_ai.as_mut() {
                        r.ndims = Some(x);
                    } else if let Some(r) = self.conf.get_ai_config_mut() {
                        r.ndims = Some(x);
                    }
                    self.ai_changed = true;
                }
            }
            Message::AiMaxDocsChanged(x) => {
                if let Ok(x) = x.parse::<usize>() {
                    if let Some(r) = self.new_ai.as_mut() {
                        r.max_docs = Some(x);
                    } else if let Some(r) = self.conf.get_ai_config_mut() {
                        r.max_docs = Some(x);
                    }
                    self.ai_changed = true;
                }
            }

            Message::AiSettingsSave => {
                if let Some(new_chat) = self.new_ai.as_ref() {
                    let new_key = crate::utils::random_name();
                    self.conf.ai_chats.insert(new_key.clone(), new_chat.clone());
                    self.conf.ai_chat = new_key;
                }
                match toml::to_string(&self.conf) {
                    Ok(s) => {
                        return iced::Task::perform(async move {
                            tokio::fs::write(CONF, s).await
                        }, |r| {
                            match r {
                                Ok(_) => Message::Close,
                                Err(e) => Message::ShowModal(e.to_string()),
                            }
                        });
                    }
                    Err(e) => {
                        return iced::Task::done(Message::ShowModal(e.to_string()));
                    }
                }
            }
            Message::AiChatSelected(c) => {
                let key = self.conf.ai_chats.iter()
                    .find(|(_,v)| v.name == c.name)
                    .map(|(k,_)| k.clone());
                if let Some(key) = key {
                    self.conf.ai_chat = key;
                }
                self.ai_changed = true;
            }
            Message::SettingsSave => {
                #[cfg(feature = "scraper")]
                if self.sc_new {
                    if let Some(l_ex) = self.conf.l_ex.as_ref() {
                        if !l_ex.is_empty() {
                            self.conf.link_ex.push(l_ex.clone());
                        } else {
                            self.conf.l_ex = None;
                        }
                    }
                    if let Some(t_ex) = self.conf.t_ex.as_ref() {
                        if !t_ex.is_empty() {
                            self.conf.text_ex.push(t_ex.clone());
                        } else {
                            self.conf.t_ex = None;
                        }
                    }
                }
                match toml::to_string(&self.conf) {
                    Ok(s) => {
                        return iced::Task::perform(async move {
                            tokio::fs::write(CONF, s).await
                        }, |r| {
                            match r {
                                Ok(_) => Message::Close,
                                Err(e) => Message::ShowModal(e.to_string()),
                            }
                        });
                    }
                    Err(e) => {
                        return iced::Task::done(Message::ShowModal(e.to_string()));
                    }
                }
            }
            Message::NewAi => {
                let provider = crate::config::Provider::Openai;

                self.new_ai = Some(AiChatConfig { 
                    name: String::new(), 
                    key: None, 
                    url: Some(crate::utils::url_for_provider(&provider)), 
                    model: String::new(),
                    ndims: None,
                    max_docs: None,
                    embedding_model: None,
                    provider });

            }
            Message::ThemeSelected(th) => {
                self.conf.window.theme = th.to_string();
            }
            Message::Language(lang) => {
                self.conf.window.lang = Some(lang);
                
            }
            Message::Simplified => {
                if let Some(cedict) = &self.cedict {
                    let s = self.text.text();
                    self.text = text_editor::Content::with_text("");
                    let res = cedict.to_sim(s.as_str());
                    self.text.perform( text_editor::Action::Edit( text_editor::Edit::Paste( Arc::new(res.clone()) ) ) );

                    self.text_md = markdown::Content::new();
                    for line in res.lines() {
                        self.text_md.push_str(format!("{}\n",line.trim()).as_str() );
                    }
                }
            }
            Message::AiChatEvent(ev) => {
                match ev {
                    ChatEvent::ChatReady(sender) => {
                        self.sender = Some(sender);
                    }
                    ChatEvent::Message(content) => {
                        match content {
                            StreamedAssistantContent::Text(text) => {
                                debug!("Received text: {:?}", text);
                                self.answer_text.push_str(text.text.as_str());
                                self.answer_raw.push_str(text.text.as_str());
                            }
                            StreamedAssistantContent::Final(_) => {
                                debug!("Received final message from stream");
                                self.answer_text.push_str("\n");
                                self.answer_raw.push_str("\n");
                            }
                            StreamedAssistantContent::ToolCall(tc) => {
                                debug!("ToolCall: {:?}", tc);
                            }
                            StreamedAssistantContent::Reasoning(re) => {
                                debug!("Reasoning: {:?}", re);
                                self.answer_text.push_str(re.reasoning.as_str());
                                self.answer_raw.push_str(re.reasoning.as_str());
                            }
                        }
                    }
                    ChatEvent::ChatError(error) => {
                        return iced::Task::done(Message::ShowModal(error.to_string()));
                    }
                }
            }
            Message::PromptMeaning => {
                self.answer_raw = String::new();
                self.answer_text = markdown::Content::new();
                let question = self.conf.get_prompts().meaning.clone();
                return self.do_prompt(question.as_str(), false);
            }
            Message::PromptExplain => {
                self.answer_raw = String::new();
                self.answer_text = markdown::Content::new();

                let question = self.conf.get_prompts().explain.clone();
                return self.do_prompt(question.as_str(), true);
            }
            Message::PromptSummary => {
                self.answer_raw = String::new();
                self.answer_text = markdown::Content::new();

                let question = self.conf.get_prompts().summary.clone();
                return self.do_prompt(question.as_str(), true);
            }
            Message::PromptGrammar => {
                self.answer_raw = String::new();
                self.answer_text = markdown::Content::new();

                let question = self.conf.get_prompts().grammar.clone();
                return self.do_prompt(question.as_str(), false);
            }
            Message::PromptUsage => {
                self.answer_raw = String::new();
                self.answer_text = markdown::Content::new();

                let question = self.conf.get_prompts().examples.clone();
                return self.do_prompt(question.as_str(), false);
            }
            Message::DictionaryCopy => {
                debug!("Copying: {}", self.result_raw);
                return clipboard::write(self.result_raw.clone());
            }
            Message::AnswerCopy => {
            }
            Message::UpdateProgress => {
                let c = self.text.cursor().position;
                let line = c.line;
                let character = c.column;

                if let Err(e) = update_progress(&mut self.doc_conn, self.loaded_text.id as usize, character, line) {
                    error!("Error updating progress {}", e);
                    return iced::Task::done(Message::ShowModal(e.to_string()));
                }
            }
            Message::SaveNote => {
                if self.loaded_text.id > 0 {
                    let c = self.text.cursor().position;
                    debug!("Cursor position: {:?}", c);
                    let line = c.line;
                    let character = c.column;
                    let content = self.sidebar_notes.text();
                    
                    self.notes.push(Note { id: 0, doc: self.loaded_text.id, line, char: character, text: content.clone() });

                    match save_note(&mut self.doc_conn, line, character, self.loaded_text.id, content.as_str()) {
                        Err(e) => error!("Error saving note: {}", e),
                        Ok(id) => {
                            info!("Successfully saved note id {}, {}:{}", id, line, character);
                            self.note_edited = false;
                            self.sidebar_notes = Content::new();
                            self.sidebar_mode = SidebarMode::Dictionary;
                        }
                    }

                }
            }
            Message::RemoveNote => {
                if self.loaded_text.id > 0 {
                    let c = self.text.cursor().position;
                    let line = c.line;
                    let character = c.column;

                    if let Err(e) = delete_note(&mut self.doc_conn, self.loaded_text.id, line, character) {
                        error!("Error deleting note: {}", e);
                        return iced::Task::done(Message::ShowModal(format!("Error deleting note\n{}", e)));
                    } else {
                        self.note_edited = false;
                    }
                }
            }
            Message::LoadText => {
                let document = self.documents.iter()
                    .find(|s| s.title == self.loaded_text.title);
                if let Some(document) = document {
                    match get_content(&self.doc_conn, document.id) {
                        Ok(Some(cc)) => {
                            self.text = text_editor::Content::with_text(cc.as_str());
                            self.state = AppState::Default;
                            match get_notes(&self.doc_conn, document.id) {
                                Ok(notes) => {
                                    self.notes = notes;
                                    info!("Loaded {} notes", self.notes.len());
                                }
                                Err(e) => {
                                    error!("Error fetching notes: {}", e);
                                    return iced::Task::done(Message::ShowModal(format!("Error fetching note\n{}", e)));
                                }
                            }
                        }
                        Ok(None) => {
                            return iced::Task::done(Message::ShowModal(format!("Text not found!")));
                        }
                        Err(e) => {
                            error!("Error retrieving document: {}", e);
                            return iced::Task::done(Message::ShowModal(format!("Error retrieving text\n{}", e)));
                        }
                    }
                }
            }
            Message::SaveText => {
                let id = if self.new_text { 0 } else { self.loaded_text.id };
                match save_text(&mut self.doc_conn, id, self.loaded_text.title.as_str(), self.text.text().as_str()) {
                    Ok(id) => {
                        info!("Text saved successfully: {id}");
                        self.loaded_text.id = id as u32;
                        self.state = AppState::Default;
                    }
                    Err(e) => {
                        error!("Error saving text: {}", e);
                        self.new_text = false;
                        return iced::Task::done(Message::ShowModal(format!("Error saving text\n{}", e)));
                    }
                }
            }
            Message::CancelLoad => {
                self.state = AppState::Default;
                self.loaded_text = Doc::default();
                self.text = text_editor::Content::new();
                self.note_edited = false;
                self.new_text = false;
                self.sidebar_notes = text_editor::Content::new();

            }
            Message::TextSelected(doc) => {
                self.loaded_text = doc;
            }
            Message::RemoveText => {
                match delete_text(&mut self.doc_conn, self.loaded_text.title.as_str()) {
                    Ok(_) => {
                        info!("Text deleted successfully");
                        self.loaded_text = Doc::default();
                        self.text = text_editor::Content::new();
                        self.documents = vec![];
                        self.note_edited = false;
                        self.sidebar_notes = text_editor::Content::new();
                    }
                    Err(e) => {
                        error!("Error saving text: {}", e);
                        return iced::Task::done(Message::ShowModal(format!("Error saving text\n{}", e)));
                    }
                }
            }
            Message::DictionaryToNotes => {
                match self.sidebar_mode {
                    SidebarMode::Dictionary => {
                        if self.loaded_text.id > 0 {
                            if !self.dtn_append {
                                self.sidebar_notes = Content::new();
                            }
                            
                            self.sidebar_notes.perform(text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(format!("{}{}", 
                                        if self.sidebar_notes.is_empty() { "" } else { "\n" },
                                        self.result_raw)))));
                            self.sidebar_mode = SidebarMode::Notes;
                        }
                    }
                    SidebarMode::AI => {
                        self.sidebar_notes = Content::new();
                        self.sidebar_notes.perform(text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(self.answer_raw.clone()))));
                    }
                    _ => {}
                }
            }
            Message::DictionaryToNotesAppend(v) => self.dtn_append = v,
            Message::AnswerToNotes => {
                self.sidebar_notes.perform(text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new(self.answer_raw.clone()))));
                self.sidebar_mode = SidebarMode::Notes;
            }
            Message::TextTitleChange(t) => {
                self.loaded_text.title = t;
            }
            #[cfg(feature = "scraper")]
            Message::LinkExtractorChanged(le) => {
                self.conf.l_ex = Some(le);
            }
            #[cfg(feature = "scraper")]
            Message::TextExtractorChanged(te) => {
                self.conf.t_ex = Some(te);
            }
            Message::LinkClicked(e) => {
                debug!("URL: {}", e);
                let data = e.to_string();
                if data.starts_with("c:") {
                    debug!("Variant clicked");
                    
                    if let Some(cedict) = &self.cedict {
                        let q = &e[2..];
                        
                        debug!("Query: {:?}", q);
                        if !q.is_empty() {
                            let res = cedict.find(q.trim());
                            for e in res {
                                self.result_text = markdown::Content::new();
                                self.result_text.push_str(e.to_md().as_str());
                                self.result_raw.push_str( format!("{}{}", self.result_raw, e.to_string()).as_str() );
                            }
                            self.sidebar_mode = SidebarMode::Dictionary;
                        }
                    }
                } else if e.starts_with("a:") {
                    debug!("Anki");
                } else if e.starts_with("n:") {
                    debug!("Note link clicked");
                    let q = &e[2..];
                    if let Some(sep_ix) = q.find(":") {
                        let line = &q[..sep_ix];
                        let column = &q[..sep_ix];
                        debug!("clicked note link: {}:{}",line,column);
                        if let Ok(line) = line.parse::<usize>()
                            && let Ok(column) = column.parse::<usize>() {

                            self.sidebar_notes = Content::new();
                            let note = self.notes.iter().find(|n| n.line == line && n.char == column);
                            if let Some(note) = note {
                                self.sidebar_notes = Content::with_text(note.text.as_str());
                                self.sidebar_mode = SidebarMode::Notes;
                            }
                        }
                    }
                } else {
                    #[cfg(target_os = "linux")]
                    if let Err(e) = std::process::Command::new("xdg-open").arg(e.to_string()).output() {
                        error!("Error handlink click event: {e}");
                    }
                    
                    #[cfg(target_os = "windows")]
                    utils::open_link(e.to_string().as_str());
                    return iced::clipboard::write(e.to_string());
                }
            }
            Message::NotesCopy(note) => {
                return iced::clipboard::write(note);
            }
            Message::NotesAction(a) => {
                if self.loaded_text.id > 0 {
                    debug!("Notes action");
                    self.sidebar_notes.perform(a);
                    self.note_edited = true;
                }
            }
            Message::NotesDelete { document, line, character } => {
                if let Err(e) = delete_note(&mut self.doc_conn, document, line, character) {
                    return iced::Task::done(Message::ShowModal(e.to_string()));
                }
                return iced::Task::done(Message::Notes);
            }
            Message::DeeplKeyChange(key) => {
                self.conf.keys.deepl = key;
            }
            Message::Deepl => {
                let text = self.text.selection();
                let lang = self.conf.window.lang();
                let key = self.conf.keys.deepl.clone();
                if let Some(text) = text {
                    return iced::Task::perform(async move {
                        let r = ask_deepl(text.as_str(), lang, key.as_str()).await;
                        r.map(|r| format!("{} - {}", text, r))
                    }, |r| {
                        match r {
                            Ok(r) => Message::SetResult(r),
                            Err(e) => Message::ShowModal(e.to_string()),
                        }
                    });
                }
            }
            Message::NewTextCheck(n) => {
                self.new_text = n;
            }
            Message::AnkiChanged(s) => {
                self.conf.anki = Some(s);
            }
            Message::NewText => {
                self.text = Content::new();
                self.loaded_text = crate::textbase::Document::default();
                self.state = AppState::Default;
            }
            Message::AppDataChanged(a) => {
                self.conf.db = Some(a);
            }
            #[cfg(feature = "scraper")]
            Message::Scraper => {
                self.state = AppState::Scraper;
            }
            #[cfg(feature = "scraper")]
            Message::StartScraper => {
                let name = self.scraper_file.clone();

                if self.conf.l_ex.is_none() || self.conf.t_ex.is_none() {
                    return iced::Task::none();
                }

                let l_ext = self.conf.l_ex.clone().unwrap();
                let t_ext = self.conf.t_ex.clone().unwrap();
                let interval = self.conf.scraper_sleep.unwrap_or(500);
                                
                if let Some(sender) = &self.sc_sender {
                    let sender = sender.clone();
                    return iced::Task::perform(async move {
                        sender.send(ScraperCommand::Start { name, interval, l_ext, t_ext }).await
                        }, |r| {
                        if let Err(e) = r {
                            Message::ShowModal(e.to_string())
                        } else {
                            Message::Void
                        }
                    });
                }
            }
            #[cfg(feature = "scraper")]
            Message::LX(nc) => {
                match self.conf.l_ex.as_ref() {
                    Some(LinkExtractorType::PatternExtractor { pattern, n_chapters: _, name }) => {
                        if let Ok(nc) = nc.parse::<usize>() {
                            self.conf.l_ex.replace(LinkExtractorType::PatternExtractor { pattern: pattern.clone(), n_chapters: nc , name: name.clone() });
                        }
                    }
                    _ => {
                        debug!("NChapters::None");
                    }
                }
            }
            #[cfg(feature = "scraper")]
            Message::LN(nm) => {
                match self.conf.l_ex.as_ref() {
                    Some(LinkExtractorType::PatternExtractor { pattern, n_chapters, name: _ }) => {
                        self.conf.l_ex.replace(LinkExtractorType::PatternExtractor { pattern: pattern.clone() , name: nm, n_chapters: *n_chapters });
                    }
                    Some(LinkExtractorType::MainPageExtractor { url, pattern, name: _ }) => {
                        self.conf.l_ex.replace(LinkExtractorType::MainPageExtractor { url: url.clone(), pattern: pattern.clone(), name: nm });
                    }
                    _ => debug!("Pattern::None"),
                }
            }
            #[cfg(feature = "scraper")]
            Message::LU(u) => {
                match self.conf.l_ex.as_ref() {
                    Some(LinkExtractorType::MainPageExtractor { url: _, pattern, name }) => {
                        self.conf.l_ex.replace(LinkExtractorType::MainPageExtractor { pattern: pattern.clone() , name: name.clone(), url: u.clone() });
                    }
                    _ => debug!("LU::None"),
                }

            }
            #[cfg(feature = "scraper")]
            Message::LP(pt) => {
                match self.conf.l_ex.as_ref() {
                    Some(LinkExtractorType::PatternExtractor { pattern: _, n_chapters, name }) => {
                        self.conf.l_ex.replace(LinkExtractorType::PatternExtractor { pattern: pt, name: name.clone(), n_chapters: *n_chapters });
                    }
                    Some(LinkExtractorType::MainPageExtractor { url, pattern: _, name }) => {
                        self.conf.l_ex.replace(LinkExtractorType::MainPageExtractor { url: url.clone(), pattern: pt, name: name.clone() });
                    }
                    _ => debug!("Pattern::None"),
                }
            }
            #[cfg(feature = "scraper")]
            Message::TP(t) => {
                match self.conf.t_ex.as_ref() {
                    Some(TextExtractorType::PatternTextExtractor { title_pattern, pattern: _, name }) => {
                        self.conf.t_ex.replace(TextExtractorType::PatternTextExtractor { title_pattern: title_pattern.clone(), pattern: t, name: name.clone() });
                    }
                    _ => debug!("TP::None"),
                }
            }
            #[cfg(feature = "scraper")]
            Message::TN(t) => {
                match self.conf.t_ex.as_ref() {
                    Some(TextExtractorType::PatternTextExtractor { title_pattern, pattern, name: _ }) => {
                        self.conf.t_ex.replace(TextExtractorType::PatternTextExtractor { title_pattern: title_pattern.clone(), pattern: pattern.clone(), name: t });
                    }
                    _ => debug!("TP::None"),
                }
            }
            #[cfg(feature = "scraper")]
            Message::TT(t) => {
                match self.conf.t_ex.as_ref() {
                    Some(TextExtractorType::PatternTextExtractor { title_pattern: _, pattern, name }) => {
                        if t.is_empty() {
                            self.conf.t_ex.replace(TextExtractorType::PatternTextExtractor { title_pattern: None, pattern: pattern.clone(), name: name.clone() });
                        } else {
                            self.conf.t_ex.replace(TextExtractorType::PatternTextExtractor { title_pattern: Some(t), pattern: pattern.clone(), name: name.clone() });
                        }
                    }
                    _ => debug!("TT::None"),
                }
            }
            #[cfg(feature = "scraper")]
            Message::ScraperSleepChanged(iv) => {
                let iv = iv.parse::<u64>().unwrap_or(500);
                self.conf.scraper_sleep = Some(iv);
                if let Some(sender) = &self.sc_sender {
                    let sender = sender.clone();
                    return iced::Task::perform(async move {
                        sender.send(ScraperCommand::AdjustInterval(iv)).await
                    }, |r| {
                        if let Err(e) = r {
                            Message::ShowModal(e.to_string())
                        } else {
                            Message::Void
                        }
                    });
                }
            }
            #[cfg(feature = "scraper")]
            Message::StopScraper => {
                if let Some(sender) = &self.sc_sender {
                    let sender = sender.clone();
                    self.scraper_progress = 0.0;
                    return iced::Task::perform(async move {
                        sender.send(ScraperCome5femand::Stop).await
                    }, |r| {
                        if let Err(e) = r {
                            Message::ShowModal(e.to_string())
                        } else {
                            Message::Void
                        }
                    });
                }
            }
            #[cfg(feature = "scraper")]
            Message::ScraperEvent(p) => {
                match p {
                    ScraperEvent::Progress(pr) => {
                        self.scraper_progress = pr.0;
                        self.scraper_to = pr.1;
                    }
                    ScraperEvent::Error(er) => {
                        self.scraper_progress = 0.0;
                        return iced::Task::done(Message::ShowModal(er));
                    }
                    ScraperEvent::Finished => {
                        self.scraper_progress = 0.0;
                    }
                    ScraperEvent::Ready(sender) => {
                        self.sc_sender = Some(sender);
                    }
                }
            }
            #[cfg(feature = "scraper")]
            Message::ScNew => {
                self.sc_new = true;
                self.conf.l_ex = Some(LinkExtractorType::PatternExtractor { pattern: String::new(), n_chapters: 0, name: String::new() });
                self.conf.t_ex = Some(TextExtractorType::PatternTextExtractor { title_pattern: None, pattern: "p".to_string(), name: String::new() });
            }
            #[cfg(feature = "scraper")]
            Message::ScraperUrlChanged(u) => {
                self.scraper_url = u;
            }
            #[cfg(feature = "scraper")]
            Message::ScraperFileChanged(r) => {
                self.scraper_file = r;
            }
            #[cfg(feature = "scraper")]
            Message::ScraperSingle(ss) => {
                self.scraper_single = ss;
            }
            #[cfg(feature = "scraper")]
            Message::ScraperCurrentFile => {
                self.scraper_file = self.conf.db.clone().unwrap_or_default();
            }
            Message::TextMode(tm) => {
                if tm == TextMode::Md {
                    let text = self.text.text();
                    self.text_md = markdown::Content::new();
                    for line in text.lines() {
                        self.text_md.push_str(line);
                        self.text_md.push_str("\n");
                    }
                }
                self.text_mode = tm;
            }
            Message::Notes => {
                if let Ok(notes) = get_notes(&self.doc_conn, self.loaded_text.id) {
                    self.notes = notes;
                    self.state = AppState::Notes;
                }
            }
            Message::DbChange => {
                let file_name = rfd::FileDialog::new()
                    
                    .pick_file();
                
                if let Some(file_name) = &file_name {
                    match crate::scraper::db::init_db(file_name) {
                        Ok(doc_conn) => {
                            debug!("Setting new file: {:?}", file_name);
                            self.conf.db = file_name.to_str().map(|r| r.to_string());
                            self.doc_conn = doc_conn;
                            self.temp_document = None;
                            self.text = Content::new();
                            self.text_md = markdown::Content::new();
                        }
                        Err(e) => {
                            error!("Error opening file: {}", e);
                            return iced::Task::done(Message::ShowModal(e.to_string()));
                        }
                    }
                }

            }
            Message::NotesExport => {
                if let Ok(result) = export_notes(&self.doc_conn, self.loaded_text.id)
                    && let Some(f) = rfd::FileDialog::new().save_file() {
                    use tokio::io::AsyncWriteExt; 
                    return iced::Task::perform(async move {
                        let mut file = tokio::fs::File::create(f).await;
                        if let Ok(file) = file.as_mut() {
                            file.write_all(result.as_bytes()).await
                                .map_err(|e| ReaderError::Io(e.to_string()) )
                        } else {
                            Err(ReaderError::Io(file.unwrap_err().to_string()))
                        }
                    }, |res| {
                        match res {
                            Ok(_) => { Message::Void }
                            Err(e) => {
                                error!("{}", e);
                                Message::ShowModal(e.to_string())
                            }
                        }
                    });
                }
            }
            Message::Void => {}
            _ => {},
        }
        iced::Task::none()
    }

    fn do_prompt(&mut self, question: &str, with_text: bool) -> iced::Task<Message> {
        if let Some(selection) = self.text.selection() {
            let question = question.replace("{}", selection.as_str());
            let prompt = rig::message::Text::from(question.as_str());
            let line = self.text.cursor().position.line;
            let text_line = self.text.text();

            let content = if with_text {
                let text = Document {
                    data: text_line.lines().nth(line).unwrap_or_default().to_string(),
                    format: Some(ContentFormat::String),
                    media_type: Some(DocumentMediaType::TXT),
                };
                OneOrMany::many(vec![UserContent::Text(prompt), UserContent::Document(text)])
            } else {
                Ok( OneOrMany::one(UserContent::Text(prompt) ) )
            };

            match  content {
                Ok(content) => {
                    let message = Rmsg::User { content };

                    let conf = self.conf.get_ai_config();
                    if conf.is_none() {
                        return iced::Task::done(Message::ShowModal(String::from("No config found")));
                    }
                    let chat_history = self.chat_history.clone();
                    if let Some(sender) = &self.sender {
                        let sender = sender.clone();
                        return iced::Task::perform(async move {
                            let message = message.clone();
                            sender.send(ChatCommand::Request { message , chat_history: chat_history }).await
                        }, |r| {
                            match r {
                                Ok(_) => Message::Void,
                                Err(e) => Message::ShowModal(e.to_string()),
                            }
                        });
                    } else {
                        iced::Task::none()
                    }
                }
                Err(e) => {
                    iced::Task::done(Message::ShowModal(e.to_string()))
                }
            }
        } else {
            iced::Task::none()
        }
    }
}



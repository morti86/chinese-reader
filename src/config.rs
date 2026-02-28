use serde::{Serialize, Deserialize};
use tracing::debug;
use std::collections::BTreeMap;
use std::fmt;
#[cfg(feature = "scraper")]
use crate::scraper::{LinkExtractorType, TextExtractorType};

use crate::utils::{Level, random_name};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Keys {
    pub elevenlabs: String,
    pub deepl: String,
}

crate::make_enum!(Language, [English, Polski, Deutsch, Русский, Française, Italiano, Español, Türkçe, 日本語]);

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Window {
    pub width: f32,
    pub height: f32,
    pub split: f32,
    pub font_size: Option<f32>,
    pub lang: Option<Language>,
    pub theme: String,
    pub settings_label_w: f32,
    pub spacing: f32,
    pub padding: f32,
    pub padding_frame: f32,
}

impl Window {
    pub fn lang(&self) -> Language {
        self.lang.unwrap_or(Language::English)
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::ALL.iter().find(|t|
            t.to_string() == self.theme)
            .cloned()
            .unwrap_or(iced::Theme::Light)
    }
}

crate::make_enum!(Provider, [Openai, Deepseek, Ollama, Gemini, Xai, Anthropic, Mistral]);

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AiChatConfig {
    pub name: String,
    pub key: Option<String>,
    pub url: Option<String>,
    pub model: String,
    pub provider: Provider,
    pub ndims: Option<usize>,
    pub max_docs: Option<usize>,
    pub embedding_model: Option<String>,
}

impl fmt::Display for AiChatConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Labels {
    pub ocr: String,
    pub load_file: String,
    pub clipboard: String,
    pub paste_here: String,
    pub load: String,
    pub save: String,
    pub ai_select: String,
    pub ask_ai: String,
    pub name: String,
    pub url: String,
    pub provider: String,
    pub model: String,
    pub notes_placeholder: String,
    pub theme: String,
    pub cancel: String,
    pub language: String,
    pub ok: String,
    pub mode: String,
    pub settings_ai: String,
    pub settings: String,
    pub new: String,
    pub add_note: String,
    pub delete_note: String,
    pub image_status: String,
    pub none: String,
    pub enter_model: String,
    pub ai_key: String,
    pub ai_role: String,
    pub load_text: String,
    pub select_text: String,
    pub save_text: String,
    pub text_name: String,
    pub delete_text: String,
    pub font_size: String,
    pub ai_preamble: String,
    pub to_notes: String,
    pub copy: String,
    pub add_text: String,
    pub ocr_file: String,
    pub deepl_key: String,
    pub eleven_key: String,
    pub simplified: String,
    pub restart_needed: String,
    pub cursor: String,
    pub deepl: String,
    pub ai_meaning: String,
    pub ai_examples: String,
    pub ai_explain: String,
    pub ai_grammar: String,
    pub load_img: String,
    pub clear_img: String,
    pub save_prog: String,
    pub ndims: String,
    pub max_docs: String,
    pub emb_mod: String,
    pub appdata: String,
    pub summary: String,
    pub scraper: String,
    pub scraper_d: String,
    pub sc_file: String,
    pub sc_c_min: String,
    pub sc_c_max: String,
    pub sc_sleep: String,
    pub start: String,
    pub sc_url: String,
    pub stop: String,
    pub sc_single: String,
    pub sc_current: String,
    pub e_pattern: String,
    pub e_chapters: String,
    pub e_type: String,
    pub e_tp: String,
    pub e_lt: String,
    pub no_text: String,
    pub line: String,
    pub column: String,
    pub text: String,
    pub sc_warn: String,
    pub p_meaning: String,
    pub p_explain: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Prompts {
    pub meaning: String,
    pub examples: String,
    pub explain: String,
    pub grammar: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub ai_chats: BTreeMap<String, AiChatConfig>,
    pub labels: BTreeMap<String, Labels>,
    pub prompts: BTreeMap<String, Prompts>,

    pub db: Option<String>,

    pub window: Window,
    pub keys: Keys,
    pub level: Level,

    pub anki: Option<String>,
    pub ocr_models: String,
    pub ai_chat: String,
    pub ai_role: String,
    pub ai_preamble: String,

    pub new_ai: Option<AiChatConfig>,
    pub scraper_sleep: Option<u64>,
    #[cfg(feature = "scraper")]
    pub link_ex: Vec<LinkExtractorType>,
    #[cfg(feature = "scraper")]
    pub text_ex: Vec<TextExtractorType>,
    #[cfg(feature = "scraper")]
    pub l_ex: Option<LinkExtractorType>,
    #[cfg(feature = "scraper")]
    pub t_ex: Option<TextExtractorType>,
}

impl Config {
    pub fn get_labels(&self) -> &Labels {
        self.labels.get(self.window.lang.unwrap_or(Language::English).as_str()).unwrap()
    }

    pub fn get_prompts(&self) -> &Prompts {
        self.prompts.get(self.window.lang.unwrap_or(Language::English).as_str()).unwrap()
    }

    pub fn get_ai_config(&self) -> Option<&AiChatConfig> {
        self.ai_chats.get(self.ai_chat.as_str())
    }

    pub fn get_ai_config_mut(&mut self) -> Option<&mut AiChatConfig> {
        self.ai_chats.get_mut(self.ai_chat.as_str())
    }
    
    pub fn get_ai_chats(&self) -> Vec<AiChatConfig> {
        self.ai_chats.iter()
            .map(|(_,c)| c.clone())
            .collect()
    }

    pub fn mod_ai_chat(&mut self, key: &str, a: AiChatConfig) {
        self.ai_chats.entry(key.to_string()).and_modify(|e| *e = a);
    }

    pub fn add_ai_chat(&mut self, a: AiChatConfig) {
        let name = random_name();
        self.ai_chats.entry(name).or_insert(a);
    }

    pub fn has_models(&self) -> bool {
        if let Some(c) = self.get_ai_config() {
            let m = crate::utils::get_models(&c.provider);
            debug!("{:?}", m);
            !m.is_empty()
        } else {
            false
        }
    }

    pub fn get_models(&self) -> Vec<String> {
        let provider = self.get_ai_config()
            .map(|c| c.provider);
        match provider {
            None => vec![],
            Some(provider) => {
                crate::utils::get_models(&provider)
            }
        }
    }

    pub fn ocr_models_found(&self) -> bool {
        Path::new(&format!("{}", self.ocr_models)).exists()
    }

    pub fn is_new(&self) -> bool {
        self.new_ai.is_some()
    }
}

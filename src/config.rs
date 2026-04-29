#![allow(non_camel_case_types)]

use serde::{Serialize, Deserialize};
use tracing::debug;
use std::{collections::BTreeMap, path::PathBuf};
use std::fmt::{self, Display};
use crate::make_enum;

use crate::utils::{Level, random_name, url_for_provider};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Keys {
    pub elevenlabs: String,
    pub deepl: String,
}

//crate::make_enum!(Language, [English, Polski, Deutsch, Русский, Française, Italiano, Español, Türkçe, 日本語]);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Window {
    pub width: f32,
    pub height: f32,
    pub split: f32,
    pub font_size: Option<f32>,
    pub lang: Option<String>,
    pub theme: String,
    pub settings_label_w: f32,
    pub spacing: f32,
    pub padding: f32,
    pub padding_frame: f32,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            width: 1400.0,
            height: 800.0,
            split: 0.65,
            font_size: Some(18.0),
            lang: Some(String::from("English")),
            theme: "Catppuccin Mocha".to_string(),
            settings_label_w: 200.0,
            spacing: 2.0,
            padding: 5.0,
            padding_frame: 20.0,
        }
    }
}

impl Window {
    pub fn lang(&self) -> String {
        self.lang.clone().unwrap_or(String::from("English"))
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::ALL.iter().find(|t|
            t.to_string() == self.theme)
            .cloned()
            .unwrap_or(iced::Theme::Light)
    }
}

crate::make_enum!(Provider, [Openai, Deepseek, Ollama, Gemini, Xai, Anthropic, Mistral, LlamaCpp]);

impl Provider {
    pub fn is_local(&self) -> bool {
        matches!(self, Provider::Ollama | Provider::LlamaCpp)
    }

    pub fn is_llama(&self) -> bool {
        matches!(self, Provider::LlamaCpp)
    }
}

make_enum!(CacheType, [f32, f16, bf16, q8_0, q4_0, q4_1, iq4_nl, q5_0, q5_1]);

/// llama.cpp type: local managed by reader, remote or no llama
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum LlamaType {
    Local { 
        cache_type_k: Option<CacheType>, 
        cache_type_v: Option<CacheType>, 
        flash_attn: Option<bool>, 
        ctx_size: Option<u32>, 
        n_cpu_moe: Option<u16>, 
        reasoning_budget: Option<u32>, 
        port: Option<u16>, 
        presence_penalty: Option<String>,
        mmproj: Option<String>,
    },
    Remote,
    #[default]
    None,
}

impl Display for LlamaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlamaType::Local { .. } => write!(f, "Local"),
            LlamaType::Remote => write!(f, "Remote"),
            LlamaType::None => write!(f, "None"),
        }
    }
}

impl LlamaType {
    pub const ALL: [LlamaType; 3] = [
        LlamaType::Local { 
            cache_type_k: None, 
            cache_type_v: None, 
            flash_attn: None, 
            ctx_size: Some(8192),
            n_cpu_moe: None, 
            reasoning_budget: None, 
            port: Some(8080), 
            presence_penalty: None,
            mmproj: None,
        },
        LlamaType::Remote,
        LlamaType::None,
    ];

    pub fn is_local(&self) -> bool {
        match self {
            LlamaType::Local { .. } => true,
            _ => false,
        }
    }

    pub fn is_vision(&self) -> bool {
        match self {
            LlamaType::Local { mmproj, .. } => mmproj.is_some(),
            _ => false,
        }
    }
}

impl From<String> for LlamaType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "local" => LlamaType::Local { 
                cache_type_k: None, 
                cache_type_v: None, 
                flash_attn: None, 
                ctx_size: Some(8192), 
                n_cpu_moe: None, 
                reasoning_budget: None, 
                port: Some(8080), 
                presence_penalty: None,
                mmproj: None,
            },
            "remote" => LlamaType::Remote,
            _ => LlamaType::None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AiChatConfig {
    pub name: String,
    pub key: Option<String>,
    pub url: Option<String>,
    pub model: String,
    pub provider: Provider,
    pub temperature: Option<f64>,
    pub llama_type: Option<LlamaType>,
}

impl fmt::Display for AiChatConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.name)
    }
}

impl AiChatConfig {
    pub fn is_llama_local(&self) -> bool {
        self.llama_type.as_ref().is_some_and(|t| t.is_local())
    }

    pub fn is_local_llama_vision(&self) -> bool {
        self.llama_type.as_ref().is_some_and(|x| x.is_vision())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Prompts {
    pub meaning: String,
    pub examples: String,
    pub explain: String,
    pub grammar: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub ai_chats: BTreeMap<String, AiChatConfig>,

    pub db: Option<String>,

    pub window: Window,
    pub keys: Keys,
    pub level: Level,

    pub anki: Option<String>,
    pub ocr_models: String,
    pub ocr_det_url: Option<String>,
    pub ocr_dict_url: Option<String>,
    pub ocr_rec_url: Option<String>,
    pub ai_chat: String,
    pub ai_role: String,
    pub ai_preamble: String,
    pub deepl_lang: Option<deepl::Lang>,

    pub new_ai: Option<AiChatConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let ocr_models = 
            if let Some(mut config_dir) = dirs::config_dir() {
                config_dir.push(crate::utils::APP_NAME);
                config_dir.push("models");
                config_dir
            } else {
                PathBuf::from("app.toml")
            }.into_os_string().into_string().unwrap();
        let mut ai_chats = BTreeMap::new();
        ai_chats.insert(String::from("gpt"), 
            AiChatConfig { 
                name: String::from("ChatGPT"), 
                provider: Provider::Openai, 
                model: String::from("gpt-5"),
                url: Some(url_for_provider(&Provider::Openai)),
                ..Default::default()
            });
        Self { 
            ai_chats, 
            db: Some(String::from("appdata.db")),
            window: Window::default(), 
            keys: Keys::default(), 
            level: Level::A1, 
            anki: None, 
            ocr_models, 
            ocr_det_url: Some( String::from("https://huggingface.co/monkt/paddleocr-onnx/blob/main/detection/v5/det.onnx") ),
            ocr_rec_url: Some( String::from("https://huggingface.co/monkt/paddleocr-onnx/blob/main/languages/chinese/rec.onnx") ),
            ocr_dict_url: Some( String::from("https://huggingface.co/monkt/paddleocr-onnx/blob/main/languages/chinese/dict.txt") ),
            ai_chat: String::from("gpt"), 
            ai_role: String::new(), 
            ai_preamble: String::new(), 
            new_ai: None,
            deepl_lang: None,
        }
    }
}

impl Config {
    pub fn get_ai_config(&self) -> Option<&AiChatConfig> {
        self.ai_chats.get(self.ai_chat.as_str())
    }

    pub fn get_ai_config_mut(&mut self) -> Option<&mut AiChatConfig> {
        self.ai_chats.get_mut(self.ai_chat.as_str())
    }
    
    pub fn get_ai_chats(&self) -> Vec<AiChatConfig> {
        self.ai_chats.values()
            .map(|c| c.clone())
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

    pub fn is_local_llama(&self) -> bool {
        self.get_ai_config()
            .is_some_and(|c| c.provider == Provider::LlamaCpp && c.llama_type.as_ref().is_some_and(|d| d.is_local()))
    }

    pub fn is_new(&self) -> bool {
        self.new_ai.is_some()
    }
}

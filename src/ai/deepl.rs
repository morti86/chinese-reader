use deepl::{DeepLApi, Lang};

use crate::error::ReaderResult;

pub const ALL_LANGS: &[Lang] = &[ 
    Lang::AR,    Lang::BG,    Lang::CS,  Lang::DA,    Lang::DE,
    Lang::EL,    Lang::EN,    Lang::EN_GB,    Lang::EN_US,    Lang::ES,
    Lang::ES_419,    Lang::ET,    Lang::FI,    Lang::FR,    Lang::HE,
    Lang::HU,    Lang::ID,    Lang::IT,    Lang::JA,    Lang::KO,
    Lang::LT,    Lang::LV,    Lang::NB,    Lang::NL,    Lang::PL,
    Lang::PT,    Lang::PT_BR,    Lang::PT_PT,    Lang::RO,    Lang::RU,
    Lang::SK,    Lang::SL,    Lang::SV,    Lang::TH,    Lang::TR,
    Lang::UK,    Lang::VI,    Lang::ZH,    Lang::ZH_HANS,    Lang::ZH_HANT
];

pub async fn ask_deepl(text: &str, lang: Lang, key: &str) -> ReaderResult<String> {
    tracing::debug!("asking deepl");
    let api = DeepLApi::with(key).new();
    let res = api.translate_text(text, lang).await?;
    Ok(res.to_string())
}

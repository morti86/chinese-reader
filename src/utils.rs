use tracing::debug;
#[cfg(target_family="unix")]
use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
#[cfg(target_family="windows")]
use clipboard_win::{formats, get_clipboard};
use std::io::Read;
use rand::prelude::*;
use crate::{config::Provider, error::ReaderResult};
use tendril::StrTendril;

#[macro_export]
macro_rules! make_enum {
    ($name:ident, [$op1:ident]) => {
        #[derive(Clone, Debug, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $op1,
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$op1
            }
        }

        impl $name {
            // Fixed array with commas
            pub const ALL: &'static [Self] = &[$name::$op1];

            pub fn to_string(&self) -> String {
                match self {
                    $name::$op1 => stringify!($op1).to_string(),
                }
            }

            pub fn as_str(&self) -> &str {
                match self {
                    $name::$op1 => stringify!($op1),
                }
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                let s = s.as_str();
                match s {
                    _ => $name::$op1,
                }

            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(self.to_string().as_str())
            }
        }


    };

    ($name:ident, [$op1:ident, $($opt:ident),*]) => {
        #[derive(Clone, Debug, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $op1,
            $(
                $opt,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$op1
            }
        }

        impl $name {
            // Fixed array with commas
            pub const ALL: &'static [Self] = &[$name::$op1, $($name::$opt),+];

            pub fn to_string(&self) -> String {
                match self {
                    $name::$op1 => stringify!($op1).to_string(),
                    $(
                        $name::$opt => stringify!($opt).to_string(),
                    )*
                }
            }

            pub fn as_str(&self) -> &str {
                match self {
                    $name::$op1 => stringify!($op1),
                    $(
                        $name::$opt => stringify!($opt),
                    )*
                }
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                let s = s.as_str();
                match s {
                    stringify!($op1) => $name::$op1,
                    $(
                        stringify!($opt) => $name::$opt,
                    )*
                        _ => $name::$op1,
                }

            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(self.to_string().as_str())
            }
        }
    };
}

// Get image from clipboard
#[cfg(target_family="windows")]
pub fn get_image() -> Vec<u8> {
    if let Ok(x) = get_clipboard(formats::Bitmap) {
        return x
    }
    vec![]
}

#[cfg(target_family="unix")]
pub fn get_image() -> Vec<u8> {
    let c = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Specific("image/png"));
    let mut content = vec![];
    if let Ok((mut pipe, _)) = c {
        pipe.read_to_end(&mut content).unwrap();
        return content
    }
    vec![]
}

pub fn get_text() -> ReaderResult<String> {
    let c = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text);
    let mut content = vec![];
    if let Ok((mut pipe, _)) = c {
        pipe.read_to_end(&mut content).unwrap();
        return Ok(String::from_utf8(content)?)
    }
    Ok(String::new())
}

pub fn random_name() -> String {
    let mut rng = rand::rng();
    const LENGTH: usize = 8; // You can adjust this length as needed

    (0..LENGTH)
        .map(|_| rng.sample(rand::distr::Alphanumeric) as char)
        .collect::<String>()

}

pub fn get_models(p: &Provider) -> Vec<String> {
    //
    match p {
        Provider::Xai => vec![
            "grok-beta",
            "grok-4"
        ],
        Provider::Openai => vec![
            "gpt-5.2",
            "gpt-5",
            "gpt-5-mini",
            "gpt-5-nano",
            "gpt-4.1",
            "pt-4.1-nano"
        ],
        Provider::Deepseek => vec!["deepseek-chat"],
        Provider::Gemini => vec![
            "gemini-3-pro-preview",
            "gemini-3-flash-preview",
            "gemini-2.5-flash",
            "gemini-2.5-pro",
            "gemini-2.0-flash",
            "gemini-2.0-flash-lite",
            "gemini-1.5-flash",
            "gemini-1.5-pro",
            "gemini-1.5-pro-8b",
            "gemini-1.0-pro"
        ],
        Provider::Anthropic => vec![
            "claude-opus-4-6",
            "claude-sonnet-4-5",
            "claude-haiku-4-5",
        ],
        Provider::Mistral => vec![
            "mistral-large-latest",
            "mistral-large-latest",
            "ministral-3b-latest",
            "ministral-8b-latest",
            "mistral-small-latest",
            "open-mistral-nemo",
        ],
        _ => vec![],
    }.iter().map(|x| x.to_string()).collect()
}

pub fn url_for_provider(p: &Provider) -> String {
    match p {
        Provider::Xai => String::from("https://api.x.ai"),
        Provider::Deepseek => String::from("https://api.deepseek.com"),
        Provider::Openai => String::from("https://api.openai.com/v1"),
        Provider::Gemini => String::from("https://generativelanguage.googleapis.com"),
        Provider::Ollama => String::from("http://localhost:11434"),
        Provider::Anthropic => String::from("https://api.anthropic.com"),
        Provider::Mistral => String::from("https://api.mistral.ai"),
    }
}

make_enum!(Level, [A1,A2,B1,B2,C1,C2]);

pub fn str_to_op(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(target_os = "windows")]
pub fn open_link(url: &str) {
    extern crate shell32;
    extern crate winapi;

    use std::ffi::CString;
    use std::ptr;

    unsafe {
        shell32::ShellExecuteA(ptr::null_mut(),
                               CString::new("open").unwrap().as_ptr(),
                               CString::new(url.replace("\n", "%0A")).unwrap().as_ptr(),
                               ptr::null(),
                               ptr::null(),
                               winapi::SW_SHOWNORMAL);
    }
}

pub fn is_chinese_char(c: &char) -> bool {
    // Basic CJK Unified Ideographs block
    ('\u{4e00}'..='\u{9fff}').contains(c) ||
    // CJK Unified Ideographs Extension A
    ('\u{3400}'..='\u{4dbf}').contains(c) ||
    // CJK Unified Ideographs Extension B
    ('\u{20000}'..='\u{2a6df}').contains(c) ||
    // CJK Unified Ideographs Extension C,D,E,F,I
    ('\u{2a700}'..='\u{2ee5f}').contains(c) ||
    // CJK Compatibility Ideographs
    ('\u{f900}'..='\u{faff}').contains(c)
}

pub fn extract_variant(mea: &str) -> String {
    debug!("ev::mea: {}", mea);
    // Look for patterns like "variant of 叱吒" and extract the Chinese word
    if let Some(pos) = mea.find("variant of ") {
        let start = pos + 11;
        let remaining = &mea[start..];
        let end = remaining.find("[");

        debug!("pos: {}, start: {}, end: {:?}, len: {}", pos, start, end, mea.len());

        if let Some(end) = end {
            return remaining[..end].to_string();
        }

        // In case some weird shit happens, like no normal [pinyin] after character
        // Extract Chinese characters until we hit a non-Chinese character
        let mut result = String::new();
        for c in remaining.chars() {
            if is_chinese_char(&c) {
                result.push(c);
            } else {
                break;
            }
        }
        return result;
    }

    String::new()
}

/// Zero-copy version of extract_variant using Tendril
/// Returns a StrTendril that references the original string data without copying
pub fn extract_variant_zero_copy(mea: &str) -> StrTendril {
    debug!("ev::mea (zero-copy): {}", mea);

    // Look for patterns like "variant of 叱吒" and extract the Chinese word
    if let Some(pos) = mea.find("variant of ") {
        let start = pos + 11;
        let remaining = &mea[start..];

        // Find the end position by scanning for non-Chinese characters
        let mut end_pos = 0;
        for (i, c) in remaining.char_indices() {
            if !is_chinese_char(&c) {
                end_pos = i;
                break;
            }
        }

        // If we found Chinese characters, create a tendril slice
        if end_pos > 0 {
            return StrTendril::from_slice(&remaining[..end_pos]);
        } else if !remaining.is_empty() && is_chinese_char(&remaining.chars().next().unwrap()) {
            // If the entire remaining string is Chinese characters
            return StrTendril::from_slice(remaining);
        }
    }

    StrTendril::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_url() {
        assert!(is_chinese_char(&'中'));
        assert!(!is_chinese_char(&'ź'));
    }

    #[test]
    fn test_extract_variant() {
        // Test basic extraction
        assert_eq!(extract_variant("variant of 叱吒"), "叱吒");
        assert_eq!(extract_variant("variant of 你好"), "你好");
        assert_eq!(extract_variant("variant of 世界"), "世界");

        // Test extraction with trailing text
        assert_eq!(extract_variant("variant of 叱吒 and some more text"), "叱吒");
        assert_eq!(extract_variant("variant of 你好, variant of 世界"), "你好");

        // Test cases where no variant is found
        assert_eq!(extract_variant("some other text"), "");
        assert_eq!(extract_variant("no variant here"), "");
        assert_eq!(extract_variant(""), "");
        assert_eq!(extract_variant("variant of "), "");
    }

    #[test]
    fn test_extract_variant_zero_copy() {
        // Test basic extraction
        assert_eq!(extract_variant_zero_copy("variant of 叱吒").as_ref(), "叱吒");
        assert_eq!(extract_variant_zero_copy("variant of 你好").as_ref(), "你好");
        assert_eq!(extract_variant_zero_copy("variant of 世界").as_ref(), "世界");

        // Test extraction with trailing text
        assert_eq!(extract_variant_zero_copy("variant of 叱吒 and some more text").as_ref(), "叱吒");
        assert_eq!(extract_variant_zero_copy("variant of 你好, variant of 世界").as_ref(), "你好");

        // Test cases where no variant is found
        assert_eq!(extract_variant_zero_copy("some other text").as_ref(), "");
        assert_eq!(extract_variant_zero_copy("no variant here").as_ref(), "");
        assert_eq!(extract_variant_zero_copy("").as_ref(), "");
        assert_eq!(extract_variant_zero_copy("variant of ").as_ref(), "");
    }
}

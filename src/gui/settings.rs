use iced::advanced::text::highlighter::PlainText;
use iced::widget::{Column, Row, TextEditor, button, checkbox, column, container, markdown, pick_list, row, scrollable, slider, space, text, table, text_editor, text_input, tooltip};
use iced::{Alignment, Element, Font, Renderer, Theme};
use crate::config::{Labels, Language, Provider, Window};
#[cfg(feature = "scraper")]
use crate::scraper::{LinkExtractorType,TextExtractorType};
use crate::utils::get_models;
use super::message::Message;
use super::SidebarMode;
use super::TextOption;
use std::path::Path;
use tracing::info;

macro_rules! button_nf {
    ($text:expr) => {
        button( text($text).align_x(iced::Alignment::Center).font(Font::with_name("Symbols Nerd Font")))
            .width(34.0)
            
    };
}

macro_rules! button_nft {
    ($text:expr, $tp:expr, $msg:ident) => {

        tooltip(
            button( text($text).align_x(iced::Alignment::Center).font(Font::with_name("Symbols Nerd Font")))
                .width(34.0)
                .on_press(Message::$msg), 
            container(text($tp)).padding(5.0).style(container::rounded_box) , tooltip::Position::FollowCursor
            )
    }
}

macro_rules! button_t {
    ($text:expr, $tp:expr, $msg:ident) => {

        tooltip(
            button( text($text).align_x(iced::Alignment::Center))
                .on_press(Message::$msg), 
            container(text($tp)).padding(5.0).style(container::rounded_box) , tooltip::Position::FollowCursor
            )
    }

}

macro_rules! text_nf {
    ($text:expr) => {
        text($text).font(Font::with_name("Symbols Nerd Font"))
    };
}

#[cfg(feature = "scraper")]
pub fn link_extractor<'a>(app: &'a super::App) -> Row<'a, Message> {
    let labels = app.conf.get_labels();
    let padding = app.conf.window.padding;
    let spacing = app.conf.window.spacing;

    if let Some(te) = &app.conf.l_ex {
    match te {
            crate::scraper::LinkExtractorType::PatternExtractor { pattern, n_chapters, name } => {
                row![
                    column![ text("Type"), text("Pattern Link") ].width(150.0).padding(padding).spacing(spacing),
                    column![ text(labels.e_pattern.as_str()), text_input("", pattern).on_input(Message::LP) ].padding(padding).spacing(spacing),
                    column![ text(labels.e_chapters.as_str()), text_input("", n_chapters.to_string().as_str()).on_input(Message::LX) ].padding(padding).spacing(spacing),
                    column![ text(labels.name.as_str()), text_input("", name).on_input(Message::LN) ].padding(padding).spacing(spacing)
                ].padding(padding).spacing(spacing)
            }
            crate::scraper::LinkExtractorType::MainPageExtractor { url, pattern, name } => {
                row![
                    column![ text("Type"), text("Main Page Link") ].width(150.0).padding(padding).spacing(spacing),
                    column![ text(labels.e_pattern.as_str()), text_input("", pattern).on_input(Message::LP) ].padding(padding).spacing(spacing),
                    column![ text("URL"), text_input("", url).on_input(Message::LU) ].padding(padding).spacing(spacing),
                    column![ text(labels.name.as_str()), text_input("", name).on_input(Message::LN) ].padding(padding).spacing(spacing)
                ]
            }
        }
    } else {
        row![text("-")]
    }

}

pub fn notes<'a>(app: &'a super::App) -> Column<'a,Message> {
    let text_id = app.loaded_text.id;
    let labels = app.conf.get_labels();
    let padding = app.conf.window.padding;
    let spacing = app.conf.window.spacing;
    let idc_close =  button_nft!("\u{ea76}", labels.cancel.as_str(), Close);
    let idc_export = button_nft!("\u{eb4a}", labels.save.as_str(), NotesExport);

    let bold = |header| {
        text(header).font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        })
    };

    let columns = [
        table::column(bold(labels.line.as_str()), |note: &crate::textbase::Note| text(note.line)  ),
        table::column(bold(labels.column.as_str()), |note: &crate::textbase::Note| text(note.char)  ),
        table::column(bold(labels.text.as_str()), |note: &crate::textbase::Note| text(note.text.as_str())  ).width(1000.0),
        table::column( "", |note: &crate::textbase::Note| button_nf!("\u{f0c5}").on_press(Message::NotesCopy(note.text.clone())) ),
        table::column( "", |note: &crate::textbase::Note| button_nf!("\u{f01b4}").on_press(Message::NotesDelete { document: note.doc, line: note.line, character: note.char } ) ),
    ];
    let idc_list = table(columns ,&app.notes).padding(5.0).separator(1.0);

    match text_id {
        0 => column![
            space::vertical(), 
            row![ space::horizontal(), text(labels.no_text.as_str()), space::horizontal() ].align_y(Alignment::Center), space::horizontal(),
            row![ space::horizontal(), idc_close, space::horizontal(), space::vertical() ],
        ].padding(padding).spacing(spacing).align_x(Alignment::Center),
        _ => column![
            row![scrollable(idc_list).spacing(5.0)].padding(padding).spacing(spacing).height(app.conf.window.height - 150.0),
            row![idc_close, idc_export].padding(padding).spacing(spacing),
        ].padding(50.0).spacing(spacing),
    }
}

#[cfg(feature = "scraper")]
pub fn text_extractor<'a>(app: &'a super::App) -> Row<'a, Message> {
    let labels = app.conf.get_labels();
    let padding = app.conf.window.padding;
    let spacing = app.conf.window.spacing;

    if let Some(te) = &app.conf.t_ex {
        match te {
            crate::scraper::TextExtractorType::PatternTextExtractor { title_pattern, pattern, name } => {
                let tp = title_pattern.clone().unwrap_or("title".to_string());
                row![
                    column![ text("Type"), text("Pattern Text") ].width(150.0).padding(padding).spacing(spacing),
                    column![ text(labels.e_pattern.as_str()), text_input("", pattern.as_str()).on_input(Message::TP) ].padding(padding).spacing(spacing),
                    column![ text(labels.name.as_str()), text_input("", name).on_input(Message::TN) ].padding(padding).spacing(spacing),
                    column![ text(labels.e_tp.as_str()), text_input("", tp.as_str()).on_input(Message::TT) ].padding(padding).spacing(spacing),
                ]
            }
            crate::scraper::TextExtractorType::CText => {
                row![ text("CText") ]
            }
        }
    } else {
        row![text("-")]
    }
}

pub fn sidebar<'a>(app: &'a super::App) -> Column<'a, Message, Theme> {
    let sm = app.sidebar_mode;
    let labels = app.conf.get_labels();
    let win = &app.conf.window;

    let ids_mode = text(labels.mode.as_str());
    let mut idr_mode = row![ids_mode].padding(win.padding).spacing(win.spacing);
    let idc_mode: Vec<Element<'a,Message>> = SidebarMode::ALL.iter().map(|mode| 
        button::<Message,Theme,Renderer>(mode.as_str()).on_press(Message::SidebarModeChanged(*mode)).into()
        ).collect::<Vec<_>>();
    for m in idc_mode {
        idr_mode = idr_mode.push(m);
    }

    match sm {
        SidebarMode::Notes => {
            let c = app.text.cursor().position;
            let pos = (c.line, c.column);
            let note = app.notes.iter().find(|n| n.line == pos.0 && n.char == pos.1);

            let a_delete = note.map(|_| Message::RemoveNote);
            let a_save = if pos.0 > 0 || pos.1 > 0 { Some(Message::SaveNote) } else { None };

            let idc_delete_note = button( labels.delete_note.as_str()).on_press_maybe(a_delete);
            let idc_save_note = button( labels.save.as_str() ).on_press_maybe(a_save);
            let idr_note = row![ idc_delete_note, idc_save_note ].spacing(win.spacing).padding(win.padding);
            let idc_notes: TextEditor<PlainText,Message,Theme> = text_editor( &app.sidebar_notes )
                .placeholder(labels.notes_placeholder.as_str())
                .on_action(Message::NotesAction)
                .height(500.0);
            
            column![container(idr_mode).style(container::bordered_box).width(380.0), idc_notes, space::vertical(), idr_note].padding(win.padding_frame).align_x(iced::Alignment::Center)
        }
        SidebarMode::AI => {
            let idc_meaning = button_t!(labels.ai_meaning.as_str(), labels.p_meaning.as_str(), PromptMeaning);
            let idc_explain = button_t!(labels.ai_explain.as_str(), labels.p_explain.as_str(), PromptExplain);
            let idc_grammar = button(labels.ai_grammar.as_str())
                .on_press(Message::PromptGrammar);
            let idc_examples = button(labels.ai_examples.as_str())
                .on_press(Message::PromptUsage);
            let idr_prompts = row![idc_meaning, idc_explain, idc_grammar, idc_examples].spacing(win.spacing).padding(win.padding);

            let idc_summary = button(labels.summary.as_str())
                .on_press(Message::PromptSummary);
            let idr_prompts2 = row![idc_summary].spacing(win.spacing).padding(win.padding);

            let idc_answer = scrollable(markdown::view(app.answer_text.items(), app.theme())
                .map(Message::LinkClicked)).height(560.0);
            let idc_to_notes = button(labels.to_notes.as_str()).on_press(Message::AnswerToNotes);
            let idr_answer = row![idc_answer].padding(win.padding).spacing(win.spacing);
            column![container(idr_mode).style(container::bordered_box).width(380.0), idr_prompts, idr_prompts2, space::vertical(), idr_answer, idc_to_notes].padding(win.padding_frame).align_x(iced::Alignment::Center)
        }
        SidebarMode::Dictionary => {
            let idc_result = scrollable(markdown::view(app.result_text.items(), app.theme())
                .map(Message::LinkClicked))
                .height(640.0)
                .width(330.0);
            let idc_copy = button(labels.copy.as_str())
                .on_press(Message::DictionaryCopy);
            let idc_to_notes = button(labels.to_notes.as_str())
                .on_press(Message::DictionaryToNotes);
            let idc_dtn = checkbox(app.dtn_append).on_toggle(Message::DictionaryToNotesAppend);
            let idr_buttons = row![idc_copy, idc_to_notes, idc_dtn, text_nf!("\u{f4d0}")].padding(win.padding).spacing(win.spacing);
            
            column![container(idr_mode).style(container::bordered_box).width(380.0), idc_result, space::vertical(), idr_buttons].padding(win.padding_frame).align_x(iced::Alignment::Center)
        }
        SidebarMode::Anki => {
            let idc_result = scrollable( markdown( app.anki_result.items(), app.theme() )
                .map(Message::LinkClicked))
                .width(330.0)
                .height(640.0);
            column![
                container(idr_mode).style(container::bordered_box).width(380.0),
                row![idc_result],
                space::vertical(),
            ].padding(win.padding_frame).align_x(iced::Alignment::Center)
        }
    }
}

pub fn default<'a>(app: &'a super::App) -> Row<'a, Message> {
    let win = &app.conf.window;
    let labels = app.conf.get_labels();
    let conf = &app.conf;
    let is_img = !app.image_empty();
    let ocr_enabled = !conf.ocr_models.is_empty() 
        && Path::new(&format!("{}{}", conf.ocr_models,crate::ocr::DET_FILE)).exists()
        && Path::new(&format!("{}{}", conf.ocr_models,crate::ocr::INDEX)).exists()
        && Path::new(&format!("{}{}", conf.ocr_models,crate::ocr::REC_FILE)).exists();
    info!("OCR enabled: {}", ocr_enabled);

    let idc_settings = button_nft!("\u{eb51}", labels.settings.as_str(), Settings);
    let idc_settings_ai = button_nft!("\u{ee9c}", labels.settings_ai.as_str(), AiSettings);
    let text_option = match app.loaded_text.id {
        0 => TextOption::Add,
        _ => TextOption::Save,
    };
    let idc_load = button_nf!("\u{e28b}").on_press(Message::TextAction(TextOption::Load));
    let idc_save = button_nf!("\u{f14f3}").on_press(Message::TextAction(text_option));

    let idc_img_load = button_nft!("\u{f03e}", labels.load_img.as_str(), LoadImage);
    let idc_img_file = button_nft!("\u{f1c5}", labels.load_file.as_str(), LoadImageFile);
    let idc_img_clear = button_nft!("\u{f1418}", labels.clear_img.as_str(), ClearImage);
    #[cfg(feature = "scraper")]
    let idc_scraper = button_nft!("\u{f167e}", labels.scraper.as_str(), Scraper );
    let idc_ocr = button_nf!("\u{f113a}")
        .on_press_maybe(if is_img && ocr_enabled { Some(Message::Ocr) } else { None });
    let idc_notes = button_nft!("\u{f1a7d}", labels.to_notes.as_str(), Notes);
    
    #[cfg(feature = "scraper")]
    let idr_left_top = row![
        idc_settings,
        idc_settings_ai, 
        idc_load, 
        idc_save, 
        idc_img_load, 
        idc_img_file, 
        idc_img_clear, 
        idc_scraper, 
        idc_ocr,
        idc_notes,
    ].padding(win.padding_frame).spacing(win.spacing);

    #[cfg(not(feature = "scraper"))]
    let idr_left_top = row![
        idc_settings,
        idc_settings_ai, 
        idc_load, 
        idc_save, 
        idc_img_load, 
        idc_img_file, 
        idc_img_clear, 
        idc_ocr,
        idc_notes,
    ].padding(win.padding_frame).spacing(win.spacing);

    let idc_title = row![
        text(app.loaded_text.title()).align_y(iced::Alignment::Center).shaping(text::Shaping::Advanced),
        space::horizontal(),
        
        button_nf!("\u{f09a8}").on_press(Message::TextMode(crate::gui::TextMode::Raw)),
        button_nf!("\u{f126f}").on_press(Message::TextMode(crate::gui::TextMode::Md)),
    ].padding(win.padding).spacing(win.spacing);

    let idc_text: Element<'_, Message> = match app.text_mode {
            super::TextMode::Raw => text_editor( &app.text )
                .placeholder("")
                .on_action(Message::EditAction)
                .height(win.height*0.70)
                .size(win.font_size.unwrap_or(18.0))
                .into(),
            super::TextMode::Md => scrollable(markdown::view(app.text_md.items(), app.theme())
                .map(Message::LinkClicked)
                )
                .height(win.height*0.7).into(),
        };    
    let idc_simplified = button(labels.simplified.as_str()).on_press(Message::Simplified);

    let deepl_enabled = !conf.keys.deepl.is_empty();
    let deepl_action = if deepl_enabled { Some(Message::Deepl) } else { None };
    let idc_deepl = button_nf!("\u{f05ca}")
        .on_press_maybe(deepl_action);

    let idc_save_prog = button_nf!("\u{eb4a}")
        .on_press(Message::UpdateProgress);

    let idr_buttons = row![idc_simplified, idc_deepl, idc_save_prog].padding(win.padding_frame).spacing(win.spacing);

    let cursor = app.position;
    let (line,column) = (cursor.line, cursor.column);
    let text_id = app.loaded_text.id.to_string();
    let idr_status = row![
        text(format!("({},{}) | {} | text: {}", line, column,
            if is_img { "img" } else { "" },
            if app.loaded_text.id > 0 { text_id } else { String::new() } )),
    ];

    let idc_sidebar = sidebar(app);

    let left_panel = column![idr_left_top, idc_title, idc_text, idr_buttons, idr_status].padding(win.padding_frame).width(win.width*0.7);
    row![left_panel, idc_sidebar].align_y(iced::Alignment::Center)
}

pub fn text_action<'a>(app: &'a super::App, option: TextOption) -> Column<'a, Message> {
    let conf = &app.conf;
    let win = &conf.window;
    let labels = conf.get_labels();
    match option {
        TextOption::Load => {
            let ids_load = text(labels.load_text.as_str());
            
            let ids_select = text(labels.select_text.as_str()).width(win.settings_label_w);
            let sel = Some(app.loaded_text.clone());
            let idc_select = pick_list(app.documents.as_slice(), sel, Message::TextSelected).width(450.0).text_shaping(text::Shaping::Advanced);
            let idr_select = row![ids_select,idc_select].padding(win.padding).spacing(win.spacing);

            let idc_load_doc = button_nf!("\u{f00be}")
                .on_press(Message::LoadText);
            let idc_delete = button_nf!("\u{f01b4}")
                .on_press(Message::RemoveText);
            let idc_close = button_nf!("\u{ea76}")
                .on_press(Message::Close);
            let idc_new = button_nf!("\u{ea7f}")
                .on_press(Message::NewText);


            let idr_buttons = row![idc_load_doc, idc_delete, idc_new, idc_close].padding(win.padding).spacing(win.spacing);
            column![
                row![ids_load],
                idr_select,
                idr_buttons,
            ].padding(conf.window.padding_frame)
                .align_x(iced::Alignment::Center)
        }
        TextOption::Save => {
            let idc_new = row![ text(labels.new.as_str()), checkbox(app.new_text).on_toggle(Message::NewTextCheck) ];
            let idc_title = text_input("",app.loaded_text.title.as_str()).on_input(Message::TextTitleChange);
            let idr_doc = row![idc_new, idc_title].padding(win.padding).spacing(win.spacing);

            let idc_save = button_nf!("\u{f0cfb}")
                .on_press(Message::SaveText);
            let idc_close = button_nf!("\u{ea76}")
                .on_press(Message::Close);
            let idr_buttons = row![idc_save, idc_close].padding(win.padding).spacing(win.spacing);

            column![idr_doc, idr_buttons].padding(conf.window.padding_frame).align_x(iced::Alignment::Center)
        }
        TextOption::Add => {
            let ids_label = text(labels.add_text.as_str());

            let idc_text = text_input("",app.loaded_text.title.as_str()).on_input(Message::TextTitleChange);
            let idr_name = row![idc_text].padding(win.padding).spacing(win.spacing);

            let idc_add = button_nf!("\u{f0cfb}")
                .on_press(Message::SaveText);
            let idc_close = button_nf!("\u{ea76}")
                .on_press(Message::Close);
            let idr_buttons = row![idc_add, idc_close].padding(win.padding).spacing(win.spacing);

            column![ids_label, idr_name, idr_buttons].padding(conf.window.padding_frame).align_x(iced::Alignment::Center)
        }
        TextOption::New => {
            let ids_label = text(labels.add_text.as_str());

            let ids_name = text(labels.text_name.as_str());
            let idc_name = text_input("", &app.loaded_text.title);
            let idr_name = row![ids_name, idc_name];

            let idc_add = button_nf!("\u{f0cfb}")
                .on_press(Message::SaveText);
            let idc_close = button_nf!("\u{ea76}")
                .on_press(Message::Close);
            let idr_buttons = row![idc_add, idc_close].padding(win.padding).spacing(win.spacing);

            column![ids_label, idr_name, idr_buttons].padding(conf.window.padding_frame).align_x(iced::Alignment::Center)
        }
        TextOption::Delete => {
            let ids_label = text(labels.delete_text.as_str());

            let idc_del = button_nf!("\u{f01b4}")
                .on_press(Message::RemoveText);
            let idc_close =  button_nf!("\u{ea76}")
                .on_press(Message::Close);
            let idr_buttons = row![idc_del, idc_close].padding(win.padding).spacing(win.spacing);

            column![ids_label, idr_buttons]
        }
    }
}

pub fn settings<'a>(app: &super::App, labels: &'a Labels) -> Element<'a, Message> {
    let win = &app.conf.window;
    let ids_theme = text(labels.theme.as_str()).width(win.settings_label_w);
    let theme = win.theme();
    let idc_theme = pick_list(Theme::ALL, Some(theme), Message::ThemeSelected);
    let idr_theme = row![ids_theme, idc_theme].padding(win.padding).spacing(win.spacing);

    let ids_lang = text(labels.language.as_str()).width(win.settings_label_w);
    let idc_lang = pick_list(Language::ALL, win.lang, Message::Language).text_shaping(text::Shaping::Advanced);
    let idr_lang = row![ids_lang, idc_lang].padding(win.padding).spacing(win.spacing);

    let fs = format!("{}: {}", labels.font_size, win.font_size.unwrap_or(15.0));
    let ids_font_size = text(fs).width(win.settings_label_w);
    let idc_font_size = slider(12.0..=24.0, win.font_size.unwrap_or(15.0), Message::FontSizeChange).step(0.5);
    let idr_font_size = row![ids_font_size, idc_font_size].padding(win.padding).spacing(win.spacing);

    let ids_deepl = text(labels.deepl_key.as_str()).width(win.settings_label_w);
    let idc_deepl = text_input("", &app.conf.keys.deepl).on_input(Message::DeeplKeyChange);
    let idr_deepl = row![ids_deepl, idc_deepl].padding(win.padding).spacing(win.spacing);

    let ids_anki = text("Anki db").width(win.settings_label_w);
    let anki = app.conf.anki.clone().unwrap_or_default();
    let idc_anki = text_input("", anki.as_str()).on_input(Message::AnkiChanged);
    let idr_anki = row![ids_anki, idc_anki].padding(win.padding).spacing(win.spacing);

    let ids_appdata = text(labels.appdata.as_str()).width(win.settings_label_w);
    let idc_appdata = text_input("", &app.conf.db.as_ref().unwrap_or(&String::new()).as_str());
    let idd_appdata = button_nf!("\u{e5fe}").on_press(Message::DbChange);
    let idr_appdata = row![ids_appdata, idc_appdata, idd_appdata].padding(win.padding).spacing(win.spacing);

    let idc_close = button_nf!("\u{ea76}").on_press(Message::Close);
    let idc_save = button_nf!("\u{f145b}").on_press(Message::SettingsSave);
    let idr_b = row![ idc_close, idc_save ].padding(win.padding).spacing(win.spacing);

    column![
        idr_theme, 
        idr_lang, 
        text(""),
        idr_font_size,
        idr_anki,
        idr_deepl,
        idr_appdata,
        iced::widget::rule::horizontal(2.0),
        idr_b
    ].padding(win.padding_frame).align_x(iced::Alignment::Center).into()
}

pub fn display_av<'a>(conf: &'a Window, msg: &'a str) -> Element<'a, Message> {
    let alert = container(
        column![ 
            text(msg),
            button(text("OK")).on_press(Message::Close) 
        ].align_x(iced::Alignment::Center)
        .spacing(10)
        ).width(conf.width).height(conf.height).padding(10).align_x(iced::Alignment::Center).align_y(iced::Alignment::Center);
    alert.into()
}

pub fn ai_settings<'a>(app: &'a super::App) -> Column<'a, Message> {
    let conf = &app.conf;
    let labels  = conf.get_labels();
    let is_new = app.new_ai.is_some();
    let changed = app.ai_changed;
    let settings_label_w = conf.window.settings_label_w;
    let idc_ai_select = text(labels.ai_select.as_str()).width(settings_label_w);
    let ai_chat = app.new_ai.as_ref().or(conf.ai_chats.get(conf.ai_chat.as_str()));
    let idc_ai_chat: Element<'a, Message> = if is_new { text("").into() } else {
        pick_list(conf.get_ai_chats(),ai_chat.clone(), Message::AiChatSelected).into()
    };
    let idr_ai_select = row![idc_ai_select, idc_ai_chat].spacing(conf.window.spacing);
    let is_ai_selected = ai_chat.is_some();

    let idc_model: Element<'a, Message> = 
        if let Some(c) = app.new_ai.as_ref() {
            if get_models(&c.provider).is_empty() {
                text_input(labels.enter_model.as_str(), ai_chat.map(|x| x.model.clone()).unwrap_or_default().as_str()).on_input(Message::AiModelChange).into() 
            } else {
                pick_list(get_models(&c.provider), ai_chat.map(|x| x.model.clone()), Message::AiModelChange).into() 
            }
        } else if let Some(c) = conf.get_ai_config() {
            if get_models(&c.provider).is_empty() {
                text_input(labels.enter_model.as_str(), ai_chat.map(|x| x.model.clone()).unwrap_or_default().as_str()).on_input(Message::AiModelChange).into() 
            } else {
                pick_list(get_models(&c.provider), ai_chat.map(|x| x.model.clone()), Message::AiModelChange).into() 
            }
        } else {
            text("<< NO MODELS >>").into()
    };

    let provider = ai_chat.map(|e| e.provider).unwrap_or_default();
    let ai_name = ai_chat.map(|e| e.name.clone()).unwrap_or_default();
    let ai_url = ai_chat.map(|e| e.url.clone().unwrap_or_default() ).unwrap_or_default();
    let ai_chat_key = ai_chat.map(|e| e.key.clone().unwrap_or_default()).unwrap_or_default();
    let ai_ndims = format!("{}", ai_chat.map(|e| e.ndims.unwrap_or_default()).unwrap_or_default());
    let ai_max_docs = format!("{}", ai_chat.map(|e| e.max_docs.unwrap_or_default()).unwrap_or_default());
    let ai_embedding_model = ai_chat.map(|e| e.embedding_model.clone().unwrap_or_default()).unwrap_or_default();

    let idc_ai_details: Element<'_, Message> = if is_ai_selected || app.new_ai.is_some() {
        column![
            row![   text(labels.name.as_str()).width(settings_label_w), 
                    text_input("", ai_name.as_str()).on_input(Message::AiNameChange) 
            ],
            row![   text(labels.url.as_str()).width(settings_label_w),
                    text_input("", ai_url.as_str()).on_input_maybe(if provider == Provider::Ollama { Some(Message::AiUrlChange) } else { None }) 
            ],
            row![   text(labels.model.as_str()).width(settings_label_w),
                    idc_model,
            ],
            row![
                    text(labels.ai_key.as_str()).width(settings_label_w),
                    text_input("", ai_chat_key.as_str()).on_input(Message::AiKeyChange) 

            ],
            row![   text(labels.provider.as_str()).width(settings_label_w),
                    pick_list(Provider::ALL, Some(provider), Message::AiProviderChange) 
            ],
            row![
                    text(labels.ndims.as_str()).width(settings_label_w),
                    text_input("", ai_ndims.as_str()).on_input(Message::AiNdimsChanged)
            ],
            row![
                    text(labels.max_docs.as_str()).width(settings_label_w),
                    text_input("", ai_max_docs.as_str()).on_input(Message::AiMaxDocsChanged)
            ],
            row![
                    text(labels.emb_mod.as_str()).width(settings_label_w),
                    text_input("", ai_embedding_model.as_str()).on_input(Message::AiMaxDocsChanged)
            ],
        ].spacing(conf.window.spacing).into()
    } else {
        text(labels.ai_select.as_str()).into()
    };
    let ids_role = text(labels.ai_preamble.as_str()).width(settings_label_w);
    let idc_role = text_input("", conf.ai_role.as_str()).on_input(Message::AiRoleChanged);
    let idc_new = button(labels.new.as_str() ).on_press_maybe(if is_new { None } else { Some(Message::NewAi) });
    let idc_close = button(labels.cancel.as_str() ).on_press(Message::Close);
    let idc_save = button(labels.save.as_str() ).on_press(Message::AiSettingsSave);
    let idr_b = row![ idc_new, idc_close, idc_save ].padding(conf.window.padding).spacing(conf.window.spacing);
    column![
        idr_ai_select,
        row![iced::widget::rule::horizontal(2.0)].height(8.0),
        idc_ai_details,
        row![iced::widget::rule::horizontal(2.0)].height(8.0),
        row![ids_role, idc_role].padding(conf.window.padding).spacing(conf.window.spacing),
        idr_b,
        if changed { text(labels.restart_needed.as_str()) } else { text("") }
        ].padding(conf.window.padding_frame).align_x(iced::Alignment::Center)
        
}

#[cfg(feature = "scraper")]
pub fn scrapper<'a>(app: &'a super::App) -> Column<'a, Message> {
    let conf = &app.conf;
    let win = &conf.window;
    let labels  = conf.get_labels();

    let ids_sleep = text(labels.sc_sleep.as_str()).width(win.settings_label_w);
    let sc_sleep = match conf.scraper_sleep {
        Some(sc) => format!("{}", sc),
        None => String::new(),
    };
    let idc_sleep = text_input("", sc_sleep.as_str())
        .width(100.0)
        .on_input(Message::ScraperSleepChanged);
    let idr_sleep = row![ids_sleep, idc_sleep, text("ms")].padding(conf.window.padding).spacing(conf.window.spacing);

    let idc_start = button_nf!("\u{ead3}")
        .on_press_maybe(if app.scraper_file.is_empty() { None } else { Some(Message::StartScraper) });
    let idc_save = button_nf!("\u{eb4b}")
        .on_press(Message::SettingsSave);
    let idc_stop = button_nf!("\u{f04d}").on_press(Message::StopScraper);
    let idc_close = button_nf!("\u{ea76}").on_press_maybe(
        if app.scraper_progress == 0.0 { Some(Message::Close) } else { None });
    let idc_new = button_nf!("\u{f0394}").on_press_maybe(if app.sc_new { None } else { Some(Message::ScNew) });
    let idr_buttons = row![
        idc_start, 
        idc_save, 
        idc_stop, 
        idc_close, 
        idc_new
    ].padding(conf.window.padding).spacing(conf.window.spacing);

    let l_list = if app.sc_new {
        LinkExtractorType::ALL
    } else {
        app.conf.link_ex.as_slice()
    };

    let t_list = if app.sc_new {
        TextExtractorType::ALL
    } else {
        app.conf.text_ex.as_slice()
    };
    
    column![
        text(labels.sc_warn.as_str()),
        idr_sleep,
        row![ text_input(labels.name.as_str(), app.scraper_file.as_str()).on_input(Message::ScraperFileChanged) ],
        row![ 
            text(labels.e_lt.as_str()), 
            pick_list(l_list, app.conf.l_ex.as_ref(), Message::LinkExtractorChanged).width(500.0),
            pick_list(t_list, app.conf.t_ex.as_ref(), Message::TextExtractorChanged).width(500.0),
        ].padding(conf.window.padding).spacing(conf.window.spacing),
        link_extractor(app),
        text_extractor(app),
        progress_bar(0.0..=app.scraper_to, app.scraper_progress),
        idr_buttons,
     ].padding(conf.window.padding_frame).spacing(win.spacing).align_x(iced::Alignment::Center)
}

pub fn anki_stats<'a>(app: &'a super::App) -> Column<'a, Message> {
    let conf = &app.conf;
    let win = &conf.window;



    column![
        button_nf!("\u{f015c}").on_press(Message::Close)
    ].padding(win.padding_frame).spacing(win.spacing).align_x(iced::Alignment::Center)
}

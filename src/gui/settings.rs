use iced::advanced::text::highlighter::PlainText;
use iced::widget::{Column, Row, TextEditor, button, checkbox, column, container, markdown, pick_list, progress_bar, row, scrollable, slider, space, table, text, text_editor, text_input, tooltip};
use iced::{Alignment, Element, Font, Padding, Renderer, Theme};
use crate::cedict::HSK_TOTAL;
use crate::config::{Provider, Window};
use crate::utils::get_models;
use super::message::Message;
use super::SidebarMode;
use super::TextOption;
use std::path::Path;
use tracing::debug;

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

/*macro_rules! button_t {
    ($text:expr, $tp:expr, $msg:ident) => {

        tooltip(
            button( text($text).align_x(iced::Alignment::Center))
                .on_press(Message::$msg), 
            container(text($tp)).padding(5.0).style(container::rounded_box) , tooltip::Position::FollowCursor
            )
    }

}*/

macro_rules! text_nf {
    ($text:expr) => {
        text($text).font(Font::with_name("Symbols Nerd Font"))
    };
}

pub fn notes<'a>(app: &'a super::App) -> Column<'a,Message> {
    let text_id = app.loaded_text.id;
    let padding = app.conf.window.padding;
    let spacing = app.conf.window.spacing;
    let idc_close =  button_nft!("\u{ea76}", t!("cancel"), Close);
    let idc_export = button_nft!("\u{eb4a}", t!("save"), NotesExport);

    let bold = |header| {
        text(header).font(Font {
            weight: iced::font::Weight::Bold,
            ..Font::DEFAULT
        })
    };

    let columns = [
        table::column(bold(t!("line")), |note: &crate::textbase::Note| text(note.line)  ),
        table::column(bold(t!("column")), |note: &crate::textbase::Note| text(note.char)  ),
        table::column(bold(t!("text")), |note: &crate::textbase::Note| text(note.text.as_str())  ).width(1000.0),
        table::column( "", |note: &crate::textbase::Note| button_nf!("\u{f0c5}").on_press(Message::NotesCopy(note.text.clone())) ),
        table::column( "", |note: &crate::textbase::Note| button_nf!("\u{f01b4}").on_press(Message::NotesDelete { document: note.doc, line: note.line , character: note.char } ) ),
    ];
    let idc_list = table(columns ,&app.notes).padding(5.0).separator(1.0);

    match text_id {
        0 => column![
            space::vertical(), 
            row![ space::horizontal(), text(t!("no_text")), space::horizontal() ].align_y(Alignment::Center), space::horizontal(),
            row![ space::horizontal(), idc_close, space::horizontal(), space::vertical() ],
        ].padding(padding).spacing(spacing).align_x(Alignment::Center),
        _ => column![
            row![scrollable(idc_list).spacing(5.0)].padding(padding).spacing(spacing).height(app.conf.window.height - 150.0),
            row![idc_close, idc_export].padding(padding).spacing(spacing),
        ].padding(50.0).spacing(spacing),
    }
}

pub fn sidebar<'a>(app: &'a super::App) -> Column<'a, Message, Theme> {
    let sm = app.sidebar_mode;
    let win = &app.conf.window;

    let ids_mode = text(t!("mode"));
    let mut idr_mode = row![ids_mode].padding(win.padding).spacing(win.spacing);
    let idc_mode: Vec<Element<'a,Message>> = SidebarMode::ALL.iter().map(|mode| 
        button::<Message,Theme,Renderer>(mode.as_str()).on_press(Message::SidebarModeChanged(*mode)).into()
        ).collect::<Vec<_>>();
    for m in idc_mode {
        idr_mode = idr_mode.push(m);
    }

    let id_mode = container(idr_mode).style(container::bordered_box).width(420.0);

    match sm {
        SidebarMode::Notes => {
            let c = app.text.cursor().position;
            let pos = (c.line as i64, c.column as i64);
            let note = app.notes.iter().find(|n| n.line == pos.0 && n.char == pos.1);

            let a_delete = note.map(|_| Message::RemoveNote);
            let a_save = if pos.0 > 0 || pos.1 > 0 { Some(Message::SaveNote) } else { None };

            let idc_delete_note = button( text(t!("delete_note"))).on_press_maybe(a_delete);
            let idc_save_note = button( text(t!("save")) ).on_press_maybe(a_save);
            let idr_note = row![ idc_delete_note, idc_save_note ].spacing(win.spacing).padding(win.padding);
            let idc_notes: TextEditor<PlainText,Message,Theme> = text_editor( &app.sidebar_notes )
                .placeholder( t!("notes_placeholder") )
                .on_action(Message::NotesAction)
                .height(500.0);
            
            column![id_mode, idc_notes, space::vertical(), idr_note].spacing(win.spacing).padding(win.padding_frame).align_x(iced::Alignment::Center)
        }
        SidebarMode::AI => {
            let idr_prompts = ai_prompter(app);
            let idc_answer = scrollable(markdown::view(app.answer_text.items(), app.theme())
                .map(Message::LinkClicked)).height(500.0);
            let idc_to_notes = button(text(t!("to_notes"))).on_press(Message::AnswerToNotes);
            let idr_answer = row![idc_answer].padding(win.padding).spacing(win.spacing);

            let is_ollama = if let Some(c) = app.conf.get_ai_config() && c.provider == Provider::Ollama { Some(Message::AiOllamaKeepAlive) } else { None };
            let idc_ka = (if app.ollama_alive { button_nf!("\u{f205}") } else { button_nf!("\u{f204}") }).on_press_maybe(is_ollama);
            let idc_to_text = button_nf!("\u{ebcc}").on_press(Message::AiCurrentTextToEditor);
            let idc_history_clear = button_nf!("\u{eabf}").on_press(Message::AiHistoryClear);
            column![id_mode, space::vertical(), idr_answer.padding(Padding::new(0.0).right(5.0)), idr_prompts,
                row![idc_to_text, idc_to_notes, idc_ka,
                    button_nf!("\u{f073a}").on_press(Message::AiCancelPrompt),
                    idc_history_clear,
                ].spacing(win.spacing).padding(win.padding),
                ].padding(win.padding_frame).align_x(iced::Alignment::Center)
        }
        SidebarMode::Dictionary => {
            let idc_result = scrollable(markdown::view(app.result_text.items(), app.theme())
                .map(Message::LinkClicked))
                .height(640.0)
                .width(330.0);
            let idc_copy = button(text(t!("copy")))
                .on_press(Message::DictionaryCopy);
            let idc_to_notes = button(text(t!("to_notes")))
                .on_press(Message::DictionaryToNotes);
            let idc_dtn = checkbox(app.dtn_append).on_toggle(Message::DictionaryToNotesAppend);
            let idr_buttons = row![idc_copy, idc_to_notes, idc_dtn, text_nf!("\u{f4d0}")].padding(win.padding).spacing(win.spacing);
            
            column![id_mode, idc_result, space::vertical(), idr_buttons].padding(win.padding_frame).align_x(iced::Alignment::Center)
        }
    }
}

pub fn default<'a>(app: &'a super::App) -> Row<'a, Message> {
    let win = &app.conf.window;
    let conf = &app.conf;
    let is_img = !app.image_empty();
    let ocr_enabled = !conf.ocr_models.is_empty() 
        && Path::new(&format!("{}{}", app.models_dir,crate::ocr::DET_FILE)).exists()
        && Path::new(&format!("{}{}", app.models_dir,crate::ocr::INDEX)).exists()
        && Path::new(&format!("{}{}", app.models_dir,crate::ocr::REC_FILE)).exists();

    let idc_settings = button_nft!("\u{eb51}", t!("settings"), Settings);
    let idc_settings_ai = button_nft!("\u{ee9c}", t!("settings_ai"), AiSettings);
    let text_option = match app.loaded_text.id {
        0 => TextOption::Add,
        _ => TextOption::Save,
    };
    let idc_load = button_nf!("\u{e28b}").on_press(Message::TextAction(TextOption::Load));
    let idc_save = button_nf!("\u{f14f3}").on_press(Message::TextAction(text_option));

    let idc_img_load = button_nft!("\u{f03e}", t!("load_img"), LoadImage);
    let idc_img_file = button_nft!("\u{f1c5}", t!("load_file"), LoadImageFile);
    let idc_img_clear = button_nft!("\u{f1418}", t!("clear_img"), ClearImage);
    let idc_ocr = button_nf!("\u{f113a}")
        .on_press_maybe(if is_img && ocr_enabled { Some(Message::Ocr) } else { None });
    let idc_notes = button_nft!("\u{f1a7d}", t!("to_notes"), Notes);
    let anki_acc = if let Some(cd) = app.cedict.as_ref()
        && cd.anki_len() > 0 {
            Some(Message::AnkiStats)
    } else {
        None
    };
    let idc_anki_db = button_nf!("\u{f1c0}").on_press_maybe(anki_acc);
    let idc_deepl = button_nf!("\u{f05ca}").on_press(Message::DeeplAsk);
    
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
        idc_anki_db,
    ].padding(win.padding_frame).spacing(win.spacing);
    let title = format!("{} | {},{}", app.loaded_text.title, app.loaded_text.line, app.loaded_text.character);

    let idc_title = row![
        text(title).align_y(iced::Alignment::Center).shaping(text::Shaping::Advanced),
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
    let idc_simplified = button(text(t!("simplified"))).on_press(Message::Simplified);

    let idc_save_prog = button_nf!("\u{eb4a}")
        .on_press(Message::UpdateProgress);

    let idr_buttons = row![idc_simplified, idc_save_prog, idc_deepl].padding(win.padding_frame).spacing(win.spacing);

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
    match option {
        TextOption::Load => {
            let ids_load = text(t!("load_text"));
            
            let ids_select = text(t!("select_text")).width(win.settings_label_w);
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
            let idc_save = button_nf!("\u{f0c7}")
                .on_press(Message::SaveText);

            let idr_buttons = row![idc_load_doc, idc_delete, idc_new, idc_close, idc_save].padding(win.padding).spacing(win.spacing);
            column![
                row![ids_load],
                idr_select,
                idr_buttons,
            ].padding(conf.window.padding_frame)
                .align_x(iced::Alignment::Center)
        }
        TextOption::Save => {
            let idc_new = row![ text(t!("new")), checkbox(app.new_text).on_toggle(Message::NewTextCheck) ];
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
            let ids_label = text(t!("add_text"));

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
            let ids_label = text(t!("add_text"));

            let ids_name = text(t!("text_name"));
            let idc_name = text_input("", &app.loaded_text.title).on_input(Message::TextTitleChange);
            let idr_name = row![ids_name, idc_name];

            let idc_add = button_nf!("\u{f0cfb}")
                .on_press(Message::SaveText);
            let idc_close = button_nf!("\u{ea76}")
                .on_press(Message::Close);
            let idr_buttons = row![idc_add, idc_close].padding(win.padding).spacing(win.spacing);

            column![ids_label, idr_name, idr_buttons].padding(conf.window.padding_frame).align_x(iced::Alignment::Center)
        }
        TextOption::Delete => {
            let ids_label = text(t!("delete_text"));

            let idc_del = button_nf!("\u{f01b4}")
                .on_press(Message::RemoveText);
            let idc_close =  button_nf!("\u{ea76}")
                .on_press(Message::Close);
            let idr_buttons = row![idc_del, idc_close].padding(win.padding).spacing(win.spacing);

            column![ids_label, idr_buttons]
        }
    }
}

pub fn settings<'a>(app: &super::App) -> Element<'a, Message> {
    let win = &app.conf.window;
    let ids_theme = text(t!("theme")).width(win.settings_label_w);
    let theme = win.theme();
    let idc_theme = pick_list(Theme::ALL, Some(theme), Message::ThemeSelected);
    let idr_theme = row![ids_theme, idc_theme].padding(win.padding).spacing(win.spacing);

    let ids_lang = text(t!("language")).width(win.settings_label_w);
    let r = rust_i18n::available_locales!().iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let idc_lang = pick_list(r, win.lang.clone(), Message::Language).text_shaping(text::Shaping::Advanced);
    let idr_lang = row![ids_lang, idc_lang].padding(win.padding).spacing(win.spacing);

    let fs = format!("{}: {}", t!("font_size"), win.font_size.unwrap_or(15.0));
    let ids_font_size = text(fs).width(win.settings_label_w);
    let idc_font_size = slider(12.0..=24.0, win.font_size.unwrap_or(15.0), Message::FontSizeChange).step(0.5);
    let idr_font_size = row![ids_font_size, idc_font_size].padding(win.padding).spacing(win.spacing);

    let ids_anki = text("Anki db").width(win.settings_label_w);
    let anki = app.conf.anki.clone().unwrap_or_default();
    let idc_anki = text_input("", anki.as_str()).on_input(Message::AnkiChanged);
    let idr_anki = row![ids_anki, idc_anki].padding(win.padding).spacing(win.spacing);

    let ids_appdata = text(t!("appdata")).width(win.settings_label_w);
    let idc_appdata = text_input("", app.conf.db.as_ref().unwrap_or(&String::new()).as_str());
    let idd_appdata = button_nf!("\u{e5fe}").on_press(Message::DbChange);
    let idr_appdata = row![ids_appdata, idc_appdata, idd_appdata].padding(win.padding).spacing(win.spacing);

    let idc_close = button_nf!("\u{ea76}").on_press(Message::Close);
    let idc_save = button_nf!("\u{f145b}").on_press(Message::SettingsSave);
    let idr_b = row![ idc_close, idc_save ].padding(win.padding).spacing(win.spacing);

    let ids_lang = text(format!("Deepl {}", t!("language")));
    let idc_lang = pick_list(crate::ai::deepl::ALL_LANGS, app.conf.deepl_lang.clone(), Message::DeeplLangSel);
    let idc_key = text_input("key", &app.conf.keys.deepl).on_input(Message::DeeplKeyChanged);
    let idr_deepl = row![ids_lang, idc_lang, idc_key].spacing(win.spacing).padding(win.padding);

    column![
        idr_theme, 
        idr_lang, 
        text(""),
        idr_font_size,
        idr_anki,
        idr_appdata,
        iced::widget::rule::horizontal(2.0),
        idr_deepl,
        idr_b,
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

pub fn ai_prompter<'a>(app: &'a super::App) -> Column<'a, Message> {
    let win = &app.conf.window;
    column![
        row![text("prompt") ,pick_list(crate::gui::Prompt::ALL, app.prompt.as_ref(), Message::AiPromptSelected)].spacing(win.spacing).padding(win.padding),
        if let Some(crate::gui::Prompt::Custom) = app.prompt {
            container(row![
                 text_editor(&app.custom_prompt).on_action(Message::AiCustomPromptChanged).width(400.0).height(100.0),
            ].spacing(win.spacing).padding(win.padding) )
        } else { container(row![].width(400.0).height(100.0)).style(container::bordered_box) },
        row![
            checkbox(app.text_ctx).label("text").on_toggle(Message::AiTextCtxToggle), checkbox(app.image_include).label("img").on_toggle(Message::AiImageIncludeToggle), 
            button_nf!("\u{f0eb4}").on_press_maybe(
                    if app.prompt.is_some() { Some(Message::Prompt) } else { None }) ].spacing(win.spacing).padding(win.padding)
    ].spacing(win.spacing).padding(win.padding)
}

pub fn ai_settings<'a>(app: &'a super::App) -> Column<'a, Message> {
    let conf = &app.conf;
    let is_new = app.new_ai.is_some();
    let changed = app.ai_changed;
    let settings_label_w = conf.window.settings_label_w;
    let idc_ai_select = text(t!("ai_select")).width(settings_label_w);
    let ai_chat = app.new_ai.as_ref().or(conf.ai_chats.get(conf.ai_chat.as_str()));
    let idc_ai_chat: Element<'a, Message> = if is_new { text("").into() } else {
        pick_list(conf.get_ai_chats(),ai_chat, Message::AiChatSelected).into()
    };

    let idr_ai_select = row![idc_ai_select, idc_ai_chat].spacing(conf.window.spacing);
    let is_ai_selected = ai_chat.is_some();
    
    let idc_model: Element<'a, Message> = 
        if let Some(c) = app.get_ai_config() {
            if get_models(&c.provider).is_empty() {
                text_input(&t!("enter_model"), ai_chat.map(|x| x.model.clone()).unwrap_or_default().as_str()).on_input(Message::AiModelChange) .into() 
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
    //let ai_chat_temp = ai_chat.map(|e| if let Some(t) = e.temperature { t.to_string() } else { String::from("") } ).unwrap();

    //let idc_temp = text_input("", &ai_chat_temp)
    //    .on_input_maybe(if provider.is_local() { Some(Message::AiTemperatureChanged) } else { None } );
    let idc_ai_details: Element<'_, Message> = if is_ai_selected || app.new_ai.is_some() {
        column![
            row![   text(t!("name")).width(settings_label_w), 
                    text_input("", ai_name.as_str()).on_input(Message::AiNameChange) 
            ],
            row![   text(t!("url")).width(settings_label_w),
                    text_input("", ai_url.as_str()).on_input_maybe(if provider.is_local() { Some(Message::AiUrlChange) } else { None }) 
            ],
            row![   text(t!("model")).width(settings_label_w),
                    idc_model,
            ],
            row![
                    text(t!("ai_key")).width(settings_label_w),
                    text_input("", ai_chat_key.as_str()).on_input(Message::AiKeyChange) 

            ],
            row![   text(t!("provider")).width(settings_label_w),
                    pick_list(Provider::ALL, Some(provider), Message::AiProviderChange) 
            ],
            //row![
            //        text("temp").width(settings_label_w),
            //        idc_temp
            //],
        ].spacing(conf.window.spacing).into()
    } else {
        text(t!("ai_select")).into()
    };
    let ids_role = text(t!("ai_preamble")).width(settings_label_w);
    let idc_role = text_input("", conf.ai_role.as_str()).on_input(Message::AiRoleChanged);
    let idc_new = button(text(t!("new"))).on_press_maybe(if is_new { None } else { Some(Message::NewAi) });
    let idc_close = button(text(t!("cancel"))).on_press(Message::Close);
    let idc_save = button(text(t!("save"))).on_press(Message::AiSettingsSave);
    let idc_delete = button_nf!("\u{f01b4}").on_press(Message::AiDelete);
    let idr_b = row![ idc_new, idc_close, idc_save, idc_delete ].padding(conf.window.padding).spacing(conf.window.spacing);
    column![
        idr_ai_select,
        row![iced::widget::rule::horizontal(2.0)].height(8.0),
        idc_ai_details,
        row![iced::widget::rule::horizontal(2.0)].height(8.0),
        row![ids_role, idc_role].padding(conf.window.padding).spacing(conf.window.spacing),
        idr_b,
        if changed { text(t!("restart_needed")) } else { text("") }
        ].padding(conf.window.padding_frame).align_x(iced::Alignment::Center)
}

pub fn anki_stats<'a>(app: &'a super::App) -> Column<'a, Message> {
    let conf = &app.conf;
    let win = &conf.window;
    let cedict = app.cedict.as_ref();
    if let Some(cd) = cedict {
        let hsk_map = cd.count_hsk_anki();
        let mut hsk: Vec<(u32,usize)> = hsk_map.iter()
            .map(|(a,b)| (*a,*b))
            .collect();
        hsk.sort_by(|a,b| a.0.cmp(&b.0));
        let total = cd.anki_len();
        debug!("hsk: {:?} / {}", hsk, cd.data_hsk_len());
        return column![
            row![text(format!("HSK1: {:5} / {:5}", std::cmp::min(hsk[0].1, HSK_TOTAL[0] as usize), HSK_TOTAL[0] )).width(400.0), progress_bar(0.0..=HSK_TOTAL[0], hsk[0].1 as f32) ].spacing(win.spacing),
            row![text(format!("HSK2: {:5} / {:5}", std::cmp::min(hsk[1].1, HSK_TOTAL[1] as usize), HSK_TOTAL[1] )).width(400.0), progress_bar(0.0..=HSK_TOTAL[1], hsk[1].1 as f32) ].spacing(win.spacing),
            row![text(format!("HSK3: {:5} / {:5}", std::cmp::min(hsk[2].1, HSK_TOTAL[2] as usize), HSK_TOTAL[2] )).width(400.0), progress_bar(0.0..=HSK_TOTAL[2], hsk[2].1 as f32) ].spacing(win.spacing),
            row![text(format!("HSK4: {:5} / {:5}", std::cmp::min(hsk[3].1, HSK_TOTAL[3] as usize), HSK_TOTAL[3] )).width(400.0), progress_bar(0.0..=HSK_TOTAL[3], hsk[3].1 as f32) ].spacing(win.spacing),
            row![text(format!("HSK5: {:5} / {:5}", std::cmp::min(hsk[4].1, HSK_TOTAL[4] as usize), HSK_TOTAL[4] )).width(400.0), progress_bar(0.0..=HSK_TOTAL[4], hsk[4].1 as f32) ].spacing(win.spacing),
            row![text(format!("HSK6: {:5} / {:5}", std::cmp::min(hsk[5].1, HSK_TOTAL[5] as usize), HSK_TOTAL[5] )).width(400.0), progress_bar(0.0..=HSK_TOTAL[5], hsk[5].1 as f32) ].spacing(win.spacing),
            row![text(format!("HSK7: {:5} / {:5}", std::cmp::min(hsk[6].1, HSK_TOTAL[6] as usize), HSK_TOTAL[6] )).width(400.0), progress_bar(0.0..=HSK_TOTAL[6], hsk[6].1 as f32) ].spacing(win.spacing),
            row![text(format!("total anki: {}", total))],
            button_nf!("\u{f015c}").on_press(Message::Close)
            ].padding(win.padding_frame).spacing(win.spacing).align_x(iced::Alignment::Center);
    }

    column![
        button_nf!("\u{f015c}").on_press(Message::Close)
    ].padding(win.padding_frame).spacing(win.spacing).align_x(iced::Alignment::Center)
}

pub fn files_dl<'a>(app: &'a super::App) -> Column<'a, Message> {
    let conf = &app.conf;
    let win = &conf.window;

    column![
        progress_bar(0.0..=1.0, app.dl_prog)
    ].padding(win.padding_frame).spacing(win.spacing).align_x(iced::Alignment::Center)
}

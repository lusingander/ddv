use std::slice;

use itsuki::zero_indexed_enum;
use ratatui::{
    crossterm::event::KeyEvent, layout::Rect, style::Stylize, text::Line, widgets::Block, Frame,
};

use crate::{
    color::ColorTheme,
    constant::APP_NAME,
    data::{
        list_attribute_keys, to_key_string, Item, KeySchemaType, PlainJsonItem, RawJsonItem,
        TableDescription,
    },
    event::{AppEvent, Sender, UserEvent, UserEventMapper},
    handle_user_events,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    view::common::{attribute_to_spans, raw_string_from_scroll_lines_state, to_highlighted_lines},
    widget::{ScrollLines, ScrollLinesOptions, ScrollLinesState},
};

pub struct ItemView {
    table_description: TableDescription,
    item: Item,
    key_string: String,

    helps: Vec<Spans>,
    short_helps: Vec<SpansWithPriority>,
    theme: ColorTheme,
    tx: Sender,

    scroll_lines_state: ScrollLinesState,

    preview_type: PreviewType,
}

#[zero_indexed_enum]
enum PreviewType {
    KeyValue,
    PlainJson,
    RawJson,
}

impl ItemView {
    pub fn new(
        table_description: TableDescription,
        item: Item,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        let schema = &table_description.key_schema_type;
        let key_string = to_key_string(&item, schema);
        let scroll_lines_state =
            ScrollLinesState::new(vec![], ScrollLinesOptions::new(false, false));
        let helps = build_helps(mapper, theme);
        let short_helps = build_short_helps(mapper);

        let mut view = ItemView {
            table_description,
            item,
            key_string,

            helps,
            short_helps,
            theme,
            tx,

            scroll_lines_state,
            preview_type: PreviewType::KeyValue,
        };
        view.update_preview();
        view
    }
}

impl ItemView {
    pub fn handle_user_key_event(&mut self, user_events: Vec<UserEvent>, _key_event: KeyEvent) {
        handle_user_events! { user_events =>
            UserEvent::Close => {
                self.tx.send(AppEvent::BackToBeforeView);
            }
            UserEvent::Down => {
                self.scroll_lines_state.scroll_forward();
            }
            UserEvent::Up => {
                self.scroll_lines_state.scroll_backward();
            }
            UserEvent::PageDown => {
                self.scroll_lines_state.scroll_page_forward();
            }
            UserEvent::PageUp => {
                self.scroll_lines_state.scroll_page_backward();
            }
            UserEvent::GoToTop => {
                self.scroll_lines_state.scroll_to_top();
            }
            UserEvent::GoToBottom => {
                self.scroll_lines_state.scroll_to_end();
            }
            UserEvent::Right => {
                self.scroll_lines_state.scroll_right();
            }
            UserEvent::Left => {
                self.scroll_lines_state.scroll_left();
            }
            UserEvent::NextPreview => {
                self.next_preview();
                self.update_preview();
            }
            UserEvent::PrevPreview => {
                self.prev_preview();
                self.update_preview();
            }
            UserEvent::ToggleWrap => {
                self.scroll_lines_state.toggle_wrap();
            }
            UserEvent::ToggleNumber => {
                self.scroll_lines_state.toggle_number();
            }
            UserEvent::CopyToClipboard => {
                self.copy_to_clipboard();
            }
            UserEvent::Help => {
                self.open_help();
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = format!(
            " {} - {} ({}) ",
            APP_NAME, self.table_description.table_name, self.key_string
        );
        let scroll = ScrollLines::default()
            .block(
                Block::bordered()
                    .title_top(Line::from(title).left_aligned())
                    .fg(self.theme.fg)
                    .bg(self.theme.bg),
            )
            .theme(&self.theme);

        f.render_stateful_widget(scroll, area, &mut self.scroll_lines_state);
    }

    pub fn short_helps(&self) -> &[SpansWithPriority] {
        &self.short_helps
    }
}

fn build_helps(mapper: &UserEventMapper, theme: ColorTheme) -> Vec<Spans> {
    #[rustfmt::skip]
    let helps = vec![
        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
        BuildHelpsItem::new(UserEvent::Close, "Back to table"),
        BuildHelpsItem::new(UserEvent::Down, "Scroll down"),
        BuildHelpsItem::new(UserEvent::Up, "Scroll up"),
        BuildHelpsItem::new(UserEvent::Right, "Scroll right"),
        BuildHelpsItem::new(UserEvent::Left, "Scroll left"),
        BuildHelpsItem::new(UserEvent::PageDown, "Scroll page down"),
        BuildHelpsItem::new(UserEvent::PageUp, "Scroll page up"),
        BuildHelpsItem::new(UserEvent::GoToTop, "Scroll to top"),
        BuildHelpsItem::new(UserEvent::GoToBottom, "Scroll to bottom"),
        BuildHelpsItem::new(UserEvent::NextPreview, "Switch to next preview"),
        BuildHelpsItem::new(UserEvent::PrevPreview, "Switch to previous preview"),
        BuildHelpsItem::new(UserEvent::ToggleWrap, "Toggle wrap"),
        BuildHelpsItem::new(UserEvent::ToggleNumber, "Toggle number"),
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy descriptions"),
    ];
    build_help_spans(helps, mapper, theme)
}

fn build_short_helps(mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
    #[rustfmt::skip]
    let helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Back", 1),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Scroll", 2),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 5),
        BuildShortHelpsItem::single(UserEvent::NextPreview, "Switch preview", 3),
        BuildShortHelpsItem::group(vec![UserEvent::ToggleWrap, UserEvent::ToggleNumber], "Toggle wrap/number", 6),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 4),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    build_short_help_spans(helps, mapper)
}

impl ItemView {
    fn next_preview(&mut self) {
        self.preview_type = self.preview_type.next();
    }

    fn prev_preview(&mut self) {
        self.preview_type = self.preview_type.prev();
    }

    fn update_preview(&mut self) {
        let item = &self.item;
        let schema = &self.table_description.key_schema_type;
        let theme = &self.theme;

        let lines = match self.preview_type {
            PreviewType::KeyValue => get_key_value_lines(item, schema, theme),
            PreviewType::PlainJson => get_plain_json_lines(item, schema),
            PreviewType::RawJson => get_raw_json_lines(item, schema),
        };
        let options = self.scroll_lines_state.current_options();

        self.scroll_lines_state = ScrollLinesState::new(lines, options);
    }

    fn copy_to_clipboard(&self) {
        let content = raw_string_from_scroll_lines_state(&self.scroll_lines_state);
        self.tx
            .send(AppEvent::CopyToClipboard("item".into(), content));
    }

    fn open_help(&self) {
        self.tx.send(AppEvent::OpenHelp(self.helps.clone()))
    }
}

fn get_key_value_lines(
    item: &Item,
    schema: &KeySchemaType,
    theme: &ColorTheme,
) -> Vec<Line<'static>> {
    let attribute_keys = list_attribute_keys(slice::from_ref(item), schema);
    let max_key_width = attribute_keys.iter().map(|k| k.len()).max().unwrap();
    let max_attr_width = attribute_keys
        .iter()
        .flat_map(|k| item.attributes.get(k).map(|a| a.as_type_str().len()))
        .max()
        .unwrap();

    let mut lines = vec![];
    for key in attribute_keys {
        if let Some(attr) = item.attributes.get(&key) {
            let mut spans = vec![];
            spans.push(format!("{key:>max_key_width$}").bold());
            spans.push(
                format!(" {:<w$} ", attr.as_type_str(), w = max_attr_width)
                    .fg(theme.item_attribute_type_fg)
                    .bold(),
            );
            spans.push(": ".into());
            spans.extend(attribute_to_spans(attr, theme));
            lines.push(Line::from(spans));
        }
    }
    lines
}

fn get_plain_json_lines(item: &Item, schema: &KeySchemaType) -> Vec<Line<'static>> {
    let json_item = PlainJsonItem::new(item, schema);
    let json_str = serde_json::to_string_pretty(&json_item).unwrap();
    to_highlighted_lines(&json_str)
}

fn get_raw_json_lines(item: &Item, schema: &KeySchemaType) -> Vec<Line<'static>> {
    let json_item = RawJsonItem::new(item, schema);
    let json_str = serde_json::to_string_pretty(&json_item).unwrap();
    to_highlighted_lines(&json_str)
}

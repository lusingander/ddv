use std::collections::HashMap;

use itsuki::zero_indexed_enum;
use laurier::highlight::highlight_matched_text;
use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, ListItem},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    color::ColorTheme,
    config::UiTableListConfig,
    data::{Table, TableDescription},
    event::{AppEvent, Sender, UserEvent, UserEventMapper},
    handle_user_events, handle_user_events_with_default,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    view::common::{raw_string_from_scroll_lines_state, to_highlighted_lines},
    widget::{ScrollLines, ScrollLinesOptions, ScrollLinesState, ScrollList, ScrollListState},
};

pub struct TableListView {
    tables: Vec<Table>,
    table_descriptions: HashMap<String, TableDescription>,

    list_helps: Vec<Spans>,
    list_filtered_helps: Vec<Spans>,
    detail_helps: Vec<Spans>,
    list_short_helps: Vec<SpansWithPriority>,
    list_filtered_short_helps: Vec<SpansWithPriority>,
    detail_short_helps: Vec<SpansWithPriority>,
    config: UiTableListConfig,
    theme: ColorTheme,
    tx: Sender,

    list_state: ScrollListState,
    scroll_lines_state: ScrollLinesState,
    filter_state: FilterState,
    filter_input: Input,
    view_indices: Vec<usize>,

    focused: Focused,
    preview_type: PreviewType,
}

enum FilterState {
    None,
    Filtering,
    Filtered,
}

#[zero_indexed_enum]
enum Focused {
    List,
    Detail,
}

#[zero_indexed_enum]
enum PreviewType {
    KeyValue,
    Json,
}

impl TableListView {
    pub fn new(
        tables: Vec<Table>,
        mapper: &UserEventMapper,
        config: UiTableListConfig,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        let list_state = ScrollListState::new(tables.len());

        let view_indices = (0..tables.len()).collect();
        let scroll_lines_state =
            ScrollLinesState::new(vec![], ScrollLinesOptions::new(false, false));
        let (list_helps, list_filtered_helps, detail_helps) = build_helps(mapper, theme);
        let (list_short_helps, list_filtered_short_helps, detail_short_helps) =
            build_short_helps(mapper);

        let mut view = TableListView {
            tables,
            table_descriptions: HashMap::new(),
            list_helps,
            list_filtered_helps,
            detail_helps,
            list_short_helps,
            list_filtered_short_helps,
            detail_short_helps,
            config,
            theme,
            tx,
            filter_state: FilterState::None,
            filter_input: Input::default(),
            view_indices,
            list_state,
            scroll_lines_state,
            focused: Focused::List,
            preview_type: PreviewType::KeyValue,
        };
        view.load_table_description();
        view.update_preview();
        view
    }
}

impl TableListView {
    pub fn handle_user_key_event(&mut self, user_events: Vec<UserEvent>, key_event: KeyEvent) {
        if let FilterState::Filtering = self.filter_state {
            handle_user_events_with_default! { user_events =>
                UserEvent::Confirm => {
                    self.apply_filter();
                }
                UserEvent::Reset => {
                    self.reset_filter();
                }
                => {
                    self.update_filter(key_event);
                    self.load_table_description();
                    self.update_preview();
                }
            }
            return;
        }

        match self.focused {
            Focused::List => {
                handle_user_events! { user_events =>
                    UserEvent::Down => {
                        self.list_state.select_next();
                        self.load_table_description();
                        self.update_preview();
                    }
                    UserEvent::Up => {
                        self.list_state.select_prev();
                        self.load_table_description();
                        self.update_preview();
                    }
                    UserEvent::PageDown => {
                        self.list_state.select_next_page();
                        self.load_table_description();
                        self.update_preview();
                    }
                    UserEvent::PageUp => {
                        self.list_state.select_prev_page();
                        self.load_table_description();
                        self.update_preview();
                    }
                    UserEvent::GoToBottom => {
                        self.list_state.select_last();
                        self.load_table_description();
                        self.update_preview();
                    }
                    UserEvent::GoToTop => {
                        self.list_state.select_first();
                        self.load_table_description();
                        self.update_preview();
                    }
                    UserEvent::QuickFilter => {
                        self.start_filtering();
                    }
                    UserEvent::Reset => {
                        self.reset_filter();
                    }
                    UserEvent::NextPane => {
                        self.next_pane();
                    }
                    UserEvent::NextPreview => {
                        self.next_preview();
                        self.update_preview();
                    }
                    UserEvent::PrevPreview => {
                        self.prev_preview();
                        self.update_preview();
                    }
                    UserEvent::Confirm => {
                        self.load_table_items();
                    }
                    UserEvent::CopyToClipboard => {
                        self.copy_table_name_to_clipboard();
                    }
                    UserEvent::Help => {
                        self.open_help();
                    }
                }
            }
            Focused::Detail => {
                handle_user_events! { user_events =>
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
                    UserEvent::NextPane => {
                        self.next_pane();
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
                        self.copy_table_descriptions_to_clipboard();
                    }
                    UserEvent::Help => {
                        self.open_help();
                    }
                }
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let [list_area, detail_area] = Layout::horizontal([
            Constraint::Length(self.config.list_width),
            Constraint::Min(0),
        ])
        .areas(area);

        self.render_list(f, list_area);
        self.render_detail(f, detail_area);
    }

    pub fn short_helps(&self) -> &[SpansWithPriority] {
        match self.focused {
            Focused::List => match self.filter_state {
                FilterState::None => &self.list_short_helps,
                FilterState::Filtering | FilterState::Filtered => &self.list_filtered_short_helps,
            },
            Focused::Detail => &self.detail_short_helps,
        }
    }
}

fn build_helps(
    mapper: &UserEventMapper,
    theme: ColorTheme,
) -> (Vec<Spans>, Vec<Spans>, Vec<Spans>) {
    #[rustfmt::skip]
    let list_helps = vec![
        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
        BuildHelpsItem::new(UserEvent::Down, "Select next item"),
        BuildHelpsItem::new(UserEvent::Up, "Select prev item"),
        BuildHelpsItem::new(UserEvent::GoToTop, "Select first item"),
        BuildHelpsItem::new(UserEvent::GoToBottom, "Select last item"),
        BuildHelpsItem::new(UserEvent::Confirm, "Open table"),
        BuildHelpsItem::new(UserEvent::QuickFilter, "Filter tables"),
        BuildHelpsItem::new(UserEvent::NextPane, "Switch to next pane"),
        BuildHelpsItem::new(UserEvent::NextPreview, "Switch to next preview"),
        BuildHelpsItem::new(UserEvent::PrevPreview, "Switch to prev preview"),
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy table name"),
    ];
    #[rustfmt::skip]
    let list_filtered_helps = vec![
        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
        BuildHelpsItem::new(UserEvent::Down, "Select next item"),
        BuildHelpsItem::new(UserEvent::Up, "Select prev item"),
        BuildHelpsItem::new(UserEvent::GoToTop, "Select first item"),
        BuildHelpsItem::new(UserEvent::GoToBottom, "Select last item"),
        BuildHelpsItem::new(UserEvent::Confirm, "Open table"),
        BuildHelpsItem::new(UserEvent::Reset, "Clear filter"),
        BuildHelpsItem::new(UserEvent::NextPane, "Switch to next pane"),
        BuildHelpsItem::new(UserEvent::NextPreview, "Switch to next preview"),
        BuildHelpsItem::new(UserEvent::PrevPreview, "Switch to prev preview"),
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy table name"),
    ];
    #[rustfmt::skip]
    let detail_helps = vec![
        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
        BuildHelpsItem::new(UserEvent::Down, "Scroll down"),
        BuildHelpsItem::new(UserEvent::Up, "Scroll up"),
        BuildHelpsItem::new(UserEvent::PageDown, "Scroll page down"),
        BuildHelpsItem::new(UserEvent::PageUp, "Scroll page up"),
        BuildHelpsItem::new(UserEvent::GoToTop, "Scroll to top"),
        BuildHelpsItem::new(UserEvent::GoToBottom, "Scroll to bottom"),
        BuildHelpsItem::new(UserEvent::Right, "Scroll right"),
        BuildHelpsItem::new(UserEvent::Left, "Scroll left"),
        BuildHelpsItem::new(UserEvent::NextPane, "Switch to next pane"),
        BuildHelpsItem::new(UserEvent::NextPreview, "Switch to next preview"),
        BuildHelpsItem::new(UserEvent::PrevPreview, "Switch to previous preview"),
        BuildHelpsItem::new(UserEvent::ToggleWrap, "Toggle wrap"),
        BuildHelpsItem::new(UserEvent::ToggleNumber, "Toggle number"),
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy table descriptions"),
    ];
    (
        build_help_spans(list_helps, mapper, theme),
        build_help_spans(list_filtered_helps, mapper, theme),
        build_help_spans(detail_helps, mapper, theme),
    )
}

fn build_short_helps(
    mapper: &UserEventMapper,
) -> (
    Vec<SpansWithPriority>,
    Vec<SpansWithPriority>,
    Vec<SpansWithPriority>,
) {
    #[rustfmt::skip]
    let list_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Select", 2),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 7),
        BuildShortHelpsItem::single(UserEvent::Confirm, "Open", 1),
        BuildShortHelpsItem::single(UserEvent::QuickFilter, "Filter", 3),
        BuildShortHelpsItem::single(UserEvent::NextPane, "Switch pane", 4),
        BuildShortHelpsItem::single(UserEvent::NextPreview, "Switch preview", 6),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 5),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    #[rustfmt::skip]
    let list_filtered_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Select", 2),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 7),
        BuildShortHelpsItem::single(UserEvent::Confirm, "Open", 1),
        BuildShortHelpsItem::single(UserEvent::Reset, "Clear filter", 3),
        BuildShortHelpsItem::single(UserEvent::NextPane, "Switch pane", 4),
        BuildShortHelpsItem::single(UserEvent::NextPreview, "Switch preview", 6),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 5),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    #[rustfmt::skip]
    let detail_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Scroll", 1),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 5),
        BuildShortHelpsItem::single(UserEvent::NextPane, "Switch pane", 2),
        BuildShortHelpsItem::single(UserEvent::NextPreview, "Switch preview", 4),
        BuildShortHelpsItem::group(vec![UserEvent::ToggleWrap, UserEvent::ToggleNumber], "Toggle wrap/number", 6),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 3),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    (
        build_short_help_spans(list_helps, mapper),
        build_short_help_spans(list_filtered_helps, mapper),
        build_short_help_spans(detail_helps, mapper),
    )
}

impl TableListView {
    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let show_items_count = area.height as usize - 2 /* border */;
        let item_width = area.width as usize - 2 /* border */ - 2 /* padding (list) */ - 2 /* padding (item) */;
        let query = self.filter_input.value().to_lowercase();
        let items: Vec<_> = self
            .filtered_tables()
            .iter()
            .skip(self.list_state.offset)
            .take(show_items_count)
            .enumerate()
            .map(|(i, t)| {
                let line = if query.is_empty() {
                    let name = console::truncate_str(&t.name, item_width, "..");
                    Line::raw(format!(" {name:item_width$} "))
                } else {
                    let i = t.name.to_lowercase().find(&query).unwrap();
                    let mut spans = highlight_matched_text(&t.name)
                        .ellipsis("..")
                        .matched_range(i, i + query.len())
                        .matched_style(
                            Style::default()
                                .fg(self.theme.quick_filter_matched_fg)
                                .bg(self.theme.quick_filter_matched_bg),
                        )
                        .into_spans();
                    spans.insert(0, " ".into());
                    spans.push(" ".into());
                    Line::from(spans)
                };
                let mut style = Style::default();
                if i + self.list_state.offset == self.list_state.selected {
                    style = style.fg(self.theme.selected_fg);
                    if self.focused == Focused::List {
                        style = style.bg(self.theme.selected_bg);
                    } else {
                        style = style.bg(self.theme.disabled);
                    }
                };
                ListItem::new(line).style(style)
            })
            .collect();
        let list = ScrollList::new(items)
            .theme(&self.theme)
            .focused(self.focused == Focused::List);
        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_detail(&mut self, f: &mut Frame, area: Rect) {
        let mut block = Block::bordered().fg(self.theme.fg).bg(self.theme.bg);
        if self.focused != Focused::Detail {
            block = block.border_style(Style::default().fg(self.theme.disabled));
        }
        let scroll = ScrollLines::default().block(block).theme(&self.theme);

        f.render_stateful_widget(scroll, area, &mut self.scroll_lines_state);
    }
}

impl TableListView {
    fn load_table_description(&self) {
        if let Some(name) = self.current_selected_table_name() {
            if self.table_descriptions.contains_key(name) {
                return;
            }
            self.tx.send(AppEvent::LoadTableDescription(name.into()));
        }
    }

    pub fn set_table_description(&mut self, desc: TableDescription) {
        let name = desc.table_name.clone();
        self.table_descriptions.insert(name, desc);

        self.update_preview();
    }

    fn load_table_items(&self) {
        if let Some(desc) = self.current_selected_table_description() {
            self.tx.send(AppEvent::LoadTableItems(desc.clone()));
        }
    }

    fn current_selected_table_name(&self) -> Option<&str> {
        self.filtered_tables()
            .get(self.list_state.selected)
            .map(|t| t.name.as_str())
    }

    fn current_selected_table_description(&self) -> Option<&TableDescription> {
        self.current_selected_table_name()
            .and_then(|name| self.table_descriptions.get(name))
    }

    fn next_pane(&mut self) {
        self.focused = self.focused.next();
    }

    fn next_preview(&mut self) {
        self.preview_type = self.preview_type.next();
    }

    fn prev_preview(&mut self) {
        self.preview_type = self.preview_type.prev();
    }

    fn update_preview(&mut self) {
        let options = self.scroll_lines_state.current_options();

        if let Some(desc) = self.current_selected_table_description() {
            let lines = match self.preview_type {
                PreviewType::KeyValue => get_key_value_lines(desc),
                PreviewType::Json => get_json_lines(desc, &self.theme),
            };
            self.scroll_lines_state = ScrollLinesState::new(lines, options);
        } else {
            self.scroll_lines_state = ScrollLinesState::new(vec![], options);
        }
    }

    fn start_filtering(&mut self) {
        match self.filter_state {
            FilterState::None | FilterState::Filtered => {
                self.filter_input.reset();
                self.filter_state = FilterState::Filtering;
                self.update_status_input();
            }
            FilterState::Filtering => {}
        }
    }

    fn update_filter(&mut self, key_event: KeyEvent) {
        let event = &ratatui::crossterm::event::Event::Key(key_event);
        self.filter_input.handle_event(event);
        self.filter_view_indices();
        self.update_status_input();
    }

    fn update_status_input(&mut self) {
        let query = format!("/{}", self.filter_input.value());
        let cursor_pos = self.filter_input.cursor() as u16 + 1; // "/"
        self.tx
            .send(AppEvent::UpdateStatusInput(query, Some(cursor_pos)));
    }

    fn apply_filter(&mut self) {
        if self.filter_input.value().is_empty() {
            self.filter_state = FilterState::None;
        } else {
            self.filter_state = FilterState::Filtered;
        }
        if self.view_indices.is_empty() {
            self.reset_filter();
            return;
        }
        self.filter_view_indices();
        self.tx.send(AppEvent::ClearStatus);
    }

    fn reset_filter(&mut self) {
        match self.filter_state {
            FilterState::Filtering | FilterState::Filtered => {
                self.filter_input.reset();
                self.filter_state = FilterState::None;
                let orig_idx = self.view_indices[self.list_state.selected];
                self.filter_view_indices();
                self.list_state.select_index(orig_idx);
                self.tx.send(AppEvent::ClearStatus);
            }
            FilterState::None => {}
        }
    }

    fn filter_view_indices(&mut self) {
        let query = self.filter_input.value().to_lowercase();
        self.view_indices = self
            .tables
            .iter()
            .enumerate()
            .filter(|(_, t)| t.name.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect();
        // reset list state
        self.list_state = self.list_state.with_new_total(self.view_indices.len());
    }

    fn filtered_tables(&self) -> Vec<&Table> {
        self.view_indices.iter().map(|&i| &self.tables[i]).collect()
    }

    fn copy_table_name_to_clipboard(&self) {
        if let Some(name) = self.current_selected_table_name() {
            self.tx
                .send(AppEvent::CopyToClipboard("table name".into(), name.into()));
        }
    }

    fn copy_table_descriptions_to_clipboard(&self) {
        let content = raw_string_from_scroll_lines_state(&self.scroll_lines_state);
        self.tx.send(AppEvent::CopyToClipboard(
            "table descriptions".into(),
            content,
        ));
    }

    fn open_help(&self) {
        match self.focused {
            Focused::List => match self.filter_state {
                FilterState::None => {
                    self.tx.send(AppEvent::OpenHelp(self.list_helps.clone()));
                }
                FilterState::Filtering | FilterState::Filtered => {
                    self.tx
                        .send(AppEvent::OpenHelp(self.list_filtered_helps.clone()));
                }
            },
            Focused::Detail => self.tx.send(AppEvent::OpenHelp(self.detail_helps.clone())),
        }
    }
}

fn get_key_value_lines(desc: &TableDescription) -> Vec<Line<'static>> {
    let key_max_width = 22;
    let separator = " : ";
    let mut lines = vec![];

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Table Name").bold());
    spans.push(separator.into());
    spans.push(desc.table_name.clone().into());
    lines.push(Line::from(spans));

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Key Schema").bold());
    spans.push(separator.into());
    spans.push(
        desc.key_schema
            .iter()
            .map(|key| format!("{} ({})", key.attribute_name, key.key_type.as_str()))
            .collect::<Vec<String>>()
            .join(" / ")
            .into(),
    );
    lines.push(Line::from(spans));

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Attribute Definitions").bold());
    spans.push(separator.into());
    spans.push(
        desc.attribute_definitions
            .iter()
            .map(|attr| format!("{} ({})", attr.attribute_name, attr.attribute_type.as_str()))
            .collect::<Vec<String>>()
            .join(" / ")
            .into(),
    );
    lines.push(Line::from(spans));

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Table Status").bold());
    spans.push(separator.into());
    spans.push(desc.table_status.as_str().to_string().into());
    lines.push(Line::from(spans));

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Creation Date").bold());
    spans.push(separator.into());
    spans.push(desc.creation_date_time.to_string().into());
    lines.push(Line::from(spans));

    if let Some(pt) = &desc.provisioned_throughput {
        let mut spans = vec![];
        spans.push(format!("{:>key_max_width$}", "Provisioned Throughput").bold());
        spans.push(separator.into());
        spans.push(
            format!(
                "Read: {} / Write: {}",
                pt.read_capacity_units, pt.write_capacity_units
            )
            .into(),
        );
        lines.push(Line::from(spans));
    }

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Item Count").bold());
    spans.push(separator.into());
    spans.push(desc.item_count.to_string().into());
    lines.push(Line::from(spans));

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Total Size").bold());
    spans.push(separator.into());
    spans.push(format_size(desc.total_size_bytes).into());
    lines.push(Line::from(spans));

    let mut spans = vec![];
    spans.push(format!("{:>key_max_width$}", "Table ARN").bold());
    spans.push(separator.into());
    spans.push(desc.table_arn.clone().into());
    lines.push(Line::from(spans));

    if let Some(lsis) = &desc.local_secondary_indexes {
        let mut spans = vec![];
        for (i, lsi) in lsis.iter().enumerate() {
            if i == 0 {
                spans.push(format!("{:>key_max_width$}", "LSI").bold());
                spans.push(separator.into());
            } else {
                spans.push(" ".repeat(key_max_width + separator.len()).into());
            }
            spans.push(
                format!(
                    "{} ({})",
                    lsi.index_name,
                    lsi.key_schema
                        .iter()
                        .map(|key| key.attribute_name.clone())
                        .collect::<Vec<String>>()
                        .join(" / "),
                )
                .into(),
            );
            lines.push(Line::from(spans));
            spans = vec![];
        }
    }

    if let Some(gsis) = &desc.global_secondary_indexes {
        let mut spans = vec![];
        for (i, gsi) in gsis.iter().enumerate() {
            if i == 0 {
                spans.push(format!("{:>key_max_width$}", "GSI").bold());
                spans.push(separator.into());
            } else {
                spans.push(" ".repeat(key_max_width + separator.len()).into());
            }
            spans.push(
                format!(
                    "{} ({})",
                    gsi.index_name,
                    gsi.key_schema
                        .iter()
                        .map(|key| key.attribute_name.clone())
                        .collect::<Vec<String>>()
                        .join(" / "),
                )
                .into(),
            );
            lines.push(Line::from(spans));
            spans = vec![];
        }
    }

    lines
}

fn get_json_lines(desc: &TableDescription, theme: &ColorTheme) -> Vec<Line<'static>> {
    let json_str = serde_json::to_string_pretty(&desc).unwrap();
    to_highlighted_lines(&json_str, theme)
}

fn format_size(size_byte: u64) -> String {
    format!(
        "{} ({} bytes)",
        humansize::format_size(size_byte as usize, humansize::DECIMAL),
        size_byte
    )
}

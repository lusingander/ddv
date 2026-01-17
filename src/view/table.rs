use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Margin, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Cell, Clear},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

use crate::{
    color::ColorTheme,
    config::UiTableConfig,
    data::{
        list_attribute_keys, Attribute, Item, KeySchemaType, RawAttributeJsonWrapper, RawJsonItem,
        TableDescription, TableInsight,
    },
    event::{AppEvent, Sender, UserEvent, UserEventMapper},
    handle_user_events, handle_user_events_with_default,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    view::common::{attribute_to_spans, cut_spans_by_width, to_highlighted_lines},
    widget::{CellItem, ScrollLines, ScrollLinesOptions, ScrollLinesState, Table, TableState},
};

const ELLIPSIS: &str = "...";

pub struct TableView {
    table_description: TableDescription,
    items: Vec<Item>,

    config: UiTableConfig,
    theme: ColorTheme,
    tx: Sender,

    helps: TableViewHelps,
    row_cell_items: Vec<Vec<CellItem<'static>>>,
    header_row_cells: Vec<Cell<'static>>,
    table_state: TableState,
    attr_expanded: bool,
    attr_scroll_lines_state: ScrollLinesState,

    filter_state: FilterState,
    filter_input: Input,
    view_indices: Vec<usize>,
}

enum FilterState {
    None,
    Filtering,
    Filtered,
}

struct TableViewHelps {
    table: Vec<Spans>,
    table_filtered: Vec<Spans>,
    attr: Vec<Spans>,
    table_short: Vec<SpansWithPriority>,
    table_filtered_short: Vec<SpansWithPriority>,
    attr_short: Vec<SpansWithPriority>,
}

impl TableView {
    pub fn new(
        table_description: TableDescription,
        items: Vec<Item>,
        mapper: &UserEventMapper,
        config: UiTableConfig,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        let (table_state, row_cell_items, header_row_cells) =
            new_table_state(&table_description, &items, &config, theme);
        let helps = TableViewHelps::new(mapper, theme);
        let attr_scroll_lines_state =
            ScrollLinesState::new(vec![], ScrollLinesOptions::new(false, false));
        let view_indices = (0..items.len()).collect();

        TableView {
            table_description,
            items,

            config,
            theme,
            tx,

            helps,
            row_cell_items,
            header_row_cells,
            table_state,
            attr_expanded: false,
            attr_scroll_lines_state,
            filter_state: FilterState::None,
            filter_input: Input::default(),
            view_indices,
        }
    }
}

impl TableView {
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
                }
            }
            return;
        }

        if self.attr_expanded {
            handle_user_events! { user_events =>
                    UserEvent::Close | UserEvent::Expand => {
                        self.close_expand_selected_attr();
                    }
                    UserEvent::Down => {
                        self.attr_scroll_lines_state.scroll_forward();
                    }
                    UserEvent::Up => {
                        self.attr_scroll_lines_state.scroll_backward();
                    }
                    UserEvent::PageDown => {
                        self.attr_scroll_lines_state.scroll_page_forward();
                    }
                    UserEvent::PageUp => {
                        self.attr_scroll_lines_state.scroll_page_backward();
                    }
                    UserEvent::GoToTop => {
                        self.attr_scroll_lines_state.scroll_to_top();
                    }
                    UserEvent::GoToBottom => {
                        self.attr_scroll_lines_state.scroll_to_end();
                    }
                    UserEvent::Right => {
                        self.attr_scroll_lines_state.scroll_right();
                    }
                    UserEvent::Left => {
                        self.attr_scroll_lines_state.scroll_left();
                    }
                    UserEvent::ToggleWrap => {
                        self.attr_scroll_lines_state.toggle_wrap();
                    }
                    UserEvent::ToggleNumber => {
                        self.attr_scroll_lines_state.toggle_number();
                    }
                    UserEvent::Reload => {
                        self.reload_table();
                    }
                    UserEvent::CopyToClipboard => {
                        self.copy_to_clipboard();
                    }
                    UserEvent::Help => {
                        self.open_help();
                    }
            }
        } else {
            handle_user_events! { user_events =>
                UserEvent::Close => {
                    self.tx.send(AppEvent::BackToBeforeView);
                }
                UserEvent::Down => {
                    self.table_state.select_next_row();
                    self.table_state.update_table_state();
                }
                UserEvent::Up => {
                    self.table_state.select_prev_row();
                    self.table_state.update_table_state();
                }
                UserEvent::PageDown => {
                    self.table_state.select_next_row_page();
                    self.table_state.update_table_state();
                }
                UserEvent::PageUp => {
                    self.table_state.select_prev_row_page();
                    self.table_state.update_table_state();
                }
                UserEvent::GoToBottom => {
                    self.table_state.select_last_row();
                    self.table_state.update_table_state();
                }
                UserEvent::GoToTop => {
                    self.table_state.select_first_row();
                    self.table_state.update_table_state();
                }
                UserEvent::GoToLeft => {
                    self.table_state.select_first_col();
                    self.table_state.update_table_state();
                }
                UserEvent::GoToRight => {
                    self.table_state.select_last_col();
                    self.table_state.update_table_state();
                }
                UserEvent::Right => {
                    self.table_state.select_next_col();
                    self.table_state.update_table_state();
                }
                UserEvent::Left => {
                    self.table_state.select_prev_col();
                    self.table_state.update_table_state();
                }
                UserEvent::QuickFilter => {
                    self.start_filtering();
                }
                UserEvent::Reset => {
                    self.reset_filter();
                }
                UserEvent::Confirm => {
                    self.open_item();
                }
                UserEvent::Insight => {
                    self.open_table_insight();
                }
                UserEvent::Expand => {
                    self.open_expand_selected_attr();
                }
                UserEvent::Widen => {
                    self.table_state.widen_col();
                    self.recalculate_cells();
                }
                UserEvent::Narrow => {
                    self.table_state.narrow_col();
                    self.recalculate_cells();
                }
                UserEvent::Reload => {
                    self.reload_table();
                }
                UserEvent::CopyToClipboard => {
                    self.copy_to_clipboard();
                }
                UserEvent::Help => {
                    self.open_help();
                }
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = format!(" {} ", self.table_description.table_name);
        let count = self.table_state.selected_count_string();
        let block = Block::bordered()
            .title_top(Line::from(title).left_aligned())
            .title_top(Line::from(count).right_aligned())
            .fg(self.theme.fg)
            .bg(self.theme.bg);
        f.render_widget(block, area);

        let table_area = area.inner(Margin::new(2, 1));
        let filtered_row_cell_items: Vec<&Vec<CellItem<'static>>> = self
            .view_indices
            .iter()
            .map(|&i| &self.row_cell_items[i])
            .collect();
        let query = self.filter_input.value();
        let table =
            Table::new(&filtered_row_cell_items, &self.header_row_cells, query).theme(&self.theme);
        f.render_stateful_widget(table, table_area, &mut self.table_state);

        if self.attr_expanded {
            self.render_expanded_item(f, table_area);
        }
    }

    pub fn short_helps(&self) -> &[SpansWithPriority] {
        if self.attr_expanded {
            &self.helps.attr_short
        } else {
            match self.filter_state {
                FilterState::None => &self.helps.table_short,
                FilterState::Filtering | FilterState::Filtered => &self.helps.table_filtered_short,
            }
        }
    }
}

impl TableViewHelps {
    fn new(mapper: &UserEventMapper, theme: ColorTheme) -> TableViewHelps {
        let (table, table_filtered, attr) = build_helps(mapper, theme);
        let (table_short, table_filtered_short, attr_short) = build_short_helps(mapper);
        TableViewHelps {
            table,
            table_filtered,
            attr,
            table_short,
            table_filtered_short,
            attr_short,
        }
    }
}

fn build_helps(
    mapper: &UserEventMapper,
    theme: ColorTheme,
) -> (Vec<Spans>, Vec<Spans>, Vec<Spans>) {
    #[rustfmt::skip]
    let table_helps = vec![
        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
        BuildHelpsItem::new(UserEvent::Close, "Back to table list"),
        BuildHelpsItem::new(UserEvent::Down, "Select next row"),
        BuildHelpsItem::new(UserEvent::Up, "Select previous row"),
        BuildHelpsItem::new(UserEvent::Right, "Select next column"),
        BuildHelpsItem::new(UserEvent::Left, "Select previous column"),
        BuildHelpsItem::new(UserEvent::PageDown, "Select next page"),
        BuildHelpsItem::new(UserEvent::PageUp, "Select previous page"),
        BuildHelpsItem::new(UserEvent::GoToTop, "Select first row"),
        BuildHelpsItem::new(UserEvent::GoToBottom, "Select last row"),
        BuildHelpsItem::new(UserEvent::GoToLeft, "Select first column"),
        BuildHelpsItem::new(UserEvent::GoToRight, "Select last column"),
        BuildHelpsItem::new(UserEvent::Confirm, "Open selected item"),
        BuildHelpsItem::new(UserEvent::QuickFilter, "Filter items"),
        BuildHelpsItem::new(UserEvent::Expand, "Expand selected attribute"),
        BuildHelpsItem::new(UserEvent::Insight, "Open table insight"),
        BuildHelpsItem::new(UserEvent::Widen, "Widen selected column"),
        BuildHelpsItem::new(UserEvent::Narrow, "Narrow selected column"),
        BuildHelpsItem::new(UserEvent::Reload, "Reload table data"),
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy selected item"),
    ];
    #[rustfmt::skip]
    let table_filtered_helps = vec![
        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
        BuildHelpsItem::new(UserEvent::Close, "Back to table list"),
        BuildHelpsItem::new(UserEvent::Down, "Select next row"),
        BuildHelpsItem::new(UserEvent::Up, "Select previous row"),
        BuildHelpsItem::new(UserEvent::Right, "Select next column"),
        BuildHelpsItem::new(UserEvent::Left, "Select previous column"),
        BuildHelpsItem::new(UserEvent::PageDown, "Select next page"),
        BuildHelpsItem::new(UserEvent::PageUp, "Select previous page"),
        BuildHelpsItem::new(UserEvent::GoToTop, "Select first row"),
        BuildHelpsItem::new(UserEvent::GoToBottom, "Select last row"),
        BuildHelpsItem::new(UserEvent::GoToLeft, "Select first column"),
        BuildHelpsItem::new(UserEvent::GoToRight, "Select last column"),
        BuildHelpsItem::new(UserEvent::Confirm, "Open selected item"),
        BuildHelpsItem::new(UserEvent::Reset, "Clear filter"),
        BuildHelpsItem::new(UserEvent::Expand, "Expand selected attribute"),
        BuildHelpsItem::new(UserEvent::Insight, "Open table insight"),
        BuildHelpsItem::new(UserEvent::Widen, "Widen selected column"),
        BuildHelpsItem::new(UserEvent::Narrow, "Narrow selected column"),
        BuildHelpsItem::new(UserEvent::Reload, "Reload table data"),
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy selected item"),
    ];
    #[rustfmt::skip]
    let attr_helps = vec![
        BuildHelpsItem::new(UserEvent::Quit, "Quit app"),
        BuildHelpsItem::new(UserEvent::Close, "Close expansion"),
        BuildHelpsItem::new(UserEvent::Down, "Scroll down"),
        BuildHelpsItem::new(UserEvent::Up, "Scroll up"),
        BuildHelpsItem::new(UserEvent::PageDown, "Scroll page down"),
        BuildHelpsItem::new(UserEvent::PageUp, "Scroll page up"),
        BuildHelpsItem::new(UserEvent::GoToTop, "Scroll to top"),
        BuildHelpsItem::new(UserEvent::GoToBottom, "Scroll to bottom"),
        BuildHelpsItem::new(UserEvent::Right, "Scroll right"),
        BuildHelpsItem::new(UserEvent::Left, "Scroll left"),
        BuildHelpsItem::new(UserEvent::ToggleWrap, "Toggle wrap"),
        BuildHelpsItem::new(UserEvent::ToggleNumber, "Toggle number"),
        BuildHelpsItem::new(UserEvent::Reload, "Reload table data"),
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy selected item"),
    ];
    (
        build_help_spans(table_helps, mapper, theme),
        build_help_spans(table_filtered_helps, mapper, theme),
        build_help_spans(attr_helps, mapper, theme),
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
    let table_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Back", 1),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Select row", 6),
        BuildShortHelpsItem::group(vec![UserEvent::Left, UserEvent::Right], "Select col", 7),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 11),
        BuildShortHelpsItem::single(UserEvent::Confirm, "Open", 2),
        BuildShortHelpsItem::single(UserEvent::QuickFilter, "Filter", 5),
        BuildShortHelpsItem::single(UserEvent::Expand, "Expand", 4),
        BuildShortHelpsItem::single(UserEvent::Insight, "Insight", 3),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 8),
        BuildShortHelpsItem::group(vec![UserEvent::Widen, UserEvent::Narrow], "Widen/Narrow", 10),
        BuildShortHelpsItem::single(UserEvent::Reload, "Reload", 9),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    #[rustfmt::skip]
    let table_filtered_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Back", 1),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Select row", 6),
        BuildShortHelpsItem::group(vec![UserEvent::Left, UserEvent::Right], "Select col", 7),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 11),
        BuildShortHelpsItem::single(UserEvent::Confirm, "Open", 2),
        BuildShortHelpsItem::single(UserEvent::Reset, "Clear filter", 5),
        BuildShortHelpsItem::single(UserEvent::Expand, "Expand", 4),
        BuildShortHelpsItem::single(UserEvent::Insight, "Insight", 3),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 8),
        BuildShortHelpsItem::group(vec![UserEvent::Widen, UserEvent::Narrow], "Widen/Narrow", 10),
        BuildShortHelpsItem::single(UserEvent::Reload, "Reload", 9),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    #[rustfmt::skip]
    let attr_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Close", 1),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Scroll", 2),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 6),
        BuildShortHelpsItem::group(vec![UserEvent::ToggleWrap, UserEvent::ToggleNumber], "Toggle wrap/number", 5),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 3),
        BuildShortHelpsItem::single(UserEvent::Reload, "Reload", 4),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    (
        build_short_help_spans(table_helps, mapper),
        build_short_help_spans(table_filtered_helps, mapper),
        build_short_help_spans(attr_helps, mapper),
    )
}

impl TableView {
    fn render_expanded_item(&mut self, f: &mut Frame, area: Rect) {
        if let Some((x, y)) = self.table_state.selected_item_position() {
            let x = area.left() + x;
            let y = area.top() + y + 1; // +1 for header row
            let w = (self.attr_scroll_lines_state.max_width() as u16)
                .min(self.config.max_expand_width)
                + 2; // +2 for border
            let h = ((area.height - 1) / 2 - 1)
                .min(self.attr_scroll_lines_state.lines().len() as u16)
                .min(self.config.max_expand_height)
                + 2; // +2 for header row

            #[allow(clippy::collapsible_else_if)]
            let (left, top) = if x + w - 1 < area.right() {
                if y + h < area.bottom() {
                    (x - 1, y + 1)
                } else {
                    (x - 1, y - h)
                }
            } else {
                if y + h < area.bottom() {
                    (area.right() - w, y + 1)
                } else {
                    (area.right() - w, y - h)
                }
            };
            let popup_area = Rect::new(left, top, w, h);

            let scroll = ScrollLines::default()
                .block(
                    Block::bordered()
                        .border_set(border::DOUBLE)
                        .fg(self.theme.fg)
                        .bg(self.theme.bg),
                )
                .theme(&self.theme);
            f.render_widget(Clear, popup_area);
            f.render_stateful_widget(scroll, popup_area, &mut self.attr_scroll_lines_state);
        }
    }
}

impl TableView {
    fn open_item(&self) {
        if let Some(item) = self.items.get(self.table_state.selected_row) {
            let desc = self.table_description.clone();
            let item = item.clone();
            self.tx.send(AppEvent::OpenItem(desc, item));
        }
    }

    fn open_table_insight(&self) {
        let insight = TableInsight::new(&self.table_description, &self.items);
        self.tx.send(AppEvent::OpenTableInsight(insight));
    }

    fn open_expand_selected_attr(&mut self) {
        if let Some(col) = self.table_state.selected_col {
            let selected_item = &self.items[self.table_state.selected_row];
            let schema = &self.table_description.key_schema_type;
            let key = &list_attribute_keys(&self.items, schema)[col];
            if let Some(attr) = selected_item.attributes.get(key) {
                let lines = get_raw_json_attribute_lines(attr, &self.theme);
                let options = self.attr_scroll_lines_state.current_options();
                self.attr_scroll_lines_state = ScrollLinesState::new(lines, options);
                self.attr_expanded = true;
            }
        }
    }

    fn recalculate_cells(&mut self) {
        if let Some(col) = self.table_state.selected_col {
            let attribute_keys =
                list_attribute_keys(&self.items, &self.table_description.key_schema_type);
            let max_attribute_width = self.table_state.selected_col_width().unwrap();
            for (i, cell_items) in self.row_cell_items.iter_mut().enumerate() {
                let item = &self.items[i];
                let key = &attribute_keys[col];
                let (cell_item, _) = item
                    .attributes
                    .get(key)
                    .map(|attr| attribute_to_cell_item(attr, max_attribute_width, &self.theme))
                    .unwrap_or(undefined_cell_item(&self.theme));
                cell_items[col] = cell_item;
            }
        }
    }

    fn close_expand_selected_attr(&mut self) {
        self.attr_expanded = false;
    }

    fn reload_table(&self) {
        let desc = self.table_description.clone();
        self.tx.send(AppEvent::LoadTableItems(desc));
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
                let orig_idx = self
                    .view_indices
                    .get(self.table_state.selected_row)
                    .cloned();
                let before_offset_idx = self.table_state.selected_row_offset_index();
                self.filter_view_indices();
                if let Some(orig_idx) = orig_idx {
                    self.table_state.select_index(orig_idx, before_offset_idx);
                    self.table_state.update_table_state();
                }
                self.tx.send(AppEvent::ClearStatus);
            }
            FilterState::None => {}
        }
    }

    fn filter_view_indices(&mut self) {
        let query = self.filter_input.value();
        self.view_indices = if query.is_empty() {
            (0..self.items.len()).collect()
        } else {
            self.row_cell_items
                .iter()
                .enumerate()
                .filter(|(_, cell_items)| {
                    cell_items
                        .iter()
                        .any(|cell_item| !cell_item.matched_indices(query).is_empty())
                })
                .map(|(i, _)| i)
                .collect()
        };
        self.table_state = self
            .table_state
            .with_new_total_rows(self.view_indices.len());
    }

    fn copy_to_clipboard(&self) {
        let selected_item = &self.items[self.table_state.selected_row];
        let schema = &self.table_description.key_schema_type;

        let (name, content) = if let Some(col) = self.table_state.selected_col {
            let key = &list_attribute_keys(&self.items, schema)[col];
            if let Some(attr) = selected_item.attributes.get(key) {
                if self.attr_expanded {
                    ("selected attribute", get_raw_json_attribute_string(attr))
                } else {
                    ("selected attribute", attr.to_simple_string())
                }
            } else {
                return;
            }
        } else {
            let raw_json_string = get_raw_json_string(selected_item, schema);
            ("selected item", raw_json_string)
        };

        self.tx
            .send(AppEvent::CopyToClipboard(name.into(), content));
    }

    fn open_help(&self) {
        if self.attr_expanded {
            self.tx.send(AppEvent::OpenHelp(self.helps.attr.clone()))
        } else {
            match self.filter_state {
                FilterState::None => {
                    self.tx.send(AppEvent::OpenHelp(self.helps.table.clone()));
                }
                FilterState::Filtering | FilterState::Filtered => {
                    self.tx
                        .send(AppEvent::OpenHelp(self.helps.table_filtered.clone()));
                }
            }
        }
    }
}

fn new_table_state(
    table_description: &TableDescription,
    items: &[Item],
    config: &UiTableConfig,
    theme: ColorTheme,
) -> (TableState, Vec<Vec<CellItem<'static>>>, Vec<Cell<'static>>) {
    let attribute_keys = list_attribute_keys(items, &table_description.key_schema_type);
    let total_rows = items.len();
    let total_cols = attribute_keys.len();

    let mut max_width_vec: Vec<usize> = vec![0; total_cols];

    let mut row_cell_items: Vec<Vec<CellItem>> = Vec::with_capacity(total_rows);
    for item in items {
        let mut cell_items: Vec<CellItem> = Vec::new();
        for (i, key) in attribute_keys.iter().enumerate() {
            let (cell_item, width) = item
                .attributes
                .get(key)
                .map(|attr| attribute_to_cell_item(attr, config.max_attribute_width, &theme))
                .unwrap_or(undefined_cell_item(&theme));
            cell_items.push(cell_item);

            if width > max_width_vec[i] {
                max_width_vec[i] = width;
            }
        }
        row_cell_items.push(cell_items);
    }

    let mut header_row_cells: Vec<Cell> = Vec::with_capacity(total_cols);
    for (i, key) in attribute_keys.iter().enumerate() {
        let (cell, width) = key_to_cell(key, config, &theme);
        header_row_cells.push(cell);
        if width > max_width_vec[i] {
            max_width_vec[i] = width;
        }
    }

    let table_state = TableState::new(total_rows, total_cols, max_width_vec);

    (table_state, row_cell_items, header_row_cells)
}

fn attribute_to_cell_item(
    attr: &Attribute,
    max_attribute_width: usize,
    theme: &ColorTheme,
) -> (CellItem<'static>, usize) {
    let spans = attribute_to_spans(attr, theme);
    let plain = spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();
    let cut_spans = cut_spans_by_width(spans, max_attribute_width, ELLIPSIS, theme);
    let width = cut_spans.iter().map(Span::width).sum();
    let plain_width = console::measure_text_width(&plain);
    (CellItem::new(cut_spans, plain, plain_width), width)
}

fn key_to_cell(key: &str, config: &UiTableConfig, theme: &ColorTheme) -> (Cell<'static>, usize) {
    let span = key.to_string().bold();
    let spans = cut_spans_by_width(vec![span], config.max_attribute_width, ELLIPSIS, theme);
    let line = Line::from(spans);
    let width = line.width();
    (Cell::new(line), width)
}

fn undefined_cell_item(theme: &ColorTheme) -> (CellItem<'static>, usize) {
    let s = "-";
    let content = vec![s.fg(theme.cell_undefined_fg)];
    (CellItem::new(content, s, 1), 1)
}

fn get_raw_json_string(item: &Item, schema: &KeySchemaType) -> String {
    let json_item = RawJsonItem::new(item, schema);
    serde_json::to_string(&json_item).unwrap()
}

fn get_raw_json_attribute_string(attr: &Attribute) -> String {
    let wrapper = RawAttributeJsonWrapper::new(attr);
    serde_json::to_string_pretty(&wrapper).unwrap()
}

fn get_raw_json_attribute_lines(attr: &Attribute, theme: &ColorTheme) -> Vec<Line<'static>> {
    let json_str = get_raw_json_attribute_string(attr);
    to_highlighted_lines(&json_str, theme)
}

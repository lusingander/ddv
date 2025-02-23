use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Margin, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Cell, Clear},
    Frame,
};

use crate::{
    color::ColorTheme,
    constant::APP_NAME,
    data::{
        list_attribute_keys, Attribute, Item, KeySchemaType, RawAttributeJsonWrapper, RawJsonItem,
        TableDescription, TableInsight,
    },
    event::{AppEvent, Sender, UserEvent, UserEventMapper},
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    view::common::{attribute_to_spans, cut_spans_by_width, to_highlighted_lines},
    widget::{ScrollLines, ScrollLinesOptions, ScrollLinesState, Table, TableState},
};

const MAX_ATTRIBUTE_ITEM_WIDTH: usize = 30;
const ELLIPSIS: &str = "...";

const EXPANDED_POPUP_WIDTH: u16 = 35;
const EXPANDED_POPUP_HEIGHT: u16 = 6;

pub struct TableView {
    table_description: TableDescription,
    items: Vec<Item>,

    table_helps: Vec<Spans>,
    attr_helps: Vec<Spans>,
    table_short_helps: Vec<SpansWithPriority>,
    attr_short_helps: Vec<SpansWithPriority>,
    theme: ColorTheme,
    tx: Sender,

    row_cells: Vec<Vec<Cell<'static>>>,
    header_row_cells: Vec<Cell<'static>>,
    table_state: TableState,
    attr_expanded: bool,
    attr_scroll_lines_state: ScrollLinesState,
}

impl TableView {
    pub fn new(
        table_description: TableDescription,
        items: Vec<Item>,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        let (table_state, row_cells, header_row_cells) =
            new_table_state(&table_description, &items, theme);
        let (table_helps, attr_helps) = build_helps(mapper, theme);
        let (table_short_helps, attr_short_helps) = build_short_helps(mapper);
        let attr_scroll_lines_state =
            ScrollLinesState::new(vec![], ScrollLinesOptions::new(false, false));

        TableView {
            table_description,
            items,

            table_helps,
            attr_helps,
            table_short_helps,
            attr_short_helps,
            theme,
            tx,

            row_cells,
            header_row_cells,
            table_state,
            attr_expanded: false,
            attr_scroll_lines_state,
        }
    }
}

impl TableView {
    pub fn handle_user_key_event(&mut self, user_event: Option<UserEvent>, _key_event: KeyEvent) {
        if let Some(user_event) = user_event {
            if self.attr_expanded {
                match user_event {
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
                    UserEvent::CopyToClipboard => {
                        self.copy_to_clipboard();
                    }
                    UserEvent::Help => {
                        self.open_help();
                    }
                    _ => {}
                }
            } else {
                match user_event {
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
                    UserEvent::Confirm => {
                        self.open_item();
                    }
                    UserEvent::Insight => {
                        self.open_table_insight();
                    }
                    UserEvent::Expand => {
                        self.open_expand_selected_attr();
                    }
                    UserEvent::CopyToClipboard => {
                        self.copy_to_clipboard();
                    }
                    UserEvent::Help => {
                        self.open_help();
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = format!(" {} - {} ", APP_NAME, self.table_description.table_name);
        let count = self.table_state.selected_count_string();
        let block = Block::bordered()
            .title_top(Line::from(title).left_aligned())
            .title_top(Line::from(count).right_aligned())
            .fg(self.theme.fg)
            .bg(self.theme.bg);
        f.render_widget(block, area);

        let table_area = area.inner(Margin::new(2, 1));
        let table = Table::new(&self.row_cells, &self.header_row_cells).theme(&self.theme);
        f.render_stateful_widget(table, table_area, &mut self.table_state);

        if self.attr_expanded {
            self.render_expanded_item(f, table_area);
        }
    }

    pub fn short_helps(&self) -> &[SpansWithPriority] {
        if self.attr_expanded {
            &self.attr_short_helps
        } else {
            &self.table_short_helps
        }
    }
}

fn build_helps(mapper: &UserEventMapper, theme: ColorTheme) -> (Vec<Spans>, Vec<Spans>) {
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
        BuildHelpsItem::new(UserEvent::Insight, "Open table insight"),
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
        BuildHelpsItem::new(UserEvent::CopyToClipboard, "Copy selected item"),
    ];
    (
        build_help_spans(table_helps, mapper, theme),
        build_help_spans(attr_helps, mapper, theme),
    )
}

fn build_short_helps(mapper: &UserEventMapper) -> (Vec<SpansWithPriority>, Vec<SpansWithPriority>) {
    #[rustfmt::skip]
    let table_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Back", 1),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Select row", 4),
        BuildShortHelpsItem::group(vec![UserEvent::Left, UserEvent::Right], "Select col", 5),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 7),
        BuildShortHelpsItem::single(UserEvent::Confirm, "Open", 2),
        BuildShortHelpsItem::single(UserEvent::Insight, "Insight", 3),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 6),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    #[rustfmt::skip]
    let attr_helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Close", 1),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Scroll", 2),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 4),
        BuildShortHelpsItem::group(vec![UserEvent::ToggleWrap, UserEvent::ToggleNumber], "Toggle wrap/number", 5),
        BuildShortHelpsItem::single(UserEvent::CopyToClipboard, "Copy", 3),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    (
        build_short_help_spans(table_helps, mapper),
        build_short_help_spans(attr_helps, mapper),
    )
}

impl TableView {
    fn render_expanded_item(&mut self, f: &mut Frame, area: Rect) {
        if let Some((x, y)) = self.table_state.selected_item_position() {
            let x = area.left() + x;
            let y = area.top() + y + 1; // +1 for header row
            let (w, h) = (EXPANDED_POPUP_WIDTH + 2, EXPANDED_POPUP_HEIGHT + 2); // +2 for border

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
                let lines = get_raw_json_attribute_lines(attr);
                let options = self.attr_scroll_lines_state.current_options();
                self.attr_scroll_lines_state = ScrollLinesState::new(lines, options);
                self.attr_expanded = true;
            }
        }
    }

    fn close_expand_selected_attr(&mut self) {
        self.attr_expanded = false;
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
            self.tx.send(AppEvent::OpenHelp(self.attr_helps.clone()))
        } else {
            self.tx.send(AppEvent::OpenHelp(self.table_helps.clone()))
        }
    }
}

fn new_table_state(
    table_description: &TableDescription,
    items: &[Item],
    theme: ColorTheme,
) -> (TableState, Vec<Vec<Cell<'static>>>, Vec<Cell<'static>>) {
    let attribute_keys = list_attribute_keys(items, &table_description.key_schema_type);
    let total_rows = items.len();
    let total_cols = attribute_keys.len();

    let mut max_width_vec: Vec<usize> = vec![0; total_cols];

    let mut row_cells: Vec<Vec<Cell>> = Vec::with_capacity(total_rows);
    for item in items {
        let mut cells: Vec<Cell> = Vec::new();
        for (i, key) in attribute_keys.iter().enumerate() {
            let (cell, width) = item
                .attributes
                .get(key)
                .map(|attr| attribute_to_cell(attr, &theme))
                .unwrap_or(undefined_cell(&theme));
            cells.push(cell);

            if width > max_width_vec[i] {
                max_width_vec[i] = width;
            }
        }
        row_cells.push(cells);
    }

    let mut header_row_cells: Vec<Cell> = Vec::with_capacity(total_cols);
    for (i, key) in attribute_keys.iter().enumerate() {
        let (cell, width) = key_to_cell(key, &theme);
        header_row_cells.push(cell);
        if width > max_width_vec[i] {
            max_width_vec[i] = width;
        }
    }

    let table_state = TableState::new(total_rows, total_cols, max_width_vec);

    (table_state, row_cells, header_row_cells)
}

fn attribute_to_cell(attr: &Attribute, theme: &ColorTheme) -> (Cell<'static>, usize) {
    let spans = attribute_to_spans(attr, theme);
    let spans = cut_spans_by_width(spans, MAX_ATTRIBUTE_ITEM_WIDTH, ELLIPSIS, theme);
    let line = Line::from(spans);
    let width = line.width();
    (Cell::new(line), width)
}

fn key_to_cell(key: &str, theme: &ColorTheme) -> (Cell<'static>, usize) {
    let span = key.to_string().bold();
    let spans = cut_spans_by_width(vec![span], MAX_ATTRIBUTE_ITEM_WIDTH, ELLIPSIS, theme);
    let line = Line::from(spans);
    let width = line.width();
    (Cell::new(line), width)
}

fn undefined_cell(theme: &ColorTheme) -> (Cell<'static>, usize) {
    (Cell::new("-").fg(theme.cell_undefined_fg), 1)
}

fn get_raw_json_string(item: &Item, schema: &KeySchemaType) -> String {
    let json_item = RawJsonItem::new(item, schema);
    serde_json::to_string(&json_item).unwrap()
}

fn get_raw_json_attribute_string(attr: &Attribute) -> String {
    let wrapper = RawAttributeJsonWrapper::new(attr);
    serde_json::to_string_pretty(&wrapper).unwrap()
}

fn get_raw_json_attribute_lines(attr: &Attribute) -> Vec<Line<'static>> {
    let json_str = get_raw_json_attribute_string(attr);
    to_highlighted_lines(&json_str)
}

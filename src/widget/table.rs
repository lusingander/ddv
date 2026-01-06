use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Cell, Row, StatefulWidget, Table as RatatuiTable, TableState as RatatuiTableState},
};

use crate::color::ColorTheme;

pub struct TableState {
    pub selected_row: usize,
    pub selected_col: Option<usize>,
    offset_row: usize,
    offset_col: usize,
    total_rows: usize,
    total_cols: usize,
    width: usize,
    height: usize,
    col_widths: Vec<usize>,

    ratatui_table_state: RatatuiTableState,
}

impl TableState {
    pub fn new(total_rows: usize, total_cols: usize, col_widths: Vec<usize>) -> TableState {
        let ratatui_table_state = RatatuiTableState::new()
            .with_selected(Some(0))
            .with_selected_column(None);

        TableState {
            selected_row: 0,
            selected_col: None,
            offset_row: 0,
            offset_col: 0,
            total_rows,
            total_cols,
            width: 0,
            height: 0,
            col_widths,

            ratatui_table_state,
        }
    }

    pub fn with_new_total_rows(&self, total_rows: usize) -> TableState {
        TableState {
            selected_row: 0,
            selected_col: self.selected_col,
            offset_row: 0,
            offset_col: self.offset_col,
            total_rows,
            total_cols: self.total_cols,
            width: self.width,
            height: self.height,
            col_widths: self.col_widths.clone(),

            ratatui_table_state: self.ratatui_table_state.with_selected(Some(0)),
        }
    }

    pub fn select_next_row(&mut self) {
        if self.total_rows == 0 {
            return;
        }
        if self.selected_row >= self.total_rows - 1 {
            self.select_first_row();
        } else {
            if self.selected_row - self.offset_row == self.height - 1 {
                self.offset_row += 1;
            }
            self.selected_row += 1;
        }
    }

    pub fn select_prev_row(&mut self) {
        if self.total_rows == 0 {
            return;
        }
        if self.selected_row == 0 {
            self.select_last_row();
        } else {
            if self.selected_row - self.offset_row == 0 {
                self.offset_row -= 1;
            }
            self.selected_row -= 1;
        }
    }

    pub fn select_next_row_page(&mut self) {
        if self.total_rows == 0 {
            return;
        }
        if self.total_rows < self.height {
            self.selected_row = self.total_rows - 1;
            self.offset_row = 0;
        } else if self.selected_row + self.height < self.total_rows - 1 {
            self.selected_row += self.height;
            if self.selected_row + self.height > self.total_rows - 1 {
                self.offset_row = self.total_rows - self.height;
            } else {
                self.offset_row = self.selected_row;
            }
        } else {
            self.selected_row = self.total_rows - 1;
            self.offset_row = self.total_rows - self.height;
        }
    }

    pub fn select_prev_row_page(&mut self) {
        if self.total_rows == 0 {
            return;
        }
        if self.total_rows < self.height {
            self.selected_row = 0;
            self.offset_row = 0;
        } else if self.selected_row > self.height {
            self.selected_row -= self.height;
            if self.selected_row < self.height {
                self.offset_row = 0;
            } else {
                self.offset_row = self.selected_row - self.height + 1;
            }
        } else {
            self.selected_row = 0;
            self.offset_row = 0;
        }
    }

    pub fn select_first_row(&mut self) {
        if self.total_rows == 0 {
            return;
        }
        self.selected_row = 0;
        self.offset_row = 0;
    }

    pub fn select_last_row(&mut self) {
        if self.total_rows == 0 {
            return;
        }
        self.selected_row = self.total_rows - 1;
        if self.height < self.total_rows {
            self.offset_row = self.total_rows - self.height;
        }
    }

    pub fn select_next_col(&mut self) {
        if self.total_cols == 0 {
            return;
        }
        if let Some(selected_col) = self.selected_col {
            let updated_selected_col = selected_col + 1;
            if selected_col < self.total_cols - 1 {
                self.selected_col = Some(updated_selected_col);
            }
            loop {
                if updated_selected_col == self.offset_col {
                    break;
                }
                let sum = self
                    .col_widths
                    .iter()
                    .enumerate()
                    .skip(self.offset_col)
                    .take_while(|(i, _)| *i <= updated_selected_col)
                    .map(|(_, w)| *w + 1) // +1 for a space between column)
                    .sum::<usize>();
                if sum < self.width {
                    break;
                }
                self.offset_col += 1;
            }
        } else {
            self.selected_col = Some(0);
        }
    }

    pub fn select_prev_col(&mut self) {
        if self.total_cols == 0 {
            return;
        }
        if let Some(selected_col) = self.selected_col {
            if selected_col > 0 {
                if selected_col == self.offset_col {
                    self.offset_col -= 1;
                }
                self.selected_col = Some(selected_col - 1);
            } else {
                self.selected_col = None;
            }
        }
    }

    pub fn select_first_col(&mut self) {
        if self.total_cols == 0 {
            return;
        }
        self.selected_col = Some(0);
        self.offset_col = 0;
    }

    pub fn select_last_col(&mut self) {
        if self.total_cols == 0 {
            return;
        }
        self.selected_col = Some(self.total_cols - 1);
        let mut sum = 0;
        let mut count = 0;
        for w in self.col_widths.iter().rev() {
            sum += w + 1; // +1 for a space between columns
            if sum > self.width {
                break;
            }
            count += 1;
        }
        self.offset_col = self.total_cols - count;
    }

    pub fn update_table_state(&mut self) {
        let row = self.selected_row - self.offset_row;
        if let Some(col) = self.selected_col {
            let col = col - self.offset_col;
            self.ratatui_table_state.select_cell(Some((row, col)));
        } else {
            self.ratatui_table_state.select(Some(row));
            self.ratatui_table_state.select_column(None);
        }
    }

    pub fn selected_count_string(&self) -> String {
        if self.total_rows == 0 {
            return "".to_string();
        }
        format!(" {} / {} ", self.selected_row + 1, self.total_rows)
    }

    pub fn selected_item_position(&self) -> Option<(u16, u16)> {
        self.selected_col.map(|col| {
            let x = self
                .col_widths
                .iter()
                .skip(self.offset_col)
                .take(col - self.offset_col)
                .map(|w| *w + 1)
                .sum::<usize>();
            let y = self.selected_row - self.offset_row;
            (x as u16, y as u16)
        })
    }

    pub fn widen_col(&mut self) {
        if let Some(col) = self.selected_col {
            self.col_widths[col] += 1;
        }
    }

    pub fn narrow_col(&mut self) {
        if let Some(col) = self.selected_col {
            if self.col_widths[col] > 1 {
                self.col_widths[col] -= 1;
            }
        }
    }

    pub fn selected_col_width(&self) -> Option<usize> {
        self.selected_col.map(|col| self.col_widths[col])
    }
}

#[derive(Debug, Default)]
struct TableColor {
    selected_fg: Color,
    selected_bg: Color,
    selected_axis_bg: Color,
}

impl TableColor {
    fn new(theme: &ColorTheme) -> TableColor {
        TableColor {
            selected_fg: theme.selected_fg,
            selected_bg: theme.selected_bg,
            selected_axis_bg: theme.selected_axis_bg,
        }
    }
}
pub struct Table<'a> {
    row_cell_items: &'a [&'a Vec<CellItem<'static>>],
    header_row_cells: &'a [Cell<'static>],
    color: TableColor,
}

impl<'a> Table<'a> {
    pub fn new(
        row_cell_items: &'a [&'a Vec<CellItem<'static>>],
        header_row_cells: &'a [Cell<'static>],
    ) -> Table<'a> {
        Table {
            row_cell_items,
            header_row_cells,
            color: Default::default(),
        }
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = TableColor::new(theme);
        self
    }
}

impl StatefulWidget for Table<'_> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.width = area.width as usize;
        state.height = area.height as usize - 1 /* header */;

        let mut sum = 0;
        let mut count = 0;
        for w in state.col_widths.iter().skip(state.offset_col) {
            sum += w + 1; // +1 for a space between columns
            count += 1;
            if sum > state.width {
                break;
            }
        }

        let rows = self
            .row_cell_items
            .iter()
            .skip(state.offset_row)
            .take(state.height)
            .map(|cell_items| {
                Row::new(
                    cell_items
                        .iter()
                        .skip(state.offset_col)
                        .take(count)
                        .map(|cell_item| cell_item.cell()),
                )
            });
        let widths = state
            .col_widths
            .iter()
            .skip(state.offset_col)
            .take(count)
            .map(|w| Constraint::Length(*w as u16));
        let header_row = Row::new(
            self.header_row_cells
                .iter()
                .skip(state.offset_col)
                .take(count)
                .cloned(),
        );

        let table = RatatuiTable::new(rows, widths)
            .header(header_row)
            .flex(Flex::Legacy)
            .row_highlight_style(Style::default().bg(self.color.selected_axis_bg))
            .cell_highlight_style(
                Style::default()
                    .bg(self.color.selected_bg)
                    .fg(self.color.selected_fg),
            );

        StatefulWidget::render(table, area, buf, &mut state.ratatui_table_state);
    }
}

pub struct CellItem<'a> {
    content: Vec<Span<'a>>,
    plain: String,
}

impl<'a> CellItem<'a> {
    pub fn new(content: Vec<Span<'a>>, plain: impl Into<String>) -> Self {
        Self {
            content,
            plain: plain.into(),
        }
    }

    pub fn cell(&self) -> Cell<'a> {
        Cell::from(Line::from(self.content.clone()))
    }

    pub fn matched_index(&self, query: &str) -> Option<usize> {
        let lower_query = query.to_lowercase();
        let lower_plain = self.plain.to_lowercase();
        lower_plain.find(&lower_query)
    }
}

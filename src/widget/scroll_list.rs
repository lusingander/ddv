use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, List, ListItem, Padding, StatefulWidget, Widget},
};

use crate::{color::ColorTheme, widget::ScrollBar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollListState {
    pub selected: usize,
    pub offset: usize,
    total: usize,
    height: usize,
}

impl ScrollListState {
    pub fn new(total: usize) -> ScrollListState {
        ScrollListState {
            selected: 0,
            offset: 0,
            total,
            height: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.selected >= self.total - 1 {
            self.select_first();
        } else {
            if self.selected - self.offset == self.height - 1 {
                self.offset += 1;
            }
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.selected == 0 {
            self.select_last();
        } else {
            if self.selected - self.offset == 0 {
                self.offset -= 1;
            }
            self.selected -= 1;
        }
    }

    pub fn select_next_page(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.total < self.height {
            self.selected = self.total - 1;
            self.offset = 0;
        } else if self.selected + self.height < self.total - 1 {
            self.selected += self.height;
            if self.selected + self.height > self.total - 1 {
                self.offset = self.total - self.height;
            } else {
                self.offset = self.selected;
            }
        } else {
            self.selected = self.total - 1;
            self.offset = self.total - self.height;
        }
    }

    pub fn select_prev_page(&mut self) {
        if self.total == 0 {
            return;
        }
        if self.total < self.height {
            self.selected = 0;
            self.offset = 0;
        } else if self.selected > self.height {
            self.selected -= self.height;
            if self.selected < self.height {
                self.offset = 0;
            } else {
                self.offset = self.selected - self.height + 1;
            }
        } else {
            self.selected = 0;
            self.offset = 0;
        }
    }

    pub fn select_first(&mut self) {
        if self.total == 0 {
            return;
        }
        self.selected = 0;
        self.offset = 0;
    }

    pub fn select_last(&mut self) {
        if self.total == 0 {
            return;
        }
        self.selected = self.total - 1;
        if self.height < self.total {
            self.offset = self.total - self.height;
        }
    }
}

#[derive(Debug, Default)]
struct ScrollListColor {
    bg: Color,
    fg: Color,
    disabled_fg: Color,
}

impl ScrollListColor {
    fn new(theme: &ColorTheme) -> ScrollListColor {
        ScrollListColor {
            bg: theme.bg,
            fg: theme.fg,
            disabled_fg: theme.disabled,
        }
    }
}

#[derive(Debug)]
pub struct ScrollList<'a> {
    items: Vec<ListItem<'a>>,
    title: Option<String>,
    color: ScrollListColor,
    focused: bool,
}

impl ScrollList<'_> {
    pub fn new(items: Vec<ListItem>) -> ScrollList {
        ScrollList {
            items,
            title: None,
            color: Default::default(),
            focused: false,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn theme(mut self, theme: &ColorTheme) -> Self {
        self.color = ScrollListColor::new(theme);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl StatefulWidget for ScrollList<'_> {
    type State = ScrollListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.height = area.height as usize - 2 /* border */;

        let mut block = Block::bordered()
            .padding(Padding::horizontal(1))
            .bg(self.color.bg)
            .fg(self.color.fg);
        if let Some(title) = self.title {
            block = block.title(title).title_alignment(Alignment::Left);
        }
        if !self.focused {
            block = block.border_style(Style::default().fg(self.color.disabled_fg));
        }
        let list = List::new(self.items).block(block);
        Widget::render(list, area, buf);

        let area = area.inner(Margin::new(2, 1));
        let scrollbar_area = Rect::new(area.right(), area.top(), 1, area.height);

        if state.total > (area.height as usize) {
            let color = if self.focused {
                self.color.fg
            } else {
                self.color.disabled_fg
            };
            let scroll_bar = ScrollBar::new(state.total, state.offset).color(color);
            Widget::render(scroll_bar, scrollbar_area, buf);
        }
    }
}

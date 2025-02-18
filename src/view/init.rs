use ratatui::{crossterm::event::KeyEvent, layout::Rect, style::Stylize, widgets::Block, Frame};

use crate::{
    color::ColorTheme,
    event::{Sender, UserEvent},
    help::SpansWithPriority,
};

pub struct InitView {
    theme: ColorTheme,
    _tx: Sender,
}

impl InitView {
    pub fn new(theme: ColorTheme, tx: Sender) -> Self {
        InitView { theme, _tx: tx }
    }
}

impl InitView {
    pub fn handle_user_key_event(&mut self, _user_event: Option<UserEvent>, _key_event: KeyEvent) {}

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::bordered().fg(self.theme.fg).bg(self.theme.bg);
        f.render_widget(block, area);
    }

    pub fn short_helps(&self) -> &[SpansWithPriority] {
        &[]
    }
}

use ratatui::{
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use crate::{
    color::ColorTheme,
    constant::{APP_DESCRIPTION, APP_HOMEPAGE, APP_NAME, APP_VERSION},
    event::{AppEvent, Sender, UserEvent, UserEventMapper},
    help::{
        build_short_help_spans, group_spans_to_fit_width, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    widget::Divider,
};

pub struct HelpView {
    target_view_helps: Vec<Spans>,
    short_helps: Vec<SpansWithPriority>,
    theme: ColorTheme,
    tx: Sender,
}

impl HelpView {
    pub fn new(
        target_view_helps: Vec<Spans>,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        let short_helps = build_short_helps(mapper);

        HelpView {
            target_view_helps,

            short_helps,
            theme,
            tx,
        }
    }
}

impl HelpView {
    pub fn handle_user_key_event(&mut self, user_events: Vec<UserEvent>, _key_event: KeyEvent) {
        for user_event in &user_events {
            match user_event {
                UserEvent::Close => {
                    self.tx.send(AppEvent::BackToBeforeView);
                }
                UserEvent::Help => {
                    self.tx.send(AppEvent::BackToBeforeView);
                }
                _ => {
                    continue;
                }
            }
            break;
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title_top(Line::from(format!(" {} ", APP_NAME)).left_aligned())
            .padding(Padding::horizontal(1));

        let content_area = block.inner(area);

        let [about_area, divider_area, help_area] = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(content_area);

        f.render_widget(block, area);
        self.render_about(f, about_area);
        self.render_divider(f, divider_area);
        self.render_help(f, help_area);
    }

    pub fn short_helps(&self) -> &[SpansWithPriority] {
        &self.short_helps
    }
}

fn build_short_helps(mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
    #[rustfmt::skip]
    let helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Close help", 1),
    ];
    build_short_help_spans(helps, mapper)
}

impl HelpView {
    fn render_about(&self, f: &mut Frame, area: Rect) {
        let lines = vec![
            Line::from(format!("{} - {}", APP_NAME, APP_DESCRIPTION)),
            Line::from(format!("Version: {}", APP_VERSION)),
            Line::from(APP_HOMEPAGE.fg(self.theme.help_link_fg)),
        ];
        let content = with_empty_lines(lines);
        let paragraph =
            Paragraph::new(content).block(Block::default().padding(Padding::uniform(1)));
        f.render_widget(paragraph, area);
    }

    fn render_divider(&self, f: &mut Frame, area: Rect) {
        let divider = Divider::default().color(self.theme.divier_fg);
        f.render_widget(divider, area);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let max_width = area.width as usize - 2;
        let lines = with_empty_lines(
            group_spans_to_fit_width(&self.target_view_helps, max_width, "  ")
                .into_iter()
                .map(Line::from)
                .collect(),
        );
        let paragrah = Paragraph::new(lines).block(Block::default().padding(Padding::uniform(1)));
        f.render_widget(paragrah, area);
    }
}

fn with_empty_lines(lines: Vec<Line>) -> Vec<Line> {
    let n = lines.len();
    let mut ret = Vec::new();
    for (i, line) in lines.into_iter().enumerate() {
        ret.push(line);
        if i != n - 1 {
            ret.push(Line::raw(""));
        }
    }
    ret
}

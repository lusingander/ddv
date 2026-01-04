use ratatui::{
    crossterm::event::KeyEvent, layout::Rect, style::Stylize, text::Line, widgets::Block, Frame,
};

use crate::{
    color::ColorTheme,
    data::TableInsight,
    event::{AppEvent, Sender, UserEvent, UserEventMapper},
    handle_user_events,
    help::{
        build_help_spans, build_short_help_spans, BuildHelpsItem, BuildShortHelpsItem, Spans,
        SpansWithPriority,
    },
    widget::{ScrollLines, ScrollLinesOptions, ScrollLinesState},
};

pub struct TableInsightView {
    table_insight: TableInsight,

    helps: TableInsightViewHelps,
    theme: ColorTheme,
    tx: Sender,

    scroll_lines_state: ScrollLinesState,
}

struct TableInsightViewHelps {
    insight: Vec<Spans>,
    insight_short: Vec<SpansWithPriority>,
}

impl TableInsightView {
    pub fn new(
        table_insight: TableInsight,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        let lines = get_insight_lines(&table_insight, &theme);
        let scroll_lines_state =
            ScrollLinesState::new(lines, ScrollLinesOptions::new(false, false));
        let helps = TableInsightViewHelps::new(mapper, theme);

        TableInsightView {
            table_insight,

            helps,
            theme,
            tx,

            scroll_lines_state,
        }
    }
}

impl TableInsightView {
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
            UserEvent::ToggleWrap => {
                self.scroll_lines_state.toggle_wrap();
            }
            UserEvent::ToggleNumber => {
                self.scroll_lines_state.toggle_number();
            }
            UserEvent::Help => {
                self.open_help();
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = format!(" {} (Insights) ", self.table_insight.table_name);
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
        &self.helps.insight_short
    }
}

impl TableInsightViewHelps {
    fn new(mapper: &UserEventMapper, theme: ColorTheme) -> Self {
        let insight = build_helps(mapper, theme);
        let insight_short = build_short_helps(mapper);
        Self {
            insight,
            insight_short,
        }
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
        BuildHelpsItem::new(UserEvent::ToggleWrap, "Toggle wrap"),
        BuildHelpsItem::new(UserEvent::ToggleNumber, "Toggle number"),
    ];
    build_help_spans(helps, mapper, theme)
}

fn build_short_helps(mapper: &UserEventMapper) -> Vec<SpansWithPriority> {
    #[rustfmt::skip]
    let helps = vec![
        BuildShortHelpsItem::single(UserEvent::Quit, "Quit", 0),
        BuildShortHelpsItem::single(UserEvent::Close, "Back", 1),
        BuildShortHelpsItem::group(vec![UserEvent::Down, UserEvent::Up], "Scroll", 2),
        BuildShortHelpsItem::group(vec![UserEvent::GoToTop, UserEvent::GoToBottom], "Top/Bottom", 3),
        BuildShortHelpsItem::group(vec![UserEvent::ToggleWrap, UserEvent::ToggleNumber], "Toggle wrap/number", 4),
        BuildShortHelpsItem::single(UserEvent::Help, "Help", 0),
    ];
    build_short_help_spans(helps, mapper)
}

impl TableInsightView {
    fn open_help(&self) {
        self.tx.send(AppEvent::OpenHelp(self.helps.insight.clone()))
    }
}

fn get_insight_lines(table_insight: &TableInsight, theme: &ColorTheme) -> Vec<Line<'static>> {
    let mut lines = vec![];

    lines.push(Line::from("Attribute Distribution:".bold()));
    lines.push(Line::raw(""));

    let max_width = table_insight
        .attribute_distributions
        .iter()
        .map(|a| a.attribute_name.len())
        .max()
        .unwrap();

    for distribution in &table_insight.attribute_distributions {
        let mut spans = vec![];
        spans.push("  ".into());
        spans.push(format!("{:>width$}", distribution.attribute_name, width = max_width).bold());
        spans.push(" : ".bold());
        for (i, (at, n)) in distribution.distributions.iter().enumerate() {
            spans.push(at.as_str().to_string().fg(theme.insight_attribute_name_fg));
            spans.push(" ".into());
            spans.push(
                format_ratio(*n, table_insight.total_items).fg(theme.insight_attribute_value_fg),
            );
            if i < distribution.distributions.len() - 1 {
                spans.push(" ".into());
            }
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn format_ratio(n: usize, total: usize) -> String {
    let mut ratio = format!("{:.1}", (n as f64 / total as f64) * 100.0);
    if let Some(r) = ratio.strip_suffix(".0") {
        ratio = r.to_string()
    };
    format!("{ratio}%")
}

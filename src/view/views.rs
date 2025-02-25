use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    color::ColorTheme,
    config::UiTableListConfig,
    data::{Item, Table, TableDescription, TableInsight},
    event::{Sender, UserEvent, UserEventMapper},
    help::{Spans, SpansWithPriority},
    view::{
        help::HelpView, init::InitView, item::ItemView, table::TableView,
        table_insight::TableInsightView, table_list::TableListView,
    },
};

pub enum View {
    Init(Box<InitView>),
    TableList(Box<TableListView>),
    Table(Box<TableView>),
    Item(Box<ItemView>),
    TableInsight(Box<TableInsightView>),
    Help(Box<HelpView>),
}

impl View {
    pub fn handle_user_key_event(&mut self, user_event: Option<UserEvent>, key_event: KeyEvent) {
        match self {
            View::Init(view) => view.handle_user_key_event(user_event, key_event),
            View::TableList(view) => view.handle_user_key_event(user_event, key_event),
            View::Table(view) => view.handle_user_key_event(user_event, key_event),
            View::Item(view) => view.handle_user_key_event(user_event, key_event),
            View::TableInsight(view) => view.handle_user_key_event(user_event, key_event),
            View::Help(view) => view.handle_user_key_event(user_event, key_event),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self {
            View::Init(view) => view.render(f, area),
            View::TableList(view) => view.render(f, area),
            View::Table(view) => view.render(f, area),
            View::Item(view) => view.render(f, area),
            View::TableInsight(view) => view.render(f, area),
            View::Help(view) => view.render(f, area),
        }
    }

    pub fn short_helps(&self) -> &[SpansWithPriority] {
        match self {
            View::Init(view) => view.short_helps(),
            View::TableList(view) => view.short_helps(),
            View::Table(view) => view.short_helps(),
            View::Item(view) => view.short_helps(),
            View::TableInsight(view) => view.short_helps(),
            View::Help(view) => view.short_helps(),
        }
    }
}

impl View {
    pub fn of_init(theme: ColorTheme, tx: Sender) -> Self {
        View::Init(Box::new(InitView::new(theme, tx)))
    }

    pub fn of_table_list(
        tables: Vec<Table>,
        mapper: &UserEventMapper,
        config: UiTableListConfig,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        View::TableList(Box::new(TableListView::new(
            tables, mapper, config, theme, tx,
        )))
    }

    pub fn of_table(
        desc: TableDescription,
        items: Vec<Item>,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        View::Table(Box::new(TableView::new(desc, items, mapper, theme, tx)))
    }

    pub fn of_item(
        desc: TableDescription,
        item: Item,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        View::Item(Box::new(ItemView::new(desc, item, mapper, theme, tx)))
    }

    pub fn of_table_insight(
        insight: TableInsight,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        View::TableInsight(Box::new(TableInsightView::new(insight, mapper, theme, tx)))
    }

    pub fn of_help(
        target_view_helps: Vec<Spans>,
        mapper: &UserEventMapper,
        theme: ColorTheme,
        tx: Sender,
    ) -> Self {
        View::Help(Box::new(HelpView::new(
            target_view_helps,
            mapper,
            theme,
            tx,
        )))
    }
}

pub struct ViewStack {
    stack: Vec<View>,
}

impl ViewStack {
    pub fn new(view: View) -> Self {
        ViewStack { stack: vec![view] }
    }

    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn push(&mut self, view: View) {
        self.stack.push(view);
    }

    pub fn current_view(&self) -> &View {
        self.stack.last().unwrap()
    }

    pub fn current_view_mut(&mut self) -> &mut View {
        self.stack.last_mut().unwrap()
    }
}

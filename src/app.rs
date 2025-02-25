use std::sync::Arc;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::Backend,
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Padding, Paragraph},
    Frame, Terminal,
};
use tokio::spawn;

use crate::{
    client::Client,
    color::ColorTheme,
    config::Config,
    data::{Item, Table, TableDescription, TableInsight},
    error::{AppError, AppResult},
    event::{AppEvent, Receiver, Sender, UserEvent, UserEventMapper},
    help::{prune_spans_to_fit_width, Spans},
    view::{View, ViewStack},
    widget::LoadingDialog,
};

enum Status {
    None,
    NotificationSuccess(String),
    NotificationWarning(String),
    NotificationError(String),
}

pub struct App {
    view_stack: ViewStack,

    config: Config,
    theme: ColorTheme,
    mapper: UserEventMapper,

    status: Status,
    loading: bool,

    client: Arc<Client>,
    tx: Sender,
}

impl App {
    pub fn new(
        config: Config,
        theme: ColorTheme,
        mapper: UserEventMapper,
        client: Client,
        tx: Sender,
    ) -> Self {
        App {
            view_stack: ViewStack::new(View::of_init(theme, tx.clone())),
            config,
            theme,
            mapper,
            status: Status::None,
            loading: true,
            client: Arc::new(client),
            tx,
        }
    }
}

impl App {
    pub fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        rx: Receiver,
    ) -> std::io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;
            match rx.recv() {
                AppEvent::Key(key_event) => {
                    let user_event = self.mapper.find_first_event(key_event);

                    if let Some(UserEvent::Quit) = user_event {
                        return Ok(());
                    }

                    if self.loading {
                        // Ignore key inputs while loading (except quit)
                        continue;
                    }

                    match self.status {
                        Status::None => {
                            // do nothing
                        }
                        Status::NotificationSuccess(_) | Status::NotificationWarning(_) => {
                            // Clear message and pass key input as is
                            self.clear_status();
                        }
                        Status::NotificationError(_) => {
                            if matches!(self.view_stack.current_view(), View::Init(_)) {
                                return Ok(());
                            }
                            // Clear message and cancel key input
                            self.clear_status();
                            continue;
                        }
                    }

                    self.view_stack
                        .current_view_mut()
                        .handle_user_key_event(user_event, key_event);
                }
                AppEvent::Resize(w, h) => {
                    let _ = (w, h);
                }
                AppEvent::Initialize => {
                    self.initialize();
                }
                AppEvent::CompleteInitialize(result) => {
                    self.complete_initialize(result);
                }
                AppEvent::LoadTableDescription(table_name) => {
                    self.load_table_description(table_name);
                }
                AppEvent::CompleteLoadTableDescription(result) => {
                    self.complete_load_table_description(result);
                }
                AppEvent::LoadTableItems(desc) => {
                    self.load_table_items(desc);
                }
                AppEvent::CompleteLoadTableItems(desc, result) => {
                    self.complete_load_table_items(desc, result);
                }
                AppEvent::OpenItem(desc, item) => {
                    self.open_item(desc, item);
                }
                AppEvent::OpenTableInsight(insight) => {
                    self.open_table_insight(insight);
                }
                AppEvent::OpenHelp(helps) => {
                    self.open_help(helps);
                }
                AppEvent::BackToBeforeView => {
                    self.back_to_before_view();
                }
                AppEvent::CopyToClipboard(name, content) => {
                    self.copy_to_clipboard(name, content);
                }
                AppEvent::NotifySuccess(msg) => {
                    self.notify_success(msg);
                }
                AppEvent::NotifyWarning(msg) => {
                    self.notify_warning(msg);
                }
                AppEvent::NotifyError(msg) => {
                    self.notify_error(msg);
                }
            }
        }
    }
}

impl App {
    fn render(&mut self, f: &mut Frame) {
        let [view_area, status_line_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(f.area());

        self.view_stack.current_view_mut().render(f, view_area);
        self.render_status_line(f, status_line_area);
        self.render_loading_dialog(f);
    }

    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        let text: Line = match &self.status {
            Status::None => {
                let helps = self.view_stack.current_view().short_helps();
                let spans = prune_spans_to_fit_width(helps, area.width as usize - 2, ", "); // -2 for padding
                Line::from(spans).fg(self.theme.short_help)
            }
            Status::NotificationSuccess(msg) => Line::from(
                msg.as_str()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.theme.notification_success),
            ),
            Status::NotificationWarning(msg) => Line::from(
                msg.as_str()
                    .add_modifier(Modifier::BOLD)
                    .fg(self.theme.notification_warning),
            ),
            Status::NotificationError(msg) => Line::from(
                format!("ERROR: {}", msg)
                    .add_modifier(Modifier::BOLD)
                    .fg(self.theme.notification_error),
            ),
        };
        let paragraph = Paragraph::new(text).block(
            Block::default()
                .style(Style::default().bg(self.theme.bg))
                .padding(Padding::horizontal(1)),
        );
        f.render_widget(paragraph, area);
    }

    fn render_loading_dialog(&self, f: &mut Frame) {
        if self.loading {
            let dialog = LoadingDialog::default().theme(self.theme);
            f.render_widget(dialog, f.area());
        }
    }
}

impl App {
    fn initialize(&self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        spawn(async move {
            let result = client.list_all_tables().await;
            tx.send(AppEvent::CompleteInitialize(result));
        });
    }

    fn complete_initialize(&mut self, result: AppResult<Vec<Table>>) {
        match result {
            Ok(tables) => {
                let view = View::of_table_list(
                    tables,
                    &self.mapper,
                    self.config.ui.table_list.clone(),
                    self.theme,
                    self.tx.clone(),
                );
                self.view_stack.pop();
                self.view_stack.push(view);
                // not update loading here
            }
            Err(e) => {
                self.tx.send(AppEvent::NotifyError(e));
                self.loading = false;
            }
        }
    }

    fn load_table_description(&mut self, name: String) {
        self.loading = true;
        let client = self.client.clone();
        let tx = self.tx.clone();
        spawn(async move {
            let result = client.describe_table(&name).await;
            tx.send(AppEvent::CompleteLoadTableDescription(result));
        });
    }

    fn complete_load_table_description(&mut self, result: AppResult<TableDescription>) {
        match result {
            Ok(desc) => {
                if let View::TableList(view) = self.view_stack.current_view_mut() {
                    view.set_table_description(desc);
                }
            }
            Err(e) => {
                self.tx.send(AppEvent::NotifyError(e));
            }
        }
        self.loading = false;
    }

    fn load_table_items(&mut self, desc: TableDescription) {
        self.loading = true;
        let client = self.client.clone();
        let tx = self.tx.clone();
        spawn(async move {
            let result = client
                .scan_all_items(&desc.table_name, &desc.key_schema_type)
                .await;
            tx.send(AppEvent::CompleteLoadTableItems(desc, result));
        });
    }

    fn complete_load_table_items(&mut self, desc: TableDescription, result: AppResult<Vec<Item>>) {
        match result {
            Ok(items) => {
                let view = View::of_table(desc, items, &self.mapper, self.theme, self.tx.clone());
                self.view_stack.push(view);
            }
            Err(e) => {
                self.tx.send(AppEvent::NotifyError(e));
            }
        }
        self.loading = false;
    }

    fn open_item(&mut self, desc: TableDescription, item: Item) {
        let view = View::of_item(desc, item, &self.mapper, self.theme, self.tx.clone());
        self.view_stack.push(view);
    }

    fn open_table_insight(&mut self, insight: TableInsight) {
        let view = View::of_table_insight(insight, &self.mapper, self.theme, self.tx.clone());
        self.view_stack.push(view);
    }

    fn open_help(&mut self, helps: Vec<Spans>) {
        let view = View::of_help(helps, &self.mapper, self.theme, self.tx.clone());
        self.view_stack.push(view);
    }

    fn back_to_before_view(&mut self) {
        self.view_stack.pop();
    }

    fn clear_status(&mut self) {
        self.status = Status::None;
    }

    fn copy_to_clipboard(&self, name: String, content: String) {
        match crate::util::copy_to_clipboard(&content) {
            Ok(_) => {
                let msg = format!("Copied {} to clipboard successfully", name);
                self.tx.send(AppEvent::NotifySuccess(msg));
            }
            Err(e) => {
                self.tx.send(AppEvent::NotifyError(e));
            }
        }
    }

    fn notify_success(&mut self, msg: String) {
        self.status = Status::NotificationSuccess(msg);
    }

    fn notify_warning(&mut self, e: AppError) {
        self.status = Status::NotificationWarning(e.msg);
    }

    fn notify_error(&mut self, e: AppError) {
        self.status = Status::NotificationError(e.msg);
    }
}

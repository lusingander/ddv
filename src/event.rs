use std::{sync::mpsc, thread};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    data::{Item, Table, TableDescription, TableInsight},
    error::{AppError, AppResult},
    help::Spans,
};

pub enum AppEvent {
    Key(KeyEvent),
    Resize(usize, usize),
    Initialize,
    CompleteInitialize(AppResult<Vec<Table>>),
    LoadTableDescription(String),
    CompleteLoadTableDescription(AppResult<TableDescription>),
    LoadTableItems(TableDescription),
    CompleteLoadTableItems(TableDescription, AppResult<Vec<Item>>),
    OpenItem(TableDescription, Item),
    OpenTableInsight(TableInsight),
    OpenHelp(Vec<Spans>),
    BackToBeforeView,
    CopyToClipboard(String, String),
    ClearStatus,
    UpdateStatusInput(String, Option<u16>),
    NotifySuccess(String),
    NotifyWarning(AppError),
    NotifyError(AppError),
}

#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<AppEvent>,
}

impl Sender {
    pub fn send(&self, event: AppEvent) {
        self.tx.send(event).unwrap();
    }
}

pub struct Receiver {
    rx: mpsc::Receiver<AppEvent>,
}

impl Receiver {
    pub fn recv(&self) -> AppEvent {
        self.rx.recv().unwrap()
    }
}

pub fn init() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::channel();
    let tx = Sender { tx };
    let rx = Receiver { rx };

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match ratatui::crossterm::event::read() {
            Ok(e) => match e {
                ratatui::crossterm::event::Event::Key(key) => {
                    event_tx.send(AppEvent::Key(key));
                }
                ratatui::crossterm::event::Event::Resize(w, h) => {
                    event_tx.send(AppEvent::Resize(w as usize, h as usize));
                }
                _ => {}
            },
            Err(e) => {
                panic!("Failed to read event: {e}");
            }
        }
    });

    (tx, rx)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEvent {
    Quit,
    Down,
    Up,
    Left,
    Right,
    GoToTop,
    GoToBottom,
    GoToLeft,
    GoToRight,
    PageDown,
    PageUp,
    ScrollDown,
    ScrollUp,
    Confirm,
    Close,
    QuickFilter,
    Reset,
    NextPane,
    NextPreview,
    PrevPreview,
    Insight,
    Expand,
    ToggleWrap,
    ToggleNumber,
    Widen,
    Narrow,
    Reload,
    CopyToClipboard,
    Help,
}

pub struct UserEventMapper {
    map: Vec<(KeyEvent, UserEvent)>,
}

impl UserEventMapper {
    pub fn new() -> Self {
        #[rustfmt::skip]
        let map = vec![
            (KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), UserEvent::Quit),
            (KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE), UserEvent::Down),
            (KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), UserEvent::Down),
            (KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE), UserEvent::Up),
            (KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), UserEvent::Up),
            (KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE), UserEvent::GoToTop),
            (KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE), UserEvent::GoToBottom),
            (KeyEvent::new(KeyCode::Char('^'), KeyModifiers::NONE), UserEvent::GoToLeft),
            (KeyEvent::new(KeyCode::Char('$'), KeyModifiers::NONE), UserEvent::GoToRight),
            (KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE), UserEvent::PageDown),
            (KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE), UserEvent::PageUp),
            (KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE), UserEvent::Right),
            (KeyEvent::new(KeyCode::Right, KeyModifiers::NONE), UserEvent::Right),
            (KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE), UserEvent::Left),
            (KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), UserEvent::Left),
            (KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL), UserEvent::ScrollDown),
            (KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL), UserEvent::ScrollUp),
            (KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), UserEvent::Confirm),
            (KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), UserEvent::Close),
            (KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL), UserEvent::Close),
            (KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE), UserEvent::QuickFilter),
            (KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), UserEvent::Reset),
            (KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE), UserEvent::NextPane),
            (KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE), UserEvent::NextPreview),
            (KeyEvent::new(KeyCode::Char('V'), KeyModifiers::SHIFT), UserEvent::PrevPreview),
            (KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE), UserEvent::Insight),
            (KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE), UserEvent::Expand),
            (KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE), UserEvent::ToggleWrap),
            (KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE), UserEvent::ToggleNumber),
            (KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE), UserEvent::Widen),
            (KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE), UserEvent::Narrow),
            (KeyEvent::new(KeyCode::Char('R'), KeyModifiers::NONE), UserEvent::Reload),
            (KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE), UserEvent::CopyToClipboard),
            (KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE), UserEvent::Help),
        ];
        UserEventMapper { map }
    }

    pub fn find_events(&self, e: KeyEvent) -> Vec<UserEvent> {
        self.map
            .iter()
            .filter_map(|(k, v)| if *k == e { Some(*v) } else { None })
            .collect()
    }

    pub fn find_keys(&self, e: UserEvent) -> Vec<KeyEvent> {
        self.map
            .iter()
            .filter_map(|(k, v)| if *v == e { Some(*k) } else { None })
            .collect()
    }

    pub fn find_first_key(&self, e: UserEvent) -> Option<KeyEvent> {
        self.map
            .iter()
            .find_map(|(k, v)| if *v == e { Some(*k) } else { None })
    }
}

pub fn key_event_to_string(key: KeyEvent, short: bool) -> String {
    if let KeyCode::Char(c) = key.code {
        if key.modifiers == KeyModifiers::SHIFT {
            return c.to_ascii_uppercase().into();
        }
    }

    let char;
    let key_code = match key.code {
        KeyCode::Backspace => {
            if short {
                "BS"
            } else {
                "Backspace"
            }
        }
        KeyCode::Enter => "Enter",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::Tab => "Tab",
        KeyCode::BackTab => "BackTab",
        KeyCode::Delete => {
            if short {
                "Del"
            } else {
                "Delete"
            }
        }
        KeyCode::Insert => {
            if short {
                "Ins"
            } else {
                "Insert"
            }
        }
        KeyCode::F(n) => {
            char = format!("F{n}");
            &char
        }
        KeyCode::Char(' ') => "Space",
        KeyCode::Char(c) => {
            char = c.to_string();
            &char
        }
        KeyCode::Esc => "Esc",
        KeyCode::Null => "",
        KeyCode::CapsLock => "",
        KeyCode::Menu => "",
        KeyCode::ScrollLock => "",
        KeyCode::Media(_) => "",
        KeyCode::NumLock => "",
        KeyCode::PrintScreen => "",
        KeyCode::Pause => "",
        KeyCode::KeypadBegin => "",
        KeyCode::Modifier(_) => "",
    };

    let mut modifiers = Vec::with_capacity(3);

    if key.modifiers.intersects(KeyModifiers::CONTROL) {
        if short {
            modifiers.push("C");
        } else {
            modifiers.push("Ctrl");
        }
    }

    if key.modifiers.intersects(KeyModifiers::SHIFT) {
        if short {
            modifiers.push("S");
        } else {
            modifiers.push("Shift");
        }
    }

    if key.modifiers.intersects(KeyModifiers::ALT) {
        if short {
            modifiers.push("A");
        } else {
            modifiers.push("Alt");
        }
    }

    let mut key = modifiers.join("-");

    if !key.is_empty() {
        key.push('-');
    }
    key.push_str(key_code);

    key
}

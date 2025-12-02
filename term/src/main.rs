use std::{
    io,
    sync::mpsc::{self, Receiver, Sender, channel},
    time::Duration,
};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

use crate::{
    backend::worker::{FromWorkerMsg, ToWorkerMessageContents, worker_thread},
    gui::{
        account::Account, formbutton::FormButton, ledger::Ledger, login::Login, waiting::Waiting,
    },
};

pub mod backend;
pub mod gui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppElement {
    Account,
    Sender,
    Receiver,
    Money,
    Submit,
    Logout,
    Ledger,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Waiting,
    Login,
    Active,
    Exit,
}

// #[derive(Debug)]
pub struct App {
    exit: bool,
    login: Login,
    account: Account,
    state: AppState,

    ledger: Ledger,
    waiting: Waiting,

    // Buttons
    sender_button: FormButton,
    receiver_button: FormButton,
    money_button: FormButton,
    submit_button: FormButton,
    logout_button: FormButton,

    highlighted: AppElement,
    selected: Option<AppElement>,

    to_worker_channel: Sender<ToWorkerMessageContents>,
    from_worker_channel: Receiver<FromWorkerMsg>,
}

impl App {
    pub fn new() -> Self {
        let (to_worker, from_master) = mpsc::channel();

        let (to_master, from_worker) = channel();

        std::thread::spawn(move || worker_thread(from_master, to_master));

        Self {
            exit: false,
            account: Account::default(),
            state: AppState::Waiting,
            login: Login::default(),
            waiting: Waiting::default(),
            ledger: Ledger::new(),
            sender_button: FormButton::locked("Sender", "Calgary"),
            receiver_button: FormButton::cycle("Receiver", &["Calgary",  "New York", "Signapore",]),
            money_button: FormButton::money("Money", 250),
            submit_button: FormButton::clickable("Submit Transaction"),
            logout_button: FormButton::clickable("Logout"),
            highlighted: AppElement::Account,
            selected: None,
            to_worker_channel: to_worker,
            from_worker_channel: from_worker,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.login.tick += 1;
            if self.login.tick == 1000 {
                self.login.tick = 0;
            }

            self.waiting.tick += 1;
            if self.waiting.tick == 1000 {
                self.waiting.tick = 0;
            }

            self.unpack_messages();
            self.refresh_selection();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn unpack_messages(&mut self) {
        while let Ok(msg) = self.from_worker_channel.try_recv() {
            match msg {
                FromWorkerMsg::ConnectionLive => {
                    if self.state == AppState::Waiting {
                        // Move to the login screen.
                        self.state = AppState::Login;
                    }
                    // self.state = AppState::Login;
                }
                FromWorkerMsg::ConnectionDead => {
                    self.state = AppState::Waiting;
                }
                FromWorkerMsg::LoggedIn => {
                    self.state = AppState::Active;
                }
                FromWorkerMsg::LoginUnlock => {
                    self.login.unlock();
                }
                FromWorkerMsg::Balance(bal) => {
                    self.account.set_balance(bal);
                }
            }
        }
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn refresh_selection(&mut self) {
        self.account.reset();
        self.sender_button.reset();
        self.receiver_button.reset();
        self.money_button.reset();
        self.submit_button.reset();
        self.logout_button.reset();
        self.ledger.reset();

        if let Some(focused) = &self.selected {
            match focused {
                AppElement::Account => {
                    self.account.set_focused(true);
                }
                AppElement::Logout => {
                    self.logout_button.set_focused(true);
                }
                AppElement::Receiver => {
                    self.receiver_button.set_focused(true);
                }
                AppElement::Money => {
                    self.money_button.set_focused(true);
                }
                AppElement::Submit => {
                    self.submit_button.set_focused(true);
                }
                AppElement::Sender => {
                    self.sender_button.set_focused(true);
                }
                AppElement::Ledger => {
                    self.ledger.set_focused(true);
                }
            }
        } else {
            match self.highlighted {
                AppElement::Account => {
                    self.account.set_selected(true);
                }
                AppElement::Logout => {
                    self.logout_button.set_selected(true);
                }
                AppElement::Receiver => {
                    self.receiver_button.set_selected(true);
                }
                AppElement::Money => {
                    self.money_button.set_selected(true);
                }
                AppElement::Submit => {
                    self.submit_button.set_selected(true);
                }
                AppElement::Sender => {
                    self.sender_button.set_selected(true);
                }
                AppElement::Ledger => {
                    self.ledger.set_selected(true);
                }
            }
        }

        if self.selected == Some(AppElement::Logout) {
            self.selected = None;
            self.state = AppState::Waiting;
        }

        if self.selected == Some(AppElement::Submit) {
            self.selected = None;
            let _ = self.to_worker_channel.send(ToWorkerMessageContents::Trade { sender: 0, receiver: self.receiver_button.get_position().unwrap(), money: self.money_button.get_money().unwrap() });
        }

        // self.account.set_focused(true);
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        self.refresh_selection();

        if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit = true;
        } else if event.code == KeyCode::Char('q') {
            self.exit = true;
        }

        if self.state == AppState::Login {
            self.login.handle_key_event(event, &mut self.to_worker_channel);
        } else {
            if [
                KeyCode::Left,
                KeyCode::Right,
                KeyCode::Up,
                KeyCode::Down,
                KeyCode::Enter,
            ]
            .contains(&event.code)
            {
                if self.selected.is_none() {
                    self.handle_browse(event);
                } else if event.code == KeyCode::Enter {
                    self.selected = None;
                } else {
                    // We send the key input to the element.
                    self.redirect_key_input(event);
                }
            }
        }
        // println!("Event: {event:?}");
    }
    fn redirect_key_input(&mut self, event: KeyEvent) {
        if let Some(selected) = self.selected {
            match selected {
                AppElement::Account => {
                    // Nothing to handle.
                }
                AppElement::Ledger => {}
                AppElement::Money => {}
                AppElement::Receiver => self.receiver_button.handle_key_event(event),
                AppElement::Submit => {}
                AppElement::Logout => {}
                AppElement::Sender => {}
            }
        }
    }
    fn handle_browse(&mut self, event: KeyEvent) {
        match &event.code {
            KeyCode::Left => {
                self.highlighted = match self.highlighted {
                    AppElement::Account => AppElement::Ledger,
                    AppElement::Sender => AppElement::Account,
                    AppElement::Receiver => AppElement::Sender,
                    AppElement::Money => AppElement::Receiver,
                    AppElement::Submit => AppElement::Money,
                    AppElement::Logout => AppElement::Submit,
                    AppElement::Ledger => AppElement::Logout,
                }
            }
            KeyCode::Right => {
                self.highlighted = match self.highlighted {
                    AppElement::Account => AppElement::Sender,
                    AppElement::Sender => AppElement::Receiver,
                    AppElement::Receiver => AppElement::Money,
                    AppElement::Money => AppElement::Submit,
                    AppElement::Submit => AppElement::Logout,
                    AppElement::Logout => AppElement::Ledger,
                    AppElement::Ledger => AppElement::Account,
                }
            }
            KeyCode::Up => {
                self.highlighted = match self.highlighted {
                    AppElement::Sender => AppElement::Account,
                    AppElement::Logout => AppElement::Sender,
                    AppElement::Ledger => AppElement::Logout,
                    AppElement::Receiver => AppElement::Account,
                    AppElement::Money => AppElement::Account,
                    // AppElement::Ledger => AppElement::Logout,
                    AppElement::Submit => AppElement::Logout,
                    AppElement::Account => AppElement::Ledger,
                }
            }
            KeyCode::Down => {
                self.highlighted = match self.highlighted {
                    AppElement::Account => AppElement::Sender,
                    AppElement::Sender => AppElement::Logout,
                    AppElement::Logout => AppElement::Ledger,
                    AppElement::Receiver => AppElement::Logout,
                    AppElement::Money => AppElement::Logout,
                    AppElement::Ledger => AppElement::Account,
                    AppElement::Submit => AppElement::Logout,
                }
            }
            KeyCode::Enter => {
                self.selected = Some(self.highlighted);
            }
            _ => unreachable!(),
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // todo!()

        match event::poll(Duration::from_millis(100)) {
            Ok(v) => {
                if v {
                    match event::read()? {
                        // it's important to check that the event is a key press event as
                        // crossterm also emits key release and repeat events on Windows.
                        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                            self.handle_key_event(key_event)
                        }
                        _ => {}
                    };
                }
            }
            Err(e) => {}
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" TRADING DESK ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "[Q] ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        block.clone().render(area, buf);

        if self.state == AppState::Active {
            let main_split = Layout::new(
                Direction::Horizontal,
                [Constraint::Fill(1), Constraint::Fill(1)],
            )
            .split(block.inner(area));

            let layout: [Rect; 3] = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Fill(7),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ],
            )
            .areas(main_split[0]);

            let panel: [Rect; 4] = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Fill(2),
                    Constraint::Fill(2),
                    Constraint::Fill(2),
                    Constraint::Fill(3),
                ],
            )
            .areas(layout[1]);

            self.sender_button.render(panel[0], buf);
            self.receiver_button.render(panel[1], buf);
            self.money_button.render(panel[2], buf);
            self.submit_button.render(panel[3], buf);
            self.logout_button.render(layout[2], buf);

            self.account.render(layout[0], buf);

            self.ledger.render(main_split[1], buf);
        } else {
            if self.state == AppState::Waiting {
                self.waiting.render(area, buf);
            } else if self.state == AppState::Login {
                self.login.render(area, buf);
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

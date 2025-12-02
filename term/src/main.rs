use std::{io, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text, ToSpan},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::gui::{account::Account, formbutton::{FormButton, FormButtonType}, ledger::Ledger, login::Login};

pub mod gui;

// #[derive(Debug)]
pub struct App {
    exit: bool,
    login: Login,
    account: Account,
    in_login: bool,

    ledger: Ledger,

    // Buttons
    sender_button: FormButton,
    receiver_button: FormButton,
    money_button: FormButton,
    submit_button: FormButton,
    logout_button: FormButton
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            account: Account::default(),
            in_login: false,
            login: Login::default(),
            
            ledger: Ledger::new(),
            sender_button: FormButton::locked("Sender", "Calgary"),
            receiver_button: FormButton::cycle("Receiver", &[
                "Calgary",
                "Signapore",
                "New York"
            ]),
            money_button: FormButton::money("Money", 250),
            submit_button: FormButton::clickable("Submit Transaction"),
            logout_button: FormButton::clickable("Logout")
        }

    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.login.tick += 1;
            if self.login.tick == 1000 {
                self.login.tick = 0;
            }
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
            self.exit = true;
        } else if event.code == KeyCode::Char('q') {
            self.exit = true;
        } 

        if !self.in_login {
            self.login.handle_key_event(event);
        }
        // println!("Event: {event:?}");
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
        let instructions = Line::from(vec![
            " Quit ".into(),
            "[Q] ".blue().bold(),
        ]);
        let block = Block::bordered()
        
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);



        block.clone().render(area, buf);


        let main_split = Layout::new(Direction::Horizontal, [
            Constraint::Fill(1), Constraint::Fill(1)
        ])
            .split(block.inner(area));


        let layout: [Rect; 3] = Layout::new(Direction::Vertical, [
            Constraint::Fill(7), Constraint::Length(3), Constraint::Length(3)
        ])
        .areas(main_split[0]);

            
        let panel: [Rect; 4] = Layout::new(Direction::Horizontal, [
            Constraint::Fill(2), Constraint::Fill(2), Constraint::Fill(2), Constraint::Fill(3)
        ])
            .areas(layout[1]);


        self.sender_button.render(panel[0], buf);
        self.receiver_button.render(panel[1], buf);
        self.money_button.render(panel[2], buf);
        self.submit_button.render(panel[3], buf);
        self.logout_button.render(layout[2], buf);

        self.account.render(layout[0], buf);

        self.ledger
            .render(main_split[1], buf);

        


       

    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::new().run(&mut terminal);
    ratatui::restore();
    app_result
}

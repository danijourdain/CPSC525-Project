use std::sync::mpsc::Sender;

use ratatui::widgets::Widget;

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph},
};

use crate::backend::worker::ToWorkerMsg;

/// The Login Widget
#[derive(Default, Debug)]
pub struct Login {
    /// The tick count, this is used to blink the cursor.
    pub tick: usize,
    /// The password buffer.
    password: Vec<char>,
    /// If the login is currently locked.
    locked: bool,
}

impl Login {
    /// Handles a key event for the login widget.
    pub fn handle_key_event(&mut self, event: KeyEvent, channel: &mut Sender<ToWorkerMsg>) {
        // If we are locked then we should not do anything.
        if !self.locked {
            if event.code == KeyCode::Backspace {
                self.password.pop();
            }
            // Add a character to the password buffer.
            if let KeyCode::Char(c) = event.code {
                self.password.push(c);
            }
            // Enter the password, which should lock
            // the field.
            if event.code == KeyCode::Enter {
                let _ = channel.send(ToWorkerMsg::LoginAttempt(
                    self.password.clone().into_iter().collect::<String>(),
                ));
                self.locked = true;
            }
        }
    }
    /// Unlock the login widget, allowing it 
    /// to be used again.
    pub fn unlock(&mut self) {
        self.password.clear();
        self.locked = false;
    }
}

impl Widget for &Login {
    /// Render the login widget.
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let instructions = Line::from(vec![" Submit ".into(), "[ENTER] ".blue().bold()]);
        let desk_title = Line::from(if self.locked {
            " LOGIN 🔒 ".bold()
        } else {
            " LOGIN ".bold()
        });
        let desk_block = Block::bordered()
            .padding(Padding::new(1, 1, 0, 0))
            .title(desk_title.left_aligned())
            .title_bottom(instructions.right_aligned())
            .border_set(border::PLAIN);

        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(1),
            ],
        )
        .split(area);

        let layout2 = Layout::new(
            Direction::Vertical,
            [
                Constraint::Fill(1),
                Constraint::Length(4),
                Constraint::Fill(1),
            ],
        )
        .split(layout[1]);

        // desk_block.render(layout2[1], buf);
        let counter_text = Text::from(vec![
            Line::from(vec!["Region:   ".into(), "Calgary".green().italic()]),
            Line::from(vec![
                "Password: ".into(),
                self.password.iter().copied().collect::<String>().yellow(),
                if !self.locked {
                    if self.tick % 10 < 5 {
                        "_".bold().rapid_blink()
                    } else {
                        " ".bold()
                    }
                } else {
                    "".bold()
                },
            ]),
        ]);

        Paragraph::new(counter_text)
            .left_aligned()
            .block(desk_block)
            .render(layout2[1], buf);
    }
}

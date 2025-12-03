use std::time::SystemTime;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

use crate::gui::format_money_accounting;

/// The account panel of the GUI.
#[derive(Debug, Default)]
pub struct Account {
    /// Current balance.
    balance: Option<i32>,
    /// When the GUI was last updated.
    last_update: Option<SystemTime>,
    /// If the pane is selected.
    selected: bool,
    /// If the pane is focused.
    focused: bool,
}

impl Account {
    /// Set if this window is selected.
    pub fn set_selected(&mut self, select: bool) {
        self.selected = select;
    }
    /// Set if this window is focused.
    pub fn set_focused(&mut self, highlight: bool) {
        self.focused = highlight;
    }
    /// Set the active balance.
    pub fn set_balance(&mut self, balance: i32) {
        self.balance = Some(balance);
        self.last_update = Some(SystemTime::now());
    }
    /// Reset the selection and focus status.
    pub fn reset(&mut self) {
        self.set_selected(false);
        self.set_focused(false);
    }
}

impl Widget for &Account {
    /// Render the account widget.
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let desk_title = Line::from(" ACCOUNT ".bold());
        let desk_block_lines = Line::from(vec![]);
        let desk_block = Block::bordered()
            .padding(Padding::horizontal(2))
            .title(desk_title.centered())
            .title_bottom(desk_block_lines.centered())
            .border_style(if self.selected {
                Style::new().fg(ratatui::style::Color::Yellow)
            } else if self.focused {
                Style::new().fg(ratatui::style::Color::Blue)
            } else {
                Style::default()
            })
            .border_set(border::ROUNDED);

        let balance_span = match self.balance {
            Some(val) => format_money_accounting(val as isize).green().bold(),
            None => "--".red().bold(),
        };

        let last_update_span = match self.last_update {
            Some(val) => format!("{:?}", val.elapsed().unwrap()).cyan(),
            None => "--".red().bold(),
        };

        let counter_text = Text::from(vec![
            Line::from(vec![
                "Region:      ".bold().into(),
                "Calgary".green().italic(),
            ]),
            Line::from(vec!["Balance:     ".bold().into(), balance_span]),
            Line::from(vec!["Last Update: ".bold().into(), last_update_span]),
        ]);

        let layout = Layout::new(Direction::Horizontal, [Constraint::Fill(1)]).split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .title("Action");

        block.render(layout[0], buf);

        Paragraph::new(counter_text)
            .left_aligned()
            .block(desk_block)
            .render(layout[0], buf);
    }
}

use std::{str::FromStr, time::SystemTime};

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text, ToSpan, ToText},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget},
};

use crate::gui::format_money;

#[derive(Debug, Clone)]
pub struct FormButton {
    name: String,
    selected: bool,
    focused: bool,
    form_type: FormButtonType,
}

#[derive(Debug, Clone)]
pub enum FormButtonType {
    Normal {
        content: String
    },
    Cycle {
        options: Vec<String>,
        position: usize,
    },
    Money {
        content: usize
    },
    Clickable
}

impl FormButton {
    pub fn new(name: &str, form_type: FormButtonType) -> Self {
        Self {
            name: name.to_string(),
            // locked,
            form_type,
            focused: false,
            selected: false
        }
    }
    pub fn locked(name: &str, content: &str) -> Self {
        Self::new(name, FormButtonType::Normal { content: content.to_string() })
    }
    pub fn cycle<V>(name: &str, options: &[V]) -> Self
    where
        V: ToString,
    {
        let options = options.iter().map(V::to_string).collect();
        Self::new(
            name,
            FormButtonType::Cycle {
                options,
                position: 0,
            },
        )
    }
    pub fn money(name: &str, initial: usize) -> Self {
        Self::new(name, FormButtonType::Money { content: initial })
    }
    pub fn clickable(name: &str) -> Self {
        Self::new(name, FormButtonType::Clickable)
    }
}

impl Widget for &FormButton {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = if matches!(self.form_type, FormButtonType::Normal { .. }) {
            format!(" {} 🔒 ", self.name)
        } else {
            format!(" {} ", self.name)
        };

        let block_a = Block::bordered()
            .title(if matches!(self.form_type, FormButtonType::Clickable) {
                "".to_span()
            } else {
                title.to_span()
            })
            
            .padding(Padding::left(1))
            .border_style(if self.selected {
                Style::new().fg(ratatui::style::Color::Yellow)
            } else if self.focused {
                Style::new().fg(ratatui::style::Color::Blue)
            } else {
                Style::default()
            })
            .border_set(border::ROUNDED);


        let inner_text = match &self.form_type {
            FormButtonType::Normal { content } => {
                content.clone()
            }
            FormButtonType::Cycle { options, position } => {
                options[*position].clone()
            }
            FormButtonType::Money { content } => {
                
                format_money(*content)
                    
            }
            FormButtonType::Clickable => {
                self.name.clone()
            }
        };


        // Split it into two layouts to render stuff on the right side.
        let layout = Layout::new(Direction::Horizontal, [
            Constraint::Fill(1), Constraint::Length(if matches!(self.form_type, FormButtonType::Cycle { .. }) {
                2
            } else {
                0
            })
        ])
            .split(block_a.inner(area));



        if matches!(self.form_type, FormButtonType::Clickable) {
            Paragraph::new(inner_text.to_text().bold())
                .centered()
                .render(layout[0], buf);
        } else {
            Paragraph::new(inner_text).render(layout[0], buf);
        }
        

        // Render the up and down arrows.
        Paragraph::new("↑↓").render(layout[1], buf);

        block_a.render(area, buf);
    }
}

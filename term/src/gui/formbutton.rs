use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::{ToSpan, ToText},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::gui::format_money;

/// This is a versatile button component used
/// for the creation of the GUI.
#[derive(Debug, Clone)]
pub struct FormButton {
    name: String,
    selected: bool,
    focused: bool,
    form_type: FormButtonType,
}

/// The exact type of the form button.
#[derive(Debug, Clone)]
pub enum FormButtonType {
    /// The normal form button type,
    /// also known as the locked type.
    Normal { content: String },
    /// A form button that can be cycled.
    Cycle {
        options: Vec<String>,
        position: usize,
    },
    /// A button representing a monetary quantity
    Money { content: usize },
    /// A normal button that can be clicked.
    Clickable,
}

impl FormButton {
    /// Creates a form button from a name (title) and a type.
    pub fn new(name: &str, form_type: FormButtonType) -> Self {
        Self {
            name: name.to_string(),
            form_type,
            focused: false,
            selected: false,
        }
    }
    /// Creates a locked type (normal) form button.
    pub fn locked(name: &str, content: &str) -> Self {
        Self::new(
            name,
            FormButtonType::Normal {
                content: content.to_string(),
            },
        )
    }
    /// Creates a cyclic form button where we have a variety
    /// of options we can cycle through.
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
    /// Gets the monetary quantity within the form
    /// button, or returns none if that is not the
    /// correct type.
    pub fn get_money(&self) -> Option<usize> {
        match &self.form_type {
            FormButtonType::Money { content } => Some(*content),
            _ => None,
        }
    }
    /// Gets the cycle list position if it is
    /// available.
    pub fn get_position(&self) -> Option<usize> {
        match &self.form_type {
            FormButtonType::Cycle {
                options: _,
                position,
            } => Some(*position),
            _ => None,
        }
    }
    /// Handle the key event, this is only triggered
    /// if the button is selected & focused, in which
    /// case it will allow cycling and such.
    pub fn handle_key_event(&mut self, event: KeyEvent) {
        if event.code == KeyCode::Up {
            // If the key is up.
            match &mut self.form_type {
                FormButtonType::Cycle { options, position } => {
                    *position = (*position + 1).min(options.len() - 1);
                }
                FormButtonType::Money { content } => {
                    *content += 50;
                }
                _ => {}
            }
        }
        if event.code == KeyCode::Down {
            // If the key is down.
            match &mut self.form_type {
                FormButtonType::Cycle {
                    options: _,
                    position,
                } => {
                    if *position > 0 {
                        *position -= 1;
                    }
                }
                FormButtonType::Money { content } => {
                    if *content > 0 {
                        *content -= 50;
                    }
                }
                _ => {}
            }
        }
    }
    /// A money type form button.
    pub fn money(name: &str, initial: usize) -> Self {
        Self::new(name, FormButtonType::Money { content: initial })
    }
    /// A clickable form button.
    pub fn clickable(name: &str) -> Self {
        Self::new(name, FormButtonType::Clickable)
    }
    /// Sets the selection.
    pub fn set_selected(&mut self, select: bool) {
        self.selected = select;
    }
    /// Sets if we are focused.
    pub fn set_focused(&mut self, focus: bool) {
        self.focused = focus;
    }
    /// Resets the focused and selection state.
    pub fn reset(&mut self) {
        self.set_selected(false);
        self.set_focused(false);
    }
}

impl Widget for &FormButton {
    /// Renders the widget.
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
            FormButtonType::Normal { content } => content.clone(),
            FormButtonType::Cycle { options, position } => options[*position].clone(),
            FormButtonType::Money { content } => format_money(*content),
            FormButtonType::Clickable => self.name.clone(),
        };

        // Split it into two layouts to render stuff on the right side.
        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1),
                Constraint::Length(if matches!(self.form_type, FormButtonType::Cycle { .. }) {
                    2
                } else {
                    0
                }),
            ],
        )
        .split(block_a.inner(area));

        if matches!(self.form_type, FormButtonType::Clickable) {
            Paragraph::new(inner_text.to_text().bold())
                .centered()
                .render(layout[0], buf);
        } else {
            Paragraph::new(inner_text).render(layout[0], buf);
        }

        // Render the up and down arrows.
        Paragraph::new(
            if matches!(self.form_type, FormButtonType::Cycle { .. })
                | matches!(self.form_type, FormButtonType::Money { .. })
            {
                if self.focused {
                    "↑↓".yellow().bold()
                } else {
                    "↑↓".to_span()
                }
            } else {
                "↑↓".to_span()
            },
        )
        .render(layout[1], buf);

        block_a.render(area, buf);
    }
}

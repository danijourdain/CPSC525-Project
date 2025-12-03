use ratatui::text::ToLine;
use ratatui::widgets::Widget;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Padding, Paragraph},
};

/// The waiting widget.
#[derive(Default, Debug)]
pub struct Waiting {
    /// The tick count, which is used for the pending animation.
    pub tick: usize,
}

impl Widget for &Waiting {
    /// Render the widget.
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {

         let desk_title = Line::from(" LOGIN ".bold());
        let desk_block = Block::bordered()
            .padding(Padding::new(1, 1, 0, 0))
            .title(desk_title.left_aligned())
            // .title_bottom(instructions.right_aligned())
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
                Constraint::Length(3),
                Constraint::Fill(1),
            ],
        )
        .split(layout[1]);

        let ye =("WAITING FOR CONNECTION".to_owned() + &['.'].iter().cycle().take((self.tick / 3) % 8).collect::<String>()).to_string();

        let counter_text = Text::from(vec![
            ye.to_line()
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(desk_block)
            .render(layout2[1], buf);
    }
}
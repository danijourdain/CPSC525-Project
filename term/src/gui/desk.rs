use std::time::SystemTime;

use ratatui::{layout::{Constraint, Direction, Layout}, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Padding, Paragraph, Widget}};



#[derive(Debug, Default)]
pub struct DeskGui {
    region: Option<String>,
    balance: Option<i32>,
    last_update: Option<SystemTime>
}




impl Widget for &DeskGui {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
          let desk_title = Line::from(" ACCOUNT ".bold());
        let desk_block_lines = Line::from(vec![]);
        let desk_block = Block::bordered()
            .padding(Padding::horizontal(2))
            .title(desk_title.centered())
            .title_bottom(desk_block_lines.centered())
            .border_set(border::PLAIN);

            
          let counter_text = Text::from(vec![
            Line::from(vec!["Region:      ".into(), "Calgary".green().italic()]),
            Line::from(vec!["Balance:     ".into(), "$9,883".cyan()]),
            Line::from(vec!["Last Update: ".into(), "12m".cyan()])

        ]);


        // self.login.render(block.inner(area), buf);

        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(1), Constraint::Fill(3)],
        )
        
        .split(area);

    Paragraph::new(counter_text)
            .left_aligned()
            .block(desk_block)
            .render(layout[0], buf);
    }
}
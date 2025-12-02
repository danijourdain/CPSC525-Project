
use chrono::Utc;
use ratatui::{
    buffer::Buffer, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style, Stylize}, text::ToLine, widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, Widget}
};

use crate::gui::format_money_accounting;

// #[derive(Debug)]
pub struct Ledger {
    trades: Vec<Trade>,
    selected: bool,
    focused: bool

}

impl Ledger {
    pub fn new() -> Self {
        Self {
            trades: vec![],
            selected: false,
            focused: false
        }
    }

    pub fn set_selected(&mut self, select: bool) {
        self.selected = select;
    }
    pub fn set_focused(&mut self, highlight: bool) {
        self.focused = highlight;
    }
    pub fn reset(&mut self) {
        self.set_focused(false);
        self.set_selected(false);
    }
}

impl Widget for &Ledger {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let outer = Block::bordered()
        .title(" LEDGER ".to_line().centered())
         .border_style(if self.selected {
                Style::new().fg(ratatui::style::Color::Yellow)
            } else if self.focused {
                Style::new().fg(ratatui::style::Color::Blue)
            } else {
                Style::default()
            })
        .border_type(BorderType::Rounded);
    outer.clone().render(area, buf);

    let inner = outer.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(inner);


    let top_part = Layout::new(Direction::Horizontal, [
        Constraint::Fill(1), Constraint::Length(11)
    ]).split(chunks[0]);

    // Render the time, ideally a little bit cut off.
    Paragraph::new(Utc::now().format("%H:%M:%SUTC").to_string())
        // .italic()
        .render(top_part[1], buf);

    // Render the Header
    Paragraph::new(" Latest Trades")
        .italic()
        .render(top_part[0], buf);


    let divider = Block::default()
        .borders(Borders::BOTTOM)
        .style(Style::default());
    divider.render(chunks[0], buf);

    render_table(buf, chunks[1], &self.trades);

    }
}


pub struct Trade {
    pub sender: usize,
    pub receiver: usize,
    pub money: usize
}


impl Trade {
    pub fn new(sender: usize, receiver: usize, money: usize) -> Self {
        Self {
            sender,
            receiver,
            money
        }
    }
}

fn id_to_region_name(id: usize) -> String {
    if id == 0 {
        "Calgary".to_string()
    } else if id == 1 { 
        "New York".to_string()
    } else if id == 2 {
        "Signapore".to_string()
    } else {
        panic!("Lookup failed.")
    }
}

fn render_table(frame: &mut Buffer, area: ratatui::layout::Rect, trades: &[Trade]) {
    // Optional: split area so table only uses part of the screen
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)].as_ref())
        .split(area);

    // Define header
    let header = Row::new(vec![
        Cell::from("Sender"),
        Cell::from("Receiver"),
        Cell::from("Money"),
    ])
    
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
            
    )
    
    .bottom_margin(1); // space between header and rows


    let rows = trades.into_iter()
        .map(|Trade { sender, receiver, money }| {

            let imoney = if *receiver == 0 {
                *money as isize
            } else {
                (*money as isize) * -1
            };
            // if *sender 

            Row::new(vec![ id_to_region_name(*sender), id_to_region_name(*receiver), format_money_accounting(imoney) ])
        });

    // Build table
    let table = Table::new(rows, [
        Constraint::Fill(1), Constraint::Fill(1), Constraint::Fill(1)
    ])
        .header(header)
        .block(Block::new().padding(Padding::left(1)))
        .column_spacing(1) // space between columns
        .highlight_symbol(">> ");

    table.render(chunks[0], frame);
}
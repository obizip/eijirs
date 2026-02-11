use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    crossterm::terminal,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, ToSpan},
    widgets::{Block, List, Paragraph},
};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::db::Db;

pub fn tui_main(db: Db) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(db).run(&mut terminal);
    ratatui::restore();
    result
}

#[derive(Debug)]
struct App {
    input: Input,
    prev: String,
    results: Vec<(String, String)>,
    db: Db,
}

impl App {
    fn new(db: Db) -> Self {
        Self {
            input: Input::default(),
            prev: String::new(),
            results: Vec::new(),
            db,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            let event = event::read()?;
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    _ => {
                        self.input.handle_event(&event);
                        let value = self.input.value();
                        if value != "" && value != self.prev {
                            self.prev = value.to_string();
                            self.update_results()?;
                        }
                    }
                }
            }
        }
    }

    fn update_results(&mut self) -> anyhow::Result<()> {
        let query = self.input.value().to_string();
        let (_cols, rows) = terminal::size()?;
        let lines = rows;
        self.results = self.db.search(query, lines.into(), 0)?;
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let [header_area, input_area, messages_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .areas(frame.area());

        self.render_help_message(frame, header_area);
        self.render_input(frame, input_area);
        self.render_results(frame, messages_area);
    }

    fn render_help_message(&self, frame: &mut Frame, area: Rect) {
        let help_message =
            Line::from_iter(["Press ".to_span(), "Esc".bold(), " to quit, ".to_span()]);
        frame.render_widget(help_message, area);
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = Style::default();
        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);

        // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
        // end of the input text and one line down from the border to the input line
        let x = self.input.visual_cursor().max(scroll) - scroll + 1;
        frame.set_cursor_position((area.x + x as u16, area.y + 1))
    }

    fn render_results(&self, frame: &mut Frame, area: Rect) {
        let results = self
            .results
            .iter()
            .map(|(key, value)| format!("{key}: {value}"));
        let results = List::new(results).block(Block::bordered().title("Results"));
        frame.render_widget(results, area);
    }
}

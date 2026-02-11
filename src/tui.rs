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
    offset: usize,
}

impl App {
    fn new(db: Db) -> Self {
        Self {
            input: Input::default(),
            prev: String::new(),
            results: Vec::new(),
            db,
            offset: 0,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            let event = event::read()?;
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Up => {
                        if self.offset > 0 {
                            self.offset -= 1;
                            self.update_results()?;
                        }
                    }
                    KeyCode::Down => {
                        let list_height = self.get_list_height()?;
                        if self.results.len() >= list_height {
                            self.offset += 1;
                            self.update_results()?;
                        }
                    }
                    _ => {
                        self.input.handle_event(&event);
                        let value = self.input.value();
                        if value != "" && value != self.prev {
                            self.prev = value.to_string();
                            // 検索ワードが変わったらオフセットをリセット
                            self.offset = 0;
                            self.update_results()?;
                        }
                    }
                }
            }
        }
    }

    fn get_list_height(&self) -> anyhow::Result<usize> {
        let (_cols, rows) = terminal::size()?;
        // Layoutで確保している Header(1) + Input(3) = 4行分を引く
        let height = rows.saturating_sub(4);
        Ok(height as usize)
    }

    fn update_results(&mut self) -> anyhow::Result<()> {
        let query = self.input.value().to_string();

        // 画面のリストエリアの高さ分だけデータを取得する
        let lines = self.get_list_height()?;

        // linesが0だと検索の意味がないので最低1確保（ウィンドウが極端に小さい場合など）
        let lines = lines.max(1);

        self.results = self.db.search(query, lines, self.offset)?;
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
        let help_message = Line::from_iter([
            "Press ".to_span(),
            "Esc".bold(),
            " to quit, ".to_span(),
            "Up/Down".bold(),
            " to scroll".to_span(),
        ]);
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

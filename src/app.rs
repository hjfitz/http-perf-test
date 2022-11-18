use tui::{
    backend::{Backend, CrosstermBackend},
    style::Color,
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph},
    Frame, Terminal,
};

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::terminal::enable_raw_mode;
use crossterm::{event::EnableMouseCapture, execute, terminal::EnterAlternateScreen};

use crate::ui::{create_layout, AppLayout};

pub struct UI {
    pub term: Terminal<CrosstermBackend<io::Stdout>>,
}

impl UI {
    pub fn new() -> Self {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend).unwrap();
        Self { term }
    }

    pub fn init_ui(&mut self) {
        enable_raw_mode().unwrap();
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();
    }
}

pub struct App {
    host: String,
    headers: Vec<String>,
    method: String,
    average_response_time: f64,
    total_responses: u64,
    // [2xx, 3xx, 4xx, 5xx]
    results: [u16; 4],

    // tui specifics
    redraw_interval: Duration,
    previous_redraw_time: Instant,
    ui: UI,
}

impl App {
    pub fn new(host: String, headers: Vec<String>, method: String) -> Self {
        let results: [u16; 4] = [0, 0, 0, 0];

        let redraw_interval = Duration::from_secs(1);
        let previous_redraw_time = Instant::now();
        let ui = UI::new();
        Self {
            ui,
            host,
            headers,
            method,
            results,
            total_responses: 0,
            average_response_time: 0.0,
            redraw_interval,
            previous_redraw_time,
        }
    }

    pub fn update_state(&mut self, code: u16, response_time: u128) {
        let code_usize = code as usize;
        let index = (code_usize / 100) - 2;
        self.results[index] += 1;
        self.total_responses += 1;

        let response_float = response_time as f64;
        let total_responses_float = self.total_responses as f64;
        self.average_response_time =
            (self.average_response_time + response_float) / total_responses_float;

        if Instant::now().duration_since(self.previous_redraw_time) >= self.redraw_interval {
           // self.draw_ui();
            self.ui.term.draw(|f| self.draw_term_ui(f));
            self.previous_redraw_time = Instant::now();
        }
    }

    pub fn init_ui(&mut self) {
        self.ui.init_ui();
    }

    fn draw_term_ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        let AppLayout {
            bar_width,
            details_area,
            headers_area,
            chart_area,
            stats_area,
        } = create_layout(f);

        let details_block = Paragraph::new(vec![
            Spans::from(Span::raw(format!(" Host: {}", self.host))),
            Spans::from(Span::raw(format!(" Method: {}", self.method))),
        ])
        .block(Block::default().title("Details").borders(Borders::ALL));

        let mut max = 0;
        for result in self.results {
            if result > max {
                max = result;
            }
        }
        let [twoxx, threexx, fourxx, fivexx] = self.results.map(u64::from);
        let chart_data = &[
                ("2xx", twoxx),
                ("3xx", threexx),
                ("4xx", fourxx),
                ("5xx", fivexx),
            ];
        let chart_bars = BarChart::default()
            .block(Block::default().title("Responses").borders(Borders::ALL))
            .bar_gap(1)
            .bar_style(Style::default().fg(Color::White).bg(Color::Black))
            .value_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .label_style(Style::default().fg(Color::White))
            .bar_width(bar_width)
            .data(chart_data)
            .max(max.into());

        f.render_widget(details_block, details_area);
        f.render_widget(chart_bars, chart_area);
    }

    fn draw_ui(&mut self) {
        print!("{esc}c", esc = 27 as char);
        println!("Host: {}", self.host);
        println!("Method: {}", self.method);

        if !self.headers.is_empty() {
            let pretty_headers = self
                .headers
                .iter_mut()
                .map(|header| {
                    let split_headers = header.split(": ").map(String::from).collect::<Vec<_>>();
                    let header_name = &split_headers[0];
                    let truncated_header_value = split_headers[1].as_str()[..12].to_string();
                    format!("{}: {}", header_name, truncated_header_value)
                })
                .collect::<Vec<String>>()
                .join("\n");

            println!("Headers:");
            println!("{}", pretty_headers);
        }

        let mut code_prefix = 2;
        for count in self.results {
            let code = format!("{}XX", code_prefix);
            println!("{}: {}", code, count);
            code_prefix += 1;
        }

        println!("Average response time: {}", self.average_response_time);

        println!("\n\n\n")
    }
}

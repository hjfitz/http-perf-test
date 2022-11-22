use tui::{
    backend::CrosstermBackend,
    style::Color,
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph, Sparkline},
    Terminal,
};

use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::DisableMouseCapture,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use crossterm::{event::EnableMouseCapture, execute, terminal::EnterAlternateScreen};

use crate::{
    args::Args,
    ui::{create_layout, AppLayout},
};

pub trait UIHandler {
    fn new() -> Self;
    fn restore_ui(&mut self);
    fn init_ui(&mut self);
}

pub struct UI {
    pub term: Terminal<CrosstermBackend<io::Stdout>>,
}

impl UIHandler for UI {
    fn new() -> Self {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend).unwrap();
        Self { term }
    }

    fn init_ui(&mut self) {
        enable_raw_mode().unwrap();
        self.term.clear().unwrap();
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture).unwrap();
    }

    fn restore_ui(&mut self) {
        // restore terminal
        disable_raw_mode().unwrap();
        execute!(
            self.term.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .unwrap();
        self.term.clear().unwrap();
        self.term.show_cursor().unwrap();
    }
}

pub struct App {
    program_args: Args,
    // metrics
    average_response_time: f64,
    total_responses: u64,
    test_begin: Instant,
    // [2xx, 3xx, 4xx, 5xx]
    results: [u16; 4],
    response_times: Vec<u128>, // BoundedVec<u128>,

    // tui specifics
    redraw_interval: Duration,
    previous_redraw_time: Instant,
    pub ui: UI,
}

impl App {
    pub fn new(test_begin: Instant, args: Args) -> Self {
        let results: [u16; 4] = [0, 0, 0, 0];

        let redraw_interval = Duration::from_secs(1);
        let previous_redraw_time = Instant::now();
        let ui = UI::new();
        Self {
            ui,
            test_begin,
            results,
            program_args: args,
            total_responses: 0,
            response_times: Vec::new(), //BoundedVec::new(Some(25)),
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

        self.response_times.push(response_time);

        if Instant::now().duration_since(self.previous_redraw_time) >= self.redraw_interval {
            // self.draw_ui();
            //self.ui.term.draw(|f| self.draw_term_ui(f));
            self.draw_term_ui();
            self.previous_redraw_time = Instant::now();
        }
    }

    fn draw_term_ui(&mut self) {
        let header_line_length = self.program_args.headers.len() + 1;
        let f = self.ui.term.get_frame();
        let AppLayout {
            bar_width,
            details_area,
            headers_area,
            chart_area,
            stats_area,
            col_max_width,
        } = create_layout(&f, header_line_length as u16);

        let headers_lines = self
            .program_args
            .headers
            .iter()
            .map(|full_pair| {
                return Spans::from(Span::raw(format!(
                    " > {:w$}",
                    full_pair.clone(),
                    w = col_max_width.into()
                )));
            })
            .collect::<Vec<_>>();

        let details = vec![
            Spans::from(Span::raw(format!(" Host: {}", self.program_args.url))),
            Spans::from(Span::raw(format!(" Method: {}", self.program_args.method))),
            Spans::from(Span::raw(format!(
                " Concurrent Requests: {}",
                self.program_args.concurrent_requests
            ))),
            Spans::from(Span::raw(" Headers: ")),
        ];

        let details_block = Paragraph::new([details, headers_lines].concat()).block(
            Block::default()
                .title("Session Details")
                .borders(Borders::ALL),
        );

        let time_taken = Instant::now().duration_since(self.test_begin).as_secs_f32();
        let mut total_requests = 0_f32;
        for request in self.results {
            total_requests += request as f32;
        }
        let avg_tps = total_requests / time_taken;
        let mut ok_results = self.results[0] + self.results[1];
        // prevent any divide by 0
        if ok_results == 0 {
            ok_results = 1;
        }
        let error_perc = (self.results[2] + self.results[3]) / ok_results;
        let stats_line = format!(
            "Average TPS: {:.2}  |  Average response time: {:.2}ms  |  Error Percentage: {:.4}%",
            avg_tps, self.average_response_time, error_perc
        );
        let stats_block = Paragraph::new(Spans::from(Span::raw(stats_line)))
            .block(Block::default().title("Stats").borders(Borders::ALL));

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


        let mut response_time_data = self.response_times.clone();
        response_time_data.reverse();
        let max_len = usize::from(headers_area.width - 2);
        let total_responses = response_time_data.len();
        let bound = if max_len >= total_responses {
            total_responses
        } else {
            max_len
        };
        let response_time_data_subset = &mut response_time_data[0..bound];

        let sparkline_data = response_time_data_subset
            .iter_mut()
            .map(|n| *n as u64)
            .collect::<Vec<u64>>();
     
        let test = Sparkline::default()
            .block(
                Block::default()
                    .title("Response Time Variation")
                    .borders(Borders::ALL),
            )
            .data(sparkline_data.as_slice());

        self.ui
            .term
            .draw(|f| {
                f.render_widget(details_block, details_area);
                f.render_widget(chart_bars, chart_area);
                f.render_widget(test, headers_area);
                f.render_widget(stats_block, stats_area);
            })
            .unwrap();
    }
}

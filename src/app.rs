use std::time::{Duration, Instant};

pub struct App {
    host: String,
    headers: Vec<String>,
    method: String,
    average_response_time: u128,
    total_responses: u128,
    // [2xx, 3xx, 4xx, 5xx]
    results: [u16; 4],

    // tui specifics
    redraw_interval: Duration,
    previous_redraw_time: Instant,
}

impl App {
    pub fn new(host: String, headers: Vec<String>, method: String) -> Self {
        let results: [u16; 4] = [0, 0, 0, 0];

        let redraw_interval = Duration::from_secs(1);
        let previous_redraw_time = Instant::now();
        Self {
            host,
            headers,
            method,
            results,
            total_responses: 0,
            average_response_time: 0,
            redraw_interval,
            previous_redraw_time,
        }
    }

    pub fn update_state(&mut self, code: u16, response_time: u128) {
        let code_usize = code as usize;
        let index = code_usize / 100;
        self.results[index] += 1;
        self.total_responses += 1;
        self.average_response_time =
            (self.average_response_time + response_time) / self.total_responses;

        if Instant::now().duration_since(self.previous_redraw_time) >= self.redraw_interval {
            self.draw_ui();
            self.previous_redraw_time = Instant::now();
        }
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

        println!("\n\n\n")
    }
}

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Rect,
    style::Color,
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{BarChart, Block, Borders, Paragraph},
    Frame, Terminal,
};

pub struct AppLayout {
    pub details_area: Rect,
    pub headers_area: Rect,
    pub chart_area: Rect,
    pub stats_area: Rect,
    pub bar_width: u16,
}

#[allow(unused_doc_comments)]
/**
 * |------------------------------------------------------------|
 * |                                                            |
 * |  |-- Request Details ---------| |-- Results ------------|  |
 * |  |                            | |                       |  |
 * |  | Host: https://google.com   | | bar                   |  |
 * |  | Method: GET                | | bar  bar              |  |
 * |  | Headers:                   | | bar  bar       bar    |  |
 * |  |   accept: application/json | | bar  bar       bar    |  |
 * |  |   authorization: bearer tk | | bar  bar  bar  bar    |  |
 * |  |                            | | 200  300  400  500    |  |
 * |  |----------------------------| |-----------------------|  |
 * |                                                            |
 * |  |-- Progress ------------------------------------------|  |
 * |  |                                                      |  |
 * |  | Avg TPS: 1900 | 2xx: 10000 3xx: 0 4xx: 320 500: 0    |  |
 * |  |                                                      |  |
 * |  |------------------------------------------------------|  |
 * |                                                            |
 * |------------------------------------------------------------|
 */
pub fn create_layout<B: Backend>(f: &Frame<B>) -> AppLayout {
    let Rect {
        width: frame_width,
        height: frame_height,
        ..
    } = f.size();

    let bar_width = ((frame_width / 2) - 4) / 4;

    let details_area = Rect {
        y: 0,
        x: 0,
        width: (frame_width / 2),
        height: 4,
    };

    let headers_area = Rect {
        y: 4,
        x: 0,
        width: (frame_width / 2),
        height: frame_height - 7,
    };

    let chart_area = Rect {
        y: 0,
        x: (frame_width / 2),
        width: (frame_width / 2),
        height: frame_height - 3,
    };

    let stats_area = Rect {
        y: (frame_height - 3),
        x: 0,
        width: frame_width,
        height: 3,
    };

    AppLayout {
        bar_width,
        details_area,
        headers_area,
        chart_area,
        stats_area,
    }
}

fn ui<B: Backend>(f: &mut Frame<B>) {
    let details_block = Block::default().title("Details").borders(Borders::ALL);
    let headers_block = Block::default().borders(Borders::ALL);
    let stats_block = Block::default().title("Block").borders(Borders::ALL);
    let chart_block = Block::default().title("Stats").borders(Borders::ALL);

    /*
    f.render_widget(para, details_area);
    f.render_widget(headers_block, headers_area);
    f.render_widget(chart_bars, chart_area);
    f.render_widget(stats_block, stats_area);
    */
}
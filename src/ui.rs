use tui::{backend::Backend, layout::Rect, Frame};

pub struct AppLayout {
    pub details_area: Rect,
    pub headers_area: Rect,
    pub chart_area: Rect,
    pub stats_area: Rect,
    pub bar_width: u16,
    pub col_max_width: u16,
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
pub fn create_layout<B: Backend>(f: &Frame<B>, header_frame_height: u16) -> AppLayout {
    let Rect {
        width: frame_width,
        height: frame_height,
        ..
    } = f.size();

    let col_max_width = (frame_width / 2) - 4;

    let bar_width = ((frame_width / 2) - 5) / 4;
    let details_height = 5 + header_frame_height;

    let details_area = Rect {
        y: 0,
        x: 0,
        width: (frame_width / 2),
        height: details_height,
    };

    let headers_area = Rect {
        y: details_height,
        x: 0,
        width: (frame_width / 2),
        height: frame_height - (details_height + 3),
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
        col_max_width,
        details_area,
        headers_area,
        chart_area,
        stats_area,
    }
}

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Host to make requests to
    #[arg(short, long)]
    pub url: String,

    /// How many requests to send at once
    #[arg(short, long, default_value = "10")]
    pub concurrent_requests: u16,

    /// How long the test should last for (seconds).
    #[arg(short, long, default_value = "30")]
    pub test_time: u16,

    /// Any request headers to send with the request
    #[arg(short = 'x', long)]
    pub headers: Vec<String>,

    // todo: validate
    #[arg(short, long, default_value = "GET")]
    /// HTTP method to use
    pub method: String,

    #[arg(short, long)]
    /// File to write logs to
    pub out_file: Option<String>,

    /// Perform some additional debug logging
    #[arg(short, long)]
    pub debug: bool,
}

use clap::Parser;
use std::{fmt, time::{Duration, Instant}};
use tokio::{sync::mpsc::channel, task};

#[derive(Parser, Debug, Clone)]
struct Args {
    /// Host to make requests to
    #[arg(short, long)]
    url: String,

    /// How many requests to send at once
    #[arg(short, long, default_value = "10")]
    concurrent_requests: u8,

    /// How long the test should last for (seconds).
    #[arg(short, long, default_value = "30")]
    test_time: u8,

    /// Any request headers to send with the request
    #[arg(short = 'x', long)]
    headers: Option<String>,

    #[arg(short, long, default_value = "GET")]
    /// HTTP method to use
    method: String,

    /// Debug
    #[arg(short, long)]
    debug: bool,
}

#[derive(Debug)]
struct Results {
    response_code: u8,
    elapsed: Duration,
}

struct Test {
    url: String,
    method: String,
    start_time: u8,
}

impl Test {
    /*
    pub fn new(url: String, method: String) -> Self {
        Self {url, method}
    }
    */
}

#[derive(Debug)]
enum Message {
    Result(String),
    NextResult(Results),
    Finished,
    Error(String),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("Hello, world!");
    if args.debug {
        println!("{:#?}", args);
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let mut threads = Vec::new();

    let test_begin_original = Instant::now();
    for n in 0..args.concurrent_requests {
        let tx_cloned = tx.clone();
        println!("spawning thread {}", n);
        let handle = tokio::spawn(async move {
            let msg_content = format!("Sent from thread {}", n);
            let msg = Message::Result(msg_content);

            tx_cloned.send(msg).unwrap();

            println!("Spawned thread {}", n);


            loop {
                let now = Instant::now();
                let since = now.duration_since(test_begin_original);

                // do test

                // record result

                if since.as_secs() >= args.test_time.into() {
                    tx_cloned.send(Message::Finished).unwrap();
                    break
                }
            }
        });

        threads.push(handle);
    }


    let mut results = Vec::new();
    let mut next_results = Vec::new();
    let mut waiting_threads = args.concurrent_requests;
    while let Some(msg) = rx.recv().await {
        match msg {
            Message::Result(result) => results.push(result),
            Message::NextResult(result) => next_results.push(result),
            Message::Finished => {
                waiting_threads -= 1;
                println!("Threads waiting: {}", waiting_threads);
            },
            _ => {}
        }

        if waiting_threads == 0 {
            println!("killing threads");

            rx.close();
        }
    }

    for result in results {
        println!("Got msg:");
        println!("> {}", result);
    }

    // cleanup

    println!("{}", threads.len());

    Ok(())
}

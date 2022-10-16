use clap::{Parser, ValueEnum};
use reqwest::{Method, Request, Url, Client};
use std::{fmt, time::{Duration, Instant}, process::exit};

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

    // todo: validate
    #[arg(short, long, default_value = "GET")]
    /// HTTP method to use
    method: String,

    /// Debug
    #[arg(short, long)]
    debug: bool,
}

#[derive(Debug)]
struct Results {
    response_code: u16,
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
    Result(Results),
    Finished,
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

    let method = match args.method.to_ascii_uppercase().as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PATCH" => Method::PATCH,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => {
            println!("Unacceptable method: {}", args.method);
            exit(1);
        }
        
    };

    let test_begin = Instant::now();
    for n in 0..args.concurrent_requests {
        let tx_thread = tx.clone();
        let req_url_orig = Url::parse(args.url.as_str()).unwrap().clone();
        let req_method_orig = method.clone();
        println!("spawning thread {}", n);
        let handle = tokio::spawn(async move {

            println!("Spawned thread {}", n);

            let client = Client::new();


            loop {
                let req_url = req_url_orig.clone();
                let req_method = req_method_orig.clone();
                let now = Instant::now();
                let since = now.duration_since(test_begin);

                // do test
                let req_begin = Instant::now();
                let req = Request::new(req_method, req_url);
                let resp = client.execute(req).await.unwrap();

                // record result
                let since_req = now.duration_since(req_begin);
                let result = Results {
                    response_code: resp.status().as_u16(),
                    elapsed: since_req,
                };

                let msg = Message::Result(result);
                tx_thread.send(msg).unwrap();


                if since.as_secs() >= args.test_time.into() {
                    tx_thread.send(Message::Finished).unwrap();
                    break
                }
            }
        });

        threads.push(handle);
    }


    let mut results = Vec::new();
    let mut waiting_threads = args.concurrent_requests;
    while let Some(msg) = rx.recv().await {
        match msg {
            Message::Result(result) => results.push(result),
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

    // print results
    println!("host,status_code,time_millis");
    for result in results {
        println!("{},{},{}", args.url, result.response_code, result.elapsed.as_micros());
    }

    Ok(())
}

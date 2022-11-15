use clap::Parser;
use reqwest::{Client, Method, Request, Url, header::{HeaderMap, HeaderValue, HeaderName}};
use std::{fmt, process::exit};
use tokio::time::{Instant};

#[derive(Parser, Debug, Clone)]
struct Args {
    /// Host to make requests to
    #[arg(short, long)]
    url: String,

    /// How many requests to send at once
    #[arg(short, long, default_value = "10")]
    concurrent_requests: u16,

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
    elapsed: u128,
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

fn get_headers(headers_raw: Option<String>) -> Option<HeaderMap> {
    let mut map = HeaderMap::new();

    if headers_raw.is_none() {
        return None;
    }

    let split = headers_raw.unwrap().split(": ").map(String::from).collect::<Vec<_>>();

    if split.len() != 2 {
        return None;
    }

    let header_name = HeaderName::from_lowercase(split[0].as_bytes()).unwrap();

    map.insert(header_name, HeaderValue::from_str(&split[1]).unwrap());

    println!("{:?}", map);

    Some(map)

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
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

    let full_headers = get_headers(args.headers);

    let test_begin = Instant::now();
    for _ in 0..args.concurrent_requests {
        let tx_thread = tx.clone();
        let req_url_orig = Url::parse(args.url.as_str()).unwrap().clone();

        let headers = full_headers.clone().unwrap_or_default();

        let req_method_orig = method.clone();
        let handle = tokio::spawn(async move {

            let client = Client::new();
            loop {
                let req_headers_raw = headers.clone();
                let req_url = req_url_orig.clone();
                let req_method = req_method_orig.clone();
                let now = Instant::now();
                let since = now.duration_since(test_begin);

                // do test
                let req_begin = Instant::now();
                //let req = Request::new(req_method, req_url);
                let req = client.request(req_method, req_url);
                let with_headers = req.headers(req_headers_raw);
                let ready_req = with_headers.build().unwrap();
                let resp = client.execute(ready_req).await.unwrap();
                let req_after = Instant::now();

                // record result
                let since_req = req_after.duration_since(req_begin);
                let result = Results {
                    response_code: resp.status().as_u16(),
                    elapsed: since_req.as_millis(),
                };

                let msg = Message::Result(result);
                tx_thread.send(msg).unwrap();

                if since.as_secs() >= args.test_time.into() {
                    tx_thread.send(Message::Finished).unwrap();
                    break;
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
                println!("Threads waiting to finish: {}", waiting_threads);
            }
        }

        if waiting_threads == 0 {
            println!("killing threads");

            rx.close();
        }
    }

    let total_requests = results.len();

    // print results
    println!("host,status_code,time_millis");
    for result in results {
        println!(
            "{},{},{}",
            args.url,
            result.response_code,
            result.elapsed,
        );
    }

    let test_end = Instant::now();
    let since_test = test_end.duration_since(test_begin);

    let since_secs = since_test.as_secs_f64();
    let total_reqs = total_requests as f64;
    
    let tps = total_reqs / since_secs;

    println!();
    println!("Time taken (seconds): {}", since_test.as_secs());
    println!("Total requests sent: {}", total_requests);
    println!("TPS: {}", tps);

    Ok(())
}

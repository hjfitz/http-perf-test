use clap::Parser;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, Request, Url,
};
use std::io::Write; // need to import this trait
use std::{fmt, fs::File, process::exit};
use tokio::time::Instant;

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
    test_time: u16,

    /// Any request headers to send with the request
    #[arg(short = 'x', long)]
    headers: Vec<String>,

    // todo: validate
    #[arg(short, long, default_value = "GET")]
    /// HTTP method to use
    method: String,

    #[arg(short, long)]
    /// File to write logs to
    out_file: Option<String>,

    /// Perform some additional debug logging
    #[arg(short, long)]
    debug: bool,
}

#[derive(Debug)]
struct Results {
    response_code: u16,
    elapsed: u128,
}

impl Results {
    pub fn to_csv_line(&self) -> String {
        format!("{},{}", self.response_code, self.elapsed)
    }
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

const CSV_HEADER: &str = "host,status_code,time_millis";

fn get_headers(headers: Vec<String>) -> HeaderMap {
    let mut map = HeaderMap::new();

    for pair in headers {
        let split = pair.split(": ").map(String::from).collect::<Vec<_>>();

        if split.len() != 2 {
            continue;
        }

        let header_name = HeaderName::from_lowercase(split[0].to_lowercase().as_bytes()).unwrap();
        let header_value = HeaderValue::from_str(&split[1]).unwrap();

        map.insert(header_name, header_value);
    }

    map
}

fn write_results(out_file: String, host: String, results: Vec<Results>) {
    let mut buffer = File::create(out_file).expect("Unable to open file");

    writeln!(buffer, "{}", CSV_HEADER).expect("Unable to write to file");
    for result in results {
        writeln!(buffer, "{},{}", host, result.to_csv_line())
            .expect("Unable to write line to file");
    }
}

struct TestEndpoint {
    method: Method,
    endpoint: Url,
    headers: HeaderMap,
    pub client: Client,
}

impl TestEndpoint {
    pub fn new(method: Method, endpoint: Url, headers: HeaderMap) -> Self {
        let client = Client::new();
        Self {
            method,
            endpoint,
            headers,
            client,
        }
    }

    pub fn create_request(&self) -> Request {
        let req = self
            .client
            .request(self.method.clone(), self.endpoint.clone())
            .headers(self.headers.clone())
            .build();

        if req.is_err() {
            std::process::exit(1);
        }

        req.unwrap()
    }
}

fn string_to_method(str_method: &str) -> Option<Method> {
    match str_method.to_ascii_uppercase().as_str() {
        "GET" => Some(Method::GET),
        "POST" => Some(Method::POST),
        "PATCH" => Some(Method::PATCH),
        "PUT" => Some(Method::PUT),
        "DELETE" => Some(Method::DELETE),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.debug {
        println!("{:#?}", args);
    }

    let method = string_to_method(args.method.as_str()).unwrap_or(Method::GET);
    let full_headers = get_headers(args.headers);


    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut threads = Vec::new();

    let test_begin = Instant::now();
    for _ in 0..args.concurrent_requests {
        let tx_thread = tx.clone();
        let req_url = Url::parse(args.url.as_str()).unwrap();

        let test_client = TestEndpoint::new(method.clone(), req_url, full_headers.clone());

        let handle = tokio::spawn(async move {
            loop {
                let req = test_client.create_request();

                let transaction_begin = Instant::now();
                let resp = test_client.client.execute(req).await;
                let transaction_end = Instant::now().duration_since(transaction_begin);

                if let Ok(test_resp) = resp {
                    let msg = Message::Result(Results {
                        response_code: test_resp.status().as_u16(),
                        elapsed: transaction_end.as_millis(),
                    });

                    tx_thread.send(msg).unwrap();
                }

                let since = Instant::now().duration_since(test_begin);
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

    if let Some(out_file) = args.out_file {
        write_results(out_file, args.url, results);
    } else {
        // print results
        println!("{}", CSV_HEADER);
        for result in results {
            println!("{},{},{}", args.url, result.response_code, result.elapsed,);
        }
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

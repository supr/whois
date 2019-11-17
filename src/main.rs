use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::runtime::Builder;

#[derive(Debug)]
struct WhoisClient {
    server: String,
}

impl WhoisClient {
    fn new() -> Self {
        WhoisClient {
            server: "whois.verisign-grs.com".to_owned(),
        }
    }

    fn set_server(mut self, server: &str) -> Self {
        self.server = server.to_owned();
        self
    }

    async fn query(&self, hostname: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(format!("{}:43", &self.server)).await?;
        let mut request = hostname.to_owned();
        request.push_str("\r\n");

        tokio::spawn(async move {
            let (rdr, mut wtr) = stream.split();
            if let Ok(_) = wtr.write_all(request.as_bytes()).await {
                let mut rdr = BufReader::with_capacity(4 * 1024, rdr);
                let mut out = String::new();
                rdr.read_to_string(&mut out).await;
                println!("{}", out);
            }
        });

        Ok(())
    }
}

fn main() {
    let threads = match num_cpus::get() {
        1 | 2 => 1,
        _ => 2,
    };
    let rt = Builder::new()
        .blocking_threads(threads)
        .core_threads(threads)
        .keep_alive(Some(Duration::from_secs(10)))
        .name_prefix("whois-thread-")
        .build()
        .unwrap();

    let mut rl = rustyline::Editor::<()>::new();
    let mut wc = WhoisClient::new();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let mut line_splits = line.split_whitespace();
                if let Some(cmd) = line_splits.next() {
                    if let Some(arg) = line_splits.next() {
                        match cmd {
                            "server" => {
                                wc = wc.set_server(arg);
                                continue;
                            }
                            _ => {}
                        };
                    } else {
                        match cmd {
                            "quit" | "QUIT" | "q" | "Q" => break,
                            _ => rt.block_on(wc.query(cmd)),
                        };
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => break,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

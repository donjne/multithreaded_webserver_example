use std::{fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, thread::{self}, time::Duration, error::Error};
use hellorust::ThreadPool;

type Resulting = Result<(), Box<dyn Error>>;

fn main() -> Resulting {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(4)?;
    for stream in listener.incoming() {
        let stream = stream?;
        pool.execute(|| {
            if let Err(e) = handle_connection(stream) {
                eprintln!("Error handling connection: {}", e);
            }
    })?;
} 

    println!("Shutting down");
    Ok(())
}

/// HTTP-Version Status-Code Reason-Phrase CRLF
/// headers CRLF
/// message-body

fn handle_connection(mut stream: TcpStream) -> Resulting {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = match buf_reader.lines().next() {
        Some(line) => line?,
        None => return Ok(())
    };

    let (status_line, file_name) = match &request_line[..] { 
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP 200 OK", "index.html")
        }   
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(file_name)?;
    let content_length = contents.len();
    let response = format!("{status_line}\r\nContent Length: {content_length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

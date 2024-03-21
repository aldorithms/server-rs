use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}, thread};
use server_rs::ThreadPool;

/// The main function.
fn main() {
    // Create a new `TcpListener` bound to `localhost:7878`.
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Create a new `ThreadPool` with 4 threads.
    let pool = ThreadPool::new(4);

    // Listen for incoming connections.
    for stream in listener.incoming().take(2) {
        // Unwrap the stream. If it's `None`, print an error and continue.
        let stream = stream.unwrap();

        // Execute the `handle_connection` function on the `ThreadPool`.
        pool.execute(|| handle_connection(stream));

        // Print a message to the console.
        print!("Shutting down.")
    }
}

/// Handle an incoming connection.
/// 
/// ## Parameters
/// - `stream`: The incoming `TcpStream`.
/// 
fn handle_connection(mut stream: TcpStream) {
    // Create a buffer to hold the incoming data.
    let mut buffer = [0; 1024];
    // Read the incoming data into the buffer.
    stream.read(&mut buffer).unwrap();

    // Define the `GET` and `SLEEP` requests.
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // Define the status line and filename based on the request. 
    let (status_line, filename) = if buffer.starts_with(get) {
        // If the request is `GET /`, return `200 OK` and `hello.html`.
        ("HTTP/1.1 200 OK", "hello.html")
    // If the request is `GET /sleep`, sleep for 5 seconds and return `200 OK` and `hello.html`.
    } else if buffer.starts_with(sleep) {
        // Sleep for 5 seconds.
        thread::sleep(std::time::Duration::from_secs(5));
        // Return `200 OK` and `hello.html`.
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        // If the request is anything else, return `404 NOT FOUND` and `404.html`.
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    // Read the contents of the file into a string. If the file doesn't exist, panic.
    let contents = fs::read_to_string(filename).unwrap();

    // Write the response to the stream. Include the status line, content length, and contents.
    let response = format!("{status_line}\r\nContent-Length: {}\r\n\r\n{contents}", contents.len(),);

    // Write the response to the stream. This will close the connection. 
    stream.write_all(response.as_bytes()).unwrap();
    // Flush the stream to ensure all data is written. This will also close the connection.
    stream.flush().unwrap();
}
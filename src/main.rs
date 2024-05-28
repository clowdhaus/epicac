use std::{
  collections::HashMap,
  error::Error,
  io::{Read, Write},
  net::{TcpStream, ToSocketAddrs},
  time::Duration,
};

fn main() -> Result<(), Box<dyn Error>> {
  let host = "localhost:1337";
  let path = "/";
  let ip_lookup = host.to_socket_addrs()?.next().unwrap();

  let mut socket = TcpStream::connect_timeout(&ip_lookup, Duration::from_millis(5000))?;

  let mut headers = HashMap::new();
  headers.insert("Host", host);

  let header = format!(
    "GET {} HTTP/1.1\n{}\n\n",
    path,
    headers
      .iter()
      .map(|(i, x)| format!("{}: {}", i, x))
      .collect::<Vec<_>>()
      .join("\n")
  );
  socket.write(header.as_bytes())?;
  socket.flush()?;

  let mut response = String::new();
  socket.read_to_string(&mut response)?;
  println!("{response}");

  Ok(())
}

// Since we are using the miniserde crate, we do not have access to automatically
// convert from snake_case (Rust) to PascalCase (API)
#![allow(non_snake_case)]

use std::{
  collections::HashMap,
  env,
  error::Error,
  fs,
  io::{BufRead, BufReader, Read, Write},
  net::{TcpStream, ToSocketAddrs},
  str,
  time::Duration,
};

use http::Uri;
use miniserde::{json, Deserialize, Serialize};

fn main() -> Result<(), Box<dyn Error>> {
  let identity = PodIdentity {};
  match get_credentials(identity) {
    Ok(creds) => println!("{creds}"),
    Err(e) => eprintln!("{e}"),
  }

  Ok(())
}

/// Wraper function to allow mocking the function call via trait/trait impl
fn get_credentials(identity: impl CredentialProvider) -> Result<String, Box<dyn Error>> {
  let creds = identity.get_assume_role_credentials()?;
  Ok(creds.to_string())
}

trait CredentialProvider {
  fn get_assume_role_credentials(&self) -> Result<AssumedRoleCreds, Box<dyn Error>>;
}

struct PodIdentity {}

impl CredentialProvider for PodIdentity {
  fn get_assume_role_credentials(&self) -> Result<AssumedRoleCreds, Box<dyn Error>> {
    let uri = env::var("AWS_CONTAINER_CREDENTIALS_FULL_URI")?.parse::<Uri>()?;
    let host = uri.host().unwrap();
    let path = uri.path();
    let ip_lookup = (host, 80).to_socket_addrs()?.next().unwrap();
    let mut socket = match TcpStream::connect_timeout(&ip_lookup, Duration::from_millis(5000)) {
      Ok(s) => s,
      Err(e) => {
        println!("Error connecting to the host:\nHost: {host}\nPath: {path}\nAddress: {ip_lookup}\nError: {e}");
        return Err(Box::new(e));
      }
    };

    let auth_token = fs::read_to_string(env::var("AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE")?)?;

    let mut headers = HashMap::new();
    headers.insert("Host", host);
    headers.insert("Accept", "application/json");
    headers.insert("Authorization", &auth_token);

    let header = format!(
      "GET {} HTTP/1.1\r\n{}\r\n\r\n",
      path,
      headers
        .iter()
        .map(|(i, x)| format!("{}: {}", i, x))
        .collect::<Vec<_>>()
        .join("\n")
    );
    socket.write_all(header.as_bytes())?;
    socket.flush()?;

    let body = extract_body(socket)?;
    let creds = convert_credentials(&body)?;

    Ok(creds)
  }
}

fn extract_body(stream: TcpStream) -> Result<String, Box<dyn Error>> {
  let mut reader = BufReader::new(stream);
  let mut response = String::new();
  loop {
    let r = reader.read_line(&mut response).unwrap();
    if r < 3 {
      //detect empty line
      break;
    }
  }
  let mut size = 0;
  let linesplit = response.split('\n');
  for l in linesplit {
    if l.starts_with("Content-Length") {
      let sizeplit = l.split(':');
      for s in sizeplit {
        if !(s.starts_with("Content-Length")) {
          size = s.trim().parse::<usize>()?;
        }
      }
    }
  }
  let mut buffer = vec![0; size];
  reader.read_exact(&mut buffer)?;

  Ok(str::from_utf8(&buffer)?.to_string())
}

/// Credentials returned from AWS_CONTAINER_CREDENTIALS_FULL_URI
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ContainerCreds {
  AccessKeyId: String,
  SecretAccessKey: String,
  Token: String,
  AccountId: String,
  Expiration: String,
}

/// Credentials used as assumed role
///
/// This is just a conversion of `ContainerAuth`
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AssumedRoleCreds {
  AccessKeyId: String,
  SecretAccessKey: String,
  SessionToken: String,
  Expiration: String,
  Version: i8,
}

/// Poor man's formatted JSON stdout
impl std::fmt::Display for AssumedRoleCreds {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "{{
  \"AccessKeyId\": \"{}\",
  \"SecretAccessKey\": \"{}\",
  \"SessionToken\": \"{}\",
  \"Expiration\": \"{}\",
  \"Version\": {}
}}",
      self.AccessKeyId, self.SecretAccessKey, self.SessionToken, self.Expiration, self.Version
    )
  }
}

fn convert_credentials(credentials: &str) -> Result<AssumedRoleCreds, Box<dyn Error>> {
  let con_creds: ContainerCreds = json::from_str(credentials)?;
  Ok(AssumedRoleCreds {
    AccessKeyId: con_creds.AccessKeyId,
    SecretAccessKey: con_creds.SecretAccessKey,
    SessionToken: con_creds.Token,
    Expiration: con_creds.Expiration,
    Version: 1,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_convert_credentials() {
    let input = r#"{
            "AccessKeyId": "aaa",
            "SecretAccessKey": "bbb",
            "Token": "ccc",
            "AccountId": "ddd",
            "Expiration": "2024-05-29T20:37:18Z"
        }"#;
    let expected = AssumedRoleCreds {
      AccessKeyId: "aaa".to_string(),
      SecretAccessKey: "bbb".to_string(),
      SessionToken: "ccc".to_string(),
      Expiration: "2024-05-29T20:37:18Z".to_string(),
      Version: 1,
    };
    let result = convert_credentials(input).unwrap();
    assert!(result.eq(&expected));
  }

  struct MockPodIdentity {}

  impl CredentialProvider for MockPodIdentity {
    fn get_assume_role_credentials(&self) -> Result<AssumedRoleCreds, Box<dyn Error>> {
      Ok(AssumedRoleCreds {
        AccessKeyId: "ASIAZTS577SRCMNGRXXF".to_string(),
        SecretAccessKey: "dDilvPo0Won/sqEJ1lIt8sjewKKs3M/1OtnaRrGf".to_string(),
        SessionToken: "IQoJb3JpZ2luX2VjEI///////////wEaCXVzLWVhc3QtMSJHMEUCIF6".to_string(),
        Expiration: "2024-05-29T20:37:18Z".to_string(),
        Version: 1,
      })
    }
  }

  #[test]
  fn test_get_credentials() {
    let identity = MockPodIdentity {};
    let credentials = get_credentials(identity).unwrap();
    insta::assert_debug_snapshot!(credentials);
  }
}

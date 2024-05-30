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

/// Wrapper function to allow mocking the function call via trait/trait impl
fn get_credentials(identity: impl CredentialProvider) -> Result<String, Box<dyn Error>> {
  let creds = identity.get_assume_role_credentials()?;

  Ok(creds.to_string())
}

trait CredentialProvider {
  fn get_assume_role_credentials(&self) -> Result<AssumedRoleCreds, Box<dyn Error>>;
}

struct PodIdentity {}

impl CredentialProvider for PodIdentity {
  /// Collects the credentials from the container credential provider and returns them in the format
  /// required for sourcing credentials from an external process (`credential_process`)
  ///
  /// https://docs.aws.amazon.com/sdkref/latest/guide/feature-container-credentials.html
  /// https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-sourcing-external.html
  fn get_assume_role_credentials(&self) -> Result<AssumedRoleCreds, Box<dyn Error>> {
    let uri = env::var("AWS_CONTAINER_CREDENTIALS_FULL_URI")?.parse::<Uri>()?;
    let host = uri.host().unwrap();
    let ip_lookup = (host, 80).to_socket_addrs()?.next().unwrap();
    let mut stream = TcpStream::connect_timeout(&ip_lookup, Duration::from_millis(5000))?;
    let auth_token = fs::read_to_string(env::var("AWS_CONTAINER_AUTHORIZATION_TOKEN_FILE")?)?;

    let mut headers = HashMap::new();
    headers.insert("Host", host);
    headers.insert("Accept", "application/json");
    headers.insert("Authorization", &auth_token);

    let header = format!(
      "GET {} HTTP/1.1\r\n{}\r\n\r\n",
      uri.path(),
      headers
        .iter()
        .map(|(i, x)| format!("{}: {}", i, x))
        .collect::<Vec<_>>()
        .join("\n")
    );
    stream.write_all(header.as_bytes())?;
    stream.flush()?;

    let body = extract_body(stream)?;
    let creds = convert_credentials(&body)?;

    Ok(creds)
  }
}

/// Extract just the body from the stream's GET response using the response's `Content-Length`
fn extract_body(stream: TcpStream) -> Result<String, Box<dyn Error>> {
  let mut reader = BufReader::with_capacity(512, stream);
  let mut response = String::new();

  loop {
    let r = reader.read_line(&mut response)?;
    if r < 3 {
      // detect empty line
      break;
    }
  }

  let cont_len = response
    .split('\n')
    .find(|l| l.starts_with("Content-Length"))
    .map(|l| l.split(':').last().unwrap().trim())
    .unwrap()
    .parse::<usize>()?;

  let mut buffer = vec![0; cont_len];
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

/// Credential format for the external credential process
///
/// The `ContainerCreds` is converted into this format as expected by the API
/// https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-sourcing-external.html
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

/// Convert the container credentials into the expected format for the external credential process
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

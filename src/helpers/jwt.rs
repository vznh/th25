// jwt.rs
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  iat: usize,
  exp: usize,
  iss: String,
}

pub fn create_jwt() -> Result<String, Box<dyn Error>> {
  println!("beginning to create jwt");
  // Hardcoded GitHub App ID and PEM file path
  let app_id = 1146309; // Replace with your actual GitHub App ID
  let pem_path = "certs/fuckyou.pem"; 
  // print this and check if it exists
  // Update to your PEM file's location

  // Attempt to read the private key file
  let key_contents = fs::read_to_string(pem_path)
    .map_err(|e| format!("Failed to read PEM file from '{}': {}", pem_path, e))?;

  println!("---------");
  println!("Key content: {}", key_contents);
  println!("---------");

  let now = Utc::now();
  println!("now: {}", Utc::now());
  let iat = (now.timestamp() - 60) as usize;
  println!("iat: {}", iat);
  let exp = (now + Duration::minutes(10)).timestamp() as usize; // JWT valid for 10 minutes
  println!("exp: {}", exp);

  let claims = Claims { iat, exp, iss: app_id.to_string() };
  println!("claims: {:?}", claims);

  // Create the header using RS256
  let header = Header::new(Algorithm::RS256);

  // Attempt to create the encoding key from the PEM contents
  let encoding_key = EncodingKey::from_rsa_pem(key_contents.as_bytes())
    .map_err(|e| format!("Failed to create encoding key: {}", e))?;

  // Encode the token
  let token =
    encode(&header, &claims, &encoding_key).map_err(|e| format!("Failed to encode JWT: {}", e))?;

  println!("JWT created successfully: {}", token);
  Ok(token)
}

pub async fn exchange_jwt_for_installation_token(
  jwt: &str,
  installation_id: u64,
) -> Result<String, Box<dyn Error>> {
  // Construct the URL to request the installation token.
  let url = format!("https://api.github.com/app/installations/{}/access_tokens", installation_id);

  // Create a reqwest client.
  let client = Client::new();

  // Perform the POST request with the required headers.
  let response = client
    .post(&url)
    .header("Authorization", format!("Bearer {}", jwt))
    .header("Accept", "application/vnd.github+json")
    .header("User-Agent", "3mechanic") // GitHub API requires a User-Agent header.
    .send()
    .await?
    .error_for_status()?;

  // Parse the JSON response.
  let json: Value = response.json().await?;

  // Attempt to extract the installation token.
  if let Some(token) = json.get("token").and_then(|v| v.as_str()) {
    println!("Installation token obtained: xq{}", token);
    Ok(token.to_string())
  } else {
    Err(format!("Failed to obtain installation token. Response: {:?}", json).into())
  }
}

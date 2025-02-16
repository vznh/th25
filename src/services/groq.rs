use base64::{Engine as _};  // Import Engine trait
use reqwest::Client;
use serde_json::{json, Value};
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::str;

/// Groq API Configuration
const GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1/chat/completions";
const API_KEY: &str = "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"; // Replace with your actual API key

/// **Fetch changed files from a commit (Async)**
pub async fn get_changed_files(owner: &str, repo: &str, commit_sha: &str) -> Vec<String> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/commits/{}",
        owner, repo, commit_sha
    );

    println!("Fetching commit details from: {}", api_url);

    let client = Client::new();
    let response = client
        .get(&api_url)
        .header("User-Agent", "rust-client")
        .send()
        .await
        .expect("Failed to fetch commit data")
        .json::<Value>()
        .await
        .expect("Failed to parse commit response");

    let files = response["files"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|f| f["filename"].as_str().map(String::from))
        .collect::<Vec<String>>();

    println!("Changed Files: {:?}", files);
    files
}

/// **Fetch file contents for changed files (Async)**
pub async fn get_file_contents(owner: &str, repo: &str, commit_sha: &str) -> Vec<(String, String)> {
    let changed_files = get_changed_files(owner, repo, commit_sha).await;
    let client = Client::new();
    let mut file_contents = Vec::new();

    for file in changed_files.iter() {
        let file_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
            owner, repo, file, commit_sha
        );

        println!("📂 Fetching file content from: {}", file_url);

        if let Ok(resp) = client.get(&file_url).header("User-Agent", "rust-client").send().await {
            if let Ok(response_text) = resp.text().await {
                if let Ok(json) = serde_json::from_str::<Value>(&response_text) {
                    if let Some(content) = json["content"].as_str() {
                        let decoded_content = base64::engine::general_purpose::STANDARD
                            .decode(content.replace("\n", ""))
                            .expect("Failed to decode base64 content");
                        let file_content_str = str::from_utf8(&decoded_content)
                            .expect("Failed to convert content to string");

                        file_contents.push((file.clone(), file_content_str.to_string()));
                    }
                }
            }
        }
    }

    println!("✅ Fetched content for {} files.", file_contents.len());
    file_contents
}

/// **Extract functions from code and save as JSON (Async)**
pub async fn extract_new_functions(owner: &str, repo: &str, commit_sha: &str) {
    let client = Client::new();
    let files_with_contents = get_file_contents(owner, repo, commit_sha).await;

    if files_with_contents.is_empty() {
        println!("No new changes. Exiting.");
        return;
    }

    let file_summaries: Vec<String> = files_with_contents
        .iter()
        .map(|(file, content)| format!("📄 File: {}\n```\n{}\n```", file, content))
        .collect();

    let prompt = format!(
        "You are analyzing a commit in a software repository. Your goal is to:
    1. **Extract high-impact functions.**
    2. Identify function dependencies.

    **Return JSON only:**
    {{
        \"functions\": [
            {{
                \"name\": \"function_name\",
                \"file\": \"file_path\",
                \"commit_id\": \"{}\",
                \"body\": \"function_code\",
                \"description\": \"Function description\",
                \"dependencies\": [\"dependency1\", \"dependency2\"]
            }}
        ]
    }}

    Changed files:\n\n{}",
        commit_sha,
        file_summaries.join("\n\n")
    );

    let response = client
        .post(GROQ_ENDPOINT)
        .header("Authorization", format!("Bearer {}", API_KEY))
        .json(&json!({
            "model": "deepseek-r1-distill-llama-70b",
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 8000,
            "reasoning_format": "hidden"
        }))
        .send()
        .await;

    let response_json: serde_json::Value = response
        .expect("Failed to send request")
        .json()
        .await
        .expect("Failed to parse response");

    println!("\n🧠 Groq AI Response:\n{:#?}\n", response_json);
    save_to_file("functions.json", &response_json);
}

/// **Convert JSON to XML (Async)**
pub async fn json_to_xml() -> String {
    // Read the JSON file asynchronously
    let json_content = match tokio::fs::read_to_string("functions.json").await {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read functions.json: {:?}", e);
            return "".to_string();
        }
    };

    let prompt = format!(
        "Convert JSON to XML:
        
        **Format Rules:**
        - Root element: `<functions>`.
        - Each function inside `<function>` tag.
        - Wrap function code inside `<![CDATA[ function_code ]]>`.
        - Dependencies inside `<dependencies>` with `<dependency>` tags.

        **JSON Input:**\n\n{}",
        json_content
    );

    let client = reqwest::Client::new();

    let request_body = json!({
        "model": "deepseek-r1-distill-llama-70b",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 8000,
        "reasoning_format": "hidden"
    });

    // Await the HTTP request
    let response = match client
        .post(GROQ_ENDPOINT)
        .header("Authorization", format!("Bearer {}", API_KEY))
        .json(&request_body)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error sending request: {:?}", e);
            return "".to_string();
        }
    };

    // Await JSON parsing
    let response_json: Value = match response.json().await {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error parsing JSON response: {:?}", e);
            return "".to_string();
        }
    };

    response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string()
}


/// **Send XML content to Groq (Async)**
pub async fn send_request_to_groq() -> Result<String, reqwest::Error> {
    let xml_path = "functions.xml";
    let xml_content = match read_xml_from_file(xml_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading XML file: {:?}", e);
            return Ok("".to_string());
        }
    };

    let prompt = format!(
        "Using the XML provided:\n{}\n\
        Generate a JSON-formatted GitHub comment for code review.",
        xml_content
    );

    let client = Client::new();

    let request_body = json!({
        "model": "deepseek-r1-distill-llama-70b",
        "messages": [{ "role": "user", "content": prompt }],
        "max_tokens": 10000,
        "reasoning_format": "hidden"
    });

    // Await the request separately
    let response = client
        .post(GROQ_ENDPOINT)
        .header("Authorization", format!("Bearer {}", API_KEY))
        .json(&request_body)
        .send()
        .await?;  // ✅ Await the request

    // Await JSON conversion separately
    let response_json: Value = response.json().await?;  // ✅ Await the JSON parsing

    Ok(response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string())
}


/// Save the XML output to a file
pub fn save_xml_to_file(xml_content: &str) {
    let file_path = "functions.xml";
    fs::write(file_path, xml_content).expect("Failed to write XML file");
    println!("XML saved as: {}", file_path);
}

/// Save JSON file
pub fn save_to_file(file_name: &str, content: &Value) {
    let file_path = std::path::Path::new(file_name);
    let mut file = File::create(file_path).expect("Failed to create file");

    let json_content = serde_json::to_string_pretty(content).expect("❌ Failed to serialize JSON");
    file.write_all(json_content.as_bytes())
        .expect("Failed to write JSON file");

    println!("File saved as: {}", file_name);
}

fn read_xml_from_file(file_path: &str) -> Result<String, io::Error> {
    let mut file = fs::File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// groq.rs
use base64;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::str;
use std::fs::File;
use std::io::Write;

// Fetch the changed files from a commit (GitHub API)
pub fn get_changed_files(owner: &str, repo: &str, commit_sha: &str) -> Vec<String> {
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
        .expect("Failed to fetch commit data")
        .json::<Value>()
        .expect("Failed to parse commit response");

    // Debugging: Print the raw API response
    //println!("🔍 Raw API Response: {:#?}", response);

    let files = response["files"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|f| f["filename"].as_str().map(String::from))
        .collect::<Vec<String>>();

    println!("Changed Files: {:?}", files);
    files
}
pub fn get_file_contents(owner: &str, repo: &str, commit_sha: &str) -> Vec<(String, String)> {
    let changed_files = get_changed_files(owner, repo, commit_sha);
    let client = Client::new();
    let mut file_contents = Vec::new();
    let mut fetched_files = Vec::new(); // Keep track of files we've already retrieved

    for file in changed_files.iter() {
        // If we've already fetched this file, skip fetching again
        if fetched_files.contains(file) {
            println!("📂 Using previously fetched file content for: {}", file);
            continue;
        }

        let file_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
            owner, repo, file, commit_sha
        );

        println!("📂 Fetching file content from: {}", file_url);

        let response = client
            .get(&file_url)
            .header("User-Agent", "rust-client")
            .send();

        if let Ok(resp) = response {
            if let Ok(response_text) = resp.text() {
                let response_json: Result<Value, serde_json::Error> = serde_json::from_str(&response_text);

                if let Ok(json) = response_json {
                    if let Some(content) = json["content"].as_str() {
                        let decoded_content = base64::decode(content.replace("\n", ""))
                            .expect("Failed to decode base64 content");
                        let file_content_str = str::from_utf8(&decoded_content)
                            .expect("Failed to convert content to string");

                        file_contents.push((file.clone(), file_content_str.to_string()));

                        // Add file to fetched list to avoid redundant API calls
                        fetched_files.push(file.clone());
                    }
                }
            }
        }
    }

    println!("✅ Fetched content for {} files.", file_contents.len());
    file_contents
}

/// Extract new functions and write to JSON/XML files
pub fn extract_new_functions(owner: &str, repo: &str, commit_sha: &str) {
    let client = Client::new();
    let files_with_contents = get_file_contents(owner, repo, commit_sha);

    if files_with_contents.is_empty() {
        println!("No new changes. Exiting.");
        return;
    }

    let file_summaries: Vec<String> = files_with_contents
        .iter()
        .map(|(file, content)| format!("File: {}\n```rust\n{}\n```", file, content))
        .collect();

    let prompt = format!(
        "You are analyzing newly committed code in a repository. Your task is:

        1. **Extract function definitions** from the changed files.
        2. Identify function dependencies that were not changed but are used.
        3. Ignore non-code files like README, images, and config files.
        4. Return results in **both JSON and XML format**.

        **JSON Format Example:**
        ```json
        {{
            \"functions\": [
                {{ \"name\": \"function_name\", \"file\": \"file_path\", \"commit_id\": \"commit_id\" }}
            ]
        }}
        ```

        **XML Format Example:**
        ```xml
        <functions>
            <function>
                <name>function_name</name>
                <file>file_path</file>
                <commit_id>commit_id</commit_id>
                <body><![CDATA[ full function code ]]></body>
            </function>
        </functions>
        ```

        Output both JSON and XML. Do **not** return anything except the formatted response.

        Here are the changed files:\n\n{}",
        file_summaries.join("\n\n")
    );

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", "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"))
        .json(&json!({
            "model": "deepseek-r1-distill-llama-70b",
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 4000
        }))
        .send();

    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error sending request: {:?}", e);
            return;
        }
    };

    let response_json: Value = match response.json() {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to parse response as JSON: {:?}", e);
            return;
        }
    };

    let extracted_text = response_json["choices"]
        .get(0)
        .and_then(|choice| choice["message"]["content"].as_str());

    if extracted_text.is_none() {
        eprintln!("Error: Missing 'choices' or 'message' content in response.");
        return;
    }

    let extracted_text = extracted_text.unwrap();

    // Separate JSON and XML parts
    let json_start = extracted_text.find("{").unwrap_or(0);
    let json_end = extracted_text.find("</functions>").map(|i| i + 11).unwrap_or(extracted_text.len());

    let json_part = &extracted_text[json_start..json_end];
    let xml_part = extracted_text[json_end..].trim();

    // Write JSON file
    if let Ok(mut file) = File::create("functions.json") {
        if let Err(e) = file.write_all(json_part.as_bytes()) {
            eprintln!("Error writing JSON file: {:?}", e);
        } else {
            println!("✅ functions.json written successfully.");
        }
    } else {
        eprintln!("Error creating functions.json file.");
    }

    // Write XML file
    if let Ok(mut file) = File::create("functions.xml") {
        if let Err(e) = file.write_all(xml_part.as_bytes()) {
            eprintln!("Error writing XML file: {:?}", e);
        } else {
            println!("✅ functions.xml written successfully.");
        }
    } else {
        eprintln!("Error creating functions.xml file.");
    }
}

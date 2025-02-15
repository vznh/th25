use dotenv::dotenv;
use reqwest::blocking::Client;
use serde_json::json;
use serde_json::Value;
use std::env;

const GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1/chat/completions";

pub fn send_request_to_groq(xml: &str) -> String {
    dotenv().ok();
    let groq_api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY must be set in the .env file");

    let prompt = format!(
        "Using the XML provided:\n{}\n\
    Generate a JSON-formatted inline code review comment payload with exactly these keys in this order: \
    1) \"body\", 2) \"commit_id\", 3) \"path\", and 4) \"position\".\n\n\
    For this task, treat each <Statement> element in the XML as a separate line of code, numbered sequentially in the order they appear. \n\n\
    Identify the statement that is most in need of optimization—that is, the code that appears redundant, indirect, or inefficient. For example, a method that only calls another method when the functionality could be inlined is likely a target for improvement.\n\n\
    In the provided XML, examine each <Statement> element and decide which one should be optimized. Then, use its sequential line number as the value for the key \"position\". If the target cannot be clearly determined, output \"N/A\" instead.\n\n\
    **body** must contain a Markdown-formatted GitHub comment with the following structure:\n\
    - A header line with file path and line range (e.g. src/SwapUtil.py | Line 13-22)\n\
    - A bolded short title describing the change (e.g. **[Title: Simplify Method Call]**)\n\
    - A concise explanation of why the change is needed, using inline code formatting where relevant\n\
    - A code block in diff syntax showing what changes to add (+) and remove (-)\n\
      *Important:* Do not include any XML, HTML, or other tags (e.g. <Statement> tags) in the code block. Only output plain text code changes.\n\
    - A short justification or conclusion\n\n\
    **commit_id** is the commit id we are looking at (e.g. \"abc123\").\n\
    **path** is the path of the file (e.g. \"src/SwapUtil.py\").\n\
    **position** must be the line number (from the sequential numbering of <Statement> elements) corresponding to the optimization target as determined above. If no clear target exists, output \"N/A\".\n\n\
    Return only this JSON object—no additional text or explanations. Focus on the most necessary code improvements from the XML, rather than random optimizations.",
    xml
    );

    let client = Client::new();

    let request_body = json!({
        "model": "deepseek-r1-distill-llama-70b",
        "messages": [{ "role": "user", "content": prompt }],
        "max_tokens": 10000
    });

    let response: Value = client
        .post(GROQ_ENDPOINT)
        .header("Authorization", format!("Bearer {}", groq_api_key))
        .json(&request_body)
        .send()
        .expect("Failed to send request to Groq API")
        .json()
        .expect("Failed to parse response");

    response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

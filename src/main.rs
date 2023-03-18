use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::error::Error;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub usage: Usage,
    pub choices: Vec<Choice>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: i64,
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: i64,
    #[serde(rename = "total_tokens")]
    pub total_tokens: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub message: Message,
    #[serde(rename = "finish_reason")]
    pub finish_reason: String,
    pub index: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub role: String,
    pub content: String,
}

async fn call_chatgpt_api(api_key: &str, prompt: &str) -> Result<String, Box<dyn Error>> {
    let endpoint = "https://api.openai.com/v1/chat/completions";
    let request_body: Value = json!({
    "model": "gpt-3.5-turbo",
    "messages": [
        {
        "role": "system",
        "content": "You are a helpful assistant."
        },
        {
        "role": "user",
        "content": prompt.to_string(),
        }
    ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(endpoint)
        .header(AUTHORIZATION, format!("Bearer {}", api_key.trim()))
        .header(CONTENT_TYPE, "application/json")
        .json(&request_body)
        .send()
        .await?;

    let response_body: Root = response.json().await?;
    //dbg!(response_body); Ok("".to_string())
    let generated_text = response_body.choices[0].message.content.clone();

    Ok(generated_text)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = tokio::io::stdout();
    stdout.write_all(b"Downloading auto subs...\n").await?;
    stdout.flush().await?;
    let mut child = Command::new("yt-dlp")
        .arg("--write-auto-subs")
        .arg("--skip-download")
        .arg("-o")
        .arg("subs")
        .arg(std::env::args().nth(1).unwrap())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();

    child.wait().await?;

    let s = String::from_utf8(tokio::fs::read("subs.en.vtt").await?)?;

    use regex::Regex;
    let re = Regex::new(r#"<.*?>"#)?;

    let output = re.replace_all(&s, "");

    let re2 = Regex::new(r"^\d{2}:(\d{2}:\d{2})\.\d{3} --> \d{2}:\d{2}:\d{2}\.\d{3}")?;
    let mut lines_map = HashSet::new();
    let mut formatted_lines = Vec::new();
    let mut cur_timestamp = None;
    let every_n_lines = 10;
    for line in output.lines() {
        let line = line.trim();
        if let Some(timestamp) = re2.captures(line).and_then(|x| x.get(1)) {
            cur_timestamp = Some(timestamp.as_str());
        } else if line.len() > 0 && cur_timestamp.is_some() && !lines_map.contains(line) {
            if formatted_lines.len() % every_n_lines == 0 {
                formatted_lines.push(format!("{}: {}", cur_timestamp.unwrap(), line));
            } else {
                formatted_lines.push(format!("{}", line));
            }
            lines_map.insert(line);
        }
    }

    let subs = formatted_lines.join("\n");
    //println!("{}", subs);

    let api_key = String::from_utf8(tokio::fs::read("api_key.txt").await?)?;

    stdout.write_all(b"Asking for a summary...\n\n").await?;
    stdout.flush().await?;
    let input = format!("Can you summarize the video whose subtitles are below? After the quick summary, provide a more detailed overview of the topics discussed in the video, and briefly touch on them. Then provide a very detailed listing of each subject discussed and the corresponding timestamps where each subject can be found.\n\nSUBTITLES START:\n{}\n\nSUBTITLES END\n", subs);
    let response = call_chatgpt_api(&api_key, &input).await?;
    stdout.write_all(response.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;

    Ok(())
}

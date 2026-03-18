use std::process::Command;
use glob::glob;
use std::fs;
use std::io::{self, Write};

const MAX_CHUNK_CHARS: usize = 12_000;

async fn call_claude(system: &str, user_message: &str, max_tokens: u32, api_key: &str) -> String {
    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": max_tokens,
        "system": system,
        "messages": [{ "role": "user", "content": user_message }]
    });

    let client = reqwest::Client::new();
    let max_retries = 3;
    let mut delay_secs = 10;

    for attempt in 0..=max_retries {
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .unwrap();

        let status = resp.status();
        let res: serde_json::Value = resp.json().await.unwrap();

        if let Some(text) = res["content"][0]["text"].as_str() {
            return text.to_string();
        }

        let is_rate_limit = status == 429
            || res["error"]["type"].as_str() == Some("rate_limit_error");

        if is_rate_limit && attempt < max_retries {
            eprintln!(
                "Rate limited, retrying in {}s... (attempt {}/{})",
                delay_secs,
                attempt + 1,
                max_retries
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
            delay_secs *= 2;
            continue;
        }

        eprintln!("API error: {}", res);
        std::process::exit(1);
    }

    unreachable!()
}

fn split_diff_into_chunks(diff: &str) -> Vec<String> {
    if diff.len() <= MAX_CHUNK_CHARS {
        return vec![diff.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for line in diff.lines() {
        if current_chunk.len() + line.len() + 1 > MAX_CHUNK_CHARS && !current_chunk.is_empty() {
            chunks.push(current_chunk);
            current_chunk = String::new();
        }
        current_chunk.push_str(line);
        current_chunk.push('\n');
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

async fn summarize_chunk(chunk: &str, chunk_index: usize, total_chunks: usize, api_key: &str) -> String {
    let system = "You summarize git diffs. Be concise but preserve all important details: files changed, what was added/removed/modified, and why.";
    let prompt = format!(
        "Summarize this git diff (part {}/{}):\n\n{}",
        chunk_index + 1,
        total_chunks,
        chunk
    );
    println!("  Summarizing chunk {}/{}...", chunk_index + 1, total_chunks);
    call_claude(system, &prompt, 512, api_key).await
}

async fn generate_commit_message(context: &str, api_key: &str) -> String {
    let system = "You generate git commit messages. Output ONLY the commit message. No markdown, no backticks.";
    let prompt = format!(
r#"You are a Git commit message generator.

Analyze the following changes and generate a commit message.

Rules:
- First line: a short commit title using Conventional Commits format (feat:, fix:, refactor:, docs:, chore:, test:, style:, perf:)
- Second line: empty
- Third line onwards: a concise description of what changed and why (2-5 lines max)
- Write in English
- Be specific, not vague (no "updated files" or "various changes")
- Focus on the WHY and WHAT, not the HOW

Output format (strictly follow this, no extra text):
<title>
<empty line>
<description>

Example output:
feat: add user authentication via OAuth2

Implement Google OAuth2 login flow with token refresh.
Add session middleware and protect /dashboard routes.
Store refresh tokens in encrypted cookie.

--- CHANGES ---
{context}"#
    );
    call_claude(system, &prompt, 512, api_key).await
}

fn get_api_key() -> String {
    match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            print!("ANTHROPIC_API_KEY not found. Enter your API key: ");
            io::stdout().flush().unwrap();

            let mut key = String::new();
            io::stdin().read_line(&mut key).unwrap();
            let key = key.trim().to_string();

            let home = std::env::var("HOME").unwrap();
            let shell_rc = if std::path::Path::new(&format!("{home}/.zshrc")).exists() {
                format!("{home}/.zshrc")
            } else {
                format!("{home}/.bashrc")
            };

            let rc_content = std::fs::read_to_string(&shell_rc).unwrap_or_default();
            if !rc_content.contains("ANTHROPIC_API_KEY") {
                let mut file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&shell_rc)
                    .unwrap();
                writeln!(file, "\nexport ANTHROPIC_API_KEY=\"{key}\"").unwrap();
            }

            println!("✓ Key saved to {shell_rc}");
            println!("  Run `source {shell_rc}` or restart your terminal for future sessions.");

            unsafe { std::env::set_var("ANTHROPIC_API_KEY", &key) };

            key
        }
    }
}

fn get_the_diff_for_github() -> Result<String, String> {
	let output = Command
		::new("git")
		.args(&["diff", "--staged"])
		.output()
		.map_err(|e| format!("Failed to execute git command: {}", e))?;
	if !output.status.success()
	{
		return Err(format!("Git command failed with status: {}", output.status));
	}
	let diff = String::from_utf8(output.stdout)
		.map_err(|e| format!("Failed to parse git output: {}", e))?;
	if diff.is_empty()
	{
		return Err("No differences found between the last two commits.".to_string());
	}
	Ok(diff)
}

const MAX_README_CHARS: usize = 2_000;

fn get_all_the_readmes() -> Result<String, String> {
	let mut readmes = String::new();
	let mut total_chars = 0;
	for entry in glob("**/README*.md").unwrap().flatten() {
		let path = entry;
		let content = fs::read_to_string(&path)
			.map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
		readmes.push_str("readme name is: \t");
		readmes.push_str(path.file_name().unwrap().to_string_lossy().as_ref());
		readmes.push_str("\n\n");
		readmes.push_str("content is: \n");
		let remaining = MAX_README_CHARS.saturating_sub(total_chars);
		if remaining == 0 {
			readmes.push_str("[truncated]\n\n");
			break;
		}
		let truncated = &content[..content.len().min(remaining)];
		readmes.push_str(truncated);
		if content.len() > remaining {
			readmes.push_str("\n[truncated]");
		}
		readmes.push_str("\n\n");
		total_chars += truncated.len();
	}
	Ok(readmes)
}

fn cut_the_prompt(prompt: &str) -> (String, String) {
	let mut lines = prompt.lines();
	let title = lines.next().unwrap_or("").to_string();
	let remaining: Vec<&str> = lines.collect();
	let body = if remaining.first() == Some(&"") {
		remaining[1..].join("\n")
	} else {
		remaining.join("\n")
	};
	(title, body)
}

#[tokio::main]
async fn main() {
	Command::new("git")
		.args(&["add", "."])
		.output()
		.expect("Failed to execute git add command");

	let diff = match get_the_diff_for_github() {
		Ok(d) => d,
		Err(e) => {
			eprintln!("Error: {}", e);
			return;
		}
	};

	let readmes = get_all_the_readmes().unwrap_or_default();

	let api_key = get_api_key();

	let chunks = split_diff_into_chunks(&diff);
	let context = if chunks.len() == 1 {
		println!("Generating commit message...");
		let mut ctx = String::new();
		ctx.push_str(&diff);
		if !readmes.is_empty() {
			ctx.push_str("\n\n--- README CONTEXT ---\n");
			ctx.push_str(&readmes);
		}
		ctx
	} else {
		println!("Diff is large ({} chars), splitting into {} chunks...", diff.len(), chunks.len());
		let mut summaries = Vec::new();
		for (i, chunk) in chunks.iter().enumerate() {
			let summary = summarize_chunk(chunk, i, chunks.len(), &api_key).await;
			summaries.push(summary);
		}
		let mut ctx = String::from("Summaries of all changes:\n\n");
		for (i, summary) in summaries.iter().enumerate() {
			ctx.push_str(&format!("--- Part {}/{} ---\n{}\n\n", i + 1, summaries.len(), summary));
		}
		if !readmes.is_empty() {
			ctx.push_str("--- README CONTEXT ---\n");
			ctx.push_str(&readmes);
		}
		ctx
	};

	let prompt_result = generate_commit_message(&context, &api_key).await;

	let (title, body) = cut_the_prompt(&prompt_result);
	println!("Generated commit message:\n{}\n{}", title, body);
	let mut cmd = Command::new("git");
	cmd.args(["commit", "-m", &title]);
	if !body.is_empty() {
		cmd.args(["-m", &body]);
	}
	let commit_output = cmd.output().expect("Failed to execute git commit");
	if commit_output.status.success() {
		println!("Commit created successfully.");
		let mut push_cmd = Command::new("git");
		push_cmd.arg("push");
		let push_output = push_cmd.output().expect("Failed to execute git push");
		if push_output.status.success() {
			println!("Changes pushed successfully.");
		} else {
			let stderr = String::from_utf8_lossy(&push_output.stderr);
			eprintln!("Git push failed: {}", stderr);
		}
	} else {
		let stderr = String::from_utf8_lossy(&commit_output.stderr);
		eprintln!("Git commit failed: {}", stderr);
	}
}

use std::process::Command;
use glob::glob;
use std::fs;
use std::io::{self, Write};

async fn ask_claude(output: &str, api_key: &str) -> String {
    let body = serde_json::json!({
        "model": "claude-haiku-4-5-20251001",
        "max_tokens": 512,
        "system": "You generate git commit messages. Output ONLY the commit message. No markdown, no backticks.",
        "messages": [{ "role": "user", "content": prepare_the_prompt(output) }]
    });

    let res: serde_json::Value = reqwest::Client::new()
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();

    match res["content"][0]["text"].as_str() {
        Some(text) => text.to_string(),
        None => {
            eprintln!("API error: {}", res);
            std::process::exit(1);
        }
    }
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

fn get_all_the_readmes() -> Result<String, String> {
	let mut readmes = String::new();
	for entry in glob("**/README*.md").unwrap().flatten() {
		let path = entry;
		readmes.push_str("readme name is: \t");
		readmes.push_str(path.file_name().unwrap().to_string_lossy().as_ref());
		readmes.push_str("\n\n");
		let content = fs::read_to_string(&path)
			.map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
		readmes.push_str("content is: \n");
		readmes.push_str(&content);
		readmes.push_str("\n\n");
	}
	Ok(readmes)
}

fn prepare_the_prompt(output: &str) -> String {
    format!(
r#"You are a Git commit message generator.

Analyze the following git diff and generate a commit message.

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

--- GIT DIFF ---
{output}"#
    )
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
	let diff;
	let mut output = String::new();
	let readmes;

	Command::new("git")
		.args(&["add", "."])
		.output()
		.expect("Failed to execute git add command");

	diff = get_the_diff_for_github();
	match &diff {
		Ok(diff) => {
			output.push_str("The following changes were detected between the last two commits:\n\n");
			output.push_str(&diff);
		}
		Err(e) => {
			eprintln!("Error: {}", e);
		}
	}
	if diff.is_ok() {
		readmes = get_all_the_readmes();
		match readmes {
			Ok(readmes) if !readmes.is_empty() => {
				output.push_str("\n\nThe following Readme files were found:\n\n");
				output.push_str(&readmes);
			}
			Ok(_) => {
				println!("No README.md files found.");
			}
			Err(e) => {
				eprintln!("Error: {}", e);
			}
		}
	}
	let prompt = prepare_the_prompt(&output);
	if prompt.is_empty() {
		eprintln!("Failed to prepare the prompt.");
	}

	let prompt_result = ask_claude(&output, &get_api_key()).await;

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

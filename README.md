<div align="center">

# auto-push

**One command to commit and push — AI writes the message.**

A CLI tool that stages your changes, generates a meaningful commit message using Claude, commits, and pushes. No more writing commit messages by hand.

![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-5C6BC0)
![License](https://img.shields.io/badge/License-MIT-green)
![Dependencies](https://img.shields.io/badge/Dependencies-5-7E57C2)

<a href="README.fr.md"><img src="https://img.shields.io/badge/%F0%9F%87%AB%F0%9F%87%B7_Lire_en_fran%C3%A7ais-blue?style=for-the-badge" alt="Lire en francais"></a>

</div>

---

## What It Does

auto-push analyzes your staged git diff, collects README context, sends it to the Claude API, and generates a **Conventional Commits** message automatically.

```
$ auto_push

Generated commit message:
feat: add user authentication via OAuth2

Implement Google OAuth2 login flow with token refresh.
Add session middleware and protect /dashboard routes.

Commit created successfully.
Changes pushed successfully.
```

### Key features

- **AI-generated commit messages** using Claude (Haiku) via the Anthropic API
- **Conventional Commits format** — `feat:`, `fix:`, `refactor:`, `docs:`, etc.
- **Context-aware** — includes README files content for better commit messages
- **Full pipeline** — stages, commits, and pushes in a single command
- **API key persistence** — saves your key to `.zshrc`/`.bashrc` on first use

---

## Quick Start

```sh
cargo install --path .
auto_push
```

On first run, you'll be prompted for your Anthropic API key. It will be saved to your shell config automatically.

To update, re-run the same command. To uninstall:

```sh
cargo uninstall auto_push
```

---

## Prerequisites

- **Git** — must be run inside a git repository
- **Anthropic API key** — get one at [console.anthropic.com](https://console.anthropic.com)

---

## Usage

```
auto_push
```

Just run it from any git repository with staged or unstaged changes. The tool handles everything:

```
git add .  →  git diff --staged  →  Claude API  →  git commit  →  git push
```

---

## How It Works

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Stage      │────▶│  Diff        │────▶│  Context     │────▶│  Claude API  │────▶│  Commit      │
│  git add .   │     │  --staged    │     │  + READMEs   │     │  (Haiku)     │     │  + Push      │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

| Stage | Function | What it does |
|:------|:---------|:-------------|
| **Stage** | `main()` | Runs `git add .` to stage all changes |
| **Diff** | `get_the_diff_for_github()` | Captures `git diff --staged` output |
| **Context** | `get_all_the_readmes()` | Collects all `README*.md` files for project context |
| **Generate** | `ask_claude()` | Sends diff + context to Claude API, returns a commit message |
| **Commit & Push** | `main()` | Runs `git commit -m` with the generated message, then `git push` |

---

## Tech Stack

| | Crate | Usage |
|:-|:------|:------|
| ![Reqwest](https://img.shields.io/badge/reqwest-0.13-5C6BC0?logoColor=white) | reqwest | HTTP client for Claude API calls |
| ![Serde](https://img.shields.io/badge/serde-1-7E57C2?logoColor=white) | serde + serde_json | JSON serialization for API request/response |
| ![Tokio](https://img.shields.io/badge/tokio-1-9575CD?logoColor=white) | tokio | Async runtime for HTTP requests |
| ![Glob](https://img.shields.io/badge/glob-0.3-7986CB?logoColor=white) | glob | File pattern matching for README discovery |

**5 dependencies.** Single-file architecture, no framework overhead.

---

## Project Structure

```
auto-push/
├── src/
│   └── main.rs        # Everything: API calls, git commands, prompt generation
├── Cargo.toml
├── Cargo.lock
├── README.md
├── README.fr.md
└── .gitignore
```

---

## License

MIT

---

<p align="center">
  <sub>Built by Mateon — Powered by Rust & Claude</sub>
</p>

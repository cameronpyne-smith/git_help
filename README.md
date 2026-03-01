# git_help

Never write a commit message again!

Tired of writing pull request descriptions? And trying to remember the changes you made last week? Git help!

## Features
**Commit** -- Get AI to write a commit message for you, based on the changes.

**PR** -- Generate a commit message, write a PR title and description and open the PR in the browser, all with one command.

## Installation

Download the latest binary from the [Releases](https://github.com/cameronpyne-smith/git_help/releases) page, or build from source:

## Configuration

Create a `.env.git_help` file in the same location as the executable, or in the current working directory that you will run the command, 
this will take precedence over the base if it exists:

```env
# AI provider: "google" or "openai"
AI_PROVIDER=openai

# Google Gemini
GOOGLE_API_KEY=your_google_api_key
GOOGLE_MODEL=gemini-3-flash-preview       # optional, this is the default

# OpenAI
OPEN_AI_API_KEY=your_openai_api_key
OPEN_AI_MODEL=gpt-4.1-nano                # optional, defaults to gpt-4.1-mini (cheapest model that works well)

# GitHub PAT (required for creating PRs)
GITHUB_TOKEN=your_github_pat
```

### Getting API keys

| Key | Where to get it |
|---|---|
| `GOOGLE_API_KEY` | [Google AI Studio](https://aistudio.google.com/apikey) (free tier available) |
| `OPEN_AI_API_KEY` | [OpenAI Platform](https://platform.openai.com/api-keys) |
| `GITHUB_TOKEN` | [GitHub Settings → Developer settings → Personal access tokens](https://github.com/settings/tokens) — needs `repo` scope |

## Usage

```bash
# Commit with a manual message
git_help commit fix the login bug

# Commit with an AI-generated message
git_help commit-ai

# Push and open the PR page in your browser
git_help pr

# Commit, push, and create a PR with AI-generated title and description
git_help pr-ai
```

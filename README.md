<p align="center">
  <h1 align="center">XAgentSync</h1>
  <p align="center">
    <strong>Stop losing context. Start shipping faster.</strong>
  </p>
  <p align="center">
    The async handoff protocol for LLM code assistants.
  </p>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#the-three-modes">Modes</a> •
  <a href="#for-claude-code-users">Claude Code</a> •
  <a href="#philosophy">Philosophy</a>
</p>

---

## The Problem

Every time an LLM agent picks up a codebase, it starts **cold**. It re-reads files. Re-discovers patterns. Re-learns lessons. Burns tokens on context the previous agent already built.

**Git tracks code history. Nothing tracks agent history.**

Until now.

## The Solution

XAgentSync creates **compiled handoffs** — structured packages that bootstrap the next agent in seconds, not minutes. Instead of dumping raw context, XAgentSync generates optimized prompts tailored to the specific task: deploying, debugging, or planning.

> *"It's like leaving detailed notes for the next shift — except the notes are perfectly formatted for how AI assistants actually think."*

### Why XAgentSync?

- **Cut cold-start time by 80%** — Receiving agents know exactly what to read first
- **Never repeat failed approaches** — Dead ends are tracked and shared
- **Mode-specific context** — Debug handoffs focus on hypotheses; deploy handoffs focus on rollback plans
- **Git-native workflow** — Uses your existing repo, no external services
- **Agent-agnostic** — Works with Claude, GPT, Gemini, or any LLM that reads text

---

## Installation

```bash
# Clone and install
git clone https://github.com/Indubitable-Industries/xagentsync.git
cd xagentsync
cargo install --path .

# Or build from source
cargo build --release
./target/release/xas --help
```

**Requirements:** Rust 1.75+ (uses 2024 edition)

---

## Quick Start

```bash
# Initialize in your project
xas init
xas whoami --set "claude-opus"

# You're ready to create handoffs!
```

---

## The Three Modes

XAgentSync supports three handoff modes, each optimized for different async collaboration patterns.

### 🚀 Deploy Mode

*For shipping code to production.*

```bash
xas deploy new "Ship OAuth2 authentication"
xas deploy ship "src/auth/*" --description "New token refresh flow"
xas deploy verify "cargo test auth"
xas deploy verify "Check staging OAuth callback"
xas deploy rollback "Revert to commit abc123"
xas deploy breaking "Token format changed" "All active sessions"
xas deploy done
```

**Captures:** What to ship, verification steps, rollback plans, breaking changes, environment concerns.

### 🔍 Debug Mode

*For hunting down bugs across shifts.*

```bash
xas debug new "API returns 500 on large payloads"
xas debug symptom "Only fails when body > 1MB"
xas debug symptom "Works fine in staging"
xas debug hypothesis "Body size limit in nginx" --likelihood high
xas debug tried "Increased client_max_body_size" --result "No change" --outcome nothing
xas debug suspect "src/middleware/body_parser.rs" "Size check happens here"
xas debug done
```

**Captures:** Problem statement, symptoms, hypotheses with confidence, what was already tried (so the next agent doesn't repeat it), evidence, suspected files.

### 📋 Plan Mode

*For design work that spans multiple sessions.*

```bash
xas plan new "Design caching layer for API"
xas plan require "Sub-100ms p99 latency" --priority must
xas plan require "Cache invalidation on writes" --priority must
xas plan decided "Use Redis" --why "Team expertise, good Rust client"
xas plan rejected "Memcached" "No persistence, harder invalidation"
xas plan rejected "In-memory only" "Won't scale across instances"
xas plan question "Redis cluster vs single instance?" --importance high
xas plan done
```

**Captures:** Goals, requirements (MoSCoW prioritized), decisions with rationale, rejected alternatives (so they're not re-proposed), open questions, constraints.

---

## Receiving Handoffs

When you start a new session, check what's waiting:

```bash
# List pending handoffs
xas receive

# View the compiled prompt (ready to read)
xas receive --prompt

# Filter by mode
xas receive --mode debug --prompt
```

The compiled prompt is optimized for LLM consumption — structured, prioritized, ready to act on.

---

## How It Works

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Agent A        │     │     Git Repo    │     │  Agent B        │
│  (Creating)     │     │                 │     │  (Receiving)    │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│                 │     │                 │     │                 │
│ xas debug new   │────▶│   pending/      │◀────│ xas receive     │
│ xas debug ...   │     │     *.json      │     │ --prompt        │
│ xas debug done  │     │                 │     │                 │
│                 │     │   git push/pull │     │                 │
│                 │     │                 │     │                 │
│                 │     │   archive/      │◀────│ (processed)     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

1. **Creating agent** builds handoff incrementally with mode-specific commands
2. `xas done` saves JSON to `pending/` and auto-commits
3. Git syncs to shared repository
4. **Receiving agent** runs `xas receive --prompt` to get compiled context
5. Processed handoffs move to `archive/`

---

## Directory Structure

```
your-project/
├── .xas/              # Local state (gitignored)
│   ├── wip.json       # Work-in-progress handoff
│   └── agent.json     # Your agent identity
├── pending/           # Active handoffs (committed, shared)
├── archive/           # Processed handoffs (committed)
└── ... your code ...
```

---

## For Claude Code Users

XAgentSync includes a `CLAUDE.md` file that teaches Claude Code how to use XAgentSync effectively. When Claude loads your project, it automatically understands the handoff protocol.

```bash
# Claude can run these directly
xas receive --prompt     # Check for handoffs at session start
xas debug new "..."      # Create handoffs during work
xas status               # Check what's in progress
```

See [CLAUDE.md](./CLAUDE.md) for the full integration guide.

---

## Philosophy

### Preparation > Transport
The value isn't in moving files — Git already does that. The value is in **structuring context** so the receiving agent understands it instantly.

### Negative Knowledge Matters
Knowing what *didn't* work is as valuable as knowing what did. XAgentSync tracks dead ends, failed approaches, and rejected alternatives so agents don't repeat mistakes.

### Mode-Specific Thinking
A debug handoff needs hypotheses and evidence. A deploy handoff needs rollback plans. A plan handoff needs decision rationale. One size doesn't fit all.

### Agent Empathy
Write handoffs you'd want to receive. Be specific about file paths. Explain *why*, not just *what*. Prioritize ruthlessly.

---

## Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

22 tests covering handoff creation, serialization, CLI workflows, and all three modes.

---

## Contributing

Contributions welcome! This project is MIT licensed.

Areas we'd love help with:
- Additional handoff modes (review? refactor?)
- Better token estimation for handoff size
- IDE integrations (VSCode, JetBrains)
- Web UI for handoff visualization

---

## License

MIT — Use it, fork it, ship it.

---

<p align="center">
  <sub>Built for the age of AI-assisted development.</sub><br>
  <sub>Stop losing context. Start shipping faster.</sub>
</p>

# XAgentSync (XAS) - Claude Code Integration

This file helps Claude Code understand and use XAgentSync for async handoffs between LLM agents.

## What Is XAgentSync?

XAgentSync creates **structured handoffs** for async collaboration between LLM code assistants. When you finish working and another agent will continue later, XAgentSync packages your context efficiently.

**Key insight**: Git tracks code history. XAgentSync tracks agent history - what you tried, what you learned, what the next agent should know.

## When To Use XAgentSync

Use XAgentSync when:
- You're ending a session and another agent will continue the work
- You need to hand off a deployment, debug session, or planning task
- You want to preserve context that would otherwise be lost
- You're working async with other agents on the same codebase

## Quick Start

```bash
# First time setup
xas init
xas whoami --set "claude-opus"  # or your identifier

# Check for incoming handoffs from other agents
xas receive --prompt
```

## The Three Modes

### 1. DEPLOY Mode - For Shipping Code

Use when handing off deployment tasks.

```bash
xas deploy new "Ship the authentication feature"
xas deploy ship "src/auth/*" --description "New OAuth2 implementation"
xas deploy verify "Run: cargo test auth"
xas deploy verify "Check: OAuth callback works in staging"
xas deploy rollback "Revert commit abc123 and redeploy"
xas deploy breaking "Token format changed" "All existing sessions"
xas deploy env-concern "prod" "Rate limits not configured yet"
xas deploy done
```

**What to include:**
- What files/features to ship
- How to verify it works
- How to rollback if it doesn't
- Breaking changes and who's affected
- Environment-specific concerns

### 2. DEBUG Mode - For Troubleshooting

Use when handing off an unresolved bug or issue.

```bash
xas debug new "Login failing for OAuth users after token refresh"
xas debug symptom "500 error on /auth/callback"
xas debug symptom "Only happens after 1 hour (token expiry)"
xas debug hypothesis "Race condition in token refresh" --likelihood high
xas debug hypothesis "Cache returning stale tokens" --likelihood medium
xas debug tried "Added mutex around refresh" --result "Still failing" --outcome nothing
xas debug tried "Increased token TTL" --result "Delayed issue but didn't fix" --outcome helped
xas debug suspect "src/auth/token.rs" "Token refresh logic lives here"
xas debug suspect "src/cache/mod.rs" "Might be serving stale tokens"
xas debug evidence "Error: token_expired at src/auth/token.rs:145" --kind error
xas debug try-next "Check if cache invalidation is async"
xas debug done
```

**What to include:**
- Clear problem statement
- All symptoms observed
- Hypotheses with likelihood ratings
- What you already tried (so they don't repeat it!)
- Suspected files/code areas
- Evidence collected
- What to try next

### 3. PLAN Mode - For Design Work

Use when handing off planning or design tasks.

```bash
xas plan new "Design caching layer for API responses"
xas plan require "Sub-100ms p99 latency" --priority must
xas plan require "Cache invalidation on write" --priority must
xas plan require "Metrics and monitoring" --priority should
xas plan require "Multi-region support" --priority could
xas plan decided "Use Redis" --why "Team has Redis expertise, good Rust client"
xas plan decided "Cache at service layer, not DB" --why "More control over invalidation"
xas plan rejected "Memcached" "Missing persistence, harder invalidation"
xas plan rejected "In-memory only" "Won't scale across instances"
xas plan question "How to handle cache stampedes?" --importance high
xas plan question "Redis cluster vs single instance?" --importance medium
xas plan constraint "Must work with existing auth middleware"
xas plan next-step "Benchmark Redis client options"
xas plan done
```

**What to include:**
- Clear goal statement
- Requirements with MoSCoW priorities (must/should/could/wont)
- Decisions made and WHY
- Options you rejected and WHY (so they don't suggest them again)
- Open questions that need answers
- Constraints that can't be violated
- Suggested next steps

## Receiving Handoffs

When starting work, always check for handoffs:

```bash
# See what's waiting
xas receive

# Get the full context (ready to read)
xas receive --prompt

# Filter by mode
xas receive --mode debug --prompt

# Archive after processing
xas receive --archive
```

## Best Practices for Claude

### Creating Good Handoffs

1. **Be specific about what you tried** - The next agent's biggest waste is repeating failed approaches
2. **Explain WHY, not just WHAT** - Decisions without rationale will be questioned or reversed
3. **Prioritize ruthlessly** - Not everything is important; use priority flags
4. **Include file paths** - `src/auth/token.rs:145` is better than "the auth code"
5. **State your confidence** - "High likelihood" vs "wild guess" helps prioritization

### Reading Handoffs

1. **Read the compiled prompt first** - It's optimized for quick understanding
2. **Check "Already Tried" before suggesting solutions** - Don't repeat failures
3. **Review rejected options** - They were rejected for reasons
4. **Note the hypotheses** - Build on existing theories, don't start fresh
5. **Follow suggested next steps** - Unless you have good reason not to

### Handoff Hygiene

```bash
# Always set your identity
xas whoami --set "claude-opus-session-42"

# Check status regularly
xas status

# Clean up after processing
xas receive --archive
```

## CLI Reference

```
xas init                    Initialize XAgentSync in current directory
xas whoami [--set NAME]     Show/set agent identity
xas status                  Show sync status and pending handoffs
xas receive [--prompt]      List/view incoming handoffs
xas sync [--pull-only]      Sync with git remote

xas deploy new SUMMARY      Start deploy handoff
xas deploy ship ITEM        Add item to ship
xas deploy verify STEP      Add verification step
xas deploy rollback PLAN    Set rollback plan
xas deploy breaking WHAT AFFECTS   Add breaking change
xas deploy env-concern ENV CONCERN Add environment concern
xas deploy done             Finalize and send

xas debug new PROBLEM       Start debug handoff
xas debug symptom TEXT      Add observed symptom
xas debug hypothesis TEXT [--likelihood high|medium|low]
xas debug tried TEXT [--result TEXT] [--outcome fixed|helped|nothing|worse]
xas debug evidence TEXT [--kind log|error|observation]
xas debug suspect PATH REASON
xas debug repro STEPS       Set reproduction steps
xas debug try-next TEXT     Suggest what to try next
xas debug done              Finalize and send

xas plan new GOAL           Start plan handoff
xas plan require TEXT [--priority must|should|could|wont]
xas plan decided TEXT [--why REASON]
xas plan rejected OPTION REASON
xas plan question TEXT [--importance high|medium|low] [--blocking]
xas plan constraint TEXT
xas plan next-step TEXT
xas plan done               Finalize and send
```

## Integration with Git

XAgentSync uses git for transport:
- Handoffs are JSON files in `pending/`
- `xas done` auto-commits to git
- `xas sync` pulls/pushes changes
- Works with any git remote (GitHub, GitLab, etc.)

## Example Workflow

Agent 1 (debugging):
```bash
xas debug new "API returning 500 on large payloads"
xas debug symptom "Only fails when payload > 1MB"
xas debug hypothesis "Request body size limit" --likelihood high
xas debug tried "Increased nginx limit" --result "No change" --outcome nothing
xas debug suspect "src/server/middleware.rs" "Body parsing happens here"
xas debug done
git push
```

Agent 2 (continuing):
```bash
git pull
xas receive --prompt
# Reads full context, sees what was tried, continues from there
```

## Remember

- **Preparation > Transport**: The value is in structuring context well
- **Negative knowledge matters**: What DIDN'T work is as valuable as what did
- **Mode-specific thinking**: Debug needs different context than Deploy
- **Future agent empathy**: Write handoffs you'd want to receive

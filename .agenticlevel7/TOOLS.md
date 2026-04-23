# TOOLS.md

> The agent doesn't work alone. These are the tools, MCP servers, and skills
> recommended for this specific project.

## Recommended MCP Servers

MCP servers extend the agent's capabilities. Install the ones relevant to your workflow.

### GitHub MCP Server
For issue tracking, PR management, and code review:
- Create and manage issues
- Review and merge pull requests
- Track project progress

## Recommended Skills

Skills are reusable workflows. These would be useful for this project:

- `/commit` — Create well-formatted commits
- `/simplify` — Review code for quality and efficiency
- `/pr` — Create pull requests with proper descriptions

## Agent-Specific Tools

**Foundation (Configuration Agent):** Files, Commands, Memory
**Schema (Data Model Agent):** Files, Commands, Prisma CLI, Database client, Memory
**Gatekeeper (Authentication Agent):** Files, Commands, Memory
**Interface (API Layer Agent):** Files, Commands, Memory
**Navigator (Routing & Endpoints Agent):** Files, Commands, Memory
**Lighthouse (SEO & Indexing Agent):** Files, Commands, Web fetch (for sitemap validation), Memory
**Launchpad (Build & Deploy Agent):** Files, Commands, Build system, Docker CLI, Memory
**Operator (Admin & Dashboard Agent):** Files, Commands, Memory
**Agent (File Storage Agent):** Files, Commands, Memory
**Agent (Monitoring & Observability Agent):** Files, Commands, Memory
**Sentinel (Testing Agent):** Files, Commands, Test runner, Coverage reporter, Memory

## Hooks

Consider setting up these hooks in `.claude/settings.json`:

- **Pre-commit:** Run lint and typecheck before every commit
- **Post-edit:** Run relevant tests after file changes
- **Notification:** Alert on build failures
# Claude Code First Actions

- Read .agenticlevel7/project-model.json to understand the full project model
- Read .agenticlevel7/workstreams.json to understand workstream decomposition
- Read .agenticlevel7/agent-manifest.json to understand available agents
- Inspect control point: src/main.rs (Runtime entry point)
- Start with Foundation (Configuration Agent): read its agent file at .agenticlevel7/agents/config-agent.md
- First file to read: Cargo.toml
- Plan before acting - understand dependencies between workstreams before making changes

## Suggested Launch Command
```
claude --print "Read .agenticlevel7/providers/claude-code/startup-brief.md and begin working through the agent queue for this rust-cli project"
```
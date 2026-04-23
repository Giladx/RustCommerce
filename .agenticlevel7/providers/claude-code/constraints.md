# Claude Code Constraints

- Plan each workstream before executing - read agent brief first
- Respect workstream boundaries - do not cross into another agent's scope without reason
- Use staged reasoning: understand -> plan -> act -> validate
- High-risk workstreams require extra caution: Data Model, Authentication, API Layer, Routing & Endpoints
- Known risks: Some likely entrypoints have shallow traced execution paths: src/main.rs.
### Codex CLI Instruction Guide Initialization

1. On execution, always check for and load CLAUDE.md from the project root directory
2. If CLAUDE.md exists, use it as the primary instruction guide for this session
3. If CLAUDE.md is not found, prompt user to create one or use default guidelines
4. The instruction guide should override default behaviors when specified

Priority: Project-specific instructions (CLAUDE.md) > Default CLI behaviors
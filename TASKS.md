<!-- DOCGUIDE HEADER
Version: 1.0
Generated: 2025-08-06
Project Type: Rust Library (FFI Wrapper)
Task ID Scheme: T-### (monotonic, never reuse)
Last Updated: 2025-08-06
Update Command: claude commands/scaffold/tasks.md
-->

# Task Management

## Part 1: Task Management Guide

### Agent Identification
When an agent picks up a task, they should identify themselves using the format: `role-name` (e.g., `architect-vision`, `dev-jarvis`, `test-groot`).

Example interaction:
```
User: "Please work on T-003"
Agent: "What is your role and name?" -> "I'm the architect agent, Vision"
Agent: Updates task assignee to "architect-vision"
```

### Task ID Scheme
- Format: `T-###` (e.g., T-001, T-002)
- IDs are monotonic and never reused
- When a task is dropped, mark as "Dropped" but keep the ID
- Always reference tasks by their ID

### Task States
- **TODO**: Not started
- **DOING**: In progress (only one per agent)
- **DONE**: Completed
- **REVIEW**: Ready for review
- **DROPPED**: No longer needed

### Task Transitions
```
TODO -> DOING -> DONE
TODO -> DOING -> REVIEW -> DONE
TODO -> DROPPED
```

### Adding New Tasks

Copy this template and increment the ID:
```markdown
| T-### | Task title | TODO | - | Links | Notes |
```

### Assignment Rules
1. Ask agent for role and name before assigning
2. Only one DOING task per agent at a time
3. Record agent transitions in assignment history
4. Update assignee field when transferring work

## Part 2: Active Tasks

| ID | Title | Status | Assignee | Links | Notes |
|----|-------|--------|----------|-------|-------|
| T-001 | Clean up build.rs formatting and spacing | TODO | - | - | Refactor mentioned in recent commits |
| T-002 | Add Windows build support and testing | TODO | - | ARCHITECTURE.md | Windows platform listed but not fully implemented |
| T-003 | Create comprehensive FFI documentation | TODO | - | wrapper.h | Document all exposed C++ functions |
| T-004 | Implement Android NDK cross-compilation tests | TODO | - | CROSS_COMPILE.md | Android support added but needs testing |
| T-005 | Add CI/CD pipeline with GitHub Actions | TODO | - | - | Build and test on all supported platforms |

### Task Assignment History

| Date | Task | From | To | Reason |
|------|------|------|----|--------|
| - | - | - | - | Initial task list created |

## Related Documents

- [PRD.md](./PRD.md) - Product requirements (R-### IDs)
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical design and platform support

## Integration Points

### Requirement IDs (R-###)
Link tasks to specific requirements from PRD.md

### Decision IDs (D-###)
Reference architectural decisions from DECISIONS.md

### PR/Issue Links
- GitHub issues: #123
- Pull requests: PR#456
- External refs: URL

## Notes

### Project Context
- FFI wrapper for Google Crashpad
- Two-crate structure: crashpad-sys (FFI) and crashpad (safe wrapper)
- Uses depot_tools and gclient for dependency management
- Cross-platform support for macOS, iOS, Linux, Android, Windows

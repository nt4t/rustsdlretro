# Progress: Web API Implementation

## Session Log

| Date | Phase | Action | Result |
|------|-------|--------|--------|
| 2025-01-XX | Planning | Codebase analysis (main.rs, lib.rs, input.rs, gui.rs) | ✅ Complete understanding of architecture |
| 2025-01-XX | Planning | WebSocket crate evaluation — selected tungstenite (binary frame support) | ✅ Ready for PNG streaming |
| 2025-01-XX | Planning | Created task_plan.md with PNG frame streaming added | ✅ Plan documented |
| 2025-01-XX | Planning | Created findings.md with PNG encoding research | ✅ Findings stored |

## Current Phase: Phase 0 — Research & Dependencies

### Completed
- [x] Analyzed existing codebase structure and patterns
- [x] Identified thread-safety constraints (single-threaded Core, Arc<Mutex> for input)
- [x] Evaluated WebSocket crates — selected tungstenite (binary frame support)
- [x] Designed JSON message protocol for client↔server communication
- [x] Determined input mapping strategy: direct state injection into InputReader
- [x] Researched PNG encoding — `png` crate, throttle to 15-30fps
- [x] API is optional via Cargo feature flag `api`

### In Progress
- [ ] Phase 0: Select exact crate versions and add to Cargo.toml

### Upcoming Phases
1. **Phase 1**: Shared State Architecture (`api.rs`)
2. **Phase 2**: WebSocket Server + PNG Streaming (`api_server.rs`)
3. **Phase 3**: Frontend Integration (`main.rs` modifications)
4. **Phase 5**: Frame Streaming (PNG encoding, throttling)

## Build Configuration

**Feature flag**: `api` (optional, off by default)
```bash
cargo build --release                    # No API — minimal binary
cargo build --release --features api     # With WebSocket + PNG streaming
```

1. **Port configuration**: Should API port be configurable via CLI flag or config file?
   - *Tentative*: `--api-port N` CLI flag is simplest; add to config later if needed

2. **PNG frame rate**: What streaming FPS target? (60 = CPU heavy, 15 = smooth enough)
   - *Tentative*: 30fps default; clients build their own renderers

3. **Concurrent WebSocket clients**: How many simultaneous connections?
   - *Tentative*: Support multiple; frame step is "last client wins" semantics

3. **Should we add a built-in web UI?**
   - *Decision*: No — Phase 5 can provide raw frame streaming for external clients to build their own UI. Keep core focused on API, not presentation.

## Errors Encountered
None yet (planning phase).

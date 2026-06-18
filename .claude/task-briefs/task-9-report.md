## Task 9 Report: Worker Restart/Backoff and Graceful Shutdown

### Status: COMPLETE

### Commits
- `115540a` feat: add worker restart backoff and graceful shutdown

### Changes Made
1. **`WorkerHandle`** -- added `restart_count: u32` and `last_restart: Instant` fields.
2. **`backoff_delay`** -- new free function: base 100ms, doubles each restart, capped at 30s.
3. **`Arbiter::worker_spec`** -- helper that builds a `WorkerSpec` from the current config/database state, eliminating the inline construction that was duplicated in `start_worker` and the restart watcher.
4. **`Arbiter::start_worker`** -- simplified to use `self.worker_spec(account_id.to_string())`.
5. **Restart watcher** -- background `tokio::spawn` in `Arbiter::run` that polls every 1s, calls `child.try_wait()`, removes clean-exited workers, and restarts crashed ones with backoff applied before respawning.
6. **`Arbiter::stop_worker`** -- rewritten with the graceful-shutdown flow:
   - Send `stop` command via the worker's control socket, wait up to 10s for response.
   - If the process hasn't exited after 10s, send SIGTERM, wait 5s.
   - If still alive, log a zombie warning.
7. **`Arbiter::stop_all_workers`** -- rewritten to collect all account IDs, call `stop_worker` for each in parallel via `futures::future::join_all`.
8. **`Cargo.toml`** -- added `futures.workspace = true` for `join_all`.
9. **Test code** -- updated `MockWorkerSpawner` return value to include `restart_count: 0` and `last_restart: Instant::now()`.

### Test Output
```
cargo check -p lilium  -- PASS (4 pre-existing dead_code warnings only)
cargo test  -p lilium  -- PASS (33/33 tests passed, 0 failed)
```

### Concerns / Blockers
- None. The `WorkerSpec` fields and `last_restart` field produce dead-code warnings because they're built from `Config`/`Database` inside the spawned closure rather than accessed as struct fields. These are consistent with the pre-existing dead-code warnings for `Config` and `WorkerSpec` fields (the crate is WIP and the fields will be used when the worker process itself is implemented).

---

## Fix: stop_all_workers silent-drop of WorkerHandle values

### Commit
- `3e933e9` fix: stop_all_workers calls self.stop_worker on empty workers map

### Bug
`stop_all_workers` did `std::mem::take(&mut *w)` to move all `WorkerHandle`s into a local `HashMap`, collected the account keys, then called `self.stop_worker(account_id)` for each. But `stop_worker` does `workers.remove(account_id)` on `self.workers` -- which was now empty. Every call hit the "Worker not running" branch and returned immediately. The actual `WorkerHandle` values (with their child processes) were silently dropped when the local `HashMap` went out of scope, relying solely on `kill_on_drop(true)` -- no control socket shutdown, no `child.wait()`, risking zombie processes.

### Fix
Replaced the `into_keys().iter().map(|a| self.stop_worker(a)).collect()` pattern with an inline loop over the owned `HashMap<String, WorkerHandle>` from `mem::take`. The loop applies the full 3-phase graceful shutdown directly to each handle: control socket stop request -> wait up to 10s -> SIGTERM -> wait 5s -> zombie warn. This eliminates the re-lookup through the empty map while preserving the same shutdown logic that `stop_worker` uses.

### Verification
```
cargo check -p lilium  -- PASS (4 pre-existing dead_code warnings only)
cargo test  -p lilium  -- PASS (33/33 tests passed, 0 failed)
```

### Final commit hash
```
3e933e9682c35ab9d0352fe5e5aa260ba70905f9
```

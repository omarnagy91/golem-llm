### Requirements Docs — *Durable Code Execution Components (****`golem:exec`**\*\*\*\*\*\*\*\*) for JavaScript & Python*

Below is a **fully reorganized, self-contained brief** that gathers every requirement, constraint, and expectation scattered through the original issue. Use it as a single source of truth before you plan or estimate work.

---

## 1. Purpose & Scope

| Item                    | Details                                                                                                                |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| **Goal**                | Provide **sandboxed, resource-limited code execution** that behaves identically across languages.                      |
| **Languages / Engines** | • **JavaScript** — QuickJS (via `rquickjs`)• **Python** — CPython compiled for WASI (`componentize-py` + `wasi-libc`). |
| **Targets**             | Each engine ships as its **own WASI-preview 0.23 component** produced by `cargo component`.                            |
| **Rust Only**           | All glue, orchestration, and WIT bindings must be written in Rust.                                                     |

---

## 2. Deliverables

| Artifact                     | Notes                                                                                 |
| ---------------------------- | ------------------------------------------------------------------------------------- |
| `exec-javascript.wasm`       | QuickJS-backed implementation.                                                        |
| `exec-python.wasm`           | CPython-backed implementation.                                                        |
| **WIT interface**            | Must **fully implement** `golem:exec@1.0.0` (see §6).                                 |
| **Unit & integration tests** | Executable with `cargo test`; WIT-level tests optional but encouraged.                |
| **Documentation**            | Brief README covering build flags, env-vars, known limitations, and how to run tests. |

---

## 3. Functional Requirements

### 3.1 Core execution flows

| Mode              | Functions                                | Key Behaviours                                                         |
| ----------------- | ---------------------------------------- | ---------------------------------------------------------------------- |
| **Stateless**     | `executor.run`, `executor.run-streaming` | One-shot execution, returns `result` or stream of `exec-event`.        |
| **Session-based** | `session.*` resource                     | Persist files between runs, manage working dir, download, list, close. |

### 3.2 Inputs to handle

* **Code files** (`types.file`) with optional per-file `encoding`.
* **Arguments** (`args`) & **environment** (`env`).
* **Resource limits** (`types.limits`).

### 3.3 Outputs

* **Structured outcome** (`types.result` / `types.error`).
* **Streaming chunks** (`exec-event.{stdout,stderr}`).

---

## 4. Non-Functional Requirements

| Concern                  | Details                                                                       |
| ------------------------ | ----------------------------------------------------------------------------- |
| **Isolation**            | Each stateless call or session must be hermetic.                              |
| **Resource Enforcement** | Respect `time-ms`, `memory-bytes`, `max-processes` *if technically feasible*. |
| **Timeout**              | Default 5000 ms via `EXEC_TIMEOUT_MS` env-var.                                |
| **Binary size**          | Favor minimal QuickJS / CPython builds; strip symbols.                        |
| **Cleanup**              | Session resources freed on `drop` or explicit `close`.                        |

---

## 5. Graceful Degradation Rules

1. **Unsupported memory limits** → ignore field, continue, document behaviour.
2. **Language version requested but not selectable** → accept & ignore.
3. **Filesystem persistence unavailable** → return `error.unsupported-language` or other suitable variant for `download/list-files`.
4. Any spec deviation **must be justified in docs & code comments**.

---

## 6. Interface Reference (abridged)

```wit
package golem:exec@1.0.0;

interface types {
  record language { kind: language-kind, version: option<string> }
  enum language-kind { javascript, python }
  enum encoding { utf8, base64, hex }

  record file { name: string, content: list<u8>, encoding: option<encoding> }
  record limits { time-ms: option<u64>, memory-bytes: option<u64>,
                  file-size-bytes: option<u64>, max-processes: option<u32> }

  record stage-result { stdout: string, stderr: string,
                        exit-code: option<s32>, signal: option<string> }
  record result { compile: option<stage-result>, run: stage-result,
                  time-ms: option<u64>, memory-bytes: option<u64> }

  variant error {
    unsupported-language, compilation-failed(stage-result),
    runtime-failed(stage-result), timeout, resource-exceeded, internal(string)
  }

  variant exec-event {
    stdout-chunk(list<u8>), stderr-chunk(list<u8>),
    finished(result), failed(error)
  }
}

interface executor {
  // one-shot
  run: func(...) -> result<result, error>
  run-streaming: func(...) -> stream<exec-event>
}

resource session { /* upload, run, download, list-files, set-working-dir, close */ }
```

---

## 7. Environment Variables

| Variable                | Default | Purpose                                                |
| ----------------------- | ------- | ------------------------------------------------------ |
| `EXEC_TIMEOUT_MS`       | `5000`  | Global wall-clock limit.                               |
| `EXEC_MEMORY_LIMIT_MB`  | *unset* | Soft cap; ignored if unenforceable.                    |
| `EXEC_JS_QUICKJS_PATH`  | *none*  | Override QuickJS binary if you prefer external binary. |
| `EXEC_PYTHON_WASI_PATH` | *none*  | Same for CPython WASI build.                           |

---

## 8. Testing Matrix

| Category              | Cases to Cover                                                        |
| --------------------- | --------------------------------------------------------------------- |
| **Stateless**         | Happy-path, syntax errors, runtime errors, oversized input, timeouts. |
| **Session lifecycle** | create → upload → run → download → list → close (+ double close).     |
| **File encoding**     | UTF-8 round-trip, Base64, Hex.                                        |
| **Limits**            | timeouts, memory pressure, process limit (QuickJS fork bombs).        |
| **Error variants**    | Each `types.error` arm triggered deliberately.                        |
| **Isolation**         | Two concurrent sessions cannot access each other’s FS.                |

---

## 9. Suggested Tech Stack & References

| Component              | Suggested Crates / Tools                                                                |
| ---------------------- | --------------------------------------------------------------------------------------- |
| **QuickJS binding**    | `rquickjs`, `quickjs-wasm-sys`, `anyhow`, `tokio` for async.                            |
| **CPython WASI**       | [`componentize-py`](https://github.com/bytecodealliance/componentize-py) & `wasi-libc`. |
| **WIT & WASI**         | `wit-bindgen` (0.23 preview), `cargo component`.                                        |
| **Limits enforcement** | `tokio::time::timeout`, `wasmtime::StoreLimiter`, `ResourceLimiter`.                    |
| **Testing**            | `assert_cmd`, `tempfile`, `serial_test` for isolation, plus WIT integration harness.    |

---

### 10. Open Questions / Risks

1. **Memory-limit enforcement under WASI 0.23**
   Needs experiments with Wasmtime’s `StoreLimiter` and the component model.

2. **Process count (********`max-processes`****\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*)**
   Not directly enforceable in single-process WASM; may need to treat as “unsupported”.

3. **Persistent FS in WASI sandbox**
   Decide between in-memory virtual FS or host-mapped dir; affects `download/list-file`.

---

####

---

## 11. Lessons & Owner Feedback (June 2025)

* **Streaming Limitation** – Golem’s `stream<a>` type is *not yet fully supported* in the host runtime. If you hit issues, switch to a manual approach that exposes pull‑based streams through `resource input‑stream` / `output‑stream` pairs while **keeping the public WIT signature unchanged** for forward‑compatibility.
* **Domain Justification** – Concerns were raised that code execution is outside the AI domain. Maintainers clarified it **will remain** inside the Golem repo because autonomous agents often need to generate & run code. No change to requirements.

> Integrate these notes into your implementation plan so your PR aligns with the maintainers’ expectations and avoids common rejection reasons.

---

## 12. Implementation Instructions

These directives are **mandatory** for any autonomous agent (or scripted workflow) that attempts this bounty:

1. **Bootstrap a Planning File**

   * Create a Markdown file named **`plan.md`** at the project root.
   * The file must contain an ordered task list (`- [ ] Task …`) covering **every requirement** captured in this document, grouped by major milestones (build pipeline, QuickJS adapter, CPython adapter, resource‑limits, tests, docs, packaging, etc.).

2. **Keep ********`plan.md`******** as the Single Source of Progress**

   * **Before starting any new milestone**, append the detailed sub‑tasks you intend to perform.
   * **After finishing a sub‑task**, immediately mark it as completed (`- [x] …`) and, if needed, add notes (e.g. commit hash, test evidence).

3. **Always Reconcile With This Spec**

   * At the beginning of every agent run, diff the current `plan.md` against this requirements doc; if gaps exist, add new tasks before coding.
   * Do **not** remove tasks unless they were added in error *and* the spec confirms they’re out of scope.

4. **Testing Gate**

   * No task is “done” until corresponding unit / integration tests pass locally (`cargo test` succeeds) **and** the result artifacts (`*.wasm`) build reproducibly with `cargo component build`.

5. **Streaming Fallback**

   * If you encounter runtime issues with `stream<…>` in WIT, automatically create a fallback task: “Implement pull‑based stream resource shim” and mark original stream task as *blocked* until resolved.

6. **Lint & Size Checks**

   * Include a recurring task to run `cargo clippy --all-targets -- -D warnings` and `wasm‑snip / wasm‑opt` size passes; only mark complete when binary size < 10 MB per component (or documented reason).

7. **Documentation Task**

   * The final milestone must produce a **README.md** that shows: build instructions, env vars, known limitations, and how to run the demo.

8. **Error Escalation**

   * If an irreconcilable spec ambiguity is detected, pause progress, add a task “Clarify with maintainers”, and set subsequent items to *blocked*.

Following this checklist—and keeping `plan.md` up‑to‑date—will ensure maintainers can audit progress and dramatically increase acceptance likelihood.

---

## 13. Rust Environment Setup (From Official Golem Docs)

> *These steps are required before any `cargo component` build will succeed. The agent must automate or document them in `plan.md` under the “Build Pipeline” milestone.*

| Step                             | Command / Action                                                                          | Notes                                                                                                                  |                                                    |
| -------------------------------- | ----------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| **1. Install stable Rust**       | macOS (recommended):<br>`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Use the official installation guide for other OSes.                                                                    |                                                    |
| **2. Add WASI target**           | `rustup target add wasm32-wasip1`                                                         | Confirms WebAssembly compilation support. Verify with:<br>\`rustup target list --installed                             | grep wasm32-wasip1`(should print`wasm32-wasip1\`). |
| **3. Install `cargo-component`** | `cargo install --force cargo-component@0.20.0`                                            | **Don’t** use `--locked`; a dependency bug in `wit-component 0.220.0` requires the patch 0.220.1 pulled in by default. |                                                    |
| **4. Verify version**            | `cargo component --version`                                                               | Expected output (example):<br>`cargo-component-component 0.20.0 (2e497ee 2025-01-07)`                                  |                                                    |

**Automation Hint:** add a `Makefile` or `justfile` target (e.g. `make setup`) that runs these steps idempotently; mark the task *done* once the verification step passes on CI.

---

## 14. Rust Component Implementation Patterns (Key Points from Golem Docs)

> **Why this matters:** The `exec-javascript` and `exec-python` artifacts are themselves Golem *components*. Understanding how the Rust ↔ WIT glue is generated and wired is essential before you start coding business logic.

### 14.1 Project Scaffolding

| Step                     | Tooling           | Command                                                                              | Outcome                                          |
| ------------------------ | ----------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------ |
| Create application shell | `golem` CLI       | `golem app new exec-runtimes rust`                                                   | Generates a Cargo workspace & `wit/` dir.        |
| Add a component          | `golem` CLI       | `golem component new rust exec-javascript`<br>`golem component new rust exec-python` | Two separate crates ready for `cargo component`. |
| Alt path                 | `cargo component` | `cargo component new --lib exec-javascript`                                          | Same result if you prefer pure Cargo.            |

### 14.2 Spec‑First Workflow

1. Place the **provided `golem:exec@1.0.0.wit`** into each component’s `wit/` directory (or reference it via a workspace‑level `wit` path).
2. On first `cargo component build`, the tool generates `src/bindings.rs` containing:

   * All WIT types (records, variants, enums, etc.)
   * A trait called `Guest` (for world exports).
   * Additional traits like `GuestSession` for each exported resource.
3. **Never edit `bindings.rs` directly**—it will be regenerated. Implement your logic in `lib.rs` (or modules) by:

   ```rust
   #[allow(warnings)]
   mod bindings; // bring generated code into scope

   struct Component;

   impl bindings::Guest for Component { /* implement executor functions */ }

   // Resource implementation example
   struct Session { /* fields */ }
   impl bindings::GuestSession for Session { /* upload, run, etc. */ }

   bindings::export!(Component with_types_in bindings);
   ```

### 14.3 Resources vs. Globals

* **Executor** functions (`run`, `run‑streaming`) can be top‑level methods on `Guest`.
* **Session** must be exported as a **resource** so multiple sessions can coexist. Map each WIT method (`upload`, `download`, …) to `impl bindings::GuestSession for Session`.
* If you need shared mutable state **not tied to a resource** (e.g., global worker cache), use:

  ```rust
  thread_local! {
      static STATE: RefCell<GlobalState> = RefCell::new(...);
  }
  ```

### 14.4 Per‑Instance State with `RefCell`

WIT resource methods receive `&self`, so interior mutability is required:

```rust
use std::cell::RefCell;
struct Session { files: RefCell<HashMap<String, Vec<u8>>> }
```

### 14.5 Logging & Debugging

* Stdout / stderr are captured automatically.
* For structured logs, add to `Cargo.toml`:

  ```toml
  [dependencies]
  log = { version = "0.4", features = ["kv"] }
  wasi-logger = "0.1"
  ```
* Expose an `init_logger()` function or use `OnceCell` to ensure `wasi_logger::Logger::install()` runs once.

### 14.6 Worker Configuration

* Access env vars with `std::env::var("KEY")`—handy for runtime limits.
* CLI args via `std::env::args()` are available if you start workers explicitly, but env vars are preferred.

### 14.7 Build & Bindings Regeneration

* Any change to `wit/*.wit` requires rebuilding (`cargo component build`) to refresh bindings.
* CI should cache the Cargo registry but **not** `bindings.rs`.

### 14.8 Plan.md Add‑on Tasks

* [ ] Scaffold workspace & components (`golem app new`, etc.).
* [ ] Copy `golem:exec.wit` into each `wit/` dir.
* [ ] Run initial `cargo component build` → verify `bindings.rs` generated.
* [ ] Implement `Guest` for executor functions.
* [ ] Implement `Session` resource with interior mutability.
* [ ] Add global logger initialization.
* [ ] Push compiling skeleton commit before implementing logic.

Incorporate these patterns into your coding workflow to stay aligned with Golem’s best practices and avoid the common “missing export!” errors that have sunk prior PRs.

---

## 15. Building & Packaging Components

> **Key takeaway:** `cargo component build` (not plain `cargo build`) is the canonical way to turn Rust crates into *WebAssembly components* that Golem accepts. This section consolidates the official guidance so you don’t miss critical flags.

### 15.1 One‑Shot Build via Golem CLI

```bash
# From the application root
$ golem app build
```

*Runs `cargo component build` under the hood on every component found in the manifest and leaves the generated `.wasm` in `target/wasm32-wasip1/*/`.*

### 15.2 Manual Build (IDE‑friendly)

If your IDE’s default *Run / Build* invokes `cargo build`, override it or add a task to call:

```bash
$ cargo component build           # dev profile, unoptimised
$ cargo component build --release # release profile, optimised + smaller
```

*Always check for the line `Creating component …example.wasm`. If it’s missing, `cargo-component` silently skipped packaging; double‑check your version (see §13).*

### 15.3 Validating the Output

```bash
$ wasm-tools print target/wasm32-wasip1/release/exec-javascript.wasm --skeleton
```

If the first node printed is **`component`** (not `module`), the build is correct.

### 15.4 CI / Automation Recommendations

| Task                  | Tool / Command                    | When                                                 |
| --------------------- | --------------------------------- | ---------------------------------------------------- |
| Build debug profile   | `cargo component build`           | Every push for fast feedback.                        |
| Build release profile | `cargo component build --release` | Tagged commits / PR ready.                           |
| Verify component      | `wasm-tools print … --skeleton`   | Post‑build check; fail CI if top node ≠ `component`. |
| Size budget           | `du -h …/*.wasm`                  | Warn if > 10 MB (matching §12 lint & size).          |

### 15.5 Plan.md Add‑on Tasks

* [ ] **Build pipeline**: add `dev` & `release` build commands.
* [ ] **CI check**: ensure wasm‑tools skeleton validation step.
* [ ] **Size gate**: assert each `.wasm` ≤ 10 MB or document waiver.

Add these tasks under the “Build Pipeline” milestone so the agent guarantees reproducible artifacts that pass Golem upload checks.

---

## 16. HTTP Client Support (WASI‑HTTP)

> **When you might need this:** The `exec-*` runtimes themselves do *not* require outbound HTTP calls, but sample tests (e.g. running user code that fetches data) or future extensions might. Golem components use the **WASI HTTP** spec, and today the only Rust convenience layer is a forked `reqwest` provided by Golem.

### 16.1 Adding the Dependency

```toml
[dependencies]
# Fork aligned with WASI‑HTTP; keep branch up‑to‑date per docs
reqwest = { git = "https://github.com/golemcloud/reqwest", branch = "update-april-2025", features = ["json"] }

# Optional—needed for JSON helpers
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 16.2 Minimal Example (Blocking API)

```rust
use reqwest::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ExampleRequest { name: String, amount: u32, comments: Vec<String> }

#[derive(Serialize, Deserialize)]
struct ExampleResponse { percentage: f64, message: Option<String> }

fn call_service() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    let req_body = ExampleRequest {
        name: "something".into(),
        amount: 42,
        comments: vec!["Hello".into(), "World".into()],
    };

    let resp: Response = client
        .post("http://example.com:8080/post-example")
        .json(&req_body)
        .header("X-Test", "Golem")
        .basic_auth("user", Some("pwd"))
        .send()?;

    let status = resp.status();
    let body = resp.json::<ExampleResponse>()?;
    println!("Received {:?} {:?}", status, body);
    Ok(())
}
```

*API surface mirrors `reqwest`’s **blocking** interface; async/await is not yet supported over WASI‑HTTP.*

### 16.3 Plan.md Add‑on (Optional)

* [ ] **HTTP support** *(only if needed)*: add Golem `reqwest` fork & `serde` crates.
* [ ] Mock or stub HTTP calls in tests to avoid outbound traffic.
* [ ] Document any external endpoints required for demo scenarios.

If your implementation never performs HTTP requests, you can skip these tasks—but having them listed makes future extensions straightforward.

---

## 17. Optional Golem Runtime APIs (Durability, Retries, Transactions, Promises)

> **Relevance:**  The `exec-*` components are mostly computation sandboxes and *should avoid external side‑effects* beyond stdout/stderr. Therefore you can safely rely on Golem’s **default durability & retry settings** (Persistence = `Smart`, Idempotence = enabled) and skip the extra boilerplate below **unless** you add outbound HTTP calls, interact with key‑value/blob stores, or perform other host operations. These notes exist so that future maintainers know what hooks are available.

### 17.1 golem‑rust SDK

```toml
[dependencies]
golem-rust = "1.3.0"  # Only include if you need durability helpers / promises / worker APIs
```

### 17.2 Durability Controls

| Feature               | Functions                                         | Default   | When to Override                                                                                 |
| --------------------- | ------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------ |
| **Persistence Level** | `use_persistence_level`, `with_persistence_level` | `Smart`   | Turn off persistence (`PersistNothing`) around idempotent compute loops to reduce journal noise. |
| **Idempotence Mode**  | `use_idempotence_mode`, `with_idempotence_mode`   | *enabled* | Disable only if you know an HTTP call is non‑idempotent and duplicates break invariants.         |
| **Atomic Regions**    | `mark_atomic_operation`, `atomically`             | n/a       | Group multiple side‑effects so they either all replay or none do.                                |

### 17.3 Retry Policy (Rarely Needed)

```rust
let _guard = use_retry_policy(RetryPolicy {
    max_attempts: 10,
    min_delay: Duration::from_secs(1),
    max_delay: Duration::from_secs(30),
    multiplier: 2.0,
    max_jitter_factory: None,
});
```

> Stick with cluster‑wide defaults unless a specific remote system requires tighter bounds.

### 17.4 Transactions & Compensation

The `golem-rust` crate offers **fallible** and **infallible** transactions with compensation handlers. *Not required* for the core execution sandbox but useful if you later expand to e.g. uploading files to S3 in a single logical unit.

### 17.5 Promises

Create/await/complete promises when you need to wait for external callbacks. Irrelevant to MVP.

### 17.6 Misc Runtime Helpers

* `generate_idempotency_key()` – stable UUID for deduplication.
* Worker metadata, enumeration, and updates – mostly for operational tooling.

### 17.7 Plan.md Add‑on (Optional)

* [ ] **Durability tuning**: add tasks only if you introduce outbound HTTP or key‑value operations.
* [ ] **Retry tuning**: capture policy override rationale.
* [ ] **Transaction wrappers**: outline compensations if batching side‑effects.
* [ ] **Promise flow**: implement create / await in demo if showcasing async external triggers.

👉 *Skip these tasks by default; check them into `plan.md` only on an as‑needed basis to keep scope tight and deliveries lean.*

---

## 18. Shared WIT Packages (Common Guides)

> **When it matters:** If you decide to move common helper types (e.g., file‑metadata records) into a separate package so both `exec-javascript` and `exec-python` can re‑use them, follow this pattern.

### 18.1 Creating a Shared Package

1. Create `wit/deps/shared/shared.wit` in the **application root** (not inside individual components).
2. Define types/interfaces there, e.g.:

   ```wit
   package exec:shared;

   interface types {
     record file-meta {
       name: string,
       size: u64,
     }
   }
   ```
3. Do **not** edit the auto‑generated root‑level `wit/common.wit` file; it’s a placeholder for tooling.

### 18.2 Importing the Package

In each component’s world file:

```wit
world exec-javascript {
  import exec:shared/types;
  export golem:exec/executor;
}
```

Then inside the interface or other interfaces, `use exec:shared/types.{file-meta}` to reference types.

### 18.3 Build Integration

`golem app build` automatically discovers packages in `wit/deps/*`, so no extra steps are required. Custom code generation helpers (if any) should include `wit-bindgen` usage similar to docs.

### 18.4 Plan.md Add‑on (Optional)

* [ ] **Shared types**: create `exec:shared` WIT package and move common records.
* [ ] Update imports in both components & regenerate bindings.

Skip these tasks if you keep all types in the main `golem:exec` WIT.

---

## 19. Worker Filesystem & Initial File System (IFS)

> **Relevance to `exec-*`:** Sessions manipulate user‑uploaded code & artifacts on disk. Understanding the sandbox FS rules helps you decide where to store temp files and how to expose `download/list-files`.

### 19.1 Sandbox Model

* Each worker gets an isolated root `/`.
* `std::fs::*` in Rust transparently maps to WASI `filesystem` calls.
* Other workers/components **cannot** access this FS.

### 19.2 Persisted Files

* Anything written under `/` persists for the lifetime of the worker (i.e., across invocations in the same session resource). This naturally supports `session.upload` / `session.download` semantics.

### 19.3 Initial File System (IFS)

* You may pre‑seed files via `golem.yaml` profiles:

  ```yaml
  components:
    exec-javascript:
      profiles:
        release:
          files:
            - sourcePath: ./ifs/quickjs/qjs
              targetPath: /bin/qjs
              permissions: read-exec
  ```
* Useful for shipping engine binaries or stdlib assets instead of inflating the WASM binary.

### 19.4 Read‑only vs Read‑write

* `permissions` can be `read-only`, `read-write`, or `read-exec`.
* Attempting to modify a read‑only file triggers a runtime error; surface this as `error.internal`.

### 19.5 Plan.md Tasks

* [ ] **IFS**: Decide whether to embed QuickJS / CPython stdlib via IFS or bake into WASM. Document choice.
* [ ] **FS tests**: unit test that `session.upload` writes the file and subsequent `download` returns identical bytes.

---

## 20. Worker‑to‑Worker RPC (FYI)

You normally won’t need intra‑component RPC for MVP, but if you split the JS and Python runtimes into separate workers that cooperate, declare dependencies in **`golem.yaml`** and let Golem auto‑generate Wasm‑RPC stubs. Refer to Common Guide **Worker to Worker Communication** for constructors and blocking/non‑blocking call patterns.

*Leave tasks out of `plan.md` unless you adopt a multi‑worker architecture.*

---

## 21. Other Golem Features (HTTP Incoming Handlers, LLMs, RDBMS, Forking)

These capabilities are **outside the immediate scope** of a sandbox runtime but can be integrated later:

| Feature                        | Potential Use                                        | Quick Note                                                                    |
| ------------------------------ | ---------------------------------------------------- | ----------------------------------------------------------------------------- |
| **WASI HTTP Incoming Handler** | Expose an HTTP API for code execution (e.g., `/run`) | Requires separate component exporting `wasi:http/incoming-handler`; see docs. |
| **golem‑llm**                  | Let user code access LLMs via tool‑calls             | Add WASM dependency & import `golem:llm/llm@1.0.0`.                           |
| **golem‑rdbms**                | Persist execution records to Postgres/MySQL          | Import `golem:rdbms/postgres@0.0.1`, create `DbConnection`.                   |
| **Fork API**                   | Parallel sub‑workers (e.g., isolate user requests)   | `use golem_rust::fork("child-name")`.                                         |

*No Plan.md tasks added; adopt only if feature creep is approved.*

---

## 22. Golem CLI Essentials (Application Manifest & Build Flow)

> **Why this matters:** The agent must automate *repeatable local builds* (for tests/CI) **and** produce a deployable artifact for maintainers to verify. Knowing which CLI commands drive those actions—and the manifest knobs that influence them—prevents environment drift.

### 22.1 Application Manifest (`golem.yaml`)

| Purpose                 | Key Fields                                                                                | Relevance to `exec-*`                                             |
| ----------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| **Component metadata**  | `components.<name>.template`, `profiles.debug/release.build`                              | Ensure each runtime crate is listed so `golem app build` sees it. |
| **Dependencies**        | `components.<name>.dependencies`                                                          | Only needed if you choose a multi-worker design (see §20).        |
| **Initial File System** | `components.<name>.profiles.*.files`                                                      | Already covered in §19 for seeding QuickJS / stdlib assets.       |
| **Build Commands**      | Usually auto‑generated per template; override with custom `build` if you add extra steps. | Keep defaults; Rust components already use `cargo component`.     |

*Auto‑discovery* – CLI searches upward from CWD for the top‑level `golem.yaml`; include all component‑level manifests via `includes:` if you split directories.

### 22.2 Common CLI Commands

| Task                   | Command                                  | Notes                                                                                                  |
| ---------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Build all components   | `golem app build`                        | Equivalent to our §15 flow.                                                                            |
| Build single component | `golem component build exec-javascript`  | Useful when iterating one runtime.                                                                     |
| Deploy all             | `golem app deploy`                       | Pushes new component versions; you can add `--try-update-workers` if you already spun up test workers. |
| Deploy one             | `golem component deploy exec-javascript` | Faster feedback.                                                                                       |
| Clean                  | `golem app clean`                        | Wipes `golem-temp` and build outputs.                                                                  |

*Profiles* – Pass `--profile debug|release` or `--local / --cloud` depending on target environment. The agent should run builds in `debug` for CI speed and `release` before final submission.

### 22.3 Plan.md Tasks

* [ ] **CLI wrapper**: add a `just build` recipe that runs `golem app build` after ensuring `cargo component build` succeeded locally.
* [ ] **Deploy smoke test** *(optional)*: create `just deploy-local` that runs `golem app deploy --local` inside a Docker‑Compose Golem if available.
* [ ] **Manifest lint**: task to validate `golem.yaml` parses (`golem app` with no args should list components without error).

---

## 23. Worker Lifecycle Commands (For Local QA)

> **Scope:** While maintainers will run their own tests, you can script a *self‑check* to spin up a worker, run a tiny JS/Python snippet, fetch logs, then delete it. This ensures the final `.wasm` behaves end‑to‑end under a CLI workflow.

### 23.1 Cheat‑Sheet

| Action           | Command                                                                      | Example                         |
| ---------------- | ---------------------------------------------------------------------------- | ------------------------------- |
| Create worker    | `golem worker new exec-javascript/my-js-worker`                              | Use `--env` or args if needed.  |
| Invoke blocking  | `golem worker invoke my-js-worker 'golem:exec/executor.{run}(…json‑wave…) '` | Use WAVE encoded args per docs. |
| Stream logs live | `golem worker connect my-js-worker`                                          | Good for debugging crashes.     |
| Delete worker    | `golem worker delete my-js-worker`                                           | Clean state between runs.       |

### 23.2 Plan.md Add‑on (Optional)

* [ ] **Smoke test script**: Bash/Justfile that builds → deploys → spins up one JS & one Python worker, executes `print("hello")`, asserts stdout, then deletes workers.

Including these steps is **optional** but can catch packaging errors (e.g., missing IFS binary) before submission.

---

## 24. API‑Definition Bindings (Exposing Runtimes via HTTP)

> **Why this matters:**  The `exec-*` components could be wrapped by an **API definition** so external clients can hit an endpoint like `POST /exec` and get code results. Understanding binding types lets you wire that up—or decide *not* to for scope control.

### 24.1 Binding Types Cheat‑Sheet

| Binding            | Primary Use                                                                     | Quick Notes                                                                                   |
| ------------------ | ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| **default**        | Invoke a worker function, craft custom response via Rib script                  | Most flexible; you choose worker (`workerName` Rib), idempotency key, and response formatter. |
| **cors‑preflight** | Handle `OPTIONS` with CORS headers                                              | Add alongside your main route to enable browser calls.                                        |
| **file‑server**    | Expose worker filesystem files (`/static/**`)                                   | Handy if you store compiled binaries or user files inside session FS and need HTTP downloads. |
| **http‑handler**   | Forward full HTTP request to a worker that exports `wasi:http/incoming-handler` | Overkill for MVP, but could wrap executor as generic HTTP microservice.                       |

### 24.2 Example — Minimal `/run` Endpoint (Optional)

```yaml
routes:
  - method: Post
    path: /run
    binding:
      type: default
      componentName: exec-javascript   # could be exec-python or gateway component
      workerName: |-                   # Ephemeral per call
        "-"                            
      response: |
        let lang = request.query.lang;
        let code = request.body;
        let result = golem:exec/executor.{run}(lang, [], null, [], [], null);
        { status: 200u64, body: result }
```

### 24.3 Rib Scripts Quick Reference

* `request.path.*` – path captures (`{var}` or `{+var}`).
* `request.body` – raw body (string/bytes); parse as JSON if needed.
* `instance(name)` – returns worker handle when calling a durable component.
* `{{…}}` double‑brace syntax – evaluate Rib expression inside string literal.

### 24.4 Plan.md Add‑on (Optional)

* [ ] **HTTP wrapper component** *(stretch goal)*: build thin proxy exporting `wasi:http/incoming-handler` that forwards to executor.
* [ ] **API definition YAML**: draft `api.yaml` with default binding for `/run` (see example).
* [ ] **CORS route**: add `OPTIONS /run` with `cors-preflight` if browser usage anticipated.

> **Scope guidance:** For bounty acceptance you are **not required** to ship an HTTP API—CLI tests plus WIT integration are enough. Only add these tasks if maintainers request an external endpoint.

---

## 25. Application Manifest Deep Dive (Advanced Controls)

> **Context:** §22 gave the quick‑start. The manifest is powerful; these extra knobs can unblock tricky situations—like multi‑profile builds, shared WIT deps, or static RPC linkage.

### 25.1 Document Layout & Discovery

| Concept            | Key Points                                                                                                          | Why it matters                                                                     |
| ------------------ | ------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| **Auto‑discovery** | CLI walks up dirs looking for a *top‑level* `golem.yaml`, then follows `includes:` globs (default `**/golem.yaml`). | Keep your component manifests inside `components-*/*/golem.yaml` so they’re found. |
| **Explicit path**  | `--app path/to/manifest.yaml` disables discovery/includes.                                                          | Rare—only if you script partial builds.                                            |
| **JSON‑Schema**    | Add `# $schema: https://schema.golem.cloud/app/golem/1.1.1/golem.schema.json` comment for IDE hints.                | Helps validate custom fields.                                                      |

### 25.2 Global Tweaks

| Field     | Purpose                                     | Typical Use                                                                                          |
| --------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `tempDir` | Overrides default `golem-temp` scratch dir. | Point to `target/golem-temp` to keep workspace tidy.                                                 |
| `witDeps` | Extra search paths for shared WIT packages. | If you have `wit/deps/shared`, list it here so every component resolves imports without duplication. |

### 25.3 Component‑Level Power Tools

| Feature                   | YAML Path                                                                                                          | Tips                                                                                                    |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| **Profiles**              | `components.<name>.profiles.{debug,release}`                                                                       | Define per‑profile `build:` arrays or different `files:` IFS entries. Remember to set `defaultProfile`. |
| **Templates**             | `templates.<name>` then `components.<name>.template: my-template`                                                  | DRY up repetitive build lines for JS & Python crates.                                                   |
| **Build steps**           | `build: [ { command: …, sources: […], targets: […] } ]`                                                            | Up‑to‑date checks skip commands when targets newer than sources—useful for codegen.                     |
| **Custom commands**       | `customCommands.<cmd>`                                                                                             | E.g., `npm-install` if you embed JS libs for tests. Run via `golem app npm-install`.                    |
| **Clean targets**         | `clean: [path/glob,…]`                                                                                             | Add `target/wasm32-*/*.wasm` to wipe artefacts with `golem app clean`.                                  |
| **Static vs Dynamic RPC** | Under `dependencies`: `type: wasm-rpc` (dynamic, default) or `static-wasm-rpc` (local link, needs Rust toolchain). | Stick to dynamic unless you *must* debug locally without server‑side stubs.                             |

### 25.4 Initial File System Re‑cap

IFS entries live under `components.<name>.files:`; identical syntax repeats inside each profile. Permissions: `read-only`, `read-write`, `read-exec`.

### 25.5 Plan.md Add‑on Tasks

* [ ] **Manifest schema comment**: add `$schema` line to every YAML for IDE validation.
* [ ] **witDeps path**: capture shared WIT path in root manifest.
* [ ] **Custom tempDir**: route to `target/golem-temp` for CI cache hygiene.
* [ ] **Debug vs Release profiles**: ensure `defaultProfile: debug` and release overrides (optimised build flags).
* [ ] **Clean command**: list artefacts so `golem app clean` leaves repo pristine.
* [ ] **Static RPC test** *(optional)*: create `static-wasm-rpc` dep sample to verify linker step; mark blocked unless needed.

> **Golden Rule:** Keep manifest DRY—prefer **templates** and **profiles** over copy‑pasta. Fewer lines → fewer merge conflicts.

---

## 26. Function‑Name Syntax & Resource Invocation

> **Why it matters:**  Unit tests, CLI smoke scripts, and any future HTTP bindings must reference exported functions using their *fully‑qualified* names. Mis‑spelling the path is a common cause of 404‑style runtime errors.

### 26.1 Basics — Functions & Interfaces

*Format:*
`<package>/<interface>.{<function>}`   or  `\<package>.{<top‑level‑func>}`
*Package* may include `@version` but that part is optional when invoking.

**Example** (from template):

```wit
package golem:component;

interface api {
  add-item:     func(...)
  remove-item:  func(...)
}

world my-world {
  export api;
  export dump: func() -> result<string, string>;
}
```

| Exported           | Fully‑Qualified Name                |
| ------------------ | ----------------------------------- |
| `add-item`         | `golem:component/api.{add-item}`    |
| `remove-item`      | `golem:component/api.{remove-item}` |
| `dump` (top‑level) | `golem:component.{dump}`            |

### 26.2 Resources (Constructors, Methods, Statics)

Given:

```wit
interface api {
  resource counter {
    constructor(name: string);
    inc-by:     func(value: u64);
    get-value:  func() -> u64;
    merge-counters: static func(counter, counter, name: string) -> counter;
  }
}
```

| Action          | Name                                                    |
| --------------- | ------------------------------------------------------- |
| **Constructor** | `golem:component/api.{counter.new}`                     |
| **Method**      | `golem:component/api.{counter.inc-by}`                  |
| **Static**      | `golem:component/api.{counter.merge-counters}`          |
| **Drop**        | `golem:component/api.{counter.drop}` *(auto‑generated)* |

### 26.3 Implicit Resource Creation (Shorthand)

You may inline constructor arguments to auto‑select or create an instance:

```text
 g﻿olem:component/api.{counter("my-counter").inc-by}
```

* Value encoding follows **WebAssembly Value Encoding** rules (strings quoted, numbers plain, etc.).

### 26.4 Plan.md Add‑on Tasks

* [ ] **Name validation**: add a test helper that attempts to invoke each exported symbol by its fully‑qualified name to catch typos early.
* [ ] **Inline‑param demo** *(optional)*: include one test that uses the `counter("demo")` shorthand to verify implicit creation works under Golem runtime.

Keeping these naming rules at hand prevents the classic “function not found” errors during integration and review.

---

## 27. Installing WebAssembly Tooling (wit‑bindgen & wasm‑tools)

> **Mandatory versions:** Golem currently pins `wit-bindgen 0.37.0` and `wasm-tools 1.223.0`. Using mismatched versions can cause cryptic runtime errors (e.g., misaligned pointer deref). Automate these installations in your setup script.

### 27.1 wit‑bindgen 0.37.0

```bash
# Download (choose the asset matching your OS/arch)
$ curl -L -O https://github.com/bytecodealliance/wit-bindgen/releases/download/v0.37.0/wit-bindgen-0.37.0-x86_64-linux.tar.gz

# Extract
$ tar xvf wit-bindgen-0.37.0-x86_64-linux.tar.gz

# Install system‑wide (or add to PATH)
$ sudo install -D -t /usr/local/bin wit-bindgen-0.37.0-x86_64-linux/wit-bindgen
```

### 27.2 wasm‑tools 1.223.0

```bash
$ curl -L -O https://github.com/bytecodealliance/wasm-tools/releases/download/1.223.0/wasm-tools-1.223.0-x86_64-linux.tar.gz
$ tar xvf wasm-tools-1.223.0-x86_64-linux.tar.gz
$ sudo install -D -t /usr/local/bin wasm-tools-1.223.0-x86_64-linux/wasm-tools
```

*Installation paths can be local (e.g., `$HOME/.local/bin`) as long as they’re on `PATH` during builds & CI.*

### 27.3 Plan.md Add‑on Tasks (Build Pipeline Milestone)

* [ ] **Toolchain check**: add a step in `make setup` that verifies `wit-bindgen --version` = 0.37.0 and `wasm-tools --version` = 1.223.0; install locally if missing.
* [ ] **CI cache**: cache downloaded tarballs to speed up re‑runs.

These steps must pass before any `cargo component build` task is marked *done*.

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

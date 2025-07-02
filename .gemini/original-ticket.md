Implement Durable Code Execution golem:exec in Rust for JavaScript (QuickJS) and Python (CPython) #33
Open
Open
Implement Durable Code Execution golem:exec in Rust for JavaScript (QuickJS) and Python (CPython)
#33
@jdegoes
Description
jdegoes
opened 2 weeks ago
Contributor
This ticket involves implementing the golem:exec interface for two target runtimes—JavaScript and Python—using the QuickJS and CPython engines, respectively (both of which must be compiled to WASM/WASI with the aid of wasi-libc, you can see componentize-py for CPython and look at the Rust QuickJS crate which can already target WASM/WASI). The golem:exec interface provides a unified abstraction for sandboxed, resource-limited code execution with both stateless and session-based modes. It is designed to provide consistent behavior across runtimes while allowing graceful degradation when capabilities vary.

This work must be performed in Rust, targeting WebAssembly components that conform to the WASI 0.23 preview and Golem component development conventions. Both implementations should expose the full golem:exec interface and handle structured inputs, outputs, and errors as defined in the WIT specification.

Target Runtimes
JavaScript (QuickJS)

Embed the QuickJS engine into Rust using available crates (e.g., rquickjs)
Python (CPython WASI)

Use the CPython WASI target; integrate via wasi-libc
Deliverables
Each runtime must be implemented as a standalone WebAssembly component:

exec-javascript.wasm
exec-python.wasm
Each component must:

Fully implement the golem:exec interface
Compile to a WASI 0.23 component via cargo component
Parse input parameters, including code, arguments, environment, and resource limits
Enforce execution timeout and return structured results with stdout, stderr, exit codes
Handle unsupported features by returning error.unsupported-language or appropriate variants
Clean up session resources on drop or explicit close
Testing Requirements
Each implementation must include thorough test coverage for:

Stateless execution via executor.run
Full session lifecycle: create, upload, run, download, list-files, close
File encoding/decoding fidelity (especially for base64 and UTF-8 content)
Language/version handling (version may be accepted but ignored if not applicable)
Runtime limits enforcement: timeout, memory, process count (where supported)
Proper error signaling for invalid code, bad input, unsupported operations
Isolation and cleanup between session executions
Tests should be runnable via cargo test and optionally exposed as WIT-driven integration tests.

Configuration
Runtime behavior should be configured via environment variables when applicable:

EXEC_TIMEOUT_MS (default: 5000)
EXEC_MEMORY_LIMIT_MB
EXEC_JS_QUICKJS_PATH (optional if using embedded)
EXEC_PYTHON_WASI_PATH (optional if using external interpreter)
Graceful Degradation Strategy
The interface is designed to support graceful fallback:

If memory limits cannot be enforced, document and ignore the constraint
If language version selection is unsupported, accept the field but proceed with default
If filesystem persistence is unsupported, return an appropriate error on download or list-files
Deviations from spec are allowed if rational is provided
package golem:exec@1.0.0;

interface types {
  /// Supported language types and optional version
  record language {
    kind: language-kind,
    version: option<string>,
  }

  enum language-kind {
    javascript, python, 
  }

  /// Supported encodings for file contents
  enum encoding {
    utf8,
    base64,
    hex,
  }

  /// Code file to execute
  record file {
    name: string,
    content: list<u8>,
    encoding: option<encoding>, // defaults to utf8
  }

  /// Resource limits and execution constraints
  record limits {
    time-ms: option<u64>,
    memory-bytes: option<u64>,
    file-size-bytes: option<u64>,
    max-processes: option<u32>,
  }

  /// Execution outcome per stage
  record stage-result {
    stdout: string,
    stderr: string,
    exit-code: option<s32>,
    signal: option<string>,
  }

  /// Complete execution result
  record result {
    compile: option<stage-result>,
    run: stage-result,
    time-ms: option<u64>,
    memory-bytes: option<u64>,
  }

  /// Execution error types
  variant error {
    unsupported-language,
    compilation-failed(stage-result),
    runtime-failed(stage-result),
    timeout,
    resource-exceeded,
    internal(string),
  }

  /// Streamed event output during execution
  variant exec-event {
    stdout-chunk(list<u8>),
    stderr-chunk(list<u8>),
    finished(result),
    failed(error),
  }
}

interface executor {
  use types.{language, file, limits, result, error, exec-event};

  /// Blocking, non-streaming execution
  run: func(
    lang: language,
    files: list<file>,
    stdin: option<string>,
    args: list<string>,
    env: list<tuple<string, string>>,
    constraints: option<limits>
  ) -> result<result, error>;

  /// Streaming execution with bidirectional I/O
  run-streaming: func(
    lang: language,
    files: list<file>,
    stdin: option<stream<list<u8>>>,
    args: list<string>,
    env: list<tuple<string, string>>,
    constraints: option<limits>
  ) -> stream<exec-event>;
}

resource session {
  use types.{language, file, limits, result, error, exec-event};

  constructor: func(lang: language) -> session;

  upload: func(self: borrow<session>, file: file) -> result<_, error>;

  /// Blocking execution
  run: func(
    self: borrow<session>,
    entrypoint: string,
    args: list<string>,
    stdin: option<string>,
    env: list<tuple<string, string>>,
    constraints: option<limits>
  ) -> result<result, error>;

  /// Streaming execution
  run-streaming: func(
    self: borrow<session>,
    entrypoint: string,
    args: list<string>,
    stdin: option<stream<list<u8>>>,
    env: list<tuple<string, string>>,
    constraints: option<limits>
  ) -> stream<exec-event>;

  download: func(self: borrow<session>, path: string) -> result<list<u8>, error>;

  list-files: func(self: borrow<session>, dir: string) -> result<list<string>, error>;

  set-working-dir: func(self: borrow<session>, path: string) -> result<_, error>;

  close: func(self: borrow<session>);
}
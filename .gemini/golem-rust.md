Rust Language Guide
Defining Components
Defining Golem Components in Rust
Creating a project
Golem's command line interface provides a set of predefined, Golem-specific templates to choose from as a starting point.

To get started from scratch, first create a new application using the Rust template:

golem app new my-app rust
cd my-app

An application can consist of multiple components. Add a new component by choosing from one of the available templates. To see the list of available templates, run:

golem component new

Then create a new component using the chosen template:

golem component new rust my-component

Specification-first approach
Golem and the Rust toolchain currently requires defining the component's interface using the WebAssembly Interface Type (WIT) format. See the official documentation of this format for reference.

Each new project generated with golem (or cargo component new --lib) contains a wit directory with at least one .wit file defining a world. This world can contain exports (exported functions and interfaces) and these exports will be the compiled Golem component's public API.

The first time a component is compiled (see the Building Components page for details), a bindings.rs file gets generated in the src directory. This module contains the Rust definitions of all the data types and interfaces defined in the WIT file(s).

To implement the specification written in WIT, the Rust code must implement some of these generated traits and export them using a macro defined in the generated bindings module.

Exporting top-level functions
WIT allows exporting one or more top-level functions in the world section, for example:

package golem:demo;
 
world example {
    export hello-world: func() -> string;
}
To implement this function in Rust, the following steps must be taken:

make sure the generated bindings module is imported
define an empty struct representing our component
implement the generated Guest trait for this struct
call the export! macro
Let's see in code:

// make sure the generated `bindings` module is imported
#[allow(warnings)]
mod bindings;
 
// define an empty struct representing our component
struct Component;
 
// implement the generated `Guest` trait for this struct
impl bindings::Guest for Component {
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}
 
// call the `export!` macro
bindings::export!(Component with_types_in bindings);
Note that in WIT, identifiers are using the kebab-case naming convention, while Rust uses the snake_case convention. The generated bindings map between the two automatically.

Exporting interfaces
WIT supports defining and exporting whole interfaces, coupling together multiple functions and possibly custom data types.

Take the following example:

package golem:demo;
 
interface api {
  add: func(value: u64);
  get: func() -> u64;
}
 
world example {
  export api;
}
This is equivalent to having the two exported functions directly exported from the world section, so the implementation is Rust is once again requires to implement the Guest trait from the generated bindings module:

#[allow(warnings)]
mod bindings;
 
struct Component;
 
impl bindings::exports::golem::demo::api::Guest for Component {
    fn add(value: u64) {
        todo!();
    }
 
    fn get() -> u64 {
        todo!();
    }
}
 
bindings::export!(Component with_types_in bindings);
See the Managing state section below to learn the recommended way of managing state in Golem components, which is required to implement these two functions.

Exporting resources
The WIT format supports defining and exporting resources - entities defined by their constructor function and the available methods on them.

Golem supports exporting these resources as part of the worker's API.

The following example modifies the previously seen counter example to define it as a resource, getting the counter's name as a constructor parameter:

package golem:demo;
 
interface api {
  resource counter {
    constructor(name: string);
    add: func(value: u64);
    get: func() -> u64;
  }
}
 
world example {
  export api;
}
Resources can have multiple instances within a worker. Their constructor returns a handle which is then used to call the methods on the resource. Learn more about how resources can be implicitly created and invoked through Golem's APIs in the Invocations page.

To implement the above defined WIT resource in Rust a few new steps must be taken:

define a struct representing the resource - it can contain data!
implement the trait generated as the resource's interface for this struct
specify this type in the Guest trait's implementation
Let's see in code:

#[allow(warnings)]
mod bindings;
 
use std::cell::RefCell;
 
// define a struct representing the resource
struct Counter {
    name: String,
    value: RefCell<u64>,
}
 
// implement the trait generated as the resource's interface for this struct
impl bindings::exports::golem::demo::api::GuestCounter for Counter {
    fn new(name: String) -> Self {
        Self {
            name,
            value: RefCell::new(0),
        }
    }
 
    fn add(&self, value: u64) {
        *self.value.borrow_mut() += value;
    }
 
    fn get(&self) -> u64 {
        *self.value.borrow()
    }
}
 
struct Component;
 
impl bindings::exports::golem::demo::api::Guest for Component {
    type Counter = crate::Counter;
}
 
bindings::export!(Component with_types_in bindings);
Note that the generated trait for the resource is passing non-mutable self references (&self) to the methods, so the resource's internal state must be wrapped in a RefCell to allow mutation.

Data types defined in WIT
The WIT specifications contains some primitive and higher level data types and also allows defining custom data types which can be used as function parameters and return values on the exported functions, interfaces and resources.

The following table shows an example of each WIT data type and its corresponding Rust type:

WIT type	Rust type
bool	bool
s8, s16, s32, s64	i8, i16, i32, i64
u8, u16, u32, u64	u8, u16, u32, u64
f32, f64	f32, f64
char	char
string	String
list<string>	Vec<String>
option<u64>	Option<u64>
result<s32, string>	Result<i32, String>
result<_, string>	Result<(), String>
result	Result<(), ()>
tuple<u64, string, char>	(u64, String, char)
record user { id: u64, name: string }	struct User { id: u64, name: String }
variant color { red, green, blue, rgb(u32) }	enum Color { Red, Green, Blue, Rgb(u32) }
enum color { red, green, blue }	enum Color { Red, Green, Blue }
flags access { read, write, lst }	bitflags! { pub struct Access: u8 { const READ = 1 << 0; const WRITE = 1 << 1; const LST = 1 << 2; }}
Worker configuration
It is often required to pass configuration values to workers when they are started.

In general Golem supports three different ways of doing this:

Defining a list of string arguments passed to the worker, available as command line arguments
Defining a list of key-value pairs passed to the worker, available as environment variables.
Using resource constructors to pass configuration values to the worker.
Command line arguments
The command line arguments associated with the Golem worker can be accessed in Rust using the standard env::args() function:

for arg in std::env::args() {
    println!("{}", arg);
}
Command line arguments can only be specified when a worker is explicitly created and they are are empty by default, including in cases when the worker was implicitly created by an invocation.

Environment variables
Environment variables can be accessed in Rust using the standard env::var() function:

let value = std::env::var("KEY").expect("KEY was not specified");
Environment variables can be specified when a worker is explicitly created, but there are some environment variables that are always set by Golem:

GOLEM_WORKER_NAME - the name of the worker
GOLEM_COMPONENT_ID - the ID of the worker's component
GOLEM_COMPONENT_VERSION - the version of the component used for this worker
In addition to these, when using Worker to Worker communication, workers created by remote calls inherit the environment variables of the caller.

This feature makes environment variables a good fit for passing configuration such as hostnames, ports, or access tokens to trees of workers.

Resource constructors
As explained earlier, Golem workers can export resources and these resources can have constructor parameters.

Although resources can be used in many ways, one pattern for Golem is only create a single instance of the exported resource in each worker, and use it to pass configuration values to the worker. This is supported by Golem's worker invocation syntax directly, allowing to implicitly create workers and the corresponding resource by a single invocation as described on the Invocations page.

Managing state
Golem workers are stateful. There are two major techniques to store and manipulate state in a Golem worker implemented in Rust:

Using a global thread_local! variable with RefCell
Using resources and RefCell
Note that wrapping the state in RefCell is necessary in both cases to allow mutation.

Using a global thread_local! variable with RefCell
When exporting top-level functions or functions defined in WIT interfaces, the worker state is global. In Rust it is not possible to have mutable global state in safe code so the recommended workaround is to use the thread_local! macro.

Note that Golem workers are always single threaded - the thread_local! macro is used here is just a convenient way to define global state without requiring use of any additional crates or unsafe code.

The following example implements the previously defined counter worker using a thread_local! variable:

#[allow(warnings)]
mod bindings;
 
struct State {
    total: u64,
}
 
thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        total: 0,
    });
}
 
struct Component;
 
impl bindings::exports::golem::demo::api::Guest for Component {
    fn add(value: u64) {
        STATE.with_borrow_mut(|state| state.total += value);
    }
 
    fn get() -> u64 {
        STATE.with_borrow(|state| state.total)
    }
}
 
bindings::export!(Component with_types_in bindings);
Using resources and RefCell
When exporting a WIT resource, it is possible to have a per-instance RefCell holding the resource's state, as it was shown above in the Exporting resources section.

Logging
Anything written to the standard output or standard error streams by a Golem worker is captured and can be observed using the worker connect API or the golem worker connect command.

The log crate https://crates.io/crates/log can be used for advanced logging by using the wasi-logger implementation https://crates.io/crates/wasi-logger.

This crate requires a one-time initialization step to set up the logger. The easiest way to do this is to expose a dedicated init function from the worker that can be called externally to initialize the worker. If this is not acceptable, the initialization can be done in a OnceCell protected static field, and each exported function must access this field to ensure the logger is initialized.

The following example demonstrates how to use the wasi-logger crate to log messages:

Add the wasi-logger and log crates to the Cargo.toml file:

log = { version = "0.4.22", features = ["kv"] } # the `kv` feature is optional
wasi-logger = { version = "0.1.2", features = ["kv"] }
Then expose an initialization function in the worker:

impl Guest for Component {
    fn init() {
        wasi_logger::Logger::install().expect("failed to install wasi_logger::Logger");
        log::set_max_level(log::LevelFilter::Trace);
    }
    // ...
}
After calling init, all calls to log::info!, etc. will be properly captured by Golem and available through the worker connect API.

Rust Language Guide
Building Components
Building Golem Components in Rust
Building Golem components having an application manifest is straightforward, just use the golem command line interface:

golem app build

If the project was created using golem new as recommended, the golem app build command will always work as expected.

The result of the golem app build command is a WebAssembly component file ready to be uploaded to Golem. It does not have to be specified explicitly, as the golem tool will automatically find the correct file when doing for example:

golem component add

IDE support
Any IDE supporting Rust can be used, however for creating the result WASM file, the cargo component build command must be used instead of the usual cargo build command that the IDE might use under the hood.

When using rust-analyzer, read the following section of the cargo-component documentation about how to configure it properly: https://github.com/bytecodealliance/cargo-component#using-rust-analyzer

Under the hood
Under the hood the golem tool performs a single call of the cargo component build command:

$ cargo component build
Generating bindings for example (src/bindings.rs)
 Compiling example v0.1.0 (/Users/golem/tmp/doc-temp/example)
 Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.33s
 Creating component target/wasm32-wasip1/debug/example.wasm
The target/wasm32-wasip1/debug/example.wasm file is a WebAssembly component ready to be uploaded to Golem.

It is recommended to compile a release build of the component before deploying it to Golem as it is more optimized and smaller in size:

$ cargo component build --release
Generating bindings for example (src/bindings.rs)
 Compiling example v0.1.0 (/Users/golem/tmp/doc-temp/example)
 Finished `release` profile [optimized] target(s) in 0.33s
 Creating component target/wasm32-wasip1/release/example.wasm
Make sure the Creating component ... line is printed. Some previous versions of cargo-component failed silently in some cases, skipping the last part of packaging the built WebAssembly module into a component.

It is possible to verify that the result .wasm is a valid WebAssembly component by using the wasm-tools CLI tool and running:

$ wasm-tools print target/wasm32-wasip1/release/example.wasm --skeleton
The top-level node must be component and not module.


CLI
Application Manifest
Golem Application Manifest
The Golem Application Manifest document format is used by golem, usually stored in files named golem.yaml, which can help working with components. Currently, the Application Manifest covers:

defining components and their metadata, including:
component type
location of user defined and generated WIT folders
location of built and composed WASM binaries
build commands
Initial File System
defining component dependencies for using Worker to Worker communication
building components
deploying components
The Application Manifest uses YAML format, see the reference for more information on the schema and for the field reference.

Application Manifest Quickstart
Application manifest documents can be explicitly passed as golem arguments, but the recommended way to use them is with auto discovery mode: when golem is called with an application manifest compatible command it keeps searching for golem.yaml documents in current working directory and parent directories. Once it found the top level golem.yaml document, more documents are searched using the includes field.

Once all manifest documents are found, the paths in them are resolved based on their directory, and then the documents are merged. For the field specific merge rules see the field reference.

Using composable templates
Golem projects can be created with golem app new command. This creates a new application that may consist of multiple components. To add a new component to an existing application, use golem component new. E.g.: let's add a new c and ts component in a new and empty directory:

golem app new app:component-a c
cd app:component-a
golem component new ts app:component-b
When using the app new command, it will create:

common directory for the given language (common-cpp and common-ts):
this directory contains the languages specific Application Manifest Template, which defines how to build the components
can be used for shared subprojects
might contain other shared configurations
directory for components for the given language (components-cpp and components-ts)
directory called wit/deps for common WIT dependencies, and populates it with WASI and Golem packages
depending on the language it might add common-adapters.
Now that we added our components, let's use the app command list our project metadata:

$ golem app
 
Build, deploy application
 
Usage: golem app [OPTIONS] <COMMAND>
 
Commands:
  new     Create new application
  build   Build all or selected components in the application
  deploy  Deploy all or selected components in the application, includes building
  clean   Clean all components in the application or by selection
  help    Print this message or the help of the given subcommand(s)
 
Options:
  -f, --format <FORMAT>
          Output format, defaults to text, unless specified by the selected profile
  -p, --profile <PROFILE>
          Select Golem profile by name
  -l, --local
          Select builtin "local" profile, to use services provided by the "golem server" command
  -c, --cloud
          Select builtin "cloud" profile to use Golem Cloud
  -a, --app-manifest-path <APP_MANIFEST_PATH>
          Custom path to the root application manifest (golem.yaml)
  -A, --disable-app-manifest-discovery
          Disable automatic searching for application manifests
  -b, --build-profile <BUILD_PROFILE>
          Select build profile
      --config-dir <CONFIG_DIR>
          Custom path to the config directory (defaults to $HOME/.golem)
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
  -h, --help
          Print help
 
Components:
  app:component-a
    Selected:     yes
    Source:       /Users/<...>/app-demo/components-cpp/app-component-a/golem.yaml
    Template:     cpp
    Profiles:     debug, release
  app:component-b
    Selected:     yes
    Source:       /Users/<...>/app-demo/components-ts/app-component-b/golem.yaml
    Template:     ts
 
Custom commands:
  npm-install
Because the ts components use npm, we have to use npm install before building the components. We can also see that this has a wrapper custom command in the manifest called npm-install. Let's use that, then build our components:

$ golem app npm-install
<..>
$ golem app build
Collecting sources
  Found sources: /Users/<...>/app-demo/common-cpp/golem.yaml, /Users/<...>/app-demo/common-ts/golem.yaml, /Users/<...>/app-demo/components-cpp/app-component-a/golem.yaml, /Users/<...>/app-demo/components-ts/app-component-b/golem.yaml, /Users/<...>/app-demo/golem.yaml
Collecting components
  Found components: app:component-a, app:component-b
Resolving application wit directories
  Resolving component wit dirs for app:component-a (/Users/<...>/app-demo/components-cpp/app-component-a/wit, /Users/<...>/app-demo/components-cpp/app-component-a/wit-generated)
  Resolving component wit dirs for app:component-b (/Users/<...>/app-demo/components-ts/app-component-b/wit, /Users/<...>/app-demo/components-ts/app-component-b/wit-generated)
Selecting profiles, no profile was requested
  Selected default profile debug for app:component-a using template cpp
  Selected default build for app:component-b using template ts
<...>
Linking RPC
  Copying app:component-a without linking, no static WASM RPC dependencies were found
  Copying app:component-b without linking, no static WASM RPC dependencies were found
Then we can check that the components are built:

$ ls golem-temp/components
app_component_a_debug.wasm app_component_b.wasm
To deploy (add or update) or components we can use

golem component deploy

in the project root folder, and the CLI will add or update all or components.

If we want to only update some components, we can do so by explicitly selecting them with the --component-name flag, or we can implicitly select them by changing our current working directory, e.g.:

$ cd components-cpp
$ golem app
<...>
Components:
  app:component-a
    Selected:     yes
    Source:       /Users/noise64/workspace/examples/app-demo/components-cpp/app-component-a/golem.yaml
    Template:     cpp
    Profiles:     debug, release
  app:component-b
    Selected:     no
    Source:       /Users/noise64/workspace/examples/app-demo/components-ts/app-component-b/golem.yaml
    Template:     ts
<...>
Notice, how only app:component-a is selected in the above example, the same selection logic is used when adding or updating components.

CLI
Components
Golem CLI Components
Golem components are WASM components deployed to Golem for execution.

To create, list, build and deploy components you can use golem component command.

Creating a new component
To create a new component in an application directory (previously created with golem app new) use the component new command in the following way:

golem component new <template-name> <component-name>

To see all the available component templates, just run the command without providing one.

This command only modifies the source code of the application, does not create anything on the server.

Building a component
To build a component, use the component build command in the following way:

golem component build <component-name>

To build the whole application, use the app build command in the following way:

golem app build

Both commands accept a --build-profile <BUILD_PROFILE> argument. Some of the built-in templates define a separate release profile which creates a more optimized version of the components. Build profiles can be fully customized by editing the application manifest files.

Deploying a component
To deploy a component, use the component deploy or app deploy commands in the following way:

golem component deploy <component-name>

or

golem app deploy <component-name>

to deploy a specific component only.

The output of the command will be something like the following:

New component created with URN urn:component:d8bc9194-a4a2-4a57-8545-43408fc38799, version 0, and size of 89735 bytes.
Component name: my-component.
Exports:
        rpc:counters/api.{[constructor]counter}(name: string) -> handle<0>
        rpc:counters/api.{[method]counter.inc-by}(self: &handle<0>, value: u64)
        rpc:counters/api.{[method]counter.get-value}(self: &handle<0>) -> u64
        rpc:counters/api.{[method]counter.get-args}(self: &handle<0>) -> list<string>
        rpc:counters/api.{[method]counter.get-env}(self: &handle<0>) -> list<tuple<string, string>>
        rpc:counters/api.{inc-global-by}(value: u64)
        rpc:counters/api.{get-global-value}() -> u64
        rpc:counters/api.{get-all-dropped}() -> list<tuple<string, u64>>
        rpc:counters/api.{bug-wasm-rpc-i32}(in: variant { leaf }) -> variant { leaf }
        rpc:counters/api.{[drop]counter}(self: handle<0>)
The returned output contains the following information:

Component URN - URN for the new component. You can use this URN whenever you want to specify the component instead of component name.
Component version - incremental component version - used for updating the workers.
Component size - size of the wasm file - it is important for the storage limit in the hosted Golem Cloud.
Exports - exported function you can call. You can copy function name (the part before parameters) to call the function. All Golem API expect function names in exactly this format. See the function name reference page for more details.
To deploy all components of an application, use:

golem app deploy

To deploy the component based on the current directory, use:

golem component deploy

Ephemeral components
Components created are durable by default. To create an ephemeral component instead, change the component's application manifest (golem.yaml) to contain:

components:
  example:component:
    componentType: ephemeral
Component search
Using component name and the latest version
If you want to get the latest version of the component using its name you can you can use the component get command.

golem get example:component

Using component name and specific version
To get a specific version of a component, just pass the desired version number as well:

golem get example:component 2

Getting component list
To get all component versions for specific component name you can use component list command with a given component name. Note if you are in a component's source directory, the command will automatically list that component's versions.

golem component list example:component

To get all component available you can use component list command this way:

golem component list

If you want to restrict component search to some specific project on Golem Cloud you can specify project via --project or --project-name option. It works for all commands that accept --component-name parameter.

Updating component
To update a component just run the component deploy (or app deploy) command again.

If you want to trigger an update all worker to the new latest version right after creating this version you can use --try-update-workers option.

It is possible to change the component's type (from durable to ephemeral, or from ephemeral to durable) when updating the component by changing the manifest file.

Updating workers
If you want to update all workers you can use component try-update-workers command.

This command gets all workers for the component that are not using the latest version and triggers an update for them one by one:

golem component try-update-workers example:component

The update request is enqueued and processed by the workers asynchronously, golem cannot await for the update to finish.

Note that automatic worker update is not guaranteed to succeed, if the updated component differs too much from the previous one.

You can use URL or --component-name instead.
Redeploying workers
During the development of a Golem Component, it is often necessary to quickly rebuild the code, update the component and just restart all the test workers from scratch to test the changes.

This is different from updating the workers as they will loose their state, but it can speed up the feedback loop during development.

This workflow is supported by the component redeploy command:

golem component redeploy example:component

This command deletes all workers that are not using the latest version of component and re-creates them with the same name, parameters and environment variables.

You can use URL or --component-name instead.


CLI
Workers
Golem CLI Workers
Using golem worker command you can:

Start and stop worker
Interrupt and resume workers
Invoke worker function
Get worker stdout, stderr and logs
Update workers
Search workers, get metadata, etc.
Revert a worker to a previous state
In all examples we are using component URN. You can use component URL or --component-name instead.

Start new worker
Even though workers can automatically start on the first invocation, it is possible to explicitly start a worker. This allows specifying command line arguments and environment variables for the worker.

golem worker new example:counter/counter1 --env A=1 --env B=2 arg1 arg2

You can see the URN for your new worker in command output. You can use URN whenever you want to specify worker instead of component and worker name.

Get worker metadata
Using worker name
You can get worker metadata using worker name and (optionally prefixed by the component name, if it is not inferrable from the current context):

golem worker get counter1
golem worker get example:counter/counter1

Search workers
You can search for workers of some components using worker list command with repeated --filter argument.

For instance lets find idle workers with component version older than 2 of the component example:counter:

golem worker list --filter "status = Idle" --filter "version < 2" example:counter

Enumerating workers is a slow operation and should only be used for debugging an administrative tasks.

Invoke functions
The folowig section shows the basics of invoking workers through the CLI. See the dedicated invocation with CLI page for more details.

Without waiting for result
You can invoke worker function without waiting for function result using worker invoke --enqueue command:

golem worker invoke --enqueue counter1 'rpc:counters/api.{inc-global-by}' 5

Function parameters can be specified using repeated --arg parameters.

Waiting for result
You can invoke worker function and wait for result using worker invoke command:

golem worker invoke counter1 'rpc:counters/api.{get-global-value}'

Invocation results in WAVE format:
- '5'
Ephemeral workers
Invoking ephemeral components does not require specifying the worker name, as Golem creates a new instance for each invocation. In this case only the component must be selected (if it is not inferred by the context), and the worker name must be -:

golem worker invoke example:ephemeral-component/- 'demo:worker/api.{test}'

Using idempotency key
If you want to make sure function was called only once even if you called CLI multiple times (for instance due to retries on network errors) you can use --idempotency-key parameter for both invoke and invoke-and-await commands.

You can use any string as idempotency key.

Live streaming worker logs
You can connect to your running worker and get logs using worker connect command this way:

golem worker connect counter1

You can also use --connect option on invoke command to connect to worker right after invoking the command.

Worker update
To update worker to some specific version of worker component you can use worker update this way:

golem worker update counter1 --target-version 2 --mode auto

You can also use worker update-many with the same --filter parameters as in worker list command to update multiple workers:

golem worker update-many example:counter --filter 'version < 2' --target-version 2 --mode auto

Interrupt and resume workers
If you want to interrupt and later resume a long-running worker you can use interrupt and resume commands:

golem worker interrupt counter1
golem worker resume counter1

Testing worker crash recovery
There is a special command to simulate unexpected worker crush and recovery - worker simulated-crash. This command can be used for tests:

golem worker simulated-crash counter1

Stopping workers
Idle worker are not actively consuming resources but they take storage as their state is persisted. A worker can be deleted using the worker delete command:

golem worker delete counter1

This command deletes worker state.

Please note that even though the worker can be deleted this way it would be started again (with the fresh state) if used:

golem worker delete counter1
golem worker invoke counter1 'rpc:counters/api.{get-global-value}'
Invocation results in WAVE format:
- '0'
Oplog query
It is possible to query an existing worker's oplog for debugging purposes. To get the full oplog of a worker, use the worker oplog command the worker must be specified just like in other golem-cli worker commands:

golem worker oplog counter1

Any other form of identifying a worker can be used (URL syntax, separate component-id and worker name, etc).

With the optional --from parameter it is possible to only get oplog entries after a certain oplog index:

golem worker oplog counter1

Oplog entries are indexed from 1, and the first entry is always a create entry that defines the initial state of the worker.

Searching for oplog entries
The same command can also be used to search for oplog entries, using the --query parameter. This parameter requries a query using lucene query syntax. The following syntax elements are supported:

search terms looks for search AND terms
AND, OR and NOT
grouping using parentheses ()
"quoted terms"
regular expression matches using /regex/
field:value to search for a specific information
The terms and fields are interpreted in the following way:

Oplog entry	Matching queries
Create	create
ImportedFunctionInvoked	imported-function-invoked, match on invoked function's name, match on function arguments(), match on result value()
ExportedFunctionInvoked	exported-function-invoked, exported-function, match on invoked function's name, match on idempotency key, match on function arguments(*)
ExportedFunctionCompleted	exported-function-completed, exported-function, match on response value(*)
Suspend	suspend
Error	error, match on error message
NoOp	noop
Jump	jump
Interrupted	interrupted
Exited	exited
ChangeRetryPolicy	change-retry-policy
BeginAtomicRegion	begin-atomic-region
EndAtomicRegion	end-atomic-region
BeginRemoteWrite	begin-remote-write
EndRemoteWrite	end-remote-write
PendingWorkerInvocation	pending-worker-invocation, match on invoked function's name, match on idempotency key, match on function arguments(*)
PendingUpdate	pending-update, update, match on target version
SuccessfulUpdate	successful-update, update, match on target version
FailedUpdate	failed-update, update, match on target version, match on error details
GrowMemory	grow-memory
CreateResource	create-resource
DropResource	drop-resource
DescribeResource	describe-resource, match on resource name, match on resource parameters(*)
Log	log, match on context, match on message
Restart	restart
ActivatePlugin	activate-plugin
DeactivatePlugin	deactivate-plugin
The cases marked with (*) can use the field:value syntax to look into the typed, structured parameter and result values.

For example to look for oplog entries that contain parameters or return values of exported functions where any of these input/output values is a record having a field product-id with either value 123 or 456, we can use the following query:

golem worker oplog --worker urn:worker:d8bc9194-a4a2-4a57-8545-43408fc38799/counter1 --query 'exported-function AND (product-id:123 OR product-id:456)'
Reverting a worker
It is possible to revert a worker to its previous state. This can be useful if a worker got into a failed state to make it usable again, or to undo some accidental invocations or updates.

There are two possible ways to specify which state to revert to:

By index: The index of the oplog entry to revert to. Use the oplog query command to check the worker's oplog and find the index of the desired state.
By undoing the last invocations: The given number is the number of invocations to revert.
To revert to a given oplog, use the worker revert command:

golem worker revert counter1 --last-oplog-index 42

To revert some of the last invocations, use the --number-of-invocations parameter instead:

golem worker revert counter1 --number-of-invocations 3


CLI
Profiles
Golem CLI Profiles
Profiles are used for golem CLI configuration. Using different profiles you can use golem with multiple installations of an open source Golem and the hosted Golem Cloud at the same time.

Interactive profile creation
To start interactive profile creation run the following command:

golem init

If you want to specify a custom profile name for an interactive profile creation process - you can use the following command:

golem profile init custom-name

On the first step you'll see 3 options:

Golem Default. Use this options for default local docker compose installation.
Golem. Use this option in case of a customised Golem installation, i.e. a custom GOLEM_ROUTER_PORT in .env file.
Golem Cloud. Use this option for a hosted version of Golem.
With Golem Default the hosted Golem Cloud options there are no other specific configuration options. To specify a custom location for your local open source Golem installation - please use the Golem option.

Non-interactive profile creation
Hosted Golem Cloud profile
To create a profile for a hosted Golem Cloud use the following command:

golem profile add --set-active golem-cloud --default-format json my-profile-name

This command creates a new Golem Cloud profile named my-profile-name with default output format json and sets it as a new active profile. If you want to keep default output format text - you can omit --default-format json part. If you don't want to make the new profile as the new active profile - you can omit --set-active.

If you are using golem-cloud-cli binary you should omit profile type (golem-cloud) since golem-cloud-cli does not support other profile types:

Terminal
golem-cloud-cli profile add my-profile-name

Local open source Golem
To create a profile for your local open source Golem installation use the following command:

golem profile add --set-active golem --component-url http://localhost:9881 my-oss-profile-name

This command creates a new open source Golem profile named my-oss-profile-name with both component and worker service location as http://localhost:9881 and sets it as the new active profile.

Additionally, you can specify --default-format option for json or yaml instead of human-readable text and --worker-url in case you want to have worker and component services on different locations.

If you are using an open source specific golem - you should omit profile type (golem). If you are using a Golem Cloud specific golem-cloud-cli - you can't create an open source Golem profile.

Profile authentication
This is Golem Cloud specific part.
To authentication your Golem Cloud profile you can run any command that requires authentication, i.e. you can run any command that wants to access Golem Cloud servers. The easiest way to authenticate your profile woutd be to run the following command:

golem account get

At the moment the only way to authenticate your account is to use Github Oauth2 authorization. Please follow the instructions in your terminal and authorize zivergetech organisation to use OAuth2:

>>
>>  Application requests to perform OAuth2
>>  authorization.
>>
>>  Visit following URL in a browser:
>>
>>   ┏---------------------------------┓
>>   ┃ https://github.com/login/device ┃
>>   ┗---------------------------------┛
>>
>>  And enter following code:
>>
>>   ┏-----------┓
>>   ┃ ADFC-A318 ┃
>>   ┗-----------┛
>>
>>  Code will expire in 14 minutes at 15:15:27.
>>
Waiting...
Account with id account-id for name Your Name with email your@email.com.
Switch profiles
To get the list of your profiles use the following command:

golem profile list

You'll get all available profiles with the active profile marked by *:

 * my-oss-profile-name
   my-profile-name
To switch active profile use profile switch command:

golem profile switch my-profile-name

Golem profile configuration
At the moment the only configurable option is default output format.

To change the default output format for the current active profile you can use profile config format command this way:

golem profile config format text

Output formats
There are 3 output formats:

text - human-readable format
json
yaml
Almost all commands can change output format based on --format option or default output format configured for the current active profile.

Profile configuration files
All golem configuration files are stored in .golem directory in your home directory. This includes the cached authentication token to your Golem Cloud profile. It is safe to remove $HOME/.golem directory in case of any issues with profiles - you will keep access to your Golem Cloud account as log as you have access to your Github account linked to your Golem Cloud account.

CLI
Permissions
Golem CLI Permissions
This page only applies to the hosted Golem Cloud.
Tokens
Tokens are API keys that allow accessing Golem Cloud APIs from external services. The golem-cloud-cli tool allows managing these tokens. To manage them programmatically, check the token API.

Listing existing tokens
The following command lists all the tokens associated with your account:

golem token list

Creating a new token
To create a new token, use the following command:

golem token add

New token created with id 08bc0eac-5c51-40a5-8bc6-5c8928efb475 and expiration date 2100-01-01 00:00:00 UTC.
Please save this token secret, you can't get this data later:
64cf566c-ed72-48e5-b786-b88aa0298fb4
Optionally, an expiration date can be specified with --expires-at. If not specified, default expiration date is 2100-01-01 00:00:00 UTC.

Deleting a token
Each token has a token ID. Use the token delete command to remove a token using it's identifier:

golem token delete 08bc0eac-5c51-40a5-8bc6-5c8928efb475

Project sharing
On Golem Cloud components are organized into projects.

Listing projects
Existing projects can be listed using the project list subcommand:

golem project list

Adding projects
A new project can be created using project add. The command expects a project name and description:

golem project add --project-name "Golem Demo" --project-description "A new project for demonstrating the project feature"

When creating components or workers, the project can be specified with the --project-name flag. Every user has a default project which is used when no explicit project is specified.

Sharing projects with other accounts
Projects can be shared among multiple Golem Cloud accounts.

To share a project, use the share subcommand:

golem share --project-name "Golem Demo" --recipient-account-id 08bc0eac-5c51-40a5-8bc6-5c8928efb475 --project-actions ViewWorker --project-actions ViewComponent

This example shares the "Golem Demo" project with the account identified by 08bc0eac-5c51-40a5-8bc6-5c8928efb475 and grants component and worker view permissions for it.

Alternatively it is possible to create and manage project policies using the project-policy subcommand, and refer to these policies in the share command later.

The following table lists all the actions that can be granted to a project:

Action	Description
ViewComponent	List, download and get metadata of components
CreateComponent	Create new components
UpdateComponent	Update existing components
DeleteComponent	Delete components
ViewWorker	List and get metadata of workers
CreateWorker	Create new workers
UpdateWorker	Update existing workers
DeleteWorker	Delete workers
ViewProjectGrants	List existing project grants
CreateProjectGrants	Grant more access for the project
DeleteProjectGrants	Revoke access for the project
ViewApiDefinition	View API definitions
CreateApiDefinition	Create new API definitions
UpdateApiDefinition	Update existing API definitions
DeleteApiDefinition	Delete API definitions

CLI
Permissions
Golem CLI Permissions
This page only applies to the hosted Golem Cloud.
Tokens
Tokens are API keys that allow accessing Golem Cloud APIs from external services. The golem-cloud-cli tool allows managing these tokens. To manage them programmatically, check the token API.

Listing existing tokens
The following command lists all the tokens associated with your account:

golem token list

Creating a new token
To create a new token, use the following command:

golem token add

New token created with id 08bc0eac-5c51-40a5-8bc6-5c8928efb475 and expiration date 2100-01-01 00:00:00 UTC.
Please save this token secret, you can't get this data later:
64cf566c-ed72-48e5-b786-b88aa0298fb4
Optionally, an expiration date can be specified with --expires-at. If not specified, default expiration date is 2100-01-01 00:00:00 UTC.

Deleting a token
Each token has a token ID. Use the token delete command to remove a token using it's identifier:

golem token delete 08bc0eac-5c51-40a5-8bc6-5c8928efb475

Project sharing
On Golem Cloud components are organized into projects.

Listing projects
Existing projects can be listed using the project list subcommand:

golem project list

Adding projects
A new project can be created using project add. The command expects a project name and description:

golem project add --project-name "Golem Demo" --project-description "A new project for demonstrating the project feature"

When creating components or workers, the project can be specified with the --project-name flag. Every user has a default project which is used when no explicit project is specified.

Sharing projects with other accounts
Projects can be shared among multiple Golem Cloud accounts.

To share a project, use the share subcommand:

golem share --project-name "Golem Demo" --recipient-account-id 08bc0eac-5c51-40a5-8bc6-5c8928efb475 --project-actions ViewWorker --project-actions ViewComponent

This example shares the "Golem Demo" project with the account identified by 08bc0eac-5c51-40a5-8bc6-5c8928efb475 and grants component and worker view permissions for it.

Alternatively it is possible to create and manage project policies using the project-policy subcommand, and refer to these policies in the share command later.

The following table lists all the actions that can be granted to a project:

Action	Description
ViewComponent	List, download and get metadata of components
CreateComponent	Create new components
UpdateComponent	Update existing components
DeleteComponent	Delete components
ViewWorker	List and get metadata of workers
CreateWorker	Create new workers
UpdateWorker	Update existing workers
DeleteWorker	Delete workers
ViewProjectGrants	List existing project grants
CreateProjectGrants	Grant more access for the project
DeleteProjectGrants	Revoke access for the project
ViewApiDefinition	View API definitions
CreateApiDefinition	Create new API definitions
UpdateApiDefinition	Update existing API definitions
DeleteApiDefinition	Delete API definitions

Function names
Function name syntax
This section explains how to map the exported function names from the component's WIT definition to fully qualified names to be passed to the invocation API or CLI when invoking workers.

Functions and interfaces
The component has a WIT package, specified in the top of its WIT definition.

For example if the component was generated using golem new without specifying a custom package name, it will be:

package golem:component
This package name can optionally specify a package version as well, such as:

package golem:component@0.3.2
The WIT definition should contain a single world (otherwise the world to be used have to be specified for the tools used during compilation). The name of this world does not matter for Golem - it won't be part of the function's fully qualified name.

For example it can be:

world my-component-world {
  // ...
}
This world can either export

one or more interface
or one or more functions directly
The following example demonstrates both:

package golem:component;
 
interface api {
  record product-item {
    product-id: string,
    name: string,
    price: f32,
    quantity: u32,
  }
 
  add-item: func(item: product-item) -> ();
  remove-item: func(product-id: string) -> ();
}
 
world my-component-world {
  export api;
  export dump: func() -> result<string, string>;
}
The name of the interface(s) and function(s) are completely user defined, there are no rules to follow other than the syntax rules of WIT itself.

In the above example we have 3 exported functions and we can refer to them with the following fully qualified names, consisting of the package name and the exported path:

golem:component/api.{add-item}
golem:component/api.{remove-item}
golem:component.{dump}
Note that the syntax is the same as the one used for the use statements in the WIT file.

Resources
WIT specifications also allow the definition of resources. These are constructed via special constructors, have methods, and can also have associated static functions. Golem supports exporting resources, enabling an alternative of having a separate worker for each entity. When exporting resources, a single worker may own an arbitrary number of instances of the exported resource, and the method invocations's first parameter must be the resource handle returned by the invoked constructor.

There is a special naming syntax provided by Golem that makes it more convenient to invoke resource constructors and methods. Take the following example:

package golem:component;
 
interface api {
  resource counter {
    constructor(name: string);
    inc-by: func(value: u64);
    get-value: func() -> u64;
 
    merge-counters: static func(counter1: counter, counter2: counter, name: string) -> counter;
  }
}
 
world my-world {
  export api;
}
For this WIT specification, the following function names are valid when using Golem's invocation API or CLI:

golem:component/api.{counter.new} - refers to the above defined constructor
golem:component/api.{counter.inc-by}
golem:component/api.{counter.get-value}
golem:component/api.{counter.merge-counters}
golem:component/api.{counter.drop} - special function that drops the instance while the worker continues running
Implicit resource creation
With the above described naming conventions it is possible to manually create instances of resources, and then use the returned resource handle to invoke methods on them.

There is an easier way to work with resources in Golem, that assumes that a given resource instance is associated with the constructor parameters it is created with. This way it is possible to target a specific instance just by using the function name, and the resource instance will be automatically selected or created if it does not exist yet.

To use this feature, pass the target constructor parameters in the function name's resource name part in parentheses. For example, with the above defined counter resource we can immediately create and a new counter and increment it by calling:

golem:component/api.{counter("my-counter").inc-by}
This will create a new counter instance with the name my-counter and increment it. If the counter with the name my-counter already exists, it will be used.

The syntax for passing inlined parameters to the constructor is using the WebAssembly Value Encoding.


Golem Host Functions
Golem Host functions
Golem provides three WIT packages of host functions for accessing Golem-specific functionality on top of the standard WebAssembly system interfaces.

These three packages are:

golem:api@1.1.6 - provides access to various aspects of Golem for the workers
golem:rpc@0.2.0 - defines the types and functions serving the Worker to Worker communication in Golem
golem:durability@1.2.0 - provides an API to implement custom durability in libraries
Please check the language specific guidelines to learn the best way to work with these APIs in your language of choice:

Rust
Python
Go
C/C++
TypeScript
JavaScript
Zig
Scala.js
MoonBit
This page provides an overview of all the exported functions and types in a language-agnostic way.

Types
An oplog index points to a specific instruction in the worker's history:

package golem:api@1.1.6;
 
interface host {
    /// An index into the persistent log storing all performed operations of a worker
    type oplog-index = u64;
}
The worker id uniquely identifies a worker in the Golem system:

package golem:rpc@0.2.0;
 
interface types {
    /// Represents a Golem worker
    record worker-id {
        component-id: component-id,
        worker-name: string
    }
}
Components are identified by a component id which is in fact a UUID:

package golem:rpc@0.2.0;
 
interface types {
    /// Represents a Golem component
    record component-id {
        uuid: uuid,
    }
 
    /// UUID
    record uuid {
    high-bits: u64,
    low-bits: u64
    }
}
Component versions are unsigned integers:

package golem:api@1.1.6;
 
interface host {
    /// Represents a Golem component's version
    type component-version = u64;
}
Promises
Types
A promise is identified by a promise-id type consisting of the owner worker's identifier and a number that points to the create promise function in the worker's history:

package golem:api@1.1.6;
 
interface host {
    /// A promise ID is a value that can be passed to an external Golem API to complete that promise
    /// from an arbitrary external source, while Golem workers can await for this completion.
    record promise-id {
        worker-id: worker-id,
        oplog-idx: oplog-index,
    }
}
Functions
The following functions define the promise API:

Function name	Definition	Description
create-promise	func() -> promise-id	Creates a new promise and returns its ID
await-promise	func(promise-id) -> list<u8>	Suspends execution until the given promise gets completed, and returns the payload passed to the promise completion.
complete-promise	func(promise-id, list<u8>)	Completes the given promise with the given payload. Returns true if the promise was completed, false if the promise was already completed. The payload is passed to the worker that is awaiting the promise.
delete-promise	func(promise-id) -> ()	Deletes the given promise.
Worker metadata
Types
Worker metadata is described by the following WIT types:

package golem:api@1.1.6;
 
interface host {
    record worker-metadata {
        worker-id: worker-id,
        args: list<string>,
        env: list<tuple<string, string>>,
        status: worker-status,
        component-version: u64,
        retry-count: u64
    }
 
    enum worker-status {
        /// The worker is running an invoked function
        running,
        /// The worker is ready to run an invoked function
        idle,
        /// An invocation is active but waiting for something (sleeping, waiting for a promise)
        suspended,
        /// The last invocation was interrupted but will be resumed
        interrupted,
        /// The last invocation failed and a retry was scheduled
        retrying,
        /// The last invocation failed and the worker can no longer be used
        failed,
        /// The worker exited after a successful invocation and can no longer be invoked
        exited,
    }
}
Functions
The following functions define the worker metadata API:

Function name	Definition	Description
get-worker-metadata	func(worker-id) -> option<worker-metadata>	Returns the metadata of a worker
get-self-metadata	func() -> worker-metadata	Returns the metadata of the worker that calls this function
Worker enumeration
The worker enumeration API allows listing all the workers belonging to a specific component. This is a slow operation and should be used for maintenance tasks and not for the core business logic.

Worker enumeration is a WIT resource, providing a method that returns a page of workers each time it is called:

package golem:api@1.1.6;
 
interface host {
    resource get-workers {
        constructor(component-id: component-id, filter: option<worker-any-filter>, precise: bool);
 
        get-next: func() -> option<list<worker-metadata>>;
    }
}
Once get-next returns none, the enumeration is done.

There are two parameters for customizing the worker enumeration:

Filters
An optional filter parameter can be passed to the worker enumeration, with the following definition:

package golem:api@1.1.6;
 
interface host {
    record worker-any-filter {
        filters: list<worker-all-filter>
    }
}
The worker-any-filter matches workers that satisfy any of the listed filters.

package golem:api@1.1.6;
 
interface host {
    record worker-all-filter {
        filters: list<worker-property-filter>
    }
}
A worker-all-filter matches workers only that satisfy all of the listed filters.

package golem:api@1.1.6;
 
interface host {
    variant worker-property-filter {
        name(worker-name-filter),
        status(worker-status-filter),
        version(worker-version-filter),
        created-at(worker-created-at-filter),
        env(worker-env-filter)
    }
}
The worker-name-filter matches workers by one of their properties, such as the worker name, status, version, creation date or environment variables.

Each of these variants take a filter record holding a value and a comparator:

package golem:api@1.1.6;
 
interface host {
    record worker-name-filter {
        comparator: string-filter-comparator,
        value: string
    }
 
    record worker-status-filter {
        comparator: filter-comparator,
        value: worker-status
    }
 
    record worker-version-filter {
        comparator: filter-comparator,
        value: u64
    }
 
    record worker-created-at-filter {
        comparator: filter-comparator,
        value: u64
    }
 
    record worker-env-filter {
        name: string,
        comparator: string-filter-comparator,
        value: string
    }
}
Where filter-comparator and string-filter-comparator are defined as:

package golem:api@1.1.6;
 
interface host {
    enum filter-comparator {
        equal,
        not-equal,
        greater-equal,
        greater,
        less-equal,
        less
    }
 
    enum string-filter-comparator {
        equal,
        not-equal,
        like,
        not-like
    }
}
Precise enumeration
The precise flag switches the worker enumeration into a more costly variant which guarantees that every worker metadata returned is the latest one available at the time the result was generated.

When precise is set to false, the enumeration returns the last cached worker metadata available for each worker, which may be lagging behind the actual state of the workers.

Transactions and persistence control
Golem's transaction API allows customizing the execution engine's durability and transactional behaviors. These are low level functions, which are wrapped by SDKs providing a higher level transaction API for each supported language.

Types
Retry policy is defined by the following record:

package golem:api@1.1.6;
 
interface host {
    /// Configures how the executor retries failures
    record retry-policy {
        /// The maximum number of retries before the worker becomes permanently failed
        max-attempts: u32,
        /// The minimum delay between retries (applied to the first retry)
        min-delay: duration,
        /// The maximum delay between retries
        max-delay: duration,
        /// Multiplier applied to the delay on each retry to implement exponential backoff
        multiplier: f64,
        /// The maximum amount of jitter to add to the delay
        max-jitter-factor: option<f64>
    }
}
It is possible to switch between persistence modes:

package golem:api@1.1.6;
 
interface host {
    /// Configurable persistence level for workers
    variant persistence-level {
        persist-nothing,
        persist-remote-side-effects,
        smart
    }
}
Functions
The following functions define the transaction API:

Function name	Definition	Description
get-oplog-index	func() -> oplog-index	Returns the current position in the persistent op log
set-oplog-index	func(oplog-idx: oplog-index) -> ()	Makes the current worker travel back in time and continue execution from the given position in the persistent op log.
oplog-commit	func(replicas: u8) -> ()	Blocks the execution until the oplog has been written to at least the specified number of replicas, or the maximum number of replicas if the requested number is higher.
mark-begin-operation	func() -> oplog-index	Marks the beginning of an atomic operation. In case of a failure within the region selected by mark-begin-operation and mark-end-operation, the whole region will be reexecuted on retry. The end of the region is when mark-end-operation is called with the returned oplog-index.
mark-end-operation	func(begin: oplog-index) -> ()	Commits this atomic operation. After mark-end-operation is called for a given index, further calls with the same parameter will do nothing.
get-retry-policy	func() -> retry-policy	Gets the current retry policy associated with the worker
set-retry-policy	func(policy: retry-policy) -> ()	Overrides the current retry policy associated with the worker. Following this call, get-retry-policy will return the new retry policy.
get-oplog-persistence-level	func() -> persistence-level	Gets the worker's current persistence level.
set-oplog-persistence-level	func(new-persistence-level: persistence-level) -> ()	Sets the worker's current persistence level. This can increase the performance of execution in cases where durable execution is not required.
get-idempotence-mode	func() -> bool	Gets the worker's current idempotence mode.
set-idempotence-mode	func(idempotent: bool) -> ()	Sets the current idempotence mode. The default is true. True means side-effects are treated idempotent and Golem guarantees at-least-once semantics. In case of false the executor provides at-most-once semantics, failing the worker in case it is not known if the side effect was already executed.
generate-idempotency-key	func() -> uuid	Generates a new idempotency key.
Update
The update API enables workers to trigger automatic or manual update of other workers.

Types
Automatic and manual updates are distinguished by the following enum:

package golem:api@1.1.6;
 
interface host {
    /// Describes how to update a worker to a different component version
    enum update-mode {
        /// Automatic update tries to recover the worker using the new component version
        /// and may fail if there is a divergence.
        automatic,
 
        /// Manual, snapshot-based update uses a user-defined implementation of the `save-snapshot` interface
        /// to store the worker's state, and a user-defined implementation of the `load-snapshot` interface to
        /// load it into the new version.
        snapshot-based
    }
}
Functions
The following function defines the update API:

Function name	Definition	Description
update-worker	func(worker-id: worker-id, target-version: component-version, mode: update-mode) -> ()	Initiates an update attempt for the given worker. The function returns immediately once the request has been processed, not waiting for the worker to get updated.
Oplog search and query
The oplog interface in golem:api provides functions to search and query the worker's persisted oplog.

The interface defines a big variant data type called oplog-entry:

package golem:api@1.1.6;
 
interface host {
    variant oplog-entry {
        /// The initial worker oplog entry
        create(create-parameters),
        /// The worker invoked a host function
        imported-function-invoked(imported-function-invoked-parameters),
        /// The worker has been invoked
        exported-function-invoked(exported-function-invoked-parameters),
        /// The worker has completed an invocation
        exported-function-completed(exported-function-completed-parameters),
        /// Worker suspended
        suspend(datetime),
        /// Worker failed
        error(error-parameters),
        /// Marker entry added when get-oplog-index is called from the worker, to make the jumping behavior
        /// more predictable.
        no-op(datetime),
        /// The worker needs to recover up to the given target oplog index and continue running from
        /// the source oplog index from there
        /// `jump` is an oplog region representing that from the end of that region we want to go back to the start and
        /// ignore all recorded operations in between.
        jump(jump-parameters),
        /// Indicates that the worker has been interrupted at this point.
        /// Only used to recompute the worker's (cached) status, has no effect on execution.
        interrupted(datetime),
        /// Indicates that the worker has been exited using WASI's exit function.
        exited(datetime),
        /// Overrides the worker's retry policy
        change-retry-policy(change-retry-policy-parameters),
        /// Begins an atomic region. All oplog entries after `BeginAtomicRegion` are to be ignored during
        /// recovery except if there is a corresponding `EndAtomicRegion` entry.
        begin-atomic-region(datetime),
        /// Ends an atomic region. All oplog entries between the corresponding `BeginAtomicRegion` and this
        /// entry are to be considered during recovery, and the begin/end markers can be removed during oplog
        /// compaction.
        end-atomic-region(end-atomic-region-parameters),
        /// Begins a remote write operation. Only used when idempotence mode is off. In this case each
        /// remote write must be surrounded by a `BeginRemoteWrite` and `EndRemoteWrite` log pair and
        /// unfinished remote writes cannot be recovered.
        begin-remote-write(datetime),
        /// Marks the end of a remote write operation. Only used when idempotence mode is off.
        end-remote-write(end-remote-write-parameters),
        /// An invocation request arrived while the worker was busy
        pending-worker-invocation(pending-worker-invocation-parameters),
        /// An update request arrived and will be applied as soon the worker restarts
        pending-update(pending-update-parameters),
        /// An update was successfully applied
        successful-update(successful-update-parameters),
        /// An update failed to be applied
        failed-update(failed-update-parameters),
        /// Increased total linear memory size
        grow-memory(grow-memory-parameters),
        /// Created a resource instance
        create-resource(create-resource-parameters),
        /// Dropped a resource instance
        drop-resource(drop-resource-parameters),
        /// Adds additional information for a created resource instance
        describe-resource(describe-resource-parameters),
        /// The worker emitted a log message
        log(log-parameters),
        /// The worker's has been restarted, forgetting all its history
        restart(datetime),
        /// Activates a plugin
        activate-plugin(activate-plugin-parameters),
        /// Deactivates a plugin
        deactivate-plugin(deactivate-plugin-parameters),
        /// Revert a worker to a previous state
        revert(revert-parameters),
        /// Cancel a pending invocation
        cancel-invocation(cancel-invocation-parameters),
        /// Start a new span in the invocation context
        start-span(start-span-parameters),
        /// Finish an open span in the invocation context
        finish-span(finish-span-parameters),
        /// Set an attribute on an open span in the invocation context
        set-span-attribute(set-span-attribute-parameters),
    }
}
and two resources for querying a worker's oplog.

the get-oplog resource enumerates through all entries of the oplog
the search-oplog resource accepts a search expression and only returns the matching entries
Both resources, once constructed, provide a get-next function that returns a chunk of oplog entries. Repeatedly calling this function goes through the whole data set, and eventually returns none.

Durability
The golem:durability package contains an API that can be leveraged by libraries to provide a custom durability implementation for their own API. This is the same interface that Golem uses under the hood to make the WASI interfaces durable. Golem applications are not supposed to use this package directly.

Types
The durable-function-type is a variant that categorizes a durable function in the following way:

package golem:api@1.1.6;
 
interface host {
    variant durable-function-type {
        /// The side-effect reads from the worker's local state (for example local file system,
        /// random generator, etc.)
        read-local,
        /// The side-effect writes to the worker's local state (for example local file system)
        write-local,
        /// The side-effect reads from external state (for example a key-value store)
        read-remote,
        /// The side-effect manipulates external state (for example an RPC call)
        write-remote,
        /// The side-effect manipulates external state through multiple invoked functions (for example
        /// a HTTP request where reading the response involves multiple host function calls)
        ///
        /// On the first invocation of the batch, the parameter should be `None` - this triggers
        /// writing a `BeginRemoteWrite` entry in the oplog. Followup invocations should contain
        /// this entry's index as the parameter. In batched remote writes it is the caller's responsibility
        /// to manually write an `EndRemoteWrite` entry (using `end_function`) when the operation is completed.
        write-remote-batched(option<oplog-index>)
    }
}
The durable-execution-state record provides information about the current execution state, and can be queried using the current-durable-execution-state function:

package golem:api@1.1.6;
 
interface host {
    record durable-execution-state {
        is-live: bool,
        persistence-level: persistence-level,
    }
}
Here the is-live field indicates whether the executor is currently replaying a worker's previously persisted state or side effects should be executed. The persistence-level is a user-configurable setting that can turn off persistence for certain sections of the code.

The persisted-durable-function-invocation is a record holding all the information about one persisted durable function. This should be used during replay to simulate the side effect instead of actually running it.

Functions
The durability API consists of a couple of low-level functions that must be called in a correct way to make it work.

The logic to be implemented is the following, in pseudocode:

observe-function-call("interface", "function")
state = current-durable-execution-state()
if state.is-live {
  result = perform-side-effect(input)
  persist-typed-durable-function-invocation("function", encode(input), encode(result), durable-function-type)
} else {
  // Execute the side effect
  persisted = read-persisted-durable-function-invocation()
  result = decode(persisted.response)
}
The input and result values must be encoded into value-and-type, the dynamic value representation from the golem:rpc package.

In cases when a durable function's execution interleaves with other calls, the begin-durable-function and end-durable-function calls can be used to mark the beginning and end of the operation.

Invocation context
Golem associates an invocation context with each invocation, which contains various information depending on how the exported function was called. This context gets inherited when making further invocations via worker-to-worker communication, and it is also possible to define custom spans and associate custom attributes to it.

The spans are not automatically sent to any tracing system but they can be reconstructed from the oplog, for example using oplog processor plugins, to provide real-time tracing information.

To get the current invocation context, use the current-context host function:

package golem:api@1.1.6;
 
/// Invocation context support
interface context {
    current-context: func() -> invocation-context;
}
The invocation-context itself is a resource with various methods for querying attributes of the invocation context:

method	description
trace-id	Returns the trace ID associated with the context, coming from either an external trace header or generated at the edge of Golem
span-id	Returns the span ID associated with the context
parent	Returns the parent invocation context, if any
get-attribute	Gets an attribute from the context by key
get-attributes	Gets all attributes from the context
get-attribute-chain	Gets all values of a given attribute from the current and parent contexts
get-attribute-chains	Get all attributes and their previous values
trace-context-headers	Gets the W3C Trace Context headers associated with the current invocation context
Custom attributes can only be set on custom spans. First start a new span using start-span

package golem:api@1.1.6;
 
/// Invocation context support
interface context {
    /// Starts a new `span` with the given name, as a child of the current invocation context
    start-span: func(name: string) -> span;
}
and then use the span resource's methods:

method	description
started-at	Returns the timestamp when the span was started
set-attribute	Sets an attribute on the span
set-attributes	Sets multiple attributes on the span
finish	Ends the current span
Dropping the resource is equivalent to calling finish on the span.

The custom spans are pushed onto the invocation context stack, so whenever an RPC call or HTTP call is made, their parent span(s) will include the user-defined custom spans as well as the rest of the invocation context.


Installing WebAssembly Tooling
Installing WebAssembly tooling
Golem only supports specific versions of wasm-tools and wit-bindgen. Please make sure to follow the guides carefully and install the correct versions.

You will need both wasm-tools and wit-bindgen to work with WebAssembly in a lot of languages. You can either download these from GitHub or build from source using cargo.

wit-bindgen
Download the correct asset for your system from the GitHub Release.

Extract the downloaded archive:

❯ tar xvf wit-bindgen-0.37.0-x86_64-linux.tar.gz
x wit-bindgen-0.37.0-x86_64-linux/
x wit-bindgen-0.37.0-x86_64-linux/README.md
x wit-bindgen-0.37.0-x86_64-linux/LICENSE-APACHE
x wit-bindgen-0.37.0-x86_64-linux/wit-bindgen
x wit-bindgen-0.37.0-x86_64-linux/LICENSE-MIT
x wit-bindgen-0.37.0-x86_64-linux/LICENSE-Apache-2.0_WITH_LLVM-exception

Install to /usr/local/bin:

❯ sudo install -D -t /usr/local/bin wit-bindgen-0.37.0-x86_64-linux/wit-bindgen

wasm-tools
Download the correct asset for your system from the GitHub Release.

Extract the downloaded archive:

❯ tar xvf wasm-tools-1.223.0-x86_64-linux.tar.gz
x wasm-tools-1.223.0-x86_64-linux/
x wasm-tools-1.223.0-x86_64-linux/README.md
x wasm-tools-1.223.0-x86_64-linux/wasm-tools
x wasm-tools-1.223.0-x86_64-linux/LICENSE-APACHE
x wasm-tools-1.223.0-x86_64-linux/LICENSE-MIT
x wasm-tools-1.223.0-x86_64-linux/LICENSE-Apache-2.0_WITH_LLVM-exception

Install to /usr/local/bin:

❯ sudo install -D -t wasm-tools-1.223.0-x86_64-linux/wasm-tools


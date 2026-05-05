#![allow(dead_code)]
// Copyright 2011 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Addendum: Port initally performed by Jordan Morris 2026 jthecybertinker@gmail.com
// above license still applies in all cases



use std::{
    collections::{BTreeSet, HashMap},
    ffi::{c_int, c_ulonglong},
};

// TODO
type EdgePriorityQueue = ();
type DependencyScan = ();
type DynDepFile = ();
type ExitStatus = ();
type JobserverClient = ();
type DepsLog = ();
type DepfileParserOptions = ();
#[repr(C)]
pub struct BuildLog;
#[repr(C)]
pub struct DiskInterface;
#[repr(C)]
pub struct Edge;
#[repr(C)]
pub struct Explanations;
#[repr(C)]
pub struct Node;
#[repr(C)]
pub struct State;

#[repr(C)]
pub struct Status;

// struct BuildConfig;
#[repr(C)]
pub enum EdgeResult {
    KEdgeFailed,
    KEdgeSucceeded,
}

/// Enumerate possible steps we want for an edge.
#[repr(C)]
pub enum Want {
    /// We do not want to build the edge, but we might want to build one of
    /// its dependents.
    KWantNothing,
    /// We want to build the edge, but have not yet scheduled it.
    KWantToStart,
    /// We want to build the edge, have scheduled it, and are waiting
    /// for it to complete.
    KWantToFinish,
}

/// Plan stores the state of a build plan: what we intend to build,
/// which steps we're ready to execute.
#[repr(C)]
pub struct Plan {
    /// Total number of edges that have commands (not phony).
    pub command_edges_: c_int,

    /// Total remaining number of wanted edges.
    pub wanted_edges_: c_int,
    pub edge_result: EdgeResult,
    // private members
    /// Keep track of which edges we want to build in this plan.  If this map does
    /// not contain an entry for an edge, we do not want to build the entry or its
    /// dependents.  If it does contain an entry, the enumeration indicates what
    /// we want for the edge.
    //   std::map<Edge*, Want> want_;
    want: HashMap<*mut Edge, Want>,
    ready_: EdgePriorityQueue,

    // for this we need to rearrange memory responsibility
    // bulder_: *mut Builder,
    //   Builder* builder_;
    /// user provided targets in build order, earlier one have higher priority
    targets: Vec<*const Node>,
    //   std::vector<const Node*> targets_;
}

// note that the ptr pattern may not work with & references
impl<'edge> Plan {
    pub fn plan() -> Self {
        todo!()
    }

    /// Add a target to our plan (including all its dependencies).
    /// Returns false if we don't need to build this target; may
    /// fill in |err| with an error message if there's a problem.
    pub fn add_target(_target: &Node, _err: Option<&str>) -> bool {
        todo!()
    }
    // Pop a ready edge off the queue of edges to build.
    // Returns NULL if there's no work to do.
    pub fn find_work() -> *mut Edge {
        todo!()
    }
    /// Returns true if there's more work to be done.
    ///
    pub fn more_to_do(&self) -> bool {
        return self.wanted_edges_ > 0 && self.command_edges_ > 0;
    }
    /// Mark an edge as done building (whether it succeeded or failed).
    /// If any of the edge's outputs are dyndep bindings of their dependents,
    /// this loads dynamic dependencies from the nodes' paths.
    /// Returns 'false' if loading dyndep info fails and 'true' otherwise.
    //   bool EdgeFinished(Edge* edge, EdgeResult result, std::string* err);
    pub fn edge_finished(_edge: *mut Edge, _result: EdgeResult, _err: &str) {
        todo!()
    }
    /// Clean the given node during the build.
    /// Return false on error.
    //   bool CleanNode(DependencyScan* scan, Node* node, std::string* err);
    pub fn clean_node(_scan: *mut DependencyScan, _node: *mut Node, _err: &str) {
        todo!()
    }
    /// Number of edges with commands to run.
    pub fn command_edge_count(&self) -> c_int {
        return self.command_edges_;
    }

    /// Reset state.  Clears want and ready sets.
    pub fn reset(&mut self) -> () {
        todo!()
    }

    // After all targets have been added, prepares the ready queue for find work.
    //   void PrepareQueue();
    pub fn prepare_queue() -> () {
        todo!()
    }

    /// Update the build plan to account for modifications made to the graph
    /// by information loaded from a dyndep file.
    //   bool DyndepsLoaded(DependencyScan* scan, node: *const Node,
    //                      const DyndepFile& ddf, std::string* err);
    fn dyndeps_loaded(
        _scan: *mut DependencyScan,
        _node: *const Node,
        _ddf: &DynDepFile,
        _err: &str,
    ) -> bool {
        todo!()
    }
    /// Dumps the current state of the plan.
    fn dump(&self) -> () {
        todo!()
    }

    //   private functions

    fn compute_critical_path() -> () {
        todo!()
    }
    fn refresh_dyndep_dependents(_scan: *mut DependencyScan, _node: *const Node, _err: &str) -> bool {
        todo!()
    }
    fn unmark_dependents(_node: *const Node, _dependends: BTreeSet<*mut Node>) {
        todo!()
    }

    fn add_subtarget(
        _node: *const Node,
        _dependent: *const Node,
        _err: &str,
        _dyndepwalk: BTreeSet<*mut Edge>,
    ) -> bool {
        todo!()
    }

    // Add edges that kWantToStart into the ready queue
    // Must be called after ComputeCriticalPath and before FindWork
    fn schedule_initial_edges() -> () {}

    /// Update plan with knowledge that the given node is up to date.
    /// If the node is a dyndep binding on any of its dependents, this
    /// loads dynamic dependencies from the node's path.
    /// Returns 'false' if loading dyndep info fails and 'true' otherwise.
    //   bool NodeFinished(Node* node, std::string* err);
    fn node_finished(_node: *mut Node) -> Result<bool,String> {
        todo!()
    }

    //   void EdgeWanted(const Edge* edge);
    fn edge_wanted(_edge: *const Edge) -> () {
        todo!()
    }
    /// want_e: std::map<Edge*, Want>::iterator
    fn edge_maybe_ready(_want_e: (),) -> Result<(),String> {
        todo!()
    }
    //   fn EdgeMaybeReady

    /// Submits a ready edge as a candidate for execution.
    /// The edge may be delayed from running, for example if it's a member of a
    /// currently-full pool.
    /// want_e: std::map<Edge*, Want>::iterator
    fn schedule_work(_want_e: ()) -> () {
        todo!()
    }
}

pub trait CommandRunner {
    // type RESULT = CommandRunnerResult;
    // fn CommandRunner_ctor(config: &BuildConfig,jobserver: Option<&JobserverClient>) -> Self where Self:Sized;
    /// the destructor function
    fn command_runner_ctor(&self) {}
    fn can_run_more(&self) -> c_ulonglong;
    fn start_command(&self, _edge: *mut Edge);
    fn wait_for_command(&mut self) -> Option<CommandRunnerResult>;
    fn get_active_edges(&self) -> Vec<*mut Edge> {
        Vec::new()
    }
    fn abort(&mut self) {}
}
fn command_runner_factory(
        _config: &BuildConfig,
        _jobserver: Option<&JobserverClient>,
    ) -> Box<dyn CommandRunner>
    // where
        // Self: Sized,
    {
        // if config.use_iocp {
        //     Box::new(IocpRunner::new(config, jobserver))
        // } else {
        //     Box::new(RealCommandRunner::new(config, jobserver))
        // }
        todo!()
    }
/// The result of waiting for a command.
#[repr(C)]
pub struct CommandRunnerResult {
    edge: *mut Edge,
    // Edge* edge = nullptr;
    status: ExitStatus,
    // ExitStatus status = ExitFailure;
    // std::string output;
    output: String,
    // bool success() const { return status == ExitSuccess; }
}
impl CommandRunnerResult {
    pub fn success(&self) -> bool {
        todo!();
        // return self.status == ExitSuccess
    }
}
#[repr(i32)] // keeps layout predictable like C++
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Verbosity {
    Quiet = 0,
    NoStatusUpdate = 1,
    Normal = 2,
    Verbose = 3,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct BuildConfig {
    pub verbosity: Verbosity,
    pub dry_run: bool,
    pub parallelism: i32,
    pub disable_jobserver_client: bool,
    pub failures_allowed: i32,
    pub max_load_average: f64,
    pub depfile_parser_options: DepfileParserOptions,
}
impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Normal,
            dry_run: false,
            parallelism: 1,
            disable_jobserver_client: false,
            failures_allowed: 1,
            max_load_average: -0.0, // yes, keep this exact if you want parity
            depfile_parser_options: (),
        }
    }
}
use std::collections::BTreeMap;

type RunningEdgeMap = BTreeMap<*const Edge, i32>;


#[repr(C)]
pub struct Builder<'a> {
    pub state: *mut State, // keep pointer for now (port phase)

    pub config: &'a BuildConfig,

    pub plan: Plan,

    pub jobserver: Option<Box<JobserverClient>>,

    pub command_runner: Box<dyn CommandRunner>,

    pub status: *mut Status,

    // map<const Edge*, int>
    running_edges: BTreeMap<*const Edge, i32>,

    start_time_millis: i64,

    lock_file_path: String,

    disk_interface: *mut DiskInterface,

    explanations: Option<Box<Explanations>>,

    scan: DependencyScan,

    exit_code: ExitStatus,
}
impl<'a> Builder<'a> {
    pub fn new(
        state: *mut State,
        config: &'a BuildConfig,
        _build_log: *mut BuildLog,
        _deps_log: *mut DepsLog,
        disk_interface: *mut DiskInterface,
        status: *mut Status,
        start_time_millis: i64,
    ) -> Self {
        Self {
            state,
            config,
            plan: Plan::plan(),
            jobserver: None,
            command_runner: command_runner_factory(config, None),
            status,
            running_edges: BTreeMap::new(),
            start_time_millis,
            lock_file_path: String::new(),
            disk_interface,
            explanations: None,
            scan:(),
            // scan: DependencyScan::new(build_log, deps_log),
            // exit_code: ExitStatus::Success,
            exit_code:()
        }
    }
    pub fn set_jobserver_client(&mut self, client: Box<JobserverClient>) {
    self.jobserver = Some(client);
}
pub fn add_target_by_name(
    &mut self,
    _name: &str,
) -> Result<*mut Node, String> {
    // return Err(err) instead of string*
    todo!()
}

pub fn add_target(
    &mut self,
    _target: *mut Node,
) -> Result<(), String> {
    todo!()
}
pub fn already_up_to_date(&self) -> bool {
    // port logic
    false
}
pub fn build(&mut self) -> Result<ExitStatus, String> {
    // instead of (ExitStatus + err*)
    todo!()
}
pub fn start_edge(
    &mut self,
    _edge: *mut Edge,
) -> Result<(), String> {
   todo!()
}
pub fn finish_command(
    &mut self,
    _result: &mut CommandRunnerResult,
) -> Result<(), String> {
    unimplemented!()
}
pub fn get_exit_code(&self) -> ExitStatus {
    self.exit_code
}
fn extract_deps(
    &mut self,
    _result: &CommandRunnerResult,
    _deps_type: &str,
    _deps_prefix: &str,
    _deps_nodes: &mut Vec<*mut Node>,
) -> Result<(), String> {
    unimplemented!()
}
fn set_failure_code(&mut self, code: ExitStatus) {
    self.exit_code = code;
}
}

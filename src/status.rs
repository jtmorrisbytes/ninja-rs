use crate::build::{BuildConfig, Edge, Verbosity};

type ExitStatus = ();
pub trait Status: Send {
    fn edge_added_to_plan(&mut self, edge: &Edge);
    fn edge_removed_from_plan(&mut self, edge: &Edge);
    fn build_edge_started(&mut self, edge: &Edge, start_time_millis: i64);
    fn build_edge_finished(
        &mut self,
        edge: &mut Edge,
        start_ms: i64,
        end_ms: i64,
        exit_code: ExitStatus,
        output: &str,
    );
    fn build_started(&mut self);
    fn build_finished(&mut self);
    // fn set_explanations(&mut self, explanations: Option<Box<dyn Explanations>>);
    fn new_line(&mut self);
    fn info(&mut self, msg: &str);
    fn warning(&mut self, msg: &str);
    fn error(&mut self, msg: &str);
}

pub struct StatusFactory;

impl StatusFactory {
    pub fn create(config: &BuildConfig) -> Box<dyn Status> {
        if config.verbosity == Verbosity::Quiet {
            return Box::new(NullStatus::new());
        }

        // If stdout isn't a TTY (like in a CI pipe), return a Simple Status
        if !atty::is(atty::Stream::Stdout) {
            return Box::new(PlainLineStatus::new());
        }

        // The "Beast Mode" choice: Crossterm/Indicatif enabled status
        Box::new(InteractiveStatus::new(config))
    }
}

// equivelent to the regular statusprinter
struct InteractiveStatus;
impl InteractiveStatus {
    pub fn new(_config: &BuildConfig)->Self{Self}
}
#[allow(unused_variables)]
impl Status for InteractiveStatus {
    fn build_edge_finished(
            &mut self,
            _edge: &mut Edge,
            _start_ms: i64,
            _end_ms: i64,
            _exit_code: ExitStatus,
            _output: &str,
        ) {
        
    }
    fn build_edge_started(&mut self, edge: &Edge, start_time_millis: i64) {
        
    }
    fn edge_added_to_plan(&mut self, edge: &Edge) {
        
    }
    fn error(&mut self, msg: &str) {
        tracing::error!("{}",msg);
    }
    fn warning(&mut self, msg: &str) {
        tracing::warn!("{}",msg);
        
    }
    fn info(&mut self, msg: &str) {
        tracing::info!("{}",msg);
        
    }
    fn new_line(&mut self) {
        
    }
    fn build_finished(&mut self) {
        
    }
    fn build_started(&mut self) {
        
    }
    fn edge_removed_from_plan(&mut self, edge: &Edge) {
        
    }
}



// equivelent to the regular statusprinter
struct PlainLineStatus;
impl PlainLineStatus {
    pub fn new()->Self{Self}
}
#[allow(unused_variables)]
impl Status for PlainLineStatus {
    fn build_edge_finished(
            &mut self,
            _edge: &mut Edge,
            _start_ms: i64,
            _end_ms: i64,
            _exit_code: ExitStatus,
            _output: &str,
        ) {
        
    }
    fn build_edge_started(&mut self, edge: &Edge, start_time_millis: i64) {
        
    }
    fn edge_added_to_plan(&mut self, edge: &Edge) {
        
    }
    fn error(&mut self, msg: &str) {
        tracing::error!("{}",msg);
    }
    fn warning(&mut self, msg: &str) {
        
    }
    fn info(&mut self, msg: &str) {
        
    }
    fn new_line(&mut self) {
        
    }
    fn build_finished(&mut self) {
        
    }
    fn build_started(&mut self) {
        
    }
    fn edge_removed_from_plan(&mut self, edge: &Edge) {
        
    }
}



struct NullStatus;
impl NullStatus {
    pub fn new()->Self{Self}
}
#[allow(unused_variables)]
impl Status for NullStatus {
    fn build_edge_finished(
            &mut self,
            _edge: &mut Edge,
            _start_ms: i64,
            _end_ms: i64,
            _exit_code: ExitStatus,
            _output: &str,
        ) {
        
    }
    fn build_edge_started(&mut self, edge: &Edge, start_time_millis: i64) {
        
    }
    fn edge_added_to_plan(&mut self, edge: &Edge) {
        
    }
    fn error(&mut self, msg: &str) {
        
    }
    fn warning(&mut self, msg: &str) {
        
    }
    fn info(&mut self, msg: &str) {
        
    }
    fn new_line(&mut self) {
        
    }
    fn build_finished(&mut self) {
        
    }
    fn build_started(&mut self) {
        
    }
    fn edge_removed_from_plan(&mut self, edge: &Edge) {
        
    }
}
// contains defs from ninja.cc

use std::{
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
};

use clap::ArgAction;

// use crate::build::Verbosity;

/// Command-line options.
#[repr(C)]
#[derive(Default)]
pub struct Options {
    /// Build file to load.
    pub input_file: String,

    /// Directory to change into before running.
    pub working_dir: String,

    /// Tool to run rather than building.
    pub tool: Option<()>,
    /// Whether phony cycles should warn or print an error.
    pub phony_cycle_should_err: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
pub enum NinjaDebugModes {
    ///stats
    Stats,
    Explain,
    Keepdepfile,
    Keeprsp,
    #[cfg(target_os = "windows")]
    NostatCache,
    List,
}
impl NinjaDebugModes {
    // returns a formatted string for the list action
    pub fn list() -> String {
        let mut s = format!(
            "\r\ndebugging modes:\r\n  \
    stats        print operation counts/timing info\r\n  \
    explain      explain what caused a command to execute\r\n  \
    keepdepfile  don't delete depfiles after they're read by ninja\r\n  \
    keeprsp      don't delete @response files on success\r\n"
        );
        #[cfg(windows)]
        {
            s = format!(
                "{s}  nostatcache  don't batch stat() calls per directory and cache them\r\n"
            );
        }
        format!("{s}multiple modes can be enabled via -dFOO and -d BAR")
    }
    // decided not to implement this for now
    // pub fn spellcheck(s:&str)-> String {
    //   format!("")
    // }
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "stats" => Ok(Self::Stats),
            "explain" => Ok(Self::Explain),
            "keepdepfile" => Ok(Self::Keepdepfile),
            "keeprsp" => Ok(Self::Keeprsp),
            #[cfg(windows)]
            "nostatcache" => Ok(Self::NostatCache),
            "list" => Ok(Self::List),
            _ => return Err(Self::list()),
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Parallelism(usize);
impl Parallelism {
    const DEFAULT_UNLIMITED_JOBS: usize = u8::MAX as usize;
    const DEFAULT_JOBS_COUNT: usize = 3;
    pub fn parse(s: &str) -> Result<Self, String> {
        let mut n: usize = s.parse().map_err(|e: ParseIntError| e.to_string())?;
        if n == 0 {
            tracing::warn!(
                "0 was specified for paralellism. \r\n\
                Ninja will guess and fall back to {}. Please specify a level > 0 if more or less paralellism is desired",
                Self::DEFAULT_UNLIMITED_JOBS
            );
            n = std::thread::available_parallelism()
                .map(|n| n.get().saturating_add(2))
                .unwrap_or(Self::DEFAULT_UNLIMITED_JOBS)
        }
        Ok(Self(n))
    }
}
impl Default for Parallelism {
    fn default() -> Self {
        let count = std::thread::available_parallelism()
            .map(|n| n.get() + 2)
            .unwrap_or(Self::DEFAULT_JOBS_COUNT);
        Self(count)
    }
}
impl std::fmt::Display for Parallelism {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Clone, Debug)]
pub struct AllowedFailures(usize);
impl AllowedFailures {
    pub const INFINITY: usize = usize::MAX;
    pub const DEFAULT_VALUE: usize = 1;
    pub fn parse(s: &str) -> Result<Self, String> {
        let n: i128 = s
            .parse()
            .map_err(|e: ParseIntError| format!("Invalid parameter for -k {s}:{e}"))?;
        if n <= 0 {
            tracing::info!("ninja: args: -k n <= 0, using {}", Self::INFINITY);
            Ok(Self(Self::INFINITY))
        } else if n > (Self::INFINITY as i128) {
            tracing::info!("ninja: args: -k n > {n}, using {}", Self::INFINITY);
            Ok(Self(Self::INFINITY))
        } else {
            tracing::info!("ninja: args: -k {n}");
            Ok(Self(n as usize))
        }
    }
}
impl Default for AllowedFailures {
    fn default() -> Self {
        tracing::info!(
            "ninja: args: -k not specified, using {}",
            Self::DEFAULT_VALUE
        );
        Self(Self::DEFAULT_VALUE)
    }
}
impl std::fmt::Display for AllowedFailures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[derive(Clone, Debug)]
pub struct LoadAverage(f64);
impl LoadAverage {
    pub const DEFAULT_VALUE: f64 = f64::INFINITY;
    pub fn parse(s: &str) -> Result<Self, String> {
        let n = s.parse().map_err(|e: ParseFloatError| {
            format!("-l parameter not numeric or out of bounds, did you mean -l 0.0 :{e}")
        })?;
        Ok(Self(n))
    }
}

impl std::fmt::Display for LoadAverage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Default for LoadAverage {
    fn default() -> Self {
        Self(Self::DEFAULT_VALUE)
    }
}
// #[derive(Debug, Clone)]
// pub struct Tool;

pub enum DebugMode {
  
}


#[derive(Debug, Clone)]
pub struct WorkDirectory(PathBuf);
impl WorkDirectory {
    pub fn parse(s: &str) -> Result<Self, String> {
        let p = match std::fs::canonicalize(s) {
            Ok(p) => Self(p),
            Err(e) => {
                tracing::warn!("ninja: Rust failed to canonicalize path {s}: {e}");
                #[cfg(target_os = "windows")]
                {
                    let p = unsafe { crate::fs::rs_absolute_path_win32_ntdll(s) };
                    Self(p)
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err(format!(
                        "ninja: Fatal: Failed to get the working directory for path {p}. Because:{e}"
                    ));
                }
            }
        };
        let r = std::env::set_current_dir(&p.0);
        if r.is_err() {
            let e = r.unwrap_err();
            tracing::debug!(
                "ninja: Rust failed to set current dir using std for path, trying fallbacks {}: {e}",
                p.0.display()
            );
            #[cfg(target_os = "windows")]
            {
                let r =
                    unsafe { crate::fs::rs_chdir_ntdll_longpath(p.0.to_str().unwrap_unchecked()) };
                if r != 0 {
                    return Err(format!(
                        "ninja: Fatal: Failed to chdir to path {} and no further backups are available.",
                        p.0.display()
                    ));
                }
            }
        }
        Ok(p)
    }
}
impl std::fmt::Display for WorkDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl Default for WorkDirectory {
    fn default() -> Self {
        match std::env::current_dir() {
            Ok(d) => Self(d),
            Err(e) => {
                tracing::debug!("ninja: Rust std failed to get the current directory: {e}");
                #[cfg(target_os = "windows")]
                {
                    Self(std::path::Path::new(&crate::fs::rs_getcwd_ntdll()).to_path_buf())
                }
                #[cfg(not(target_os = "windows"))]
                {
                    panic!(
                        "ninja: FATAL: Could not determine current working directory. Please pass -C"
                    );
                }
            }
        }
    }
}

// #[derive(Debug,Clone,Default)]
// pub enum Verbosity {
//     /// --quiet: Don't print the [1/1000] status line

//     Quiet,
//     /// Default: Print the [1/1000] status line
//     #[default]
//     Info,
//     /// -v, --verbose: Show all command lines while building
//     Verbose,
// }

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum NinjaTool {
    /// browse dependency graph in a web browser
    Browse,
    /// build helper for MSVC cl.exe (DEPRECATED)
    #[cfg(windows)]
    #[value(name("msvc"))]
    Msvc,

    /// clean built files
    Clean,
    /// list all commands required to rebuild given targets
    Commands,
    /// list all inputs required to rebuild given targets
    Inputs,
    /// print one or more sets of inputs required to build targets
    #[value(name("multi-inputs"))]
    
    MultiInputs,
    /// show dependencies stored in the deps log
    Deps,
    /// check deps log dependencies on generated files
    MissingDeps,
    /// output graphviz dot file for targets
    Graph,
    /// show inputs/outputs for a path
    Query,
    /// list targets by their rule or depth in the DAG
    Targets,
    /// dump JSON compilation database to stdout
    Compdb,
    /// dump JSON compilation database for a given list of targets to stdout
    #[value(name("compdb-targets"))]
    CompdbTargets,
    /// recompacts ninja-internal data structures
    Recompact,
    /// restats all outputs in the build log
    Restat,
    /// list all rules
    Rules,
    /// clean built files that are no longer produced by the manifest
    CleanDead,
    // (no description in original)
    Urtle,

    /// print the Windows code page used by ninja
    #[cfg(windows)]
    WinCodePage,
}

// I decided to use clap here, and keep some of the feel of getopt_long
#[derive(clap::Parser, Debug)]
#[command(
    name = "ninja",
    version,
    about = "A small build system with a focus on speed."
)]
pub struct NinjaCLIArgs {
    /// Sets the working directory
    #[arg(short='C',value_parser=WorkDirectory::parse,default_value_t)]
    working_dir: WorkDirectory,
    #[arg(short = 'd', value_name = "DEBUG_MODE", action=ArgAction::Append,num_args = 0..=1,default_missing_value = "list",value_parser=NinjaDebugModes::parse,hide_possible_values=false)]
    debug_mode: Vec<NinjaDebugModes>,
    #[arg(short = 'f', default_value = "build.ninja")]
    input_file: PathBuf,
    /// -jN -j N jobs. The amount of parallel jobs ninja will attempt to run at once. default vaule
    /// is std::thread::available_parallelism + 2 or 3 if it fails, otherwise if N ==0, will guess or u8::MAX (255)
    /// stored as a u32 on 32 bit platforms and u64 on 64 bit platforms
    #[arg(short='j',value_name = "NUM_JOBS", value_parser=Parallelism::parse,default_value_t)]
    paralellism: Parallelism,
    /// -k keep going until: the number of errors that can occur before ninja stops accepts i128 but stored as usize
    #[arg(short='k',value_name = "i128",value_parser=AllowedFailures::parse,default_value_t)]
    allowed_failures: AllowedFailures,
    /// max load average
    #[arg(short='l',value_parser=LoadAverage::parse,default_value_t)]
    max_load_average: LoadAverage,
    /// no run (or dry run)
    #[arg(short = 'n', value_name="bool",default_value_t = false)]
    no_run: bool,
    #[arg(short = 't')]
    // TODO: Tool fn pointers?
    tool: Option<NinjaTool>,
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    verbose_build: bool,
    // #[arg(long="version",default_value_t=false)]
    // print_version:bool,
    #[arg(long = "quiet", default_value_t = false)]
    quiet_build: bool,
    // todo: w is a subcommand or subvalue
    #[arg(short = 'w', default_value_t = false)]
    build_enable_warnings: bool,
}


// status

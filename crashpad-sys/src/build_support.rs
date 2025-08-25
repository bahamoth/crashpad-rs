/// Build support functions that can be used by both build.rs and xtask
/// This module exposes the depot_build functionality as a library

pub mod depot_build {
    pub use crate::build::depot_build::{
        build_crashpad_with_depot, 
        CrashpadBuildOutput
    };
}

pub mod tools {
    pub use crate::build::tools::{
        depot_cmd,
        ensure_depot_tools,
        setup_depot_tools_env,
    };
}
/// `format` submodule.
pub mod format;
/// `procinfo` submodule.
pub mod procinfo;
/// `provenance` submodule — code-identity (Publishers) attribution.
pub mod provenance;
/// `resolver` submodule.
pub mod resolver;

pub use procinfo::lookup_process;

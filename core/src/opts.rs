use std::path::PathBuf;

/// a TUI that lets you manage your github branches.
#[derive(clap::Parser, Clone, Debug)]
pub struct Opts {
    pub dir: Option<PathBuf>,
}

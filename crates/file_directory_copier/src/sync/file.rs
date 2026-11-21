use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct FileCopier;

impl FileCopier {
    pub fn new() -> Self {
        Self
    }

    pub fn copy<P: AsRef<Path>>(&self, from: P, to: P) -> Result<()> {
        todo!()
    }

    pub fn move_file<P: AsRef<Path>>(&self, from: P, to: P) -> Result<()> {
        todo!()
    }
}

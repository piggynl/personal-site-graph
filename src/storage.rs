use std::fs;
use std::path::Path;

use anyhow::Result;
use spdlog::prelude::*;

pub fn read(path: &Path) -> Result<String> {
    let buf = fs::read(path)?;
    let data = String::from_utf8(buf)?;
    trace!(
        "storage: read {} len={}",
        path.to_string_lossy(),
        data.len()
    );
    Ok(data)
}

pub fn write(path: &Path, data: &String) -> Result<()> {
    trace!(
        "storage: write {} len={}",
        path.to_string_lossy(),
        data.len()
    );
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, data)?;
    Ok(())
}

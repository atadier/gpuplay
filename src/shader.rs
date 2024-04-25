use std::{fs, io, path::Path, string::FromUtf8Error};

use thiserror::Error;
use wgpu::naga::{
    front::wgsl::{self, ParseError},
    Module,
};

pub fn load_shader<P: AsRef<Path>>(path: P) -> Result<Module, LoadShaderError> {
    let shader_bytes = fs::read(path).map_err(LoadShaderError::Io)?;
    let shader_content = String::from_utf8(shader_bytes).map_err(LoadShaderError::Encoding)?;
    let shader_prelude = include_str!("shaders/prelude.wgsl");
    wgsl::parse_str(&format!("{shader_prelude}{shader_content}")).map_err(LoadShaderError::Parse)
}

#[derive(Error, Debug)]
pub enum LoadShaderError {
    #[error(transparent)]
    Io(io::Error),
    #[error(transparent)]
    Encoding(FromUtf8Error),
    #[error(transparent)]
    Parse(ParseError),
}

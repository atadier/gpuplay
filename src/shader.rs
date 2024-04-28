use std::{ffi::OsStr, fs, io, path::Path, string::FromUtf8Error};

use thiserror::Error;
use wgpu::naga::{
    front::{glsl, wgsl},
    Module, ShaderStage,
};

pub fn load_shader<P: AsRef<Path>>(path: P) -> Result<Module, LoadShaderError> {
    let shader_extension = path
        .as_ref()
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or_default();

    let shader_bytes = fs::read(&path).map_err(LoadShaderError::Io)?;
    let shader_content = String::from_utf8(shader_bytes).map_err(LoadShaderError::Encoding)?;
    let shader_prelude = match shader_extension {
        "wgsl" => include_str!("shaders/prelude.wgsl"),
        "frag" => include_str!("shaders/prelude.frag"),
        _ => panic!("unable to identify shader language by file extension"),
    };
    let shader_source = format!("{shader_prelude}{shader_content}");

    match shader_extension {
        "wgsl" => wgsl::parse_str(&shader_source).map_err(LoadShaderError::ParseWgsl),
        "frag" => {
            let options = glsl::Options::from(ShaderStage::Fragment);
            glsl::Frontend::default()
                .parse(&options, &shader_source)
                .map_err(|errors| LoadShaderError::ParseGlsl(errors.into_iter().next().unwrap()))
        }
        _ => unreachable!(),
    }
}

#[derive(Error, Debug)]
pub enum LoadShaderError {
    #[error(transparent)]
    Io(io::Error),
    #[error(transparent)]
    Encoding(FromUtf8Error),
    #[error(transparent)]
    ParseWgsl(wgsl::ParseError),
    #[error(transparent)]
    ParseGlsl(glsl::Error),
}

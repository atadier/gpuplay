use encase::ShaderType;
use mint::{Vector3, Vector4};

#[derive(Debug, Clone, ShaderType)]
pub struct BufferUniforms {
    pub time: f32,
    pub resolution: Vector3<f32>,
    pub mouse: Vector4<f32>,
    pub frame: i32,
    pub delta_time: f32,
}

impl Default for BufferUniforms {
    fn default() -> Self {
        Self {
            time: 0.,
            resolution: Vector3 {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            mouse: Vector4 {
                x: 0.,
                y: 0.,
                z: 0.,
                w: 0.,
            },
            frame: 0,
            delta_time: 0.,
        }
    }
}

impl BufferUniforms {
    pub fn as_bytes(&self) -> encase::internal::Result<Vec<u8>> {
        let mut buffer = encase::StorageBuffer::new(Vec::new());
        buffer.write(self)?;
        Ok(buffer.into_inner())
    }
}

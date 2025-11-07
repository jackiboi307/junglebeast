use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexWrapper {
    pub position: Vec3,
    pub uv: Vec2,
    pub color: [u8; 4],
    pub normal: Vec4,
}

impl From<&VertexWrapper> for Vertex {
    fn from(v: &VertexWrapper) -> Vertex {
        Vertex {
            position: v.position,
            uv: v.uv,
            color: v.color,
            normal: v.normal,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageWrapper {
    pub bytes: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeshWrapper {
    pub vertices: Vec<VertexWrapper>,
    pub indices: Vec<u16>,
    pub texture: Option<ImageWrapper>,
}

impl MeshWrapper {
    pub fn to_mesh(&self) -> Mesh {
        use crate::miniquad::TextureWrap;

        let texture = if let Some(texture) = &self.texture {
            Some(Texture2D::from_rgba8(
                texture.width,
                texture.height,
                &texture.bytes
            ))
        } else {
            None
        };

        let mesh = Mesh {
            vertices: self.vertices.iter()
                .map(|v| v.into()).collect(),
            indices: (*self.indices).to_vec(),
            texture,
        };

        mesh
    }
}

use gltf::Gltf;
use sdl3::pixels::FColor;
use sdl3::render::FPoint;
use std::error::Error;

// model viewed from a fixed camera, drawn by the gpu each frame
const FOV_Y: f32 = 35.0;
const FILL: f32 = 0.72;
const AMBIENT: f32 = 0.30;
const DIFFUSE: f32 = 0.85;
const SPECULAR: f32 = 0.35;
const TILT: f32 = 10.0;

#[derive(Clone, Copy, Default)]
struct V3 {
    x: f32,
    y: f32,
    z: f32,
}

impl V3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn add(self, o: Self) -> Self {
        Self::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    fn sub(self, o: Self) -> Self {
        Self::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    fn mul(self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    fn normalized(self) -> Self {
        self.mul(1.0 / self.length().max(1e-6))
    }

    fn min(self, o: Self) -> Self {
        Self::new(self.x.min(o.x), self.y.min(o.y), self.z.min(o.z))
    }

    fn max(self, o: Self) -> Self {
        Self::new(self.x.max(o.x), self.y.max(o.y), self.z.max(o.z))
    }
}

// column-major matrix, same layout as gltf
#[derive(Clone, Copy)]
struct M4([f32; 16]);

impl M4 {
    fn from_gltf(m: [[f32; 4]; 4]) -> Self {
        let mut out = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                out[col * 4 + row] = m[col][row];
            }
        }
        Self(out)
    }

    fn mul(self, o: Self) -> Self {
        let mut m = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.0[k * 4 + row] * o.0[col * 4 + k];
                }
                m[col * 4 + row] = sum;
            }
        }
        Self(m)
    }

    fn point(self, p: V3) -> V3 {
        let m = self.0;
        V3::new(
            m[0] * p.x + m[4] * p.y + m[8] * p.z + m[12],
            m[1] * p.x + m[5] * p.y + m[9] * p.z + m[13],
            m[2] * p.x + m[6] * p.y + m[10] * p.z + m[14],
        )
    }

    fn dir(self, d: V3) -> V3 {
        let m = self.0;
        V3::new(
            m[0] * d.x + m[4] * d.y + m[8] * d.z,
            m[1] * d.x + m[5] * d.y + m[9] * d.z,
            m[2] * d.x + m[6] * d.y + m[10] * d.z,
        )
    }

    fn rot_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    fn rot_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self([
            c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }
}

#[derive(Clone, Copy, Default)]
struct Vertex {
    pos: V3,
    normal: V3,
    uv: [f32; 2],
}

#[derive(Clone, Copy)]
struct Tri {
    v: [Vertex; 3],
    base: V3,
}

// decoded base color texture
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct Model {
    tris: Vec<Tri>,
    pub texture: Option<Image>,
}

impl Model {
    // project the model into screen-space triangles, sorted back to front
    pub fn vertices(&self, width: u32, height: u32) -> Vec<sdl3::render::Vertex> {
        let mut tris = self.tris.clone();

        let mut lo = V3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut hi = V3::new(f32::MIN, f32::MIN, f32::MIN);
        for t in &tris {
            for v in &t.v {
                lo = lo.min(v.pos);
                hi = hi.max(v.pos);
            }
        }
        let center = lo.add(hi).mul(0.5);

        // lay the model across the screen and tilt it slightly
        let rot = M4::rot_x(TILT.to_radians()).mul(M4::rot_y(90f32.to_radians()));
        lo = V3::new(f32::MAX, f32::MAX, f32::MAX);
        hi = V3::new(f32::MIN, f32::MIN, f32::MIN);
        for t in &mut tris {
            for v in &mut t.v {
                v.pos = rot.dir(v.pos.sub(center));
                v.normal = rot.dir(v.normal).normalized();
                lo = lo.min(v.pos);
                hi = hi.max(v.pos);
            }
        }
        let center = lo.add(hi).mul(0.5);
        let half = hi.sub(lo).mul(0.5);

        let aspect = width as f32 / height as f32;
        let tan_y = (FOV_Y.to_radians() * 0.5).tan();
        let tan_x = tan_y * aspect;
        let dist = (half.x / (tan_x * FILL))
            .max(half.y / (tan_y * FILL))
            + half.z
            + 1.0;

        let light = V3::new(-0.45, 0.8, 0.55).normalized();
        let half_vec = light.add(V3::new(0.0, 0.0, 1.0)).normalized();

        let mut tris: Vec<(f32, [Vertex; 3])> = tris
            .into_iter()
            .filter_map(|t| {
                let mut out = [Vertex::default(); 3];
                let mut depth = 0.0;
                for k in 0..3 {
                    let v = t.v[k];
                    let p = v.pos.sub(center);
                    let vz = p.z - dist;
                    if vz >= -0.05 {
                        return None;
                    }
                    let inv = 1.0 / -vz;
                    let sx = (p.x * inv / tan_x * 0.5 + 0.5) * width as f32;
                    let sy = (0.5 - p.y * inv / tan_y * 0.5) * height as f32;

                    let ndl = v.normal.dot(light).max(0.0);
                    let spec = v.normal.dot(half_vec).max(0.0).powf(28.0) * SPECULAR;
                    let lit = AMBIENT + ndl * DIFFUSE;

                    out[k] = Vertex {
                        pos: V3::new(sx, sy, 0.0),
                        normal: t.base.mul(lit).add(V3::new(spec, spec, spec)),
                        uv: v.uv,
                    };
                    depth += dist - p.z;
                }

                // drop backfaces so the painter's order stays clean
                let area = (out[1].pos.x - out[0].pos.x) * (out[2].pos.y - out[0].pos.y)
                    - (out[1].pos.y - out[0].pos.y) * (out[2].pos.x - out[0].pos.x);
                if area >= 0.0 {
                    return None;
                }

                Some((depth / 3.0, out))
            })
            .collect();

        tris.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut verts = Vec::with_capacity(tris.len() * 3);
        for (_, tri) in tris {
            for v in tri {
                verts.push(sdl3::render::Vertex {
                    position: FPoint::new(v.pos.x, v.pos.y),
                    color: FColor::RGBA(
                        v.normal.x.clamp(0.0, 1.0),
                        v.normal.y.clamp(0.0, 1.0),
                        v.normal.z.clamp(0.0, 1.0),
                        1.0,
                    ),
                    tex_coord: FPoint::new(v.uv[0], v.uv[1]),
                });
            }
        }
        verts
    }
}

// load the glb once, keeping triangles in model space
pub fn load(path: &str) -> Result<Model, Box<dyn Error>> {
    let gltf = Gltf::open(path)?;
    let document = gltf.document;
    let buffers = gltf::import_buffers(&document, None, gltf.blob)?;
    let mut tris = Vec::new();
    let mut texture = None;

    for node in document.nodes() {
        let Some(mesh) = node.mesh() else { continue };
        let transform = M4::from_gltf(node.transform().matrix());
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
            let Some(positions) = reader.read_positions().map(|p| p.collect::<Vec<_>>()) else {
                continue;
            };
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_default();
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|t| t.into_f32().collect())
                .unwrap_or_default();
            let indices: Vec<u32> = reader
                .read_indices()
                .map(|i| i.into_u32().collect())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());

            let pbr = primitive.material().pbr_metallic_roughness();
            let factor = pbr.base_color_factor();
            let base = V3::new(factor[0], factor[1], factor[2]);
            if texture.is_none()
                && let Some(info) = pbr.base_color_texture()
            {
                texture = load_image(&info.texture().source(), &buffers);
            }

            for chunk in indices.chunks(3) {
                if chunk.len() < 3 {
                    break;
                }
                let mut v = [Vertex::default(); 3];
                for (k, &idx) in chunk.iter().enumerate() {
                    let idx = idx as usize;
                    let p = positions.get(idx).copied().unwrap_or([0.0; 3]);
                    let n = normals.get(idx).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let uv = uvs.get(idx).copied().unwrap_or([0.0, 0.0]);
                    v[k] = Vertex {
                        pos: transform.point(V3::new(p[0], p[1], p[2])),
                        normal: transform.dir(V3::new(n[0], n[1], n[2])).normalized(),
                        uv,
                    };
                }
                tris.push(Tri { v, base });
            }
        }
    }

    if tris.is_empty() {
        return Err("model has no triangles".into());
    }
    Ok(Model { tris, texture })
}

// decode only the base color png we actually draw with
fn load_image(image: &gltf::Image<'_>, buffers: &[gltf::buffer::Data]) -> Option<Image> {
    let gltf::image::Source::View { view, .. } = image.source() else {
        return None;
    };
    let data = &buffers[view.buffer().index()].0[view.offset()..view.offset() + view.length()];
    let image = image::load_from_memory(data).ok()?.to_rgba8();
    Some(Image {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}

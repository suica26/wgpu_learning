use std::ops::Index;
use std::process::id;

use log::debug;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

pub const VERTICES: &[Vertex] = &[
    // Changed
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641] }, // E
];

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }

    pub fn create_sphere(div: u16, radius: f32) -> (Vec<Vertex>, Vec<u16>) {
        let pi = std::f32::consts::PI;

        let mut vertices = vec![];
        let mut indices = vec![];

        let div_f32 = div as f32;

        let y_unit = 2.0 * radius / div_f32;
        let y_angle_unit = pi / div_f32;
        let xz_angle_unit = 2.0 * pi / div_f32;

        let top = radius;

        // 上側の頂点
        vertices.push(Vertex {
            position: [0.0, top, 0.0],
            tex_coords: [0.0, 1.0],
        });

        // y軸方向のループ
        for i in 1..=(div - 1) {
            let i_f32 = i as f32;
            let cos_y = (y_angle_unit * i_f32).cos();

            let y = radius * cos_y;
            let xy_radius = radius * (y_angle_unit * i_f32).sin();

            // xz平面上の角度ループ
            for j in 0..div {
                let j_f32 = j as f32;
                let cos_xz = (xz_angle_unit * j_f32).cos();
                let sin_xz = (xz_angle_unit * j_f32).sin();

                let x = xy_radius * cos_xz;
                let z = xy_radius * sin_xz;

                vertices.push(Vertex {
                    position: [x, y, z],
                    tex_coords: [cos_xz * 0.5 + 0.5, sin_xz * 0.5 + 0.5],
                });
            }
        }

        // 下側の頂点
        vertices.push(Vertex {
            position: [0.0, -top, 0.0],
            tex_coords: [0.0, 0.0],
        });

        // 上側の三角形
        for i in 0..div {
            let next_i = if i == div - 1 { 1 } else { i + 1 };

            indices.push(0);
            indices.push(next_i + 1);
            indices.push(i + 1);
        }

        // 中間部分の三角形
        for i in 0..(div - 2) {
            let offset = 1 + i * div;

            for j in 0..div {
                let next_j = if j == div - 1 { 0 } else { j + 1 };
                let next_offset = offset + div;

                indices.push(offset + j);
                indices.push(next_offset + next_j);
                indices.push(next_offset + j);

                indices.push(offset + j);
                indices.push(offset + next_j);
                indices.push(next_offset + next_j);
            }
        }

        // 下側の三角形
        let last = (vertices.len() as u16) - 1;
        let offset = last - div;
        for i in 0..div {
            let next_i = if i == div - 1 { 0 } else { i + 1 };

            indices.push(offset + i);
            indices.push(offset + next_i);
            indices.push(last);
        }

        Self::log_create_sphere(&vertices, &indices, div);

        (vertices, indices)
    }

    fn log_create_sphere(vertices: &Vec<Vertex>, indices: &Vec<u16>, div: u16) {
        let mut output_str = String::new();
        for (i, v) in vertices.iter().enumerate() {
            if i > 0 {
                if (i as u16 - 1) % div == 0 {
                    output_str.push_str("\n");
                }
            }

            output_str.push_str(&format!("{:?} {:?}\n", i, v.position));
        }

        output_str.push_str("\n");

        for i in 0..indices.len() / 3 {
            if i as u16 % div == 0 {
                output_str.push_str("\n");
            }

            let idx = i * 3;
            output_str.push_str(&format!("{:?} {:?} {:?}\n", indices[idx], indices[idx + 1], indices[idx + 2]));
        }
        debug!("{}", output_str);

        {
            use std::io::prelude::*;
            use std::io::BufWriter;
            use std::fs::File;
            use std::path::Path;

            let path = Path::new("vertices_indices.txt");
            let mut file = match File::create(&path) {
                Ok(f) => f,
                Err(e) => panic!("File Create Error: {:?}", e),
            };

            debug!("File created: {:?}", file.metadata());

            match file.write(output_str.as_bytes()) {
                Ok(_) => debug!("Write success"),
                Err(e) => panic!("File Write Error: {:?}", e),
            };
        }
    }
}
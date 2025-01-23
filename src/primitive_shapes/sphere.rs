use log::{debug, error};

use crate::system::vertex::Vertex;

pub struct Sphere {
    pub div: u16,
    pub radius: f32,

    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Sphere {
    pub fn new(div: u16, radius: f32) -> Self {
        if let Err(e) = Self::validate(div, radius) {
            error!("{}", e);
        }

        let (vertices, indices) = if div % 2 != 0 {
            Self::create_by_indivisible(div, radius)
        } else if 180 % div == 0 {
            Self::create_by_divisible(div, radius)
        } else {
            Self::create_by_indivisible(div, radius)
        };

        Self {
            div,
            radius,
            vertices,
            indices,
        }
    }
}

impl Sphere {
    fn validate(div: u16, radius: f32) -> Result<(), String> {
        if div <= 2 {
            return Err(String::from("can't create sphere by div less than 2."));
        }
        if radius < 0.0 {
            return Err(String::from("can't create sphere by minus radius"));
        }

        return Ok(());
    }

    fn create_by_indivisible(div: u16, radius: f32) -> (Vec<Vertex>, Vec<u16>) {
        use std::f32::consts::*;

        let mut vertices = vec![];
        let mut indices = vec![];

        let div_f32 = div as f32;

        let y_unit_angle = PI / div_f32;
        let xz_unit_angle = TAU / div_f32;

        let top = radius;

        // 上側の頂点
        vertices.push(Vertex {
            position: [0.0, top, 0.0],
            tex_coords: [0.0, 1.0],
        });

        // y軸方向のループ
        for i in 1..=(div - 1) {
            let i_f32 = i as f32;
            let cos_y = (y_unit_angle * i_f32).cos();

            let y = radius * cos_y;
            let xz_radius = radius * (y_unit_angle * i_f32).sin();

            // xz平面上の角度ループ
            for j in 0..div {
                let j_f32 = j as f32;
                let cos_xz = (xz_unit_angle * j_f32).cos();
                let sin_xz = (xz_unit_angle * j_f32).sin();

                let x = xz_radius * cos_xz;
                let z = xz_radius * sin_xz;

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
        for i in 1..=div {
            let next_i = if i == div { 1 } else { i + 1 };

            indices.push(0);
            indices.push(next_i);
            indices.push(i);
        }

        // 中間部分の三角形
        for i in 0..(div - 2) {
            let offset = 1 + (i * div);

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
        for i in offset..last {
            let next_i = if i == last - 1 { offset } else { i + 1 };

            indices.push(i);
            indices.push(next_i);
            indices.push(last);
        }

        Self::log_create_sphere(&vertices, &indices, div);

        (vertices, indices)
    }

    fn create_by_divisible(div: u16, radius: f32) -> (Vec<Vertex>, Vec<u16>) {
        use std::f32::consts::*;

        let mut vertices = vec![];
        let mut indices = vec![];

        let div_f32 = div as f32;

        let y_unit_angle = PI / div_f32;
        let xz_unit_angle = TAU / div_f32;

        let top = radius;

        // 上側の頂点
        vertices.push(Vertex {
            position: [0.0, top, 0.0],
            tex_coords: [0.0, 1.0],
        });

        // y軸方向のループ
        for i in 1..=(div - 1) {
            let i_f32 = i as f32;
            let cos_y = (y_unit_angle * i_f32).cos();

            let y = radius * cos_y;
            let xz_radius = radius * (y_unit_angle * i_f32).sin();

            // xz平面上の角度ループ
            for j in 0..div {
                let j_f32 = j as f32;
                let cos_xz = (xz_unit_angle * j_f32).cos();
                let sin_xz = (xz_unit_angle * j_f32).sin();

                let x = xz_radius * cos_xz;
                let z = xz_radius * sin_xz;

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
        for i in 1..=div {
            let next_i = if i == div { 1 } else { i + 1 };

            indices.push(0);
            indices.push(next_i);
            indices.push(i);
        }

        // 中間部分の三角形
        for i in 0..(div - 2) {
            let offset = 1 + (i * div);

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
        for i in offset..last {
            let next_i = if i == last - 1 { offset } else { i + 1 };

            indices.push(i);
            indices.push(next_i);
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
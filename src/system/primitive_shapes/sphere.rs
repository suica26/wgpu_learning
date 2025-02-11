use log::{debug, error};

use crate::system::game_object::transform::Transform;
use crate::system::rendering::shape_geometry::ShapeGeometry;
use crate::system::rendering::vertex::Vertex;

pub struct Sphere {
    pub div: u16,
    pub transform: Transform,
    pub geometry: ShapeGeometry,
}

impl Sphere {
    pub fn new(div: u16, radius: f32) -> Self {
        if let Err(e) = Self::validate(div, radius) {
            error!("{}", e);
        }

        let geometry = Self::create_geometry(div);

        let mut transform = Transform::new();
        transform.set_scale(cgmath::Vector3::new(radius, radius, radius));

        Self {
            div,
            transform,
            geometry,
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

    fn create_geometry(div: u16) -> ShapeGeometry {
        let (vertices, indices) = if div % 2 == 0 {
            Self::create_by_divisible(div)
        } else {
            Self::create_by_indivisible(div)
        };

        // Self::log_create_sphere(&vertices, &indices, div);

        ShapeGeometry::from(vertices, indices)
    }

    /// divが奇数の場合の球体の生成
    fn create_by_indivisible(div: u16) -> (Vec<Vertex>, Vec<u16>) {
        use std::f32::consts::*;

        let mut vertices = vec![];

        let half_div = (div - 1) / 2;
        let div_f32 = div as f32;
        let half_div_f32 = half_div as f32;

        let y_unit_angle = TAU / div_f32;

        // 上側の頂点
        vertices.push(Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 1.0, 0.0],
        });

        // y軸方向のループ
        for i in 1..=half_div {
            let i_f32 = i as f32;
            let cos_y = (y_unit_angle * i_f32).cos();

            // 円周上の頂点を追加
            let circle_radius = (y_unit_angle * i_f32).sin();
            let circle_y = cos_y;
            let circle_vertices = Self::get_circle_vertices(
                div,
                circle_radius,
                circle_y,
            );
            vertices.extend(circle_vertices);
        }

        // 下側の頂点
        let bottom = (y_unit_angle * half_div_f32).cos();
        vertices.push(Vertex {
            position: [0.0, bottom, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, -1.0, 0.0],
        });

        let indices = Self::create_indices(div, vertices.len() as u16);

        (vertices, indices)
    }

    /// divが偶数の場合の球体の生成
    fn create_by_divisible(div: u16) -> (Vec<Vertex>, Vec<u16>) {
        use std::f32::consts::*;

        let mut vertices = vec![];

        let half_div = div / 2;
        let half_div_f32 = half_div as f32;
        let y_unit_angle = PI / half_div_f32;

        // 上側の頂点
        vertices.push(Vertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 1.0, 0.0],
        });

        // y軸方向のループ
        for i in 1..half_div {
            let i_f32 = i as f32;
            let cos_y = (y_unit_angle * i_f32).cos();

            // 円周上の頂点を追加
            let circle_radius = (y_unit_angle * i_f32).sin();
            let circle_y = cos_y;
            let circle_vertices = Self::get_circle_vertices(
                div,
                circle_radius,
                circle_y,
            );
            vertices.extend(circle_vertices);
        }

        // 下側の頂点
        vertices.push(Vertex {
            position: [0.0, -1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, -1.0, 0.0],
        });

        let indices = Self::create_indices(div, vertices.len() as u16);

        (vertices, indices)
    }

    fn get_circle_vertices(div: u16, radius: f32, y: f32) -> Vec<Vertex> {
        use std::f32::consts::*;

        let mut vertices = vec![];

        let div_f32 = div as f32;
        let xz_unit_angle = TAU / div_f32;

        for i in 0..div {
            let i_f32 = i as f32;
            let cos_xz = (xz_unit_angle * i_f32).cos();
            let sin_xz = (xz_unit_angle * i_f32).sin();

            let x = radius * cos_xz;
            let z = radius * sin_xz;

            let dist = (x * x + y * y + z * z).sqrt();

            vertices.push(Vertex {
                position: [x, y, z],
                tex_coords: [cos_xz * 0.5 + 0.5, sin_xz * 0.5 + 0.5],
                normal: [x / dist, y / dist, z / dist],
            });
        }

        vertices
    }

    fn create_indices(div: u16, vertices_num: u16) -> Vec<u16> {
        let mut indices = vec![];

        for i in 1..=div {
            let next_i = if i == div { 1 } else { i + 1 };

            indices.push(0);
            indices.push(next_i);
            indices.push(i);
        }

        let loop_num = if div % 2 == 0 {
            div / 2
        } else {
            (div + 1) / 2
        } - 2;

        for i in 0..loop_num {
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

        let last = vertices_num - 1;
        let offset = last - div;
        for i in offset..last {
            let next_i = if i == last - 1 { offset } else { i + 1 };

            indices.push(i);
            indices.push(next_i);
            indices.push(last);
        }

        indices
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
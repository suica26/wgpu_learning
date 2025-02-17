use log::error;

use crate::system::shape_geometry::ShapeGeometry;
use crate::system::vertex::ModelVertex;

pub fn create_sphere_geometry(div: u16) -> ShapeGeometry {
    if div <= 2 {
        error!("can't create sphere by div less than 2.");
    }

    let (vertices, indices) = if div % 2 == 0 {
        create_by_divisible(div)
    } else {
        create_by_indivisible(div)
    };

    ShapeGeometry::from(vertices, indices)
}

/// divが奇数の場合の球体の生成
fn create_by_indivisible(div: u16) -> (Vec<ModelVertex>, Vec<u16>) {
    use std::f32::consts::*;

    let mut vertices = vec![];

    let half_div = (div - 1) / 2;
    let div_f32 = div as f32;
    let half_div_f32 = half_div as f32;

    let y_unit_angle = TAU / div_f32;

    // 上側の頂点
    vertices.push(ModelVertex {
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
        let circle_vertices = get_circle_vertices(
            div,
            circle_radius,
            circle_y,
        );
        vertices.extend(circle_vertices);
    }

    // 下側の頂点
    let bottom = (y_unit_angle * half_div_f32).cos();
    vertices.push(ModelVertex {
        position: [0.0, bottom, 0.0],
        tex_coords: [0.0, 0.0],
        normal: [0.0, -1.0, 0.0],
    });

    let indices = create_indices(div, vertices.len() as u16);

    (vertices, indices)
}

/// divが偶数の場合の球体の生成
fn create_by_divisible(div: u16) -> (Vec<ModelVertex>, Vec<u16>) {
    use std::f32::consts::*;

    let mut vertices = vec![];

    let half_div = div / 2;
    let half_div_f32 = half_div as f32;
    let y_unit_angle = PI / half_div_f32;

    // 上側の頂点
    vertices.push(ModelVertex {
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
        let circle_vertices = get_circle_vertices(
            div,
            circle_radius,
            circle_y,
        );
        vertices.extend(circle_vertices);
    }

    // 下側の頂点
    vertices.push(ModelVertex {
        position: [0.0, -1.0, 0.0],
        tex_coords: [0.0, 0.0],
        normal: [0.0, -1.0, 0.0],
    });

    let indices = create_indices(div, vertices.len() as u16);

    (vertices, indices)
}

fn get_circle_vertices(div: u16, radius: f32, y: f32) -> Vec<ModelVertex> {
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

        vertices.push(ModelVertex {
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
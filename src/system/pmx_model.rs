use std::rc::Rc;

use log::info;
use wgpu::{self, util::DeviceExt};

use crate::system::{texture, vertex};

pub struct LoadedPMXModel {
    pub model_info: PMXUtil::types::ModelInfo,
    pub vertices: Vec<PMXUtil::types::Vertex>,
    pub faces: Vec<PMXUtil::types::Face>,
    pub textures: Vec<Rc<texture::Texture>>,
    pub materials: Vec<PMXUtil::types::Material>,
    pub bones: Vec<PMXUtil::types::Bone>,
    pub morphs: Vec<PMXUtil::types::Morph>,
    pub frames: Vec<PMXUtil::types::Frame>,
    pub rigids: Vec<PMXUtil::types::Rigid>,
    pub joints: Vec<PMXUtil::types::Joint>,
    pub soft_bodies: Vec<PMXUtil::types::SoftBody>,
}

pub struct PMXModelParts {
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: PMXUtil::types::Material,
    pub texture: Rc<texture::Texture>,
    pub texture_bind_group: wgpu::BindGroup,
}

pub struct PMXModel {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub parts: Vec<PMXModelParts>,
}

impl PMXModel {
    pub fn new(
        device: &wgpu::Device,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        loaded_model: LoadedPMXModel,
    ) -> Self {
        let name = loaded_model.model_info.name.clone();

        let vertices = loaded_model
            .vertices
            .iter()
            .map(|v| vertex::ModelVertex {
                position: v.position,
                tex_coords: v.uv,
                normal: v.norm,
                tangent: [0.0; 3],
                bitangent: [0.0; 3],
            })
            .collect::<Vec<_>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("{:?} PMX Vertex Buffer", loaded_model.model_info.name).as_str()),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices = loaded_model
            .faces
            .iter()
            .map(|x| x.vertices)
            .flat_map(|x| x)
            .map(|x| x as u16)
            .collect::<Vec<_>>();

        let parts_num = loaded_model.materials.len();

        let mut start_index: usize = 0;
        let mut parts = Vec::with_capacity(parts_num);

        // materialの情報を元にindex bufferを作成
        for i in 0..parts_num {
            let material = loaded_model.materials[i].clone();
            let texture = loaded_model.textures[material.texture_index as usize].clone();

            let end_index = start_index + material.num_face_vertices as usize;
            let indices_slice = &indices[start_index..end_index];

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("{:?} PMX Index Buffer", name).as_str()),
                contents: bytemuck::cast_slice(indices_slice),
                usage: wgpu::BufferUsages::INDEX,
            });

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: None,
            });

            start_index = end_index;

            parts.push(PMXModelParts {
                index_buffer,
                num_elements: material.num_face_vertices as u32,
                material,
                texture,
                texture_bind_group,
            });
        }

        Self {
            name,
            vertex_buffer,
            parts,
        }
    }
}

pub trait DrawPMXModel<'a> {}

impl<'a, 'b> DrawPMXModel<'b> for wgpu::RenderPass<'a> where 'b: 'a {}

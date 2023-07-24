use std::collections::HashMap;
use glium::{Display, VertexBuffer, Program, Frame, uniform, Surface, IndexBuffer};
use rcrefcell::RcCell;
use ultraviolet::Mat4;
use crate::{assets::{mesh_asset::MeshAsset, shader_asset::ShaderAsset, texture_asset::TextureAsset}, managers::{render::{Vertex, self}, debugger::error}, math_utils::deg_to_rad, object::Object};
use super::{component::Component, transform::BasicTransform};

pub struct Mesh {
    pub mesh_asset: MeshAsset,
    pub shader_asset: ShaderAsset,
    pub texture_asset: Option<TextureAsset>,
    texture: Option<glium::texture::Texture2d>,
    vertex_buffer: Vec<VertexBuffer<Vertex>>,
    program: Vec<Program>,
    owner: Option<RcCell<Object>>,
    started: bool,
    error: bool
}

impl Component for Mesh {
    fn opaque_render(&mut self, display: &Display, target: &mut Frame) {
        if self.error {
            return;
        }
        if !self.started {
            self.start_mesh(display);
        }

        for i in 0..self.mesh_asset.objects.len() {
            let object = &self.mesh_asset.objects[i];
            let indices = IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &object.indices);

            let model_matrix: Mat4;

            match self.owner.as_ref().unwrap().borrow().get_component("Transform") {
                Some(pos) => model_matrix = self.setup_mat(pos.get_data().unwrap()),
                None => {
                    error(
                        &format!(
                            "Mesh component can't access transform component!\nObject's name: {}",
                            self.owner.as_ref().unwrap().borrow().name)
                        );
                    return;
                }
            }

            let view_mat = render::get_view_matrix();
            let projection_mat = render::get_projection_matrix();

            let texture_option = self.texture.as_ref();

            let empty_texture = glium::texture::Texture2d::empty(display, 1, 1).unwrap();
            let texture: &glium::texture::Texture2d;
            match texture_option {
                Some(tx) => texture = tx,
                None => texture = &empty_texture,
            }
            
            let uniforms = uniform!
            {
                model: [
                    *model_matrix.cols[0].as_array(),
                    *model_matrix.cols[1].as_array(),
                    *model_matrix.cols[2].as_array(),
                    *model_matrix.cols[3].as_array(),
                ],
                view: [
                    *view_mat.cols[0].as_array(),
                    *view_mat.cols[1].as_array(),
                    *view_mat.cols[2].as_array(),
                    *view_mat.cols[3].as_array(),
                ],
                projection: [
                    *projection_mat.cols[0].as_array(),
                    *projection_mat.cols[1].as_array(),
                    *projection_mat.cols[2].as_array(),
                    *projection_mat.cols[3].as_array(),
                ],
                tex: texture,
            };

            let draw_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    .. Default::default()
                },
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                .. Default::default()
            };

            target.draw(
                &self.vertex_buffer[i],
                &indices.unwrap(),
                &self.program[i],
                &uniforms,
                &draw_params)
                .unwrap();
        }
    }

    fn get_component_type(&self) -> &str {
        "Mesh"
    }

    fn set_owner(&mut self, owner: RcCell<Object>) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> &Option<RcCell<Object>> {
        &self.owner
    }
}

impl Mesh {
    pub fn new(mesh_asset: MeshAsset, texture_asset: Option<TextureAsset>, shader_asset: ShaderAsset) -> Mesh {
        Mesh {
            mesh_asset,
            shader_asset,
            texture_asset,
            texture: None,
            vertex_buffer: vec![],
            program: vec![],
            owner: None,
            started: false,
            error: false
        }
    }

    fn setup_mat(&self, transform_data: HashMap<&str, String>) -> Mat4 {
        let transform_component = BasicTransform::from(transform_data);
        let rotation = transform_component.rotation;

        let mut transform = Mat4::identity(); 
        transform = transform * Mat4::from_euler_angles(
            deg_to_rad(rotation.x),
            deg_to_rad(rotation.y),
            deg_to_rad(rotation.z));
        transform = transform *
            Mat4::from_nonuniform_scale(transform_component.scale);
        transform = transform *
            Mat4::from_translation(transform_component.position);

        transform
    }

    fn start_mesh(&mut self, display: &Display) {
        for i in &self.mesh_asset.objects {
            let vertex_buffer = VertexBuffer::new(display, &i.vertices);
            match vertex_buffer {
                Ok(buff) => self.vertex_buffer.push(buff),
                Err(err) => {
                    error(&format!("Mesh component error:\nvertex buffer creation error!\nErr: {}", err));
                    self.error = true;
                    return;
                },
            }
        }

        let vertex_shader_source = &self.shader_asset.vertex_shader_source;
        let fragment_shader_source = &self.shader_asset.fragment_shader_source;

        for _i in &self.mesh_asset.objects {
            let program = Program::from_source(display, &vertex_shader_source, &fragment_shader_source, None);
            match program {
                Ok(prog) => self.program.push(prog),
                Err(err) => {
                    error(&format!("Mesh component error:\nprogram creation error!\nErr: {}", err));
                    self.error = true;
                    return;
                },
            }
        }

        if self.texture_asset.is_some() {
            let asset = self.texture_asset.as_ref().unwrap();
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&asset.image_raw, asset.image_dimensions);
            let texture = glium::texture::texture2d::Texture2d::new(display, image);

            match texture {
                Ok(tx) => self.texture = Some(tx),
                Err(err) => {
                    error(&format!("Mesh component error:\ntexture creating error!\nErr: {}", err));
                    self.texture = None;
                },
            }
        } 

        self.started = true;
    }
}


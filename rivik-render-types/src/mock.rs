//! A Mock renderer used for testing

use crate::{light::Light, mesh::DynMesh, Model};

#[derive(Default)]
pub struct Frame {
    meshes: Vec<(DynMesh, [[f32; 3]; 3])>,
    lights: Vec<Light>,
    camera: ([f32; 3], [f32; 3]),
    ui_set: bool,
}

impl crate::Frame for Frame {
    fn draw_mesh<M: crate::mesh::AsMesh>(
        &mut self,
        model: &Model<M>,
        pos: [f32; 3],
        rot: [f32; 3],
        scale: [f32; 3],
    ) {
        let mesh = DynMesh {
            verts: Vec::from(model.mesh.verts()),
            faces: Vec::from(model.mesh.faces()),
        };

        self.meshes.push((mesh, [pos, rot, scale]));
    }

    fn draw_light(&mut self, light: crate::light::Light) {
        self.lights.push(light);
    }

    fn set_camera(&mut self, pos: [f32; 3], rot: [f32; 3]) {
        self.camera = (pos, rot);
    }

    fn set_ui<'a>(
        &'a mut self,
        clip_prim: &'a [egui::ClippedPrimitive],
        textures: egui::TexturesDelta,
    ) {
        self.ui_set = true;
    }
}

impl crate::Renderer for () {
    type FRAME = Frame;

    fn frame(&mut self) -> Self::FRAME {
        Frame::default()
    }

    fn init<W>(_w: &W) -> Self
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        ()
    }

    fn upload_mesh<M: crate::mesh::AsMesh>(&mut self, label: Option<&str>, mesh: &M) {
        todo!()
    }

    fn upload_texture<T: crate::tex::AsTex>(&mut self, label: Option<&str>, tex: &T) {
        todo!()
    }
}

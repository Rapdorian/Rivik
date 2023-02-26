use std::borrow::Borrow;

use ultraviolet::Mat4;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages,
};

use crate::context::{device, queue};

/// A Handle around a transformation uniform buffer
///
/// TODO: This should probably also cache the current value to allow for easier manipulation methods
pub struct Transform {
    buffer: Buffer,
}

impl Transform {
    /// Create a new transform buffer
    pub fn new(proj: Mat4, view: Mat4, model: Mat4) -> Self {
        let device = device();

        let mut buffer = Vec::new();
        buffer.extend_from_slice((proj * view * model).as_byte_slice());
        buffer.extend_from_slice((view * model).as_byte_slice());
        buffer.extend_from_slice((view * model).inversed().transposed().as_byte_slice());

        // create buffer
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &buffer,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        Self { buffer }
    }

    /// Updates the contents of the transform buffer
    pub fn update(&self, proj: Mat4, view: Mat4, model: Mat4) {
        self.set_mvp(proj * view * model);
        self.set_mv(view * model);
        self.set_transform(view, model);
    }

    /// Set the model-view-projection matrix
    fn set_mvp(&self, proj: Mat4) {
        queue().write_buffer(&self.buffer, 0, proj.as_byte_slice());
    }

    /// Set the model-view matrix for this transform
    fn set_mv(&self, view: Mat4) {
        queue().write_buffer(&self.buffer, 64, view.as_byte_slice());
    }

    /// Set the model and normalized model matrix for this transform
    fn set_transform(&self, view: Mat4, model: Mat4) {
        queue().write_buffer(
            &self.buffer,
            128,
            (view * model).inversed().transposed().as_byte_slice(),
        );
    }

    /// Get the underlying buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl Borrow<Buffer> for Transform {
    fn borrow(&self) -> &Buffer {
        &self.buffer
    }
}

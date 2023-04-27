/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Utilities for working with a transform buffer
//!
//! See the [Transform] type

use std::{borrow::Borrow, num::NonZeroU64};

use mint::ColumnMatrix4;
use once_cell::sync::OnceCell;
use ultraviolet::Mat4;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages,
};

use crate::context::{device, queue};

/// A Handle around a transformation uniform buffer
///
/// TODO: This should probably also cache the current value to allow for easier manipulation methods
pub struct Transform {
    buffer: Buffer,
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Mat4::default(), Mat4::default(), Mat4::default())
    }
}

/// An object that has a transform buffer
pub trait Spatial {
    /// Fetch this object's transform buffer
    fn transform(&self) -> &Transform;
}

static TRANFORM_LAYOUT: OnceCell<BindGroupLayout> = OnceCell::new();

/// Layout of a transform buffer
pub fn layout() -> &'static BindGroupLayout {
    if let Some(layout) = TRANFORM_LAYOUT.get() {
        layout
    } else {
        // generate bind group layout
        let layout = device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZeroU64::new(64 * 3),
                },
                count: None,
            }],
        });
        TRANFORM_LAYOUT.try_insert(layout).unwrap()
    }
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
    pub fn update(
        &self,
        proj: impl Into<ColumnMatrix4<f32>>,
        view: impl Into<ColumnMatrix4<f32>>,
        model: impl Into<ColumnMatrix4<f32>>,
    ) {
        let proj = Mat4::from(proj.into());
        let view = Mat4::from(view.into());
        let model = Mat4::from(model.into());
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

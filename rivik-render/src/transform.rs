/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Utilities for working with a transform buffer
//!
//! See the [Transform] type

use std::{
    borrow::Borrow,
    num::NonZeroU64,
    sync::{Arc, RwLock},
};

use glam::Mat4;
use mint::ColumnMatrix4;
use once_cell::sync::{Lazy, OnceCell};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages,
};

use crate::{
    context::{device, queue},
    jobs::frustum_cull::{Frustum, AABB},
};

/// A Handle around a transformation uniform buffer
///
/// TODO: This should probably also cache the current value to allow for easier manipulation methods
pub struct Transform {
    buffer: Buffer,
    model: Arc<RwLock<Mat4>>,
    view: Arc<RwLock<Mat4>>,
    proj: Arc<RwLock<Mat4>>,
}

/// All transforms are going to share a camera
static VIEW: Lazy<Arc<RwLock<Mat4>>> = Lazy::new(|| Arc::new(RwLock::new(Mat4::default())));
static PROJ: Lazy<Arc<RwLock<Mat4>>> = Lazy::new(|| Arc::new(RwLock::new(Mat4::default())));

impl Default for Transform {
    fn default() -> Self {
        Self::new(Mat4::default(), Mat4::default(), Mat4::default())
    }
}

/// An object that has a transform buffer
pub trait Spatial {
    /// Fetch this object's transform buffer
    fn transform(&self) -> &Transform;

    /// Fetch this object's bounding box
    fn bound(&self) -> AABB;
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

        let buffer = [
            proj * view * model,
            view * model,
            (view * model).inverse().transpose(),
        ];

        // create buffer
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&buffer),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            model: Arc::new(RwLock::new(model)),
            view: Arc::clone(&VIEW),
            proj: Arc::clone(&PROJ),
        }
    }

    /// Create a bind group for this transform
    pub fn binding(&self) -> wgpu::BindGroup {
        device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: layout(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer().as_entire_binding(),
            }],
        })
    }

    /// Updates the contents of the transform buffer
    pub fn update(&self, model: impl Into<ColumnMatrix4<f32>>) {
        let model = Mat4::from(model.into());
        *self.model.write().unwrap() = model;
        let view = *self.view.read().unwrap();
        let proj = *self.proj.read().unwrap();
        self.set_mvp(proj * view * model);
        self.set_mv(view * model);
        self.set_transform(view, model);
    }

    /// Updates just the camera component of this transform
    pub fn update_view(
        &self,
        proj: impl Into<ColumnMatrix4<f32>>,
        view: impl Into<ColumnMatrix4<f32>>,
    ) {
        let proj = Mat4::from(proj.into());
        let view = Mat4::from(view.into());
        let model = *self.model.read().unwrap();
        *self.view.write().unwrap() = view;
        *self.proj.write().unwrap() = proj;
        self.set_mvp(proj * view * model);
        self.set_mv(view * model);
        self.set_transform(view, model);
    }

    /// Set the model-view-projection matrix
    fn set_mvp(&self, proj: Mat4) {
        queue().write_buffer(&self.buffer, 0, bytemuck::bytes_of(&proj));
    }

    /// Set the model-view matrix for this transform
    fn set_mv(&self, view: Mat4) {
        queue().write_buffer(&self.buffer, 64, bytemuck::bytes_of(&view));
    }

    /// Set the model and normalized model matrix for this transform
    fn set_transform(&self, view: Mat4, model: Mat4) {
        queue().write_buffer(
            &self.buffer,
            128,
            bytemuck::bytes_of(&(view * model).inverse().transpose()),
        );
    }

    /// Get the underlying buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Get a reference to the model transform
    pub fn model(&self) -> Arc<RwLock<Mat4>> {
        Arc::clone(&self.model)
    }

    /// Create a frustum from the camera
    pub fn frustum() -> Frustum {
        Frustum::new(*PROJ.read().unwrap() * *VIEW.read().unwrap())
    }
}

impl Borrow<Buffer> for Transform {
    fn borrow(&self) -> &Buffer {
        &self.buffer
    }
}

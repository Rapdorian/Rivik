/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::{fmt, ops};

#[derive(Clone)]
pub struct F32(String);

impl F32 {
    pub fn new(name: f32) -> Self {
        Self(name.to_string())
    }
    pub fn bind(name: &str) -> Self {
        Self(name.to_owned())
    }
}

#[derive(Clone)]
pub struct Vec3(String);

impl Vec3 {
    pub fn new(x: F32, y: F32, z: F32) -> Self {
        Self(format!("vec3<f32>({x}, {y}, {z})"))
    }

    pub fn bind(name: &str) -> Self {
        Self(name.to_owned())
    }

    pub fn normalize(&self) -> Self {
        Self(format!("normalize({self})"))
    }
}

#[derive(Clone)]
pub struct Vec4(String);

impl Vec4 {
    pub fn new(x: F32, y: F32, z: F32, w: F32) -> Self {
        Self(format!("vec4<f32>({x}, {y}, {z}, {w})"))
    }

    pub fn bind(name: &str) -> Self {
        Self(name.to_owned())
    }
}

#[derive(Clone)]
pub struct Mat4(String);

impl Mat4 {
    pub fn bind(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl_display!(F32);
impl_display!(Vec3);
impl_display!(Vec4);
impl_display!(Mat4);

impl_mul!(F32);
impl_mul!(Vec3, F32, Vec3);
impl_mul!(Vec3);
impl_mul!(Vec4);
impl_mul!(Vec4, F32, Vec4);
impl_mul!(Mat4);
impl_mul!(Mat4, Vec4, Vec4);
impl_mul!(F32, f32);
impl_mul!(Vec3, f32);
impl_mul!(Vec4, f32);

impl_add!(F32);
impl_add!(Vec3);
impl_add!(Vec4);
impl_add!(F32, f32);
impl_add!(Vec3, f32);
impl_add!(Vec4, f32);

impl_sub!(F32);
impl_sub!(Vec3);
impl_sub!(Vec4);
impl_sub!(F32, f32);
impl_sub!(Vec3, f32);
impl_sub!(Vec4, f32);

impl_div!(F32);
impl_div!(Vec3, F32, Vec3);
impl_div!(Vec4, F32, Vec4);
impl_div!(Vec3);
impl_div!(Vec4);
impl_div!(F32, f32);
impl_div!(Vec3, f32);
impl_div!(Vec4, f32);

impl From<f32> for F32 {
    fn from(value: f32) -> Self {
        F32(value.to_string())
    }
}

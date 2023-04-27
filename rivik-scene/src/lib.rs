/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! this project provides a scene graph system for use with the rivik engine

use std::{
    ops::Mul,
    sync::{Arc, RwLock},
};

/// Scenegraph node which can have children and manage them
#[derive(Default)]
pub struct Node<T: Clone + Default> {
    local: T,
    parent: T,
    children: Vec<Arc<RwLock<Node<T>>>>,
}

impl<T: Clone + Default + Mul<T, Output = T>> Node<T> {
    pub fn new(transform: T) -> Self {
        Self {
            local: transform,
            parent: T::default(),
            children: Vec::new(),
        }
    }

    /// Add a child node to this node
    pub fn insert(&mut self, child: T) -> Arc<RwLock<Node<T>>> {
        let mut node = Node::new(child);
        node.parent = self.local.clone();
        let node = Arc::new(RwLock::new(node));
        self.children.push(Arc::clone(&node));
        node
    }

    /// Changes this nodes local space and update its children's parent space
    pub fn update(&mut self, transform: T) {
        self.local = transform;
        for child in &mut self.children {
            child
                .write()
                .unwrap()
                .update_parent_space(self.local.clone());
        }
    }

    /// Internal method which recursively updates nodes parent space fields
    fn update_parent_space(&mut self, parent: T) {
        self.parent = parent;
        let global = self.global();
        for child in &mut self.children {
            child.write().unwrap().update_parent_space(global.clone());
        }
    }

    /// Get this nodes localspace
    pub fn local(&self) -> &T {
        &self.local
    }

    /// Get this node's parent's space
    pub fn parent(&self) -> &T {
        &self.parent
    }

    /// Get this node's global position
    pub fn global(&self) -> T {
        self.parent.clone() * self.local.clone()
    }
}

//! A definition for parsing scenes using toml

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LightColor {
    Power(f32),
    Color([f32; 3]),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeData {
    Object {
        path: String,
        #[serde(alias = "pos")]
        #[serde(default)]
        position: [f32; 3],
    },
    PointLight {
        #[serde(alias = "brightness")]
        #[serde(alias = "power")]
        #[serde(alias = "color")]
        light: LightColor,
        #[serde(alias = "pos")]
        #[serde(default)]
        position: [f32; 3],
    },
    DirectionalLight {
        #[serde(alias = "brightness")]
        #[serde(alias = "power")]
        #[serde(alias = "color")]
        light: LightColor,
        #[serde(alias = "dir")]
        direction: [f32; 3],
    },
    SubScene {
        include: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(flatten)]
    data: NodeData,
    #[serde(alias = "node")]
    #[serde(default)]
    nodes: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    ambient: Option<LightColor>,
    #[serde(alias = "node")]
    #[serde(default)]
    nodes: Vec<Node>,
}

use std::{
    io::{BufRead, BufReader},
    num::{ParseFloatError, ParseIntError},
};

use log::{error, warn};
use mint::{Point2, Point3};
use snafu::{Backtrace, OptionExt, ResultExt, Snafu};

use crate::{formats::Format, Path, ReaderCreationError};

use super::{Mesh, Scene};

#[derive(Snafu, Debug)]
pub enum ObjError {
    #[snafu(display("Failed loading OBJ asset"))]
    CreationError {
        #[snafu(backtrace)]
        source: ReaderCreationError,
    },
    #[snafu(display("Invalid coordinate {value} on line {line}"))]
    InvalidCoord {
        line: usize,
        value: String,
        source: ParseFloatError,
        backtrace: Backtrace,
    },
    #[snafu(display(
        "on line {line}: vertex index '{value}' is out of range, max value is {max}"
    ))]
    IndexOutOfRange {
        line: usize,
        value: usize,
        max: usize,
        backtrace: Backtrace,
    },
    #[snafu(display("on line {line}: invalid face index {index}"))]
    InvalidFaceIndex {
        line: usize,
        index: String,
        source: ParseIntError,
        backtrace: Backtrace,
    },
    #[snafu(display("missing face index on line {line}"))]
    MissingFaceIndex { line: usize, backtrace: Backtrace },
}

/// File format definition for Wavefront obj files
#[derive(Clone, Copy)]
pub struct ObjMesh;

impl Format for ObjMesh {
    type Output = Mesh<f32>;
    type Error = ObjError;

    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        let mut scene = (ObjScene).parse(path)?;
        Ok(scene.nodes.pop().unwrap().0)
    }
}

/// File format definition for Wavefront obj files
pub struct ObjScene;

impl Format for ObjScene {
    type Output = Scene<f32>;
    type Error = ObjError;

    fn parse(&self, path: &Path) -> Result<Self::Output, Self::Error> {
        let reader = BufReader::new(path.reader().context(CreationSnafu)?);

        let mut verts: Vec<Point3<f32>> = vec![];
        let mut normals: Vec<Point3<f32>> = vec![];
        let mut uvs: Vec<Point2<f32>> = vec![];
        let mut indices: Vec<(usize, Option<usize>, Option<usize>)> = vec![];
        let mut scene: Vec<(Mesh<f32>, String)> = vec![];
        let mut cur_obj: Option<String> = None;

        for (n, line) in reader.lines().enumerate() {
            let n = n + 1; // files usually aren't 0 indexed
                           //let line = throw!(line, "Failed to parse line {n}");
            let line = line.expect("line present");

            let tokens: Vec<&str> = line.split_whitespace().collect();

            match tokens[..] {
                ["#", ..] => { /* do nothing this is a comment */ }
                ["v", x, y, z] => verts.push(Point3 {
                    x: x.parse().context(InvalidCoordSnafu { line: n, value: x })?,
                    y: y.parse().context(InvalidCoordSnafu { line: n, value: y })?,
                    z: z.parse().context(InvalidCoordSnafu { line: n, value: z })?,
                }),
                ["vt", u, v] => uvs.push(Point2 {
                    x: u.parse().context(InvalidCoordSnafu { line: n, value: u })?,
                    y: v.parse().context(InvalidCoordSnafu { line: n, value: v })?,
                }),
                ["vn", x, y, z] => normals.push(Point3 {
                    x: x.parse().context(InvalidCoordSnafu { line: n, value: x })?,
                    y: y.parse().context(InvalidCoordSnafu { line: n, value: y })?,
                    z: z.parse().context(InvalidCoordSnafu { line: n, value: z })?,
                }),
                ["o", name] => {
                    if let Some(name) = cur_obj {
                        // do some validation of the parsed data
                        if indices.len() % 3 != 0 {
                            warn!("object does not have a valid number of indices ({}), expected a multiple of 3", indices.len());
                        }
                        if verts.len() > normals.len() {
                            warn!("found {} vertices and {} normals, some vertices will be missing normals", verts.len(), normals.len());
                        }
                        if verts.len() > uvs.len() {
                            warn!("found {} vertices and {} uv coordinates, some vertices will be missing uv coords", verts.len(), uvs.len());
                        }

                        // build a mesh from parsed data
                        let mut mesh = Mesh::default();
                        for (v, uv, norm) in &indices {
                            let out_of_range = |max| IndexOutOfRangeSnafu {
                                line: n,
                                value: *v,
                                max,
                            };

                            mesh.verts
                                .push(*verts.get(*v - 1).context(out_of_range(verts.len()))?);
                            if let Some(norm) = norm {
                                mesh.normals.push(
                                    *normals
                                        .get(*norm - 1)
                                        .context(out_of_range(normals.len()))?,
                                );
                            }
                            if let Some(uv) = uv {
                                mesh.uvs
                                    .push(*uvs.get(*uv - 1).context(out_of_range(uvs.len()))?);
                            }
                        }

                        // add mesh to scene
                        scene.push((mesh, name.to_string()));

                        // clear the current info
                        indices.clear();
                        verts.clear();
                        normals.clear();
                        uvs.clear();
                    }
                    // record the name of the last object
                    cur_obj = Some(name.to_string());
                }
                ["f", a, b, c] => {
                    let mut parse_index = |index: &str| {
                        let mut tokens = index
                            .split('/')
                            .map(|s| -> Result<Option<_>, ObjError> {
                                if s.is_empty() {
                                    Ok(None)
                                } else {
                                    Ok(Some(s.parse::<usize>().context(InvalidFaceIndexSnafu {
                                        line: n,
                                        index: index.to_string(),
                                    })?))
                                }
                            })
                            .map(|r| r.transpose());

                        let Some(Some(v)) = tokens.next() else { return MissingFaceIndexSnafu{line: n}.fail() };
                        let v = v?;
                        let uv = tokens.next().flatten().transpose()?;
                        let norm = tokens.next().flatten().transpose()?;
                        indices.push((v, uv, norm));

                        Ok(())
                    };
                    (parse_index)(a)?;
                    (parse_index)(b)?;
                    (parse_index)(c)?;
                }
                _ => error!("Unrecognized .obj command '{line}'"),
            }
        }

        // record the last mesh since it won't have an `o` tag
        // build a mesh from parsed data
        let mut mesh = Mesh::default();
        for (v, uv, norm) in &indices {
            mesh.verts.push(verts[*v - 1]);
            if let Some(norm) = norm {
                mesh.normals.push(normals[*norm - 1]);
            }
            if let Some(uv) = uv {
                mesh.uvs.push(uvs[*uv - 1]);
            }
        }

        // add mesh to scene
        scene.push((mesh, cur_obj.unwrap_or_else(|| String::from("<anonymous>"))));

        let mut out_scene = Scene::default();
        for elem in scene {
            out_scene.nodes.push(elem);
        }

        Ok(out_scene)
    }
}

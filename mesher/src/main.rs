use std::{fs::File, io::Write};

use clap::{arg, Parser};

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_name = "FILE")]
    source: String,
    #[arg(short, long)]
    out: Option<String>,
}

fn main() {
    let args = Args::parse();

    let (obj, _) = tobj::load_obj(&args.source, &dbg!(tobj::GPU_LOAD_OPTIONS)).unwrap();

    assert_eq!(obj.len(), 1);
    let obj = &obj[0].mesh;

    let mut verts = vec![];
    let mut faces = vec![];

    for idx in obj.indices.iter() {
        faces.push(*idx as u16);
    }

    for i in 0..obj.positions.len() / 3 {
        let x = obj.positions[i * 3];
        let y = obj.positions[i * 3 + 1];
        let z = obj.positions[i * 3 + 2];

        let nx = obj.normals[i * 3];
        let ny = obj.normals[i * 3 + 1];
        let nz = obj.normals[i * 3 + 2];

        let u = obj.texcoords[i * 2];
        let v = obj.texcoords[i * 2 + 1];

        verts.extend_from_slice(&[x, y, z, nx, ny, nz, u, v]);
    }

    // write mesh to file
    let outfile = args.out.unwrap_or(format!("{}.obj", args.source));
    let mut file = File::create(outfile).unwrap();

    let v_buffer = bytemuck::cast_slice(&verts);
    let i_buffer = bytemuck::cast_slice(&faces);

    let sizes = [v_buffer.len() as u16, i_buffer.len() as u16];
    let size_buffer = bytemuck::cast_slice(&sizes);

    file.write_all(size_buffer).unwrap();
    file.write_all(v_buffer).unwrap();
    file.write_all(i_buffer).unwrap();
}

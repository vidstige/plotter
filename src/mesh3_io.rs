use std::{
    fs::File,
    io::{self, BufRead, BufReader, ErrorKind},
    path::Path,
};

use nalgebra_glm::Vec3;

use crate::mesh3::{FaceVertex, Mesh3};

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message.into())
}

fn parse_f32(
    parts: &mut dyn Iterator<Item = &str>,
    kind: &str,
    component: &str,
) -> io::Result<f32> {
    parts
        .next()
        .ok_or_else(|| invalid_data(format!("missing {component} in {kind}")))?
        .parse::<f32>()
        .map_err(|_| invalid_data(format!("invalid {kind} component")))
}

fn parse_vec3(parts: &mut dyn Iterator<Item = &str>, kind: &str) -> io::Result<Vec3> {
    let x = parse_f32(parts, kind, "x")?;
    let y = parse_f32(parts, kind, "y")?;
    let z = parse_f32(parts, kind, "z")?;
    Ok(Vec3::new(x, y, z))
}

fn parse_index(token: &str, kind: &str, len: usize) -> io::Result<usize> {
    let raw = token
        .parse::<isize>()
        .map_err(|_| invalid_data(format!("invalid {kind} index")))?;
    if raw == 0 {
        return Err(invalid_data(format!("invalid {kind} index")));
    }
    let index = if raw > 0 {
        raw as usize - 1
    } else {
        len.checked_sub((-raw) as usize)
            .ok_or_else(|| invalid_data(format!("{kind} index out of bounds")))?
    };
    if index >= len {
        return Err(invalid_data(format!("{kind} index out of bounds")));
    }
    Ok(index)
}

fn parse_face_vertex(token: &str, mesh: &Mesh3) -> io::Result<FaceVertex> {
    let mut parts = token.split('/');
    let vertex = parts
        .next()
        .filter(|part| !part.is_empty())
        .ok_or_else(|| invalid_data("missing vertex index in face"))?;
    let second = parts.next();
    let third = parts.next();
    if parts.next().is_some() {
        return Err(invalid_data("unsupported face vertex format"));
    }
    if matches!(second, Some(value) if !value.is_empty()) {
        return Err(invalid_data("texture coordinates are not supported"));
    }

    let vertex = parse_index(vertex, "vertex", mesh.vertices.len())?;
    let normal = match third {
        Some(value) if !value.is_empty() => Some(parse_index(value, "normal", mesh.normals.len())?),
        _ => None,
    };

    Ok(FaceVertex { vertex, normal })
}

pub fn read_obj(reader: impl BufRead) -> io::Result<Mesh3> {
    let mut mesh = Mesh3::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(keyword) = parts.next() else {
            continue;
        };

        match keyword {
            "v" => mesh.vertices.push(parse_vec3(&mut parts, "vertex")?),
            "vn" => mesh.normals.push(parse_vec3(&mut parts, "normal")?),
            "f" => {
                let face: io::Result<Vec<_>> =
                    parts.map(|part| parse_face_vertex(part, &mesh)).collect();
                let face = face?;
                if face.len() < 3 {
                    return Err(invalid_data("face must have at least 3 vertices"));
                }
                mesh.faces.push(face);
            }
            "o" | "g" | "s" | "mtllib" | "usemtl" => {}
            _ => return Err(invalid_data(format!("unsupported OBJ statement: {keyword}"))),
        }
    }

    Ok(mesh)
}

pub fn load_obj(path: impl AsRef<Path>) -> io::Result<Mesh3> {
    let reader = BufReader::new(File::open(path)?);
    read_obj(reader)
}

#[cfg(test)]
mod tests {
    use super::read_obj;

    #[test]
    fn reads_vertices_normals_and_faces() {
        let obj = "\
v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 0
vn 0 0 1
f 1//1 2//1 3//1 4//1
";

        let mesh = read_obj(obj.as_bytes()).unwrap();

        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.normals.len(), 1);
        assert_eq!(mesh.faces.len(), 1);
        assert_eq!(mesh.faces[0].len(), 4);
        assert_eq!(mesh.faces[0][0].vertex, 0);
        assert_eq!(mesh.faces[0][0].normal, Some(0));
        assert_eq!(mesh.faces[0][3].vertex, 3);
    }

    #[test]
    fn reads_negative_face_indices() {
        let obj = "\
v 0 0 0
v 1 0 0
v 0 1 0
vn 0 0 1
f -3//-1 -2//-1 -1//-1
";

        let mesh = read_obj(obj.as_bytes()).unwrap();

        assert_eq!(mesh.faces.len(), 1);
        assert_eq!(mesh.faces[0][0].vertex, 0);
        assert_eq!(mesh.faces[0][1].vertex, 1);
        assert_eq!(mesh.faces[0][2].vertex, 2);
        assert_eq!(mesh.faces[0][0].normal, Some(0));
    }

    #[test]
    fn rejects_unknown_statements() {
        let obj = "\
v 0 0 0
curv 0 1 2
";

        let error = read_obj(obj.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("unsupported OBJ statement"));
    }

    #[test]
    fn rejects_texture_coordinates() {
        let obj = "\
v 0 0 0
v 1 0 0
v 0 1 0
vt 0 0
f 1/1 2/1 3/1
";

        let error = read_obj(obj.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("unsupported OBJ statement"));
    }
}

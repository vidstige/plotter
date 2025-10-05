use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[derive(Debug, Default)]
pub struct AudioAnalysis {
    beats: Vec<f32>,
    onsets: Vec<f32>,
}

pub fn load_dat<P: AsRef<Path>>(path: P) -> io::Result<HashMap<String, Vec<f32>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut sections: HashMap<String, Vec<f32>> = HashMap::new();
    let mut current_section: Option<String> = None;

    for (line_no, line) in reader.lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(stripped) = trimmed.strip_suffix(':') {
            let name = stripped.trim();
            if name.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Empty section name at line {}", line_no + 1),
                ));
            }
            let name = name.to_string();
            sections.entry(name.clone()).or_default();
            current_section = Some(name);
            continue;
        }

        let section = current_section.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Value without section header at line {}", line_no + 1),
            )
        })?;

        let value: f32 = trimmed.parse().map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse value `{}` at line {}: {}", trimmed, line_no + 1, err),
            )
        })?;

        if let Some(values) = sections.get_mut(section) {
            values.push(value);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Missing section `{}` when adding value at line {}", section, line_no + 1),
            ));
        }
    }

    Ok(sections)
}

impl AudioAnalysis {
    pub fn load_dat_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut sections = load_dat(path)?;

        let beats = sections.remove("beats").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing `beats` section")
        })?;

        let onsets = sections.remove("onsets").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing `onsets` section")
        })?;

        Ok(Self { beats, onsets })
    }

    pub fn beats(&self) -> &[f32] {
        &self.beats
    }

    pub fn onsets(&self) -> &[f32] {
        &self.onsets
    }
}

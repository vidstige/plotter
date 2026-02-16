use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[derive(Debug, Clone, Copy)]
pub struct Beat {
    pub time: f32,
    pub strength: f32,
}

#[derive(Debug, Default)]
pub struct AudioAnalysis {
    beats: Vec<Beat>,
    onsets: Vec<f32>,
}

type SectionEntries = HashMap<String, Vec<String>>;

fn load_sections<P: AsRef<Path>>(path: P) -> io::Result<SectionEntries> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut sections: SectionEntries = HashMap::new();
    let mut current_section: Option<String> = None;

    for line in reader.lines() {
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
                    "Empty section name",
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
                "Value without section header",
            )
        })?;

        if let Some(values) = sections.get_mut(section) {
            values.push(trimmed.to_string());
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Missing section `{}`", section),
            ));
        }
    }

    Ok(sections)
}

fn parse_scalar_value(value: &str) -> io::Result<f32> {
    let token = value.split(',').next().unwrap_or_default().trim();
    token.parse::<f32>().map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse value `{}`: {}", value, err),
        )
    })
}

pub fn load_dat<P: AsRef<Path>>(path: P) -> io::Result<HashMap<String, Vec<f32>>> {
    let entries = load_sections(path)?;
    let mut sections: HashMap<String, Vec<f32>> = HashMap::new();

    for (section, values) in entries {
        let mut parsed = Vec::with_capacity(values.len());
        for value in values {
            parsed.push(parse_scalar_value(&value)?);
        }
        sections.insert(section, parsed);
    }

    Ok(sections)
}

impl AudioAnalysis {
    pub fn load_dat_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut sections = load_sections(path)?;

        let beats = sections.remove("beats").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing `beats` section")
        })?;
        let beats: io::Result<Vec<Beat>> = beats
            .into_iter()
            .map(|value| {
                let mut parts = value.split(',').map(str::trim);
                let time = parts.next().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Missing beat time")
                })?;
                let strength = parts.next().unwrap_or("1.0");

                if parts.next().is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unexpected extra beat fields",
                    ));
                }

                let time = time.parse::<f32>().map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to parse beat time `{}`: {}", time, err),
                    )
                })?;
                let strength = strength.parse::<f32>().map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to parse beat strength `{}`: {}", strength, err),
                    )
                })?;

                Ok(Beat { time, strength })
            })
            .collect();
        let beats = beats?;

        let onsets = sections.remove("onsets").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Missing `onsets` section")
        })?;
        let onsets: io::Result<Vec<f32>> = onsets
            .into_iter()
            .map(|value| parse_scalar_value(&value))
            .collect();
        let onsets = onsets?;

        Ok(Self { beats, onsets })
    }

    pub fn beats(&self) -> &[Beat] {
        &self.beats
    }

    pub fn onsets(&self) -> &[f32] {
        &self.onsets
    }
}

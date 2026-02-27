use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[derive(Debug, Default)]
pub struct AudioAnalysis {
    beats: Vec<f32>,
    claps: Vec<f32>,
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

        let beats = sections
            .remove("beat")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing `beat` section"))?;
        let beats: Vec<f32> = beats
            .into_iter()
            .map(|value| parse_scalar_value(&value))
            .collect::<io::Result<Vec<f32>>>()?;

        let claps = sections
            .remove("claps")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing `claps` section"))?;
        let claps: Vec<f32> = claps
            .into_iter()
            .map(|value| parse_scalar_value(&value))
            .collect::<io::Result<Vec<f32>>>()?;
        Ok(Self { beats, claps })
    }

    pub fn beats(&self) -> &[f32] {
        &self.beats
    }

    pub fn claps(&self) -> &[f32] {
        &self.claps
    }
}

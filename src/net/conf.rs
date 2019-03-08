use failure;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

// WireGuard configuration files are based on the Windows INI format, but allow multiple sections.

pub struct Conf {
    pub sections: Vec<Section>
}

pub struct Section {
    pub name: String,
    pub values: HashMap<String, String>,
}

pub fn parse(file: File) -> Result<Conf, failure::Error> {
    let mut conf = Conf { sections: vec![] };

    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line_num = i + 1;
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line.len() > 2 && line.starts_with("[") && line.ends_with("]") {
            conf.sections.push(Section {
                name: line[1..line.len()-1].to_string(),
                values: HashMap::new(),
            });

            continue;
        }

        match line.splitn(2, "=").collect::<Vec<&str>>().as_slice() {
            [field, value] => {
                let section = conf.sections.last_mut()
                    .ok_or(ConfNoSectionFound)?;

                let field = field.trim().to_string();
                let value = value.trim().to_string();

                if section.values.contains_key(&field) {
                    return Err(ConfDuplicateField { line_num }.into());
                }
                section.values.insert(field, value);
            },
            _ => return Err(ConfUnrecognizedLine { line_num }.into()),
        };
    }

    Ok(conf)
}

#[derive(Debug, failure::Fail)]
#[fail(display = "no start of section found before first line of conf file")]
pub struct ConfNoSectionFound;

#[derive(Debug, failure::Fail)]
#[fail(display = "line {} is not a field value pair or a section", line_num)]
pub struct ConfUnrecognizedLine {
    line_num: usize
}

#[derive(Debug, failure::Fail)]
#[fail(display = "line {} contains a duplicate field", line_num)]
pub struct ConfDuplicateField {
    line_num: usize
}

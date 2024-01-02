//! Naive implementation

use std::io::BufRead;

#[derive(Debug, Clone, Copy)]
struct Records {
    count: u64,
    min: f32,
    max: f32,
    sum: f32,
}

impl Records {
    fn update(&mut self, item: f32) {
        self.count += 1;
        self.min = self.min.min(item);
        self.max = self.max.max(item);
        self.sum += item;
    }

    fn from_item(item: f32) -> Self {
        Self {
            count: 1,
            min: item,
            max: item,
            sum: item,
        }
    }

    fn mean(&self) -> f32 {
        let mean = self.sum / (self.count as f32);
        (mean * 10.0).round() / 10.0
    }
}

type Map<'a> = std::collections::HashMap<String, Records>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::open("measurements.txt")?;
    let reader = std::io::BufReader::new(file);
    let mut map = Map::new();

    for line in reader.lines() {
        // single allocation per line
        let mut line = line?;
        let split_point = line.find(';').ok_or_else(|| format!("no ';' in {line}"))?;
        let temp = &line[split_point + 1..];
        let temp: f32 = temp
            .parse()
            .map_err(|err| format!("parsing {temp}: {err}"))?;
        line.truncate(split_point);
        let city = line;

        map.entry(city)
            .and_modify(|records| records.update(temp))
            .or_insert_with(|| Records::from_item(temp));
    }

    let mut keys = map.keys().collect::<Vec<_>>();
    keys.sort_unstable();

    for key in keys {
        let record = map[key];
        let min = record.min;
        let mean = record.mean();
        let max = record.max;

        println!("{key}: {min}/{mean}/{max}");
    }

    Ok(())
}

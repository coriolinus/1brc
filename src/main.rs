use std::{
    fs::File,
    os::unix::fs::{FileExt, MetadataExt},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
};

/// Size of chunk that each thread will process at a time
const CHUNK_SIZE: u64 = 16 * 1024 * 1024;
/// How much extra space we back the chunk start up by, to ensure we capture the full initial record
///
/// Must be greater than the longest line in the table
const CHUNK_EXCESS: u64 = 64;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

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

    fn merge(self, other: Self) -> Self {
        Self {
            count: self.count + other.count,
            min: self.min.min(other.min),
            max: self.max.max(other.max),
            sum: self.sum + other.sum,
        }
    }
}

#[cfg(not(feature = "fxhash"))]
type Hasher = std::collections::hash_map::RandomState;

#[cfg(feature = "fxhash")]
type Hasher = fxhash::FxBuildHasher;

type GenericMap<K, V> = std::collections::HashMap<K, V, Hasher>;

type Map = GenericMap<String, Records>;
// note that we defer parsing the slice into a string until as late as possible, which hopefully
// minimizes access time
type BorrowedMap<'a> = GenericMap<&'a [u8], Records>;

macro_rules! new_map {
    ($t:ty) => {{
        let hasher = Hasher::default();
        <$t>::with_hasher(hasher)
    }};
}

/// Get an aligned buffer from the given file.
///
/// "Aligned" in this case means that the first byte of the returned buffer is the
/// first byte of a record, and if `offset != 0` then the previous byte of the source file is `\n`,
/// and the final byte of the returned buffer is `\n`.
fn get_aligned_buffer<'a>(file: &File, offset: u64, mut buffer: &'a mut [u8]) -> Result<&'a [u8]> {
    assert!(
        offset == 0 || offset > CHUNK_EXCESS,
        "offset must never be less than chunk excess"
    );
    let metadata = file.metadata()?;
    let file_size = metadata.size();
    if offset > file_size {
        return Ok(&[]);
    }

    let buffer_size = buffer.len().min((file_size - offset) as usize);
    buffer = &mut buffer[..buffer_size];

    let mut head;
    let read_from;

    if offset == 0 {
        head = 0;
        read_from = 0;
    } else {
        head = CHUNK_EXCESS as usize;
        read_from = offset - CHUNK_EXCESS;
    };

    file.read_exact_at(buffer, read_from)?;

    // step backwards until we find the end of the previous record
    // then drop all elements before that
    while head > 0 {
        if buffer[head - 1] == b'\n' {
            break;
        }
        head -= 1;
    }

    // find the end of the final valid record
    let mut tail = buffer.len() - 1;
    while buffer[tail] != b'\n' {
        tail -= 1;
    }

    Ok(&buffer[head..=tail])
}

fn process_chunk(
    file: &File,
    offset: u64,
    outer_map: &mut Arc<Mutex<Map>>,
    buffer: &mut [u8],
) -> Result<()> {
    let aligned_buffer = get_aligned_buffer(file, offset, buffer)?;
    let mut map = new_map!(BorrowedMap);

    for line in aligned_buffer
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
    {
        let split_point = line
            .iter()
            .enumerate()
            .find_map(|(idx, &b)| (b == b';').then_some(idx))
            .ok_or_else(|| {
                let line = std::str::from_utf8(line).unwrap_or("<invalid utf8>");
                format!("no ';' in {line}")
            })?;

        let temp = std::str::from_utf8(&line[split_point + 1..])
            .map_err(|err| format!("non-utf8 temp: {err}"))?;
        let temp: f32 = temp
            .parse()
            .map_err(|err| format!("parsing {temp}: {err}"))?;

        let city = &line[..split_point];

        map.entry(city)
            .and_modify(|records| records.update(temp))
            .or_insert_with(|| Records::from_item(temp));
    }

    // that should have taken a while; long enough that we can now cheaply update the outer map
    // without worrying too much about contention from other threads
    let mut outer = outer_map.lock().expect("non-poisoned mutex");
    for (city, records) in map.into_iter() {
        let city =
            String::from_utf8(city.to_owned()).map_err(|err| format!("non-utf8 city: {err}"))?;
        outer
            .entry(city)
            .and_modify(|outer_records| *outer_records = outer_records.merge(records))
            .or_insert(records);
    }

    Ok(())
}

fn distribute_work(file: &File) -> Result<Map> {
    let metadata = file.metadata()?;
    let file_size = metadata.size();

    let offset = Arc::new(AtomicU64::new(0));
    let map = Arc::new(Mutex::new(new_map!(Map)));

    thread::scope(|scope| {
        for _ in 0..thread::available_parallelism().map(Into::into).unwrap_or(1) {
            let offset = offset.clone();
            let mut map = map.clone();
            scope.spawn(move || {
                let mut buffer = vec![0; (CHUNK_SIZE + CHUNK_EXCESS) as usize];
                loop {
                    let offset = offset.fetch_add(CHUNK_SIZE, Ordering::SeqCst);
                    if offset > file_size {
                        break;
                    }

                    process_chunk(file, offset, &mut map, &mut buffer)
                        .expect("processing a chunk should always succeed");
                }
            });
        }
    });

    Ok(Arc::into_inner(map)
        .expect("all other references to map have gone out of scope")
        .into_inner()
        .expect("no poisoned mutexes in this program"))
}

fn main() -> Result<()> {
    let file = std::fs::File::open("measurements.txt")?;
    let map = distribute_work(&file)?;

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

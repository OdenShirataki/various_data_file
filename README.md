# various_data_file

Store variable-length binary data in a file.
Uses mmap, and reuses the space of deleted data for new data.

## Example

```rust
use various_data_file::VariousDataFile;

let dir = std::env::temp_dir().join("various_data_file-readme");
std::fs::create_dir_all(&dir).unwrap();
let path = dir.join("readme.d");
let _ = std::fs::remove_file(&path);
let _ = std::fs::remove_file(dir.join("readme.d.f"));

let mut file = VariousDataFile::new(&path);

// insert returns the address of the stored data.
let noah = file.insert(b"Noah").into_address();
let liam = file.insert(b"Liam").into_address();

assert_eq!(file.bytes(&noah), b"Noah");
assert_eq!(file.bytes(&liam), b"Liam");

// the space of deleted data is reused by later inserts.
file.delete(noah);
let olivia = file.insert(b"Oli").into_address();
assert_eq!(file.bytes(&olivia), b"Oli");
assert_eq!(file.bytes(&liam), b"Liam");
```

//! Inserts and deletes random byte strings and checks after every operation
//! that all live data is still intact, i.e. reused free space never overlaps.

use various_data_file::{DataAddress, VariousDataFile};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: u32) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n as u64) as u32
    }
}

#[test]
fn random_insert_delete_keeps_live_data() {
    let dir = std::env::temp_dir().join("various_data_file-model-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    for seed in 0..10 {
        let mut rng = Lcg(seed);
        let mut file = VariousDataFile::new(dir.join(format!("model{seed}.d")));
        let mut live: Vec<(DataAddress, Vec<u8>)> = Vec::new();

        for step in 0..3000u32 {
            if live.is_empty() || rng.next(10) < 6 {
                let len = rng.next(40) as usize;
                let byte = (step % 251) as u8;
                let content = vec![byte; len];
                let address = file.insert(&content).into_address();
                live.push((address, content));
            } else {
                let index = rng.next(live.len() as u32) as usize;
                let (address, _) = live.swap_remove(index);
                file.delete(address);
            }

            for (address, content) in &live {
                assert_eq!(file.bytes(address), &content[..], "seed {seed} step {step}");
            }
        }

        // reopen: data and the free list must survive.
        drop(file);
        let mut file = VariousDataFile::new(dir.join(format!("model{seed}.d")));
        for (address, content) in &live {
            assert_eq!(
                file.bytes(address),
                &content[..],
                "seed {seed} after reopen"
            );
        }
        for step in 0..500u32 {
            let content = vec![(step % 7) as u8 + 1; rng.next(40) as usize];
            let address = file.insert(&content).into_address();
            live.push((address, content));
            for (address, content) in &live {
                assert_eq!(
                    file.bytes(address),
                    &content[..],
                    "seed {seed} reopen step {step}"
                );
            }
        }
    }
}

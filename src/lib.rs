mod search_items;

use memchr::memmem;
use std::time::Instant;

static _SEARCH_BOUND: usize = 0x1000;

#[derive(Clone)]
struct HookAddress {
    name: String,
    old_address: Option<usize>,
    bytes: &'static [u8],
}

impl HookAddress {
    fn find_offset(&self) {
        let started = Instant::now();
        match self.old_address {
            Some(old_address) => {
                println!(
                    "Looking for {}, old address is {:#x}",
                    self.name, old_address
                )
            }
            None => println!("Looking for {}, old address not known", self.name),
        };
        if let Some(offset) = exact_search(self.bytes) {
            println!("Found exact match for {} at {:#x}", self.name, offset);
        } else if let Some((offset, bytes)) = fuzzy_search(self.bytes) {
            println!("Found fuzzy match for {} at {:#x}", self.name, offset);
            println!("Update the bytes to {:#x?}", bytes)
        } else {
            println!("Something went wrong, couldn't find {}", self.name);
        }
        println!("Done looking for {} in {:?}", self.name, started.elapsed());
    }
}

pub fn exact_search(needle: &[u8]) -> Option<usize> {
    let haystack = unsafe {
        let start = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as *const u8;
        let end = skyline::hooks::getRegionAddress(skyline::hooks::Region::Rodata) as *const u8;
        let length = end.offset_from(start) as usize;
        std::slice::from_raw_parts(start, length)
    };
    let first = memmem::find(haystack, needle)?;
    let last = memmem::rfind(haystack, needle)?;
    if first == last { Some(first) } else { None }
}

pub fn fuzzy_search(needle: &[u8]) -> Option<(usize, &[u8])> {
    let haystack = unsafe {
        let start = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as *const u8;
        let end = skyline::hooks::getRegionAddress(skyline::hooks::Region::Rodata) as *const u8;
        let length = end.offset_from(start) as usize;
        std::slice::from_raw_parts(start, length)
    };

    Some(
        haystack
            .windows(needle.len())
            .enumerate()
            .min_by_key(|&(_, w)| (hamming::distance_fast(w, needle).unwrap_or(u64::MAX), w))
            .expect("Haystack is empty!"),
    )
}

#[skyline::main(name = "smash-offset-locator")]
pub fn main() {
    let all_items = search_items::SearchItems::all();
    for item in all_items {
        let _ = std::thread::Builder::new().spawn(move || item.clone().find_offset());
    }
}

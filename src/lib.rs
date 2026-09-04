mod search_items;

use memchr::memmem;
use std::time::Instant;

static SEARCH_BOUND: usize = 0x1000;

#[derive(Clone)]
struct HookAddress {
    name: &'static str,
    old_offset: Option<usize>,
    bytes: &'static [u8],
}

impl HookAddress {
    fn find_offset(&self) {
        let started = Instant::now();
        match self.old_offset {
            Some(old_offset) => {
                println!(
                    "[OffsetLocator] Looking for {}, old offset is {:#x}",
                    self.name, old_offset
                )
            }
            None => println!(
                "[OffsetLocator] Looking for {}, old offset not known",
                self.name
            ),
        };
        if let Some(offset) = exact_search(self.bytes) {
            println!(
                "[OffsetLocator] Found exact match for {} at offset {:#x}",
                self.name, offset
            );
        } else if let Some((offset, bytes)) = fuzzy_search(self.old_offset, self.bytes) {
            println!(
                "[OffsetLocator] Found fuzzy match for {} at offset {:#x}",
                self.name, offset
            );
            println!(
                "[OffsetLocator] Update {} search bytes to {:#x?}",
                self.name, bytes
            )
        } else {
            println!(
                "[OffsetLocator] Something went wrong, couldn't find {}",
                self.name
            );
        }
        println!(
            "[OffsetLocator] Done looking for {} in {:?}",
            self.name,
            started.elapsed()
        );
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

pub fn fuzzy_search(old_offset: Option<usize>, needle: &[u8]) -> Option<(usize, &[u8])> {
    let search_window_start = old_offset.unwrap_or(0).saturating_sub(SEARCH_BOUND);
    let haystack = unsafe {
        let region_start = skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as usize;
        let region_end = skyline::hooks::getRegionAddress(skyline::hooks::Region::Rodata) as usize;
        let region_size = region_end - region_start;

        let search_window_end = old_offset
            .unwrap_or(region_size)
            .saturating_add(SEARCH_BOUND)
            .clamp(0, region_size);
        let start =
            (region_start + search_window_start).clamp(region_start, region_end) as *const u8;
        let end = (region_start + search_window_end).clamp(region_start, region_end) as *const u8;

        let length = end.offset_from(start) as usize;
        std::slice::from_raw_parts(start, length)
    };

    let ret = haystack
        .windows(needle.len())
        .enumerate()
        .min_by_key(|&(_, w)| (hamming::distance_fast(w, needle).unwrap_or(u64::MAX), w));

    if let Some((ind, bytes)) = ret {
        Some((ind + search_window_start, bytes))
    } else {
        println!(
            "[OffsetLocator] Haystack is empty! Check that the offset {:#x?} is valid?",
            old_offset
        );
        None
    }
}

#[skyline::main(name = "smash-offset-locator")]
pub fn main() {
    let all_items = search_items::SearchItems::all();
    for item in all_items {
        let _ = std::thread::Builder::new().spawn(move || item.clone().find_offset());
    }
}

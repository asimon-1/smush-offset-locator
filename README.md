# smush-offset-locator

A diagnostic Skyline plugin for locating the current offsets of instruction hook locations as they change across game versions.

## Search behavior

The plugin first performs an exact search across the text region for the provided input `bytes`.

If that does not find one unique match, it performs a fuzzy search using Hamming distance. When a previous offset is supplied, fuzzy search is limited to a small region either side of that offset, to improve the search speed. When no previous offset is supplied, it searches the entire text region.

If a fuzzy match has a Hamming distance above `MAX_HAMMING_DISTANCE`, then it is reported as suspicious because too many bytes differ, and should be checked manually before using the result. The plugin still prints the best candidate and replacement bytes for suspicious matches so they can be inspected.

## Output

For each item, the plugin reports the discovered offset and the matching bytes. The reported offset is always relative to the start of the text region. For example:

```text
[OffsetLocator] Found fuzzy match for STALE_MENU at offset 0x13e88e0
[OffsetLocator] Update STALE_MENU search bytes to [...]
```

A low-confidence result is reported like this:

```text
[OffsetLocator] Found suspicious fuzzy match for LOAD_PRC_FILE at offset 0x22ce20 with hamming distance 17. This probably is incorrect.
```

## Usage

### Updating Search Items

The offsets to search for are found in `src/search_items.rs`. You can edit the list with your own search terms as needed by adding or editing, the relevant `HookAddress` entry with the name, offset, and assembly bytes. Set `old_offset` to `None` when no previous offset is available; this causes the fuzzy fallback to scan the complete text region.

The default limited fuzzy search scope `SEARCH_BOUND` is +/- 0x1000 bytes from the `old_offset`.
The default confidence threshold `MAX_HAMMING_DISTANCE` is 4 changed bytes between the match and the input `bytes`.

### Build and Running the plugin

Build the plugin using `cargo skyline`

```bash
cargo skyline build --release
```

And upload the resulting plugin `target\aarch64-skyline-switch\release\libsmush_offset_locator.nro` to the usual Smash Ultimate plugins directory directory `/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/`

In your terminal, listen to the Skyline log, replacing the ip address here with the address of your Switch.

Windows terminal:
```cmd
cargo skyline set-ip 192.168.1.100
cargo skyline listen | findstr "OffsetLocator"
```

Powershell:
```powershell
cargo skyline set-ip 192.168.1.100
cargo skyline listen | Select-String "OffsetLocator"
```

Bash:
```bash
cargo skyline set-ip 192.168.1.100
cargo skyline listen | grep OffsetLocator
```

Once you start up smash you will see the log lines with your new updates
```
[OffsetLocator] Looking for CLOUD_ADD_LIMIT, old offset is 0x8dc160
[OffsetLocator] Found exact match for CLOUD_ADD_LIMIT at offset 0x8dc160
[OffsetLocator] Done looking for CLOUD_ADD_LIMIT in 112.856302ms
[OffsetLocator] Looking for STALE_MENU, old offset is 0x13e88e0
[OffsetLocator] Found fuzzy match for STALE_MENU at offset 0x13e88c0
[OffsetLocator] Update STALE_MENU search bytes to [..]
[OffsetLocator] Done looking for STALE_MENU in 32.039844ms
[OffsetLocator] Looking for LOAD_PRC_FILE, old offset is 0x3720910
[OffsetLocator] Found suspicious fuzzy match for LOAD_PRC_FILE at offset 0x3720ec0 with hamming distance 69
[OffsetLocator] Update LOAD_PRC_FILE search bytes to [..]
[OffsetLocator] Done looking for LOAD_PRC_FILE in 33.604635ms
[OffsetLocator] Looking for TRAINING_RESET_CHECK, old offset not known
[OffsetLocator] Found fuzzy match for TRAINING_RESET_CHECK at offset 0x1378e50
[OffsetLocator] Update TRAINING_RESET_CHECK search bytes to [..]
[OffsetLocator] Done looking for TRAINING_RESET_CHECK in 13.565043334s
[OffsetLocator] Looking for BAD_MATCH, old offset is 0x1000000
[OffsetLocator] Found suspicious fuzzy match for BAD_MATCH at offset 0xfff382 with hamming distance 86
[OffsetLocator] Update BAD_MATCH search bytes to [..]
[OffsetLocator] Done looking for BAD_MATCH in 34.650104ms
```
# Graph Report - Auto-Mode  (2026-06-17)

## Corpus Check
- 14 files · ~31,479 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 218 nodes · 358 edges · 14 communities
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ef09de34`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]

## God Nodes (most connected - your core abstractions)
1. `SharedState` - 19 edges
2. `device_loop()` - 13 edges
3. `AppState` - 12 edges
4. `run()` - 12 edges
5. `MacAddr6` - 10 edges
6. `apply_and_report()` - 10 edges
7. `String` - 9 edges
8. `DeviceTasks` - 9 edges
9. `spawn_devices()` - 9 edges
10. `connected_session()` - 9 edges

## Surprising Connections (you probably didn't know these)
- `apply_profile()` --references--> `Result`  [EXTRACTED]
  src-tauri/src/worker.rs → src-tauri/src/worker.rs  _Bridges community 0 → community 9_

## Import Cycles
- 1-file cycle: `src-tauri/src/config.rs -> src-tauri/src/config.rs`
- 1-file cycle: `src-tauri/src/lib.rs -> src-tauri/src/lib.rs`
- 1-file cycle: `src-tauri/src/worker.rs -> src-tauri/src/worker.rs`

## Communities (14 total, 0 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.13
Nodes (43): Arc, FnOnce, HashMap, JoinHandle, OpenSCQ30Device, OpenSCQ30Session, ScanResult, Send (+35 more)

### Community 1 - "Community 1"
Cohesion: 0.11
Nodes (36): AppHandle, CategoryDto, ScanItemDto, SettingDto, apply_now(), AppState, build_categories(), CategoryDto (+28 more)

### Community 2 - "Community 2"
Cohesion: 0.12
Nodes (15): Default, Self, Config, DeviceConfig, resolve_path(), SettingEntry, DeviceConfig, DeviceModel (+7 more)

### Community 3 - "Community 3"
Cohesion: 0.10
Nodes (20): author, bugs, url, description, devDependencies, sharp, @tauri-apps/cli, homepage (+12 more)

### Community 4 - "Community 4"
Cohesion: 0.11
Nodes (18): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+10 more)

### Community 5 - "Community 5"
Cohesion: 0.14
Nodes (6): batteryPct(), Device(), Header(), pretty(), settingsMap(), ToggleRow()

### Community 6 - "Community 6"
Cohesion: 0.12
Nodes (15): dependencies, lucide-react, react, react-dom, devDependencies, tailwindcss, @tailwindcss/vite, vite (+7 more)

### Community 7 - "Community 7"
Cohesion: 0.17
Nodes (11): Build, Configure, Credits, How it works, Modules, Notes, Platform support, Run (+3 more)

### Community 8 - "Community 8"
Cohesion: 0.56
Nodes (8): autostart_path(), current_exe(), disable(), enable(), reconcile(), remove_if_exists(), PathBuf, Result

### Community 9 - "Community 9"
Cohesion: 0.33
Nodes (7): Cow, Select, Result, Setting, one_of(), parse_value(), Value

### Community 10 - "Community 10"
Cohesion: 0.40
Nodes (4): description, identifier, permissions, windows

## Knowledge Gaps
- **75 isolated node(s):** `name`, `version`, `description`, `main`, `test` (+70 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What connects `name`, `version`, `description` to the rest of the system?**
  _75 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.12525252525252525 - nodes in this community are weakly interconnected._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.11411411411411411 - nodes in this community are weakly interconnected._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.12333333333333334 - nodes in this community are weakly interconnected._
- **Should `Community 3` be split into smaller, more focused modules?**
  _Cohesion score 0.09523809523809523 - nodes in this community are weakly interconnected._
- **Should `Community 4` be split into smaller, more focused modules?**
  _Cohesion score 0.10526315789473684 - nodes in this community are weakly interconnected._
- **Should `Community 5` be split into smaller, more focused modules?**
  _Cohesion score 0.13970588235294118 - nodes in this community are weakly interconnected._
use crate::trace_event::TraceEvent;

use std::path::Path;

pub struct DependencyNode {
    pub name: String,
    pub pid: u32,
    pub timestamp_ns: u64,
    pub duration_ns: u64,
    pub dependencies: Vec<String>,
}

impl DependencyNode {
    pub fn from_trace_event(event: TraceEvent) -> Self {
        let cmd = &event.cmds;
        let mut i = 1usize;
        let mut name = String::new();
        let mut dependencies = Vec::new();
        let mut prev_is_option = false;
        while i < event.cmds.len() {
            let is_option = cmd[i].starts_with('-');
            if !prev_is_option && !is_option {
                // an input file
                let path = Path::new(event.cwd.as_str()).join(&cmd[i]);
                dependencies.push(path.to_string_lossy().to_string());
            } else if is_option && cmd[i] == "-o" {
                // output file
                let path = Path::new(event.cwd.as_str()).join(&cmd[i + 1]);
                name = path.to_string_lossy().to_string();
            }
            prev_is_option = is_option;
            i = i + 1;
        }
        return DependencyNode {
            name,
            pid: event.pid,
            timestamp_ns: event.timestamp_ns,
            duration_ns: event.duration_ns,
            dependencies,
        };
    }
}

pub struct DependencyManager {
    events_map: std::collections::HashMap<String, DependencyNode>,
}

impl DependencyManager {
    pub fn new() -> Self {
        DependencyManager {
            events_map: std::collections::HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event: TraceEvent) {
        let dep_event = DependencyNode::from_trace_event(event);
        self.events_map.insert(dep_event.name.clone(), dep_event);
    }

    pub fn iterate_dependencies<F>(&self, mut f: F)
    where
        F: FnMut(&DependencyNode, &DependencyNode),
    {
        for event in self.events_map.values() {
            for dep_name in &event.dependencies {
                if let Some(dep_event) = self.events_map.get(dep_name) {
                    f(&dep_event, &event)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace_event::TraceEvent;

    #[test]
    fn test_dependency_node_from_trace_event() {
        let cmds_str = "g++ -D_GNU_SOURCE -D__SANE_USERSPACE_TYPES__ -I../src/include/ -include ../config-host.h -D_LARGEFILE_SOURCE -D_FILE_OFFSET_BITS=64 -g -O3 -Wall -Wextra -Wno-unused-parameter -Wno-sign-compare -Wstringop-overflow=0 -Warray-bounds=0 -DLIBURING_BUILD_TEST -Wno-unused-parameter -Wno-sign-compare -Wstringop-overflow=0 -Warray-bounds=0 -std=c++11 -DLIBURING_BUILD_TEST -o sq-full-cpp.t sq-full-cpp.cc helpers.o -L../src/ -luring -lpthread";
        let cmds = cmds_str
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let event = TraceEvent {
            pid: 1234,
            timestamp_ns: 1000000,
            duration_ns: 500000,
            cmds: cmds,
            cwd: "/home/user/project".to_string(),
        };
        let dep_node = DependencyNode::from_trace_event(event);
        assert_eq!(dep_node.name, "/home/user/project/sq-full-cpp.t".to_string());
        assert_eq!(dep_node.pid, 1234);
        assert_eq!(dep_node.timestamp_ns, 1000000);
        assert_eq!(dep_node.duration_ns, 500000);
        assert_eq!(
            dep_node.dependencies,
            vec![
                "/home/user/project/sq-full-cpp.cc".to_string(),
                "/home/user/project/helpers.o".to_string()
            ]
        );
    }
}

use crate::trace_event::TraceEvent;

pub struct DependencyNode {
    pub name: String,
    pub pid: u32,
    pub timestamp_ns: u64,
    pub duration_ns: u64,
    pub children: Vec<String>,
}

impl DependencyNode {
    pub fn from_trace_event(event: TraceEvent) -> Self {
        let cmd = &event.cmds;
        let mut i = 0usize;
        let mut name = String::new();
        let mut children = Vec::new();
        while i < event.cmds.len() {
            if cmd[i] == "-o" && i + 1 < event.cmds.len() {
                name = cmd[i + 1].clone();
                i = i + 2;
                continue;
            }
            if cmd[i].ends_with(".o") {
                children.push(cmd[i].clone());
            }
            i = i + 1;
        }
        return DependencyNode {
            name,
            pid: event.pid,
            timestamp_ns: event.timestamp_ns,
            duration_ns: event.duration_ns,
            children,
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
            for child_name in &event.children {
                if let Some(child_event) = self.events_map.get(child_name) {
                    f(&event, &child_event)
                }
            }
        }
    }
}

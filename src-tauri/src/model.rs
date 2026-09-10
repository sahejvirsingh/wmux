use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PaneNode {
    Terminal {
        id: String,
        pty_id: String,
        shell: String,
        cwd: String,
        command: Option<String>,
        title: String,
        zoomed: bool,
        attention: bool,
    },
    Browser {
        id: String,
        url: String,
        title: String,
        zoomed: bool,
        attention: bool,
    },
    Split {
        id: String,
        direction: SplitDirection,
        ratio: f64,
        children: Vec<PaneNode>,
    },
}

impl PaneNode {
    pub fn new_terminal(id: String, shell: String, cwd: String) -> Self {
        PaneNode::Terminal {
            id,
            pty_id: String::new(),
            shell,
            cwd,
            command: None,
            title: "terminal".into(),
            zoomed: false,
            attention: false,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, PaneNode::Terminal { .. })
    }

    pub fn find(&mut self, node_id: &str) -> Option<&mut PaneNode> {
        if self.id() == node_id {
            return Some(self);
        }
        if let PaneNode::Split { children, .. } = self {
            for child in children {
                if let Some(found) = child.find(node_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn find_ref(&self, node_id: &str) -> Option<&PaneNode> {
        if self.id() == node_id {
            return Some(self);
        }
        if let PaneNode::Split { children, .. } = self {
            for child in children {
                if let Some(found) = child.find_ref(node_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn find_terminal_by_pty(&mut self, pty_id: &str) -> Option<&mut PaneNode> {
        match self {
            PaneNode::Terminal { pty_id: pid, .. } if pid == pty_id => Some(self),
            PaneNode::Split { children, .. } => children
                .iter_mut()
                .find_map(|c| c.find_terminal_by_pty(pty_id)),
            _ => None,
        }
    }

    pub fn id(&self) -> &str {
        match self {
            PaneNode::Terminal { id, .. } => id,
            PaneNode::Browser { id, .. } => id,
            PaneNode::Split { id, .. } => id,
        }
    }

    pub fn collect_pty_ids(&self, out: &mut Vec<String>) {
        match self {
            PaneNode::Terminal { pty_id, .. } => {
                if !pty_id.is_empty() {
                    out.push(pty_id.clone());
                }
            }
            PaneNode::Browser { .. } => {}
            PaneNode::Split { children, .. } => {
                for child in children {
                    child.collect_pty_ids(out);
                }
            }
        }
    }

    pub fn collect_browser_ids(&self, out: &mut Vec<String>) {
        match self {
            PaneNode::Browser { id, .. } => out.push(id.clone()),
            PaneNode::Terminal { .. } => {}
            PaneNode::Split { children, .. } => {
                for child in children {
                    child.collect_browser_ids(out);
                }
            }
        }
    }

    pub fn terminal_ids(&self, out: &mut Vec<String>) {
        match self {
            PaneNode::Terminal { id, .. } => out.push(id.clone()),
            PaneNode::Browser { id, .. } => out.push(id.clone()),
            PaneNode::Split { children, .. } => {
                for child in children {
                    child.terminal_ids(out);
                }
            }
        }
    }

    pub fn first_terminal_id(&self) -> Option<String> {
        let mut ids = Vec::new();
        self.terminal_ids(&mut ids);
        ids.first().cloned()
    }

    pub fn next_terminal_id(&self, current: &str) -> Option<String> {
        let mut ids = Vec::new();
        self.terminal_ids(&mut ids);
        let pos = ids.iter().position(|i| i == current)?;
        ids.get((pos + 1) % ids.len()).cloned()
    }

    pub fn prev_terminal_id(&self, current: &str) -> Option<String> {
        let mut ids = Vec::new();
        self.terminal_ids(&mut ids);
        let pos = ids.iter().position(|i| i == current)?;
        ids.get((pos + ids.len() - 1) % ids.len()).cloned()
    }

    pub fn set_zoomed(&mut self, node_id: &str, zoomed: bool) {
        match self {
            PaneNode::Terminal { id, zoomed: z, .. } if id == node_id => *z = zoomed,
            PaneNode::Split { children, .. } => {
                for child in children {
                    child.set_zoomed(node_id, zoomed);
                }
            }
            _ => {}
        }
    }

    pub fn unzoom_all(&mut self) {
        match self {
            PaneNode::Terminal { zoomed, .. } => *zoomed = false,
            PaneNode::Browser { zoomed, .. } => *zoomed = false,
            PaneNode::Split { children, .. } => {
                for child in children {
                    child.unzoom_all();
                }
            }
        }
    }

    pub fn zoomed_node(&self) -> Option<&PaneNode> {
        match self {
            PaneNode::Terminal { zoomed, .. } if *zoomed => Some(self),
            PaneNode::Split { children, .. } => {
                for child in children {
                    if let Some(n) = child.zoomed_node() {
                        return Some(n);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn set_terminal_pty(&mut self, node_id: &str, pty_id: String) {
        match self {
            PaneNode::Terminal { id, pty_id: p, .. } if id == node_id => *p = pty_id,
            PaneNode::Split { children, .. } => {
                for child in children {
                    child.set_terminal_pty(node_id, pty_id.clone());
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub cwd: String,
    pub branch: Option<String>,
    pub ports: Vec<u32>,
    pub notifications: u32,
    pub layout: PaneNode,
    pub active_pane: Option<String>,
}

impl Workspace {
    pub fn new(id: String, name: String, shell: String, cwd: String) -> Self {
        let layout = PaneNode::new_terminal(format!("pane-{id}-1"), shell, cwd.clone());
        Self {
            id,
            name,
            cwd,
            branch: None,
            ports: Vec::new(),
            notifications: 0,
            active_pane: Some(layout.id().to_string()),
            layout,
        }
    }

    pub fn pty_ids(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.layout.collect_pty_ids(&mut out);
        out
    }

    pub fn browser_ids(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.layout.collect_browser_ids(&mut out);
        out
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateJson {
    pub active_workspace: Option<String>,
    pub workspaces: Vec<Workspace>,
}

pub struct WorkspaceManager {
    workspaces: Vec<Workspace>,
    active: Option<String>,
    seq: u64,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: Vec::new(),
            active: None,
            seq: 0,
        }
    }

    pub fn next_id(&mut self, prefix: &str) -> String {
        self.seq += 1;
        format!("{prefix}-{}", self.seq)
    }

    pub fn state_json(&self) -> AppStateJson {
        AppStateJson {
            active_workspace: self.active.clone(),
            workspaces: self.workspaces.clone(),
        }
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    pub fn active_id(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn active_workspace(&self) -> Option<&Workspace> {
        self.active.as_deref().and_then(|id| self.get(id))
    }

    pub fn active_workspace_mut(&mut self) -> Option<&mut Workspace> {
        let id = self.active.clone()?;
        self.get_mut(&id)
    }

    pub fn get(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|w| w.id == id)
    }

    pub fn create(&mut self, name: String, shell: String, cwd: String) -> &Workspace {
        let id = self.next_id("ws");
        self.workspaces.push(Workspace::new(id.clone(), name, shell, cwd));
        self.active = Some(id);
        self.workspaces.last().unwrap()
    }

    pub fn delete(&mut self, id: &str) -> Option<Workspace> {
        let pos = self.workspaces.iter().position(|w| w.id == id)?;
        let removed = self.workspaces.remove(pos);
        if self.active.as_deref() == Some(id) {
            self.active = self.workspaces.first().map(|w| w.id.clone());
        }
        Some(removed)
    }

    pub fn rename(&mut self, id: &str, name: String) {
        if let Some(ws) = self.get_mut(id) {
            ws.name = name;
        }
    }

    pub fn select(&mut self, id: &str) -> bool {
        if self.get(id).is_some() {
            self.active = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn restore(&mut self, ws: Workspace) {
        if self.workspaces.is_empty() {
            self.active = Some(ws.id.clone());
        }
        self.workspaces.push(ws);
    }

    pub fn reorder(&mut self, ids: Vec<String>) {
        let mut reordered: Vec<Workspace> = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(pos) = self.workspaces.iter().position(|w| w.id == id) {
                reordered.push(self.workspaces.remove(pos));
            }
        }
        for ws in self.workspaces.drain(..) {
            reordered.push(ws);
        }
        self.workspaces = reordered;
    }

    pub fn set_branch(&mut self, id: &str, branch: Option<String>) {
        if let Some(ws) = self.get_mut(id) {
            ws.branch = branch;
        }
    }

    pub fn set_ports(&mut self, id: &str, ports: Vec<u32>) {
        if let Some(ws) = self.get_mut(id) {
            ws.ports = ports;
        }
    }

    pub fn set_workspace_cwd(&mut self, id: &str, cwd: String) {
        if let Some(ws) = self.get_mut(id) {
            ws.cwd = cwd;
        }
    }

    pub fn mark_attention(&mut self, id: &str, pty_id: &str) {
        if let Some(ws) = self.get_mut(id) {
            ws.notifications += 1;
            if let Some(node) = ws.layout.find_terminal_by_pty(pty_id) {
                if let PaneNode::Terminal { attention, .. } = node {
                    *attention = true;
                }
            }
        }
    }

    pub fn clear_notifications(&mut self, id: &str, pty_id: Option<&str>) {
        if let Some(ws) = self.get_mut(id) {
            if let Some(pid) = pty_id {
                if let Some(node) = ws.layout.find_terminal_by_pty(pid) {
                    if let PaneNode::Terminal { attention, .. } = node {
                        *attention = false;
                    }
                }
                let still_attentive = any_attention(&ws.layout);
                if !still_attentive {
                    ws.notifications = 0;
                }
            } else {
                ws.notifications = 0;
                let layout = ws.layout.clone();
                ws.layout = strip_attention(layout);
            }
        }
    }

    pub fn find_workspace_by_pty(&self, pty_id: &str) -> Option<(String, String)> {
        for ws in &self.workspaces {
            if let Some(node) = find_pty_in_node(&ws.layout, pty_id) {
                return Some((ws.id.clone(), node));
            }
        }
        None
    }

    pub fn split(
        &mut self,
        id: &str,
        direction: SplitDirection,
        shell: String,
        cwd: String,
        command: Option<String>,
    ) -> Result<SplitOutcome, String> {
        self.seq += 1;
        let new_id = format!("pane-{id}-{}", self.seq);
        let title = command.clone().unwrap_or_else(|| "terminal".into());
        let new_node = PaneNode::Terminal {
            id: new_id,
            pty_id: String::new(),
            shell,
            cwd,
            command,
            title,
            zoomed: false,
            attention: false,
        };
        self.split_leaf(id, direction, new_node)
    }

    pub fn split_browser(&mut self, id: &str, url: String) -> Result<SplitOutcome, String> {
        self.seq += 1;
        let new_id = format!("pane-{id}-{}", self.seq);
        let new_node = PaneNode::Browser {
            id: new_id,
            title: url_host(&url),
            url,
            zoomed: false,
            attention: false,
        };
        self.split_leaf(id, SplitDirection::Vertical, new_node)
    }

    fn split_leaf(
        &mut self,
        id: &str,
        direction: SplitDirection,
        new_node: PaneNode,
    ) -> Result<SplitOutcome, String> {
        let node_id = {
            let ws = self
                .get(id)
                .ok_or_else(|| "workspace not found".to_string())?;
            let target = ws
                .active_pane
                .clone()
                .filter(|p| ws.layout.find_ref(p).is_some())
                .or_else(|| ws.layout.first_terminal_id())
                .ok_or_else(|| "no pane to split".to_string())?;
            match ws.layout.find_ref(&target) {
                Some(node) if !matches!(node, PaneNode::Split { .. }) => node.id().to_string(),
                _ => return Err("cannot split a split node".into()),
            }
        };

        let new_node_id = new_node.id().to_string();
        self.seq += 1;
        let split_id = format!("split-{id}-{}", self.seq);

        let ws = self
            .get_mut(id)
            .ok_or_else(|| "workspace not found".to_string())?;
        let old = ws.layout.find(&node_id).unwrap().clone();
        let node = ws.layout.find(&node_id).unwrap();
        *node = PaneNode::Split {
            id: split_id,
            direction,
            ratio: 0.5,
            children: vec![old, new_node],
        };

        ws.active_pane = Some(new_node_id.clone());

        Ok(SplitOutcome {
            node_id,
            new_node_id,
        })
    }

    pub fn close_pane(&mut self, id: &str, pane_id: &str) -> Result<Vec<String>, String> {
        let (to_kill, is_root_pane, root_shell) = {
            let ws = self
                .get_mut(id)
                .ok_or_else(|| "workspace not found".to_string())?;
            let mut to_kill = Vec::new();
            if let Some(PaneNode::Terminal { pty_id, .. }) = ws.layout.find_ref(pane_id) {
                if !pty_id.is_empty() {
                    to_kill.push(pty_id.clone());
                }
            }
            let is_root_pane =
                !matches!(ws.layout, PaneNode::Split { .. }) && ws.layout.id() == pane_id;
            let root_shell = match &ws.layout {
                PaneNode::Terminal { shell, .. } => shell.clone(),
                _ => String::new(),
            };
            (to_kill, is_root_pane, root_shell)
        };

        if is_root_pane {
            self.seq += 1;
            let seq = self.seq;
            let ws = self
                .get_mut(id)
                .ok_or_else(|| "workspace not found".to_string())?;
            ws.layout =
                PaneNode::new_terminal(format!("pane-{id}-{seq}"), root_shell, ws.cwd.clone());
            ws.active_pane = ws.layout.first_terminal_id();
        } else {
            let ws = self
                .get_mut(id)
                .ok_or_else(|| "workspace not found".to_string())?;
            if !detach_terminal(&mut ws.layout, pane_id) {
                return Err("pane not found".into());
            }
            if ws.active_pane.as_deref() == Some(pane_id) {
                ws.active_pane = ws.layout.first_terminal_id();
            }
        }

        Ok(to_kill)
    }

    pub fn focus_pane(&mut self, id: &str, pane_id: &str) -> bool {
        if let Some(ws) = self.get_mut(id) {
            if ws.layout.find(pane_id).is_some() {
                ws.active_pane = Some(pane_id.to_string());
                return true;
            }
        }
        false
    }

    pub fn navigate(&mut self, id: &str, direction: NavDirection) -> Option<String> {
        let ws = self.get_mut(id)?;
        let current = ws.active_pane.clone().unwrap_or_default();
        let next = match direction {
            NavDirection::Next => ws.layout.next_terminal_id(&current),
            NavDirection::Prev => ws.layout.prev_terminal_id(&current),
        };
        if let Some(n) = next {
            ws.active_pane = Some(n.clone());
            Some(n)
        } else {
            None
        }
    }

    pub fn zoom_toggle(&mut self, id: &str, pane_id: Option<&str>) -> bool {
        let Some(ws) = self.get_mut(id) else {
            return false;
        };
        let target = pane_id
            .map(|p| p.to_string())
            .unwrap_or_else(|| ws.active_pane.clone().unwrap_or_default());
        if ws.layout.zoomed_node().is_some() {
            ws.layout.unzoom_all();
        } else {
            ws.layout.unzoom_all();
            ws.layout.set_zoomed(&target, true);
        }
        true
    }

    pub fn set_ratio(&mut self, id: &str, split_id: &str, ratio: f64) -> bool {
        let Some(ws) = self.get_mut(id) else {
            return false;
        };
        let ratio = ratio.clamp(0.1, 0.9);
        if let Some(PaneNode::Split { ratio: r, .. }) = ws.layout.find(split_id) {
            *r = ratio;
            true
        } else {
            false
        }
    }

    pub fn set_pane_title(&mut self, id: &str, pty_id: &str, title: String) {
        if let Some(ws) = self.get_mut(id) {
            if let Some(node) = ws.layout.find_terminal_by_pty(pty_id) {
                if let PaneNode::Terminal { title: t, .. } = node {
                    *t = title;
                }
            }
        }
    }

    pub fn set_pane_cwd(&mut self, id: &str, pty_id: &str, cwd: String) {
        if let Some(ws) = self.get_mut(id) {
            if let Some(node) = ws.layout.find_terminal_by_pty(pty_id) {
                if let PaneNode::Terminal { cwd: c, .. } = node {
                    *c = cwd.clone();
                }
            }
            ws.cwd = cwd;
        }
    }

    pub fn set_pane_url(&mut self, id: &str, pane_id: &str, url: String) -> bool {
        let Some(ws) = self.get_mut(id) else {
            return false;
        };
        let Some(node) = ws.layout.find(pane_id) else {
            return false;
        };
        if let PaneNode::Browser { url: u, title, .. } = node {
            *u = url;
            *title = url_host(u);
            true
        } else {
            false
        }
    }

    pub fn attach_pty(&mut self, id: &str, pane_id: &str, pty_id: String) -> bool {
        if let Some(ws) = self.get_mut(id) {
            ws.layout.set_terminal_pty(pane_id, pty_id.clone());
            ws.active_pane = Some(pane_id.to_string());
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NavDirection {
    Next,
    Prev,
}

#[derive(Debug, Clone)]
pub struct SplitOutcome {
    pub node_id: String,
    pub new_node_id: String,
}

fn find_pty_in_node(node: &PaneNode, pty_id: &str) -> Option<String> {
    match node {
        PaneNode::Terminal { id, pty_id: pid, .. } if pid == pty_id => Some(id.clone()),
        PaneNode::Split { children, .. } => children.iter().find_map(|c| find_pty_in_node(c, pty_id)),
        _ => None,
    }
}

fn url_host(url: &str) -> String {
    let trimmed = url.trim();
    let rest = trimmed
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(trimmed);
    let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    if host.is_empty() {
        "browser".into()
    } else {
        host.to_string()
    }
}

fn any_attention(node: &PaneNode) -> bool {
    match node {
        PaneNode::Terminal { attention, .. } => *attention,
        PaneNode::Browser { attention, .. } => *attention,
        PaneNode::Split { children, .. } => children.iter().any(any_attention),
    }
}

fn strip_attention(mut node: PaneNode) -> PaneNode {
    match &mut node {
        PaneNode::Terminal { attention, .. } => *attention = false,
        PaneNode::Browser { attention, .. } => *attention = false,
        PaneNode::Split { children, .. } => {
            for c in children {
                *c = strip_attention(c.clone());
            }
        }
    }
    node
}

fn detach_terminal(node: &mut PaneNode, pane_id: &str) -> bool {
    if let PaneNode::Split { children, .. } = node {
        for i in 0..children.len() {
            if !matches!(children[i], PaneNode::Split { .. }) && children[i].id() == pane_id {
                let sibling = children[1 - i].clone();
                *node = sibling;
                return true;
            }
            if detach_terminal(&mut children[i], pane_id) {
                return true;
            }
        }
    }
    false
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager_with_workspace() -> (WorkspaceManager, String) {
        let mut m = WorkspaceManager::new();
        let ws = m.create("test".into(), "pwsh.exe".into(), "C:\\work".into());
        let id = ws.id.clone();
        (m, id)
    }

    #[test]
    fn create_and_select() {
        let (mut m, id) = manager_with_workspace();
        assert_eq!(m.active_id(), Some(id.as_str()));
        assert_eq!(m.workspaces().len(), 1);
        assert!(!m.select("nope"));
        assert!(m.select(&id));
    }

    #[test]
    fn split_replaces_active_terminal() {
        let (mut m, id) = manager_with_workspace();
        let outcome = m
            .split(&id, SplitDirection::Vertical, "pwsh.exe".into(), "C:\\work".into(), None)
            .unwrap();
        let ws = m.get(&id).unwrap();
        assert!(matches!(ws.layout, PaneNode::Split { .. }));
        let mut ids = Vec::new();
        ws.layout.terminal_ids(&mut ids);
        assert_eq!(ids.len(), 2);
        assert_eq!(ws.active_pane.as_deref(), Some(outcome.new_node_id.as_str()));
    }

    #[test]
    fn split_with_command_sets_title() {
        let (mut m, id) = manager_with_workspace();
        let outcome = m
            .split(&id, SplitDirection::Horizontal, "pwsh.exe".into(), "C:\\work".into(), Some("npm run dev".into()))
            .unwrap();
        let ws = m.get_mut(&id).unwrap();
        let node = ws.layout.find(&outcome.new_node_id).unwrap();
        if let PaneNode::Terminal { title, .. } = node {
            assert_eq!(title, "npm run dev");
        } else {
            panic!("expected terminal");
        }
    }

    #[test]
    fn close_pane_collapses_to_sibling() {
        let (mut m, id) = manager_with_workspace();
        let first = m.get(&id).unwrap().layout.id().to_string();
        let outcome = m
            .split(&id, SplitDirection::Vertical, "pwsh.exe".into(), "C:\\work".into(), None)
            .unwrap();
        let killed = m.close_pane(&id, &outcome.new_node_id).unwrap();
        assert!(killed.iter().all(|k| k.is_empty()));
        assert!(matches!(m.get(&id).unwrap().layout, PaneNode::Terminal { .. }));
        assert_eq!(m.get(&id).unwrap().layout.id(), first);
    }

    #[test]
    fn close_root_terminal_resets() {
        let (mut m, id) = manager_with_workspace();
        let pane = m.get(&id).unwrap().active_pane.clone().unwrap();
        m.close_pane(&id, &pane).unwrap();
        assert!(matches!(m.get(&id).unwrap().layout, PaneNode::Terminal { .. }));
        assert!(m.get(&id).unwrap().pty_ids().is_empty());
    }

    #[test]
    fn navigate_cycles() {
        let (mut m, id) = manager_with_workspace();
        m.split(&id, SplitDirection::Vertical, "pwsh.exe".into(), "C:\\work".into(), None)
            .unwrap();
        let current = m.get(&id).unwrap().active_pane.clone().unwrap();
        let next = m.navigate(&id, NavDirection::Next).unwrap();
        assert_ne!(next, current);
        let back = m.navigate(&id, NavDirection::Prev).unwrap();
        assert_eq!(back, current);
    }

    #[test]
    fn zoom_toggle() {
        let (mut m, id) = manager_with_workspace();
        let pane = m.get(&id).unwrap().active_pane.clone().unwrap();
        assert!(m.zoom_toggle(&id, Some(&pane)));
        assert!(m.get(&id).unwrap().layout.zoomed_node().is_some());
        assert!(m.zoom_toggle(&id, Some(&pane)));
        assert!(m.get(&id).unwrap().layout.zoomed_node().is_none());
    }

    #[test]
    fn attention_marks_pane() {
        let (mut m, id) = manager_with_workspace();
        let pane = m.get(&id).unwrap().active_pane.clone().unwrap();
        let pty = "pty-1".to_string();
        m.get_mut(&id)
            .unwrap()
            .layout
            .set_terminal_pty(&pane, pty.clone());
        m.mark_attention(&id, &pty);
        let ws = m.get(&id).unwrap();
        assert_eq!(ws.notifications, 1);
        m.clear_notifications(&id, None);
        assert_eq!(m.get(&id).unwrap().notifications, 0);
    }

    #[test]
    fn reorder_workspaces() {
        let (mut m, _) = manager_with_workspace();
        let a = m.workspaces()[0].id.clone();
        let b = m.create("b".into(), "pwsh.exe".into(), "C:\\b".into()).id.clone();
        m.reorder(vec![b.clone(), a.clone()]);
        assert_eq!(m.workspaces()[0].id, b);
        assert_eq!(m.workspaces()[1].id, a);
    }
}
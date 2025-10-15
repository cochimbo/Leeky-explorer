use std::collections::HashSet;
use std::path::PathBuf;

use crate::app::PanelSide;

/// Manages selection state for both panels
/// Items are marked independently per panel
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    left_marked: HashSet<PathBuf>,
    right_marked: HashSet<PathBuf>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            left_marked: HashSet::new(),
            right_marked: HashSet::new(),
        }
    }

    /// Toggle mark on a single item
    /// Returns true if item is now marked, false if unmarked
    pub fn toggle_mark(&mut self, panel: PanelSide, path: PathBuf) -> bool {
        let marked_set = self.get_marked_set_mut(panel);
        
        if marked_set.contains(&path) {
            marked_set.remove(&path);
            false
        } else {
            marked_set.insert(path);
            true
        }
    }

    /// Mark all items in the list
    pub fn mark_all(&mut self, panel: PanelSide, paths: Vec<PathBuf>) {
        let marked_set = self.get_marked_set_mut(panel);
        
        // Toggle behavior: if all are marked, unmark all; otherwise mark all
        let all_marked = paths.iter().all(|p| marked_set.contains(p));
        
        if all_marked {
            // Unmark all
            for path in paths {
                marked_set.remove(&path);
            }
        } else {
            // Mark all
            for path in paths {
                marked_set.insert(path);
            }
        }
    }

    /// Clear all marks for a panel
    pub fn clear(&mut self, panel: PanelSide) {
        let marked_set = self.get_marked_set_mut(panel);
        marked_set.clear();
    }

    /// Clear all marks for both panels
    pub fn clear_all(&mut self) {
        self.left_marked.clear();
        self.right_marked.clear();
    }

    /// Get list of marked items for a panel
    pub fn get_marked(&self, panel: PanelSide) -> Vec<PathBuf> {
        let marked_set = self.get_marked_set(panel);
        marked_set.iter().cloned().collect()
    }

    /// Check if an item is marked
    pub fn is_marked(&self, panel: PanelSide, path: &PathBuf) -> bool {
        let marked_set = self.get_marked_set(panel);
        marked_set.contains(path)
    }

    /// Get count of marked items for a panel
    pub fn count(&self, panel: PanelSide) -> usize {
        let marked_set = self.get_marked_set(panel);
        marked_set.len()
    }

    /// Check if any items are marked in a panel
    pub fn has_marked(&self, panel: PanelSide) -> bool {
        self.count(panel) > 0
    }

    // Private helpers

    fn get_marked_set(&self, panel: PanelSide) -> &HashSet<PathBuf> {
        match panel {
            PanelSide::Left => &self.left_marked,
            PanelSide::Right => &self.right_marked,
        }
    }

    fn get_marked_set_mut(&mut self, panel: PanelSide) -> &mut HashSet<PathBuf> {
        match panel {
            PanelSide::Left => &mut self.left_marked,
            PanelSide::Right => &mut self.right_marked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_mark() {
        let mut state = SelectionState::new();
        let path = PathBuf::from("/test/file.txt");

        // Mark item
        let is_marked = state.toggle_mark(PanelSide::Left, path.clone());
        assert!(is_marked);
        assert!(state.is_marked(PanelSide::Left, &path));
        assert_eq!(state.count(PanelSide::Left), 1);

        // Unmark item
        let is_marked = state.toggle_mark(PanelSide::Left, path.clone());
        assert!(!is_marked);
        assert!(!state.is_marked(PanelSide::Left, &path));
        assert_eq!(state.count(PanelSide::Left), 0);
    }

    #[test]
    fn test_mark_all_toggle() {
        let mut state = SelectionState::new();
        let paths = vec![
            PathBuf::from("/test/file1.txt"),
            PathBuf::from("/test/file2.txt"),
            PathBuf::from("/test/file3.txt"),
        ];

        // Mark all
        state.mark_all(PanelSide::Left, paths.clone());
        assert_eq!(state.count(PanelSide::Left), 3);
        assert!(state.is_marked(PanelSide::Left, &paths[0]));

        // Toggle all (should unmark)
        state.mark_all(PanelSide::Left, paths.clone());
        assert_eq!(state.count(PanelSide::Left), 0);
        assert!(!state.is_marked(PanelSide::Left, &paths[0]));
    }

    #[test]
    fn test_mark_all_partial() {
        let mut state = SelectionState::new();
        let paths = vec![
            PathBuf::from("/test/file1.txt"),
            PathBuf::from("/test/file2.txt"),
            PathBuf::from("/test/file3.txt"),
        ];

        // Mark one manually
        state.toggle_mark(PanelSide::Left, paths[0].clone());

        // mark_all should mark the rest (not unmark)
        state.mark_all(PanelSide::Left, paths.clone());
        assert_eq!(state.count(PanelSide::Left), 3);
    }

    #[test]
    fn test_clear() {
        let mut state = SelectionState::new();
        state.toggle_mark(PanelSide::Left, PathBuf::from("/test/file1.txt"));
        state.toggle_mark(PanelSide::Left, PathBuf::from("/test/file2.txt"));
        state.toggle_mark(PanelSide::Right, PathBuf::from("/test/file3.txt"));

        assert_eq!(state.count(PanelSide::Left), 2);
        assert_eq!(state.count(PanelSide::Right), 1);

        // Clear left panel only
        state.clear(PanelSide::Left);
        assert_eq!(state.count(PanelSide::Left), 0);
        assert_eq!(state.count(PanelSide::Right), 1);
    }

    #[test]
    fn test_panels_independent() {
        let mut state = SelectionState::new();
        let path = PathBuf::from("/test/file.txt");

        // Mark in left panel
        state.toggle_mark(PanelSide::Left, path.clone());
        assert!(state.is_marked(PanelSide::Left, &path));
        assert!(!state.is_marked(PanelSide::Right, &path));

        // Mark in right panel
        state.toggle_mark(PanelSide::Right, path.clone());
        assert!(state.is_marked(PanelSide::Left, &path));
        assert!(state.is_marked(PanelSide::Right, &path));
    }

    #[test]
    fn test_get_marked() {
        let mut state = SelectionState::new();
        let path1 = PathBuf::from("/test/file1.txt");
        let path2 = PathBuf::from("/test/file2.txt");

        state.toggle_mark(PanelSide::Left, path1.clone());
        state.toggle_mark(PanelSide::Left, path2.clone());

        let marked = state.get_marked(PanelSide::Left);
        assert_eq!(marked.len(), 2);
        assert!(marked.contains(&path1));
        assert!(marked.contains(&path2));
    }
}

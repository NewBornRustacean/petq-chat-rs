use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone,Serialize, Deserialize )]
pub struct SlidingWindow {
    window: VecDeque<String>,
    capacity: usize,
}

impl SlidingWindow {
    // Initialize a new sliding window with a given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    // Add a new message to the window, removing the oldest if necessary
    pub fn add(&mut self, message: String) {
        if self.window.len() == self.capacity {
            self.window.pop_front(); // Remove the oldest message
        }
        self.window.push_back(message); // Add the new message
    }

    // Convert the window's contents into a single string
    pub fn to_string(&self) -> String {
        self.window.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::SlidingWindow;

    #[test]
    fn test_initialization() {
        let window = SlidingWindow::new(3);
        assert_eq!(window.window.len(), 0);
        assert_eq!(window.capacity, 3);
    }

    #[test]
    fn test_add_single_message() {
        let mut window = SlidingWindow::new(3);
        window.add("Hello".to_string());
        assert_eq!(window.window.len(), 1);
        assert_eq!(window.to_string(), "Hello");
    }

    #[test]
    fn test_add_multiple_messages() {
        let mut window = SlidingWindow::new(3);
        window.add("Hello".to_string());
        window.add("World".to_string());
        assert_eq!(window.window.len(), 2);
        assert_eq!(window.to_string(), "Hello\nWorld");
    }

    #[test]
    fn test_sliding_behavior() {
        let mut window = SlidingWindow::new(3);
        window.add("First".to_string());
        window.add("Second".to_string());
        window.add("Third".to_string());
        assert_eq!(window.to_string(), "First\nSecond\nThird");

        // Add a new message; the first one should be removed
        window.add("Fourth".to_string());
        assert_eq!(window.to_string(), "Second\nThird\nFourth");
    }

    #[test]
    fn test_full_capacity() {
        let mut window = SlidingWindow::new(2);
        window.add("First".to_string());
        window.add("Second".to_string());
        window.add("Third".to_string()); // This should remove "First"
        assert_eq!(window.to_string(), "Second\nThird");
    }

    #[test]
    fn test_empty_window() {
        let window = SlidingWindow::new(3);
        assert_eq!(window.to_string(), "");
    }
}

use std::collections::VecDeque;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct UndoHistory<T> {
    history: VecDeque<T>,
    current_index: usize,
}

impl<T> UndoHistory<T> {
    pub fn new(capacity: usize, initial: T) -> Self {
        if capacity == 0 {
            panic!("Capacity must be > 0");
        }

        let mut history = VecDeque::with_capacity(capacity);
        history.push_back(initial);

        Self {
            history,
            current_index: 0,
        }
    }

    pub fn undo(&mut self) -> Option<&T> {
        if self.current_index == 0 {
            return None;
        }

        self.current_index -= 1;

        self.history.get(self.current_index)
    }

    pub fn redo(&mut self) -> Option<&T> {
        if self.current_index + 1 == self.history.len() {
            return None;
        }

        self.current_index += 1;
        
        self.history.get(self.current_index)
    }

    pub fn commit_change(&mut self, value: T) {
        self.history.truncate(self.current_index + 1);

        if self.history.len() == self.history.capacity() {
            self.history.pop_front();
        }else {
            self.current_index += 1;
        }

        self.history.push_back(value);
    }

    pub fn current(&self) -> &T {
        &self.history[self.current_index]
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn capacity(&self) -> usize {
        self.history.capacity()
    }

    pub fn clear(&mut self) {
        //Last element of history is current value and should be the new initial value
        self.history.swap_remove_back(0);
        self.history.truncate(1);
        self.current_index = 0;
    }

    pub fn clear_with_new_initial(&mut self, initial_value: T) {
        self.history.clear();
        self.history.push_back(initial_value);
        self.current_index = 0;
    }
}

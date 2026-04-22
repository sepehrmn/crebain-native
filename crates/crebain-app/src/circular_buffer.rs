pub struct CircularBuffer<T> {
    buffer: Vec<Option<T>>,
    head: usize,
    count: usize,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "CircularBuffer capacity must be positive");
        let mut buffer = Vec::with_capacity(capacity);
        buffer.extend(std::iter::repeat_with(|| None).take(capacity));
        Self {
            buffer,
            head: 0,
            count: 0,
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.count {
            return None;
        }
        let tail = if self.count == self.capacity {
            self.head
        } else {
            0
        };
        let actual = (tail + index) % self.capacity;
        self.buffer[actual].as_ref()
    }

    pub fn newest(&self) -> Option<&T> {
        if self.count == 0 {
            return None;
        }
        let idx = (self.head + self.capacity - 1) % self.capacity;
        self.buffer[idx].as_ref()
    }

    pub fn oldest(&self) -> Option<&T> {
        if self.count == 0 {
            return None;
        }
        self.get(0)
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == self.capacity
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
        for i in 0..self.capacity {
            self.buffer[i] = None;
        }
    }
}

impl<T: Clone> CircularBuffer<T> {
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.count);
        for i in 0..self.count {
            if let Some(item) = self.get(i) {
                result.push(item.clone());
            }
        }
        result
    }
}
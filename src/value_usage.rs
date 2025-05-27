#[derive(Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct NumberId(usize);

impl NumberId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn id(&self) -> usize {
        self.0
    }
}

struct NumberUsage {
    values: Vec<f64>,
}

impl NumberUsage {
    pub(crate) fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub(crate) fn number_node(&mut self, number: f64) {
        self.values.push(number);
    }

    pub(crate) fn number_value(&self, number_id: NumberId) -> f64 {
        self.values[number_id.id()]
    }
}

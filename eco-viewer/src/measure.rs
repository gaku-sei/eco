use tracing::info;

/// Poor man's performance measure

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum Precision {
    Ns,
    Ms,
    S,
}

#[allow(unused)]
pub struct Measure {
    label: String,
    start: std::time::Duration,
    precision: Precision,
}

impl Clone for Measure {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            start: self.start,
            precision: self.precision,
        }
    }
}

impl Measure {
    #[allow(unused)]
    #[must_use]
    /// ## Panics
    pub fn new(label: &str, precision: Precision) -> Self {
        Self {
            label: label.to_string(),
            start: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap(),
            precision,
        }
    }

    #[must_use]
    #[allow(unused)]
    pub fn set_label(mut self, new_label: &str) -> Self {
        self.label = new_label.to_string();
        self
    }
}

impl Drop for Measure {
    fn drop(&mut self) {
        let end = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let duration = end - self.start;
        match self.precision {
            Precision::Ns => info!("{}: {}ns", self.label, duration.as_nanos()),
            Precision::Ms => info!("{}: {}ms", self.label, duration.as_millis()),
            Precision::S => info!("{}: {}s", self.label, duration.as_secs()),
        };
    }
}

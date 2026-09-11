use std::sync::{Arc, Mutex, MutexGuard};

use flume::{Direction, FlumeElem, FlumeStyle, FlumeWrap, Size, Space};
use verdant::shapes::Style;

#[derive(Clone)]
pub struct SenStyle {
    pub verdant: Arc<Mutex<Style>>,
    pub flume: FlumeElem,
}

impl SenStyle {
    pub fn new() -> Self {
        Self {
            verdant: Arc::new(Mutex::new(Default::default())),
            flume: FlumeStyle::mutable(Size::take(0., 0.), Space::ShrinkWrapped, None).wrap()
        }
    }

    pub fn sync() -> Self {
        Self {
            verdant: Arc::new(Mutex::new(Default::default())),
            flume: FlumeStyle::sync(Size::take(0., 0.), Space::ShrinkWrapped, None).wrap()
        }
    }

    pub fn verdant(&self) -> MutexGuard<'_, Style> {
        match self.verdant.lock() {
            Ok(v) => v,
            Err(e) => e.into_inner(),
        }
    }
}

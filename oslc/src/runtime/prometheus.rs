use std::sync::Arc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

pub struct MetricsRegistry {
    counters: HashMap<String, Arc<AtomicU64>>,
    gauges: HashMap<String, Arc<AtomicUsize>>,
    histograms: HashMap<String, Histogram>,
}

pub struct Histogram {
    values: Arc<std::sync::Mutex<Vec<u64>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        MetricsRegistry {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }
    
    pub fn counter(&mut self, name: &str) -> Arc<AtomicU64> {
        let counter = Arc::new(AtomicU64::new(0));
        self.counters.insert(name.to_string(), Arc::clone(&counter));
        counter
    }
    
    pub fn gauge(&mut self, name: &str) -> Arc<AtomicUsize> {
        let gauge = Arc::new(AtomicUsize::new(0));
        self.gauges.insert(name.to_string(), Arc::clone(&gauge));
        gauge
    }
    
    pub fn histogram(&mut self, name: &str) -> &Histogram {
        let hist = Histogram {
            values: Arc::new(std::sync::Mutex::new(Vec::new())),
        };
        self.histograms.insert(name.to_string(), hist);
        self.histograms.get(name).unwrap()
    }
    
    pub fn scrape(&self) -> String {
        let mut output = String::new();
        for (name, counter) in &self.counters {
            output.push_str(&format!("{} {}\n", name, counter.load(Ordering::Relaxed)));
        }
        for (name, gauge) in &self.gauges {
            output.push_str(&format!("{} {}\n", name, gauge.load(Ordering::Relaxed)));
        }
        output
    }
}

impl Histogram {
    pub fn observe(&self, value: u64) {
        self.values.lock().unwrap().push(value);
    }
}

pub fn counter(name: &str) -> Arc<AtomicU64> {
    static REGISTRY: std::sync::Mutex<Option<MetricsRegistry>> = std::sync::Mutex::new(None);
    let mut guard = REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(MetricsRegistry::new());
    }
    if let Some(ref mut reg) = *guard {
        reg.counter(name)
    } else {
        Arc::new(AtomicU64::new(0))
    }
}

pub fn gauge(name: &str) -> Arc<AtomicUsize> {
    static REGISTRY: std::sync::Mutex<Option<MetricsRegistry>> = std::sync::Mutex::new(None);
    let mut guard = REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(MetricsRegistry::new());
    }
    if let Some(ref mut reg) = *guard {
        reg.gauge(name)
    } else {
        Arc::new(AtomicUsize::new(0))
    }
}

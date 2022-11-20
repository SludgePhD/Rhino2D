use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

#[derive(Debug)]
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    pub fn into_inner(self) -> f32 {
        f32::from_bits(self.0.into_inner())
    }

    pub fn load(&self, order: Ordering) -> f32 {
        f32::from_bits(self.0.load(order))
    }

    pub fn store(&self, val: f32, order: Ordering) {
        self.0.store(val.to_bits(), order);
    }

    pub fn swap(&self, val: f32, order: Ordering) -> f32 {
        f32::from_bits(self.0.swap(val.to_bits(), order))
    }
}

#[derive(Debug)]
pub struct AtomicF32x2(AtomicU64);

impl AtomicF32x2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self(AtomicU64::new(f32x2_to_u64(x, y)))
    }

    pub fn into_inner(self) -> [f32; 2] {
        u64_to_f32x2(self.0.into_inner())
    }

    pub fn load(&self, order: Ordering) -> [f32; 2] {
        u64_to_f32x2(self.0.load(order))
    }

    pub fn store(&self, x: f32, y: f32, order: Ordering) {
        self.0.store(f32x2_to_u64(x, y), order);
    }

    pub fn swap(&mut self, x: f32, y: f32, order: Ordering) -> [f32; 2] {
        u64_to_f32x2(self.0.swap(f32x2_to_u64(x, y), order))
    }
}

fn f32x2_to_u64(x: f32, y: f32) -> u64 {
    u64::from(x.to_bits()) | (u64::from(y.to_bits()) << 32)
}

fn u64_to_f32x2(u: u64) -> [f32; 2] {
    let x = f32::from_bits(u as u32);
    let y = f32::from_bits((u >> 32) as u32);
    [x, y]
}

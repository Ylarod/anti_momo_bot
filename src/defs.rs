use std::sync::atomic::AtomicBool;

pub const BAN_DURATION: i64 = 60 * 60;
pub static USE_OCR: AtomicBool = AtomicBool::new(false);

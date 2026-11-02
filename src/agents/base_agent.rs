#![allow(dead_code)]

pub trait Agent {
    fn name(&self) -> &str;
}

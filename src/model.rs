#![allow(unused)]

use crate::{Error, Result};
#[derive(Debug)]
struct Edge {
    id: String,
    source: String,
    sink: String,
}

#[derive(Debug)]
struct Node {
    id: String,
    pos: (u64, u64),
}

#[derive(Debug)]
struct Layout {
    id: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

pub struct ModelController {}

impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}

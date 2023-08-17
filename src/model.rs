#![allow(unused)]

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};

use crate::routes::InputLayout;
use crate::{Error, Result};

// these are mostly copy of input* struct defined in routes.rs
// the point being the model (MVC) can/should be different from input
#[derive(Debug, Clone)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub sink: String,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub position: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub id: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Default)]
pub struct Model(Option<Arc<Layout>>);

// Slightly complex datastructure here
//
// the server start with an empty layout.
//
// - any /query queries will always return an error till a valid map is updated
// - queries on /query will Arc::clone the model after a read-lock
//      - during a readlock, map updates (write) will be blocked,
//      - and during write, read locks (/query will be blocked)
//      - however the lock are held only for Arc::clone() and simple assignment
//      - which should be pretty fast
//
//
// The update is the tricky part;
// the currently running queries on /query should continue their evaluation
// while being able to concurrently update the the layout.

#[derive(Clone)]
pub struct ModelController {
    layout: Arc<RwLock<Model>>,
}

impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            layout: Arc::default(),
        })
    }

    pub async fn layout(&self) -> Result<Arc<Layout>> {
        match self.layout.clone().read() {
            Ok(m) => (*m)
                .0
                .clone()
                .and_then(|l| Some(l.clone()))
                .ok_or(Error::NoValidMapDefined),
            // this is very much unlikely.
            // this means a thread holding a write lock died, given thtat
            // it is an assingment only critical section
            Err(_) => Err(Error::LayoutUpdateFailureDetected),
        }
    }
    pub async fn update(&self, input_layout: InputLayout) -> Result<()> {
        let nodes: Vec<_> = input_layout
            .nodes
            .iter()
            .map(|node| Node {
                id: node.id.to_owned(),
                position: (node.position.x, node.position.y),
            })
            .collect();
        let edges = input_layout
            .edges
            .iter()
            .map(|edge| Edge {
                id: edge.id.to_owned(),
                source: edge.source.to_owned(),
                sink: edge.sink.to_owned(),
            })
            .collect();
        let model = Model(Some(Arc::new(Layout {
            id: input_layout.id.to_owned(),
            nodes,
            edges,
        })));
        match self.layout.write() {
            Ok(mut m) => {
                *m = model;
                Ok(())
            }
            // this is very much unlikely as at this point the needed lock is already held
            Err(_) => Err(Error::LayoutUpdateFailureDetected),
        }
    }
}

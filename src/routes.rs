#![allow(unused)]
use crate::{Error, Result};
use std::{
    alloc::Layout,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct InputEdge {
    id: String,
    source: String,
    sink: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
struct InputPosition {
    x: u64,
    y: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct InputNode {
    id: String,
    position: InputPosition,
}

#[derive(Debug, Deserialize)]
struct InputLayout {
    id: String,
    nodes: Vec<InputNode>,
    edges: Vec<InputEdge>,
}

pub fn routes() -> Router {
    Router::new().route("/validate", post(handler_validate))
}

#[derive(Debug)]
struct Edge {
    id: String,
    source: Arc<Node>,
    sink: Arc<Node>,
}

#[derive(Debug)]
struct Node {
    input_node: Arc<InputNode>,
    edges: Vec<Arc<Edge>>,
}

async fn handler_validate(Json(layout): Json<InputLayout>) -> Result<()> {
    let ids = layout
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();

    layout.nodes.iter().try_for_each(|node| {
        ids.contains(&node.id.as_str())
            .then_some({})
            .ok_or(Error::NodesMustBeUnique)
    })?;

    {
        let positions = layout.nodes.iter().map(|node| node.position);
        let uniq_count = positions
            .clone()
            .collect::<HashSet<_>>()
            .into_iter()
            .count();
        if positions.count() != uniq_count {
            return Err(Error::NodesMustBeUnique);
        }
    }

    layout.edges.iter().try_for_each(|edge| {
        edge;
        (ids.contains(&edge.source.as_str()) && ids.contains(&edge.sink.as_str()))
            .then_some({})
            .ok_or(Error::InvalidNodeInEdge)
    })?;

    let mut node_count: HashMap<&str, u32> = HashMap::new();
    layout.edges.iter().for_each(|edge| {
        node_count
            .entry(edge.source.as_str())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        node_count
            .entry(edge.sink.as_str())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    });
    println!("{node_count:?}");
    node_count.values().try_for_each(|count| {
        (dbg!(*count) >= 2)
            .then_some({})
            .ok_or(Error::NodeNeedsMoreDriveways)
    })?;

    //     let mut node_map: HashMap<&str, Arc<Node>> = layout
    //         .nodes
    //         .iter()
    //         .map(|node| {
    //             (
    //                 dbg!(node.id.as_str()),
    //                 Arc::new(Node {
    //                     input_node: Arc::new(node.clone()),
    //                     edges: vec![],
    //                 }),
    //             )
    //         })
    //         .collect();
    //
    //     println!("{node_map:?}");
    //
    //     let edges: Vec<_> = layout
    //         .edges
    //         .iter()
    //         .map(|edge| {
    //             let source_n = node_map.get(edge.source.as_str()).unwrap().clone();
    //             let sink_n = node_map.get(edge.sink.as_str()).unwrap().clone();
    //             let e = Arc::new(Edge {
    //                 id: edge.id.to_owned(),
    //                 source: source_n,
    //                 sink: sink_n,
    //             });
    //             let node = Arc::get_mut(node_map.get_mut(edge.source.as_str()).unwrap()).unwrap();
    //             node.edges.push(e.clone());
    //             e
    //         })
    //         .collect();
    //
    //     println!("{edges:?}");

    Ok(())
}

#![allow(unused)]
use crate::{model::ModelController, Error, Result};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct InputEdge {
    pub id: String,
    pub source: String,
    pub sink: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct InputPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InputNode {
    pub id: String,
    pub position: InputPosition,
}

#[derive(Debug, Deserialize)]
pub struct InputLayout {
    pub id: String,
    pub nodes: Vec<InputNode>,
    pub edges: Vec<InputEdge>,
}

pub fn routes(mc: ModelController) -> Router {
    Router::new()
        .route("/query", get(handler_query))
        .route("/validate", post(handler_validate))
        .with_state(mc)
}
async fn handler_validate(
    State(mc): State<ModelController>,
    Json(layout): Json<InputLayout>,
) -> Result<()> {
    println!("->> {:12} - handler_validate", "HANDLER");

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
        let seen = vec![];
        let posi = layout
            .nodes
            .iter()
            .map(|node| (node.position.x, node.position.y))
            .try_for_each(|(x, y)| {
                if seen.contains(&(x, y)) {
                    Err(Error::NodesMustBeUnique)
                } else {
                    Ok(())
                }
            });
    }

    {
        // another way to handle uniques, extra scope to explicitly drop the intermediaries
        let edges = layout.edges.iter().map(|edge| edge.id.as_str());
        let uniq_count = edges.clone().collect::<HashSet<_>>().into_iter().count();
        if edges.count() != uniq_count {
            return Err(Error::EdgesMustBeUnique);
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

    node_count.values().try_for_each(|count| {
        ((*count) >= 2)
            .then_some({})
            .ok_or(Error::NodeNeedsMoreDriveways)
    })?;

    mc.update(layout).await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct PathQueryParams {
    start: String,
    goal: String,
}

/// handler for /query path
///
/// needs to query params
/// start and goal. Both should be valid node in the map
pub async fn handler_query(
    State(mc): State<ModelController>,
    Query(mut params): Query<PathQueryParams>,
) -> Result<Json<Vec<String>>> {
    println!("->> {:12} - handler_query {params:?}", "HANDLER");
    let layout = mc.layout().await?;

    let ids = layout
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();

    (ids.contains(&params.start.as_str()) && ids.contains(&params.goal.as_str()))
        .then_some({})
        .ok_or(Error::NodesNotFound)?;

    let mut map = HashMap::new();

    layout.edges.iter().for_each(|edge| {
        map.entry(edge.source.as_str())
            .and_modify(|sinks: &mut Vec<_>| sinks.push(edge.sink.as_str()))
            .or_insert(vec![edge.sink.as_str()]);
    });

    let mut visited: Vec<(&str, Vec<&str>)> = vec![];
    visited.push((params.start.as_str(), vec![]));

    // this does not handle cycles at all
    // there are various ways to handle them but would leave it out of scope for now
    let path = while !visited.is_empty() {
        let (n, mut path) = visited.pop().unwrap(); // just checked to be not empty
        if n == params.goal.as_str() {
            path.push(params.goal.as_str());
            return Ok(Json(path.iter().map(|node| node.to_string()).collect()));
        }
        map.get(n).unwrap().iter().for_each(|nn| {
            let mut pp = path.clone();
            pp.push(n);
            visited.push((nn, pp));
        });
    };

    Err(Error::PathNotFound)
}

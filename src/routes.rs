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
#[derive(Debug, Deserialize)]
pub struct InputEdge {
    pub id: String,
    pub source: String,
    pub sink: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
pub struct InputPosition {
    pub x: u64,
    pub y: u64,
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
    println!(" ->> {:12} - handler_validate", "HANDLER");

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
        // another way to handle uniques, extra scope to explicitly drop the intermediaries
        let posi = layout.nodes.iter().map(|node| node.position);
        let uniq_count = posi.clone().collect::<HashSet<_>>().into_iter().count();
        if posi.count() != uniq_count {
            return Err(Error::NodesMustBeUnique);
        }
    }

    {
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

#[derive(Deserialize)]
pub struct PathQueryParams {
    start: String,
    goal: String,
}

pub async fn handler_query(
    State(mc): State<ModelController>,
    Query(params): Query<PathQueryParams>,
) -> Result<()> {
    println!(" ->> {:12} - handler_query", "HANDLER");
    let layout = mc.layout().await?;

    let ids: Vec<&str> = layout
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();

    (ids.contains(&params.start.as_str()) && ids.contains(&params.goal.as_str()))
        .then_some({})
        .ok_or(Error::NodesNotFound)?;
    Ok(())
}

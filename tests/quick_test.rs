#![allow(unused)]

use anyhow::Result;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn quick_test() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8001")?;

    hc.do_get("/ping?echo=HelloWorld").await?.print().await?;
    hc.do_get("/ping").await?.print().await?;

    Ok(())
}
//
#[tokio::test]
async fn test_duplicate_id() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8001")?;
    //
    let resp = hc
        .do_post(
            "/api/validate",
            json!({
                "id": "valid_map",
                "nodes":[{"id": "node1", "position" : {"x": 1,"y":1 }},
                         {"id": "node1", "position" : {"x": 1,"y":1 }}],
                "edges": [{"id": "BL_2_BC", "source": "Node_BL", "sink": "Node_BC" }]
            }),
        )
        .await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json_body()?,
        json!({"error":{"details": "nodes must be unique"}})
    );
    //
    Ok(())
}
//
#[tokio::test]
async fn test_invalid_node_in_edge() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8001")?;
    //
    let resp = hc
        .do_post(
            "/api/validate",
            json!({
                "id": "valid_map",
                "nodes":[{"id": "node1", "position" : {"x": 1,"y":1 }} ],
                "edges": [{"id": "BL_2_BC", "source": "Node_BL", "sink": "Node_BC" }]
            }),
        )
        .await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json_body()?,
        json!({"error":{"details": "invalid node in edge"}})
    );
    //
    Ok(())
}

#[tokio::test]
async fn test_invalid_number_of_nodes() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8001")?;

    let resp = hc
        .do_post(
            "/api/validate",
            json!({
                "id": "valid_map",
                "nodes":[{"id": "node1", "position" : {"x": 1,"y":1 }},
                         {"id": "node2", "position" : {"x": 2,"y":1 }}],
                "edges": [{"id": "BL_2_BC", "source": "node2", "sink": "node1" }]
            }),
        )
        .await?;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json_body()?,
        json!({"error":{"details": "node does not have enough number of driveways"}})
    );

    Ok(())
}

#[tokio::test]
async fn test_invalid_nodes_in_query() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8001")?;

    let resp = hc
        .do_post(
            "/api/validate",
            json!({
                "id": "valid_map",
                "nodes":[{"id": "node1", "position" : {"x": 1,"y":1 }},
                         {"id": "node2", "position" : {"x": 2,"y":1 }}],
                "edges": [{"id": "2 to 1", "source": "node2", "sink": "node1" },
                          {"id": "1 to 2", "source": "node1", "sink": "node2" }
                ]
            }),
        )
        .await?;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = hc.do_get("/api/query?start=node1&goal=node2").await?;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = hc
        .do_post(
            "/api/validate",
            json!({
                "id": "valid_map",
                "nodes":[{"id": "A", "position" : {"x": 1,"y":1 }},
                         {"id": "B", "position" : {"x": 2,"y":1 }}],
                "edges": [{"id": "2 to 1", "source": "A", "sink": "B" },
                          {"id": "1 to 2", "source": "B", "sink": "A" }
                ]
            }),
        )
        .await?;
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = hc.do_get("/api/query?start=node1&goal=node2").await?;
    assert_eq!(
        resp.json_body()?,
        json!({"error":{"details": "the provided nodes were not found"}})
    );
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    Ok(())
}

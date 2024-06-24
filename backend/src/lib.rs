mod railway;

use railway::Railway;

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use derive_get::Getters;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use tracing::{error, info};

pub type Result<T, E = Error> = std::result::Result<T, E>;

static PROJECTS: &str = include_str!("graphql/projects.gql");
static SERVICES: &str = include_str!("graphql/services.gql");

pub fn router() -> Router {
    Router::new().route("/v1/projects", post(projects)).route("/v1/services", post(services))
}

pub async fn serve(app: Router, port: u16) -> color_eyre::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening: {addr}");
    Ok(axum::serve(listener, app).await?)
}

#[derive(Getters, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    id: String,
    name: String,
}

pub async fn projects(headers: HeaderMap) -> Result<Json<Vec<Project>>> {
    let mut token = headers
        .get("Authorization")
        .ok_or(Error::AuthorizationMissing)?
        .to_str()?
        .to_owned();
    if token.starts_with("Bearer ") {
        token.drain(.."Bearer ".len());
    } else {
        return Err(Error::AuthorizationMissing);
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ProjectsRailwayResponseEdgesEdgeNode {
        id: String,
        name: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ProjectsRailwayResponseEdgesEdge {
        node: ProjectsRailwayResponseEdgesEdgeNode
    }

    #[derive(Deserialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    struct ProjectsRailwayResponseEdges {
        edges: Vec<ProjectsRailwayResponseEdgesEdge>,
    }

    #[derive(Deserialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    struct ProjectsRailwayResponse {
        projects: ProjectsRailwayResponseEdges,
    }

    let response: ProjectsRailwayResponse = Railway::query(
        serde_json::json!({ "query": PROJECTS }),
        &token,
    )
    .await?;
    
    let mut projects = Vec::with_capacity(response.projects.edges.len());
    for project in response.projects.edges {
        projects.push(Project { id: project.node.id, name: project.node.name });
    }

    Ok(Json(projects))
}

#[derive(Getters, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicesRequest {
    project_id: String,
}

#[derive(Getters, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    id: String,
    name: String,
    health_check_url: Option<String>,
}

pub async fn services(headers: HeaderMap, Json(req): Json<ServicesRequest>) -> Result<Json<Vec<Service>>> {
    let mut token = headers
        .get("Authorization")
        .ok_or(Error::AuthorizationMissing)?
        .to_str()?
        .to_owned();
    if token.starts_with("Bearer ") {
        token.drain(.."Bearer ".len());
    } else {
        return Err(Error::AuthorizationMissing);
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponseProjectServicesEdgeNodeInstancesEdgesNode {
        healthcheck_path: Option<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponseProjectServicesEdgeNodeInstancesEdges {
        node: ServicesRailwayResponseProjectServicesEdgeNodeInstancesEdgesNode
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponseProjectServicesEdgeNodeInstances {
        edges: Vec<ServicesRailwayResponseProjectServicesEdgeNodeInstancesEdges>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponseProjectServicesEdgeNode {
        id: String,
        name: String,
        service_instances: ServicesRailwayResponseProjectServicesEdgeNodeInstances,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponseProjectServicesEdge {
        node: ServicesRailwayResponseProjectServicesEdgeNode
    }

    #[derive(Deserialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponseProjectServices {
        edges: Vec<ServicesRailwayResponseProjectServicesEdge>,
    }

    #[derive(Deserialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponseProject {
        services: ServicesRailwayResponseProjectServices,
    }

    #[derive(Deserialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    struct ServicesRailwayResponse {
        project: ServicesRailwayResponseProject,
    }

    let response: ServicesRailwayResponse = Railway::query(
        serde_json::json!({ "query": SERVICES, "variables": { "projectId": req.project_id } }),
        &token,
    )
    .await?;
    
    let mut services = Vec::with_capacity(response.project.services.edges.len());
    for service in response.project.services.edges {
        let health_check_url = service.node.service_instances.edges.iter().find(|i| i.node.healthcheck_path.is_some()).and_then(|i| i.node.healthcheck_path.to_owned());
        services.push(Service { id: service.node.id, name: service.node.name,
            health_check_url,
        });
    }

    Ok(Json(services))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ToStr(#[from] axum::http::header::ToStrError),
    #[error("authorization missing")]
    AuthorizationMissing,
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("railway responded with: {0:?}")]
    Railway(Vec<String>),
    #[error("railway data missing: {0}")]
    RailwayDataMissing(&'static str),
    #[error("railway request failed with status {0}: {1}")]
    RailwayStatusFailure(u16, String),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::AuthorizationMissing => (StatusCode::UNAUTHORIZED, "Unauthorized".to_owned()),
            // Internal failures
            err => {
                error!("{err})");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_owned(),
                )
            }
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

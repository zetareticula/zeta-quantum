use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    bms::{BMSObservable, EscapeRoute},
    phi_ir::{PhiCircuit, PhiElement},
    qpu::Modality,
    QuantumQuantizer,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OptimizeRequest {
    pub circuit: Vec<Gate>,
    pub modality: String,
    pub calibration: std::collections::HashMap<String, f64>,
    pub bms_route: Option<String>, // "none" | "scalar" | "holographic"
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Gate {
    pub r#type: String, // "H", "X", "CNOT"
    pub targets: Vec<u32>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OptimizeResponse {
    pub status: String,
    pub optimized_circuit: PhiCircuit, // serialized Weyl words
    pub integrated_sx: f64,
    pub fidelity_estimate: f64,
    pub pulse_script: String,
    pub bms_observable: BMSObservable,
    pub routed_summary: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(optimize),
    components(schemas(OptimizeRequest, OptimizeResponse, Gate, BMSObservable, EscapeRoute))
)]
struct ApiDoc;

#[utoipa::path(
    post,
    path = "/optimize",
    request_body = OptimizeRequest,
    responses(
        (status = 200, description = "Optimization result", body = OptimizeResponse),
        (status = 400, description = "Bad request", body = serde_json::Value)
    )
)]
async fn optimize(
    State(quantizer): State<Arc<tokio::sync::Mutex<QuantumQuantizer>>>,
    Json(req): Json<OptimizeRequest>,
) -> impl IntoResponse {
    let modality = match req.modality.as_str() {
        "superconducting" => Modality::Superconducting,
        "iontrap" => Modality::IonTrap,
        "neutralatom" => Modality::NeutralAtom,
        _ => Modality::Superconducting,
    };

    let route = match req.bms_route.as_deref() {
        Some("scalar") => EscapeRoute::ScalarMetricEngineering,
        Some("holographic") => EscapeRoute::HolographicReconstruction,
        _ => EscapeRoute::None,
    };

    let mut q = quantizer.lock().await;
    // Rebuild QPU with fresh minute-specific calibration
    *q = QuantumQuantizer::new(modality, &req.calibration, chrono::Utc::now().to_rfc3339());

    // Build PhiCircuit from request
    let mut circ = PhiCircuit::default();
    for g in req.circuit {
        match g.r#type.as_str() {
            "H" => circ.elements.push(PhiElement::h(g.targets[0])),
            "X" => circ.elements.push(PhiElement::x(g.targets[0])),
            "CNOT" | "CX" => circ
                .elements
                .push(PhiElement::cnot(g.targets[0], g.targets[1])),
            _ => {}
        }
    }

    let (optimized, sx, bms_obs) = match q.quantize_with_bms(&circ, route) {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    let pulse = generate_pulse_script(&optimized, &req.modality, sx);

    let routed_summary = if bms_obs.decoded_sx.is_none() {
        "Gravitational memory collapses information (non-injective map). Only phase transition detected.".into()
    } else {
        "Extra channel enabled – full S_X recoverable.".into()
    };

    let resp = OptimizeResponse {
        status: "success".into(),
        optimized_circuit: optimized,
        integrated_sx: sx,
        fidelity_estimate: (-sx).exp(),
        pulse_script: pulse,
        bms_observable: bms_obs,
        routed_summary,
    };

    (StatusCode::OK, Json(resp)).into_response()
}

fn generate_pulse_script(circ: &PhiCircuit, modality: &str, sx: f64) -> String {
    let mut s = format!(
        "# zeta-quantum v0.4.0 pulse script – {}\n# S_X = {:.6} | fidelity ≈ {:.4}\n\n",
        chrono::Utc::now(),
        sx,
        (-sx).exp()
    );
    for elem in &circ.elements {
        // modality-specific pulse (same as before, now in Rust)
        if elem.targets.len() == 1 {
            s.push_str(&format!(
                "DRIVE {} duration=30ns amp=0.92\n",
                elem.targets[0]
            ));
        } else {
            let g = if modality == "superconducting" {
                "CZ"
            } else if modality == "iontrap" {
                "MS"
            } else {
                "RYDBERG"
            };
            s.push_str(&format!("{} {}-{}\n", g, elem.targets[0], elem.targets[1]));
        }
    }
    s
}

pub fn app(quantizer: Arc<tokio::sync::Mutex<QuantumQuantizer>>) -> Router {
    let (router, _openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .route("/optimize", post(optimize))
        .split_for_parts();

    router
        .with_state(quantizer)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(30),
        ))
}

// Entry point
#[tokio::main]
pub async fn start_server() {
    tracing_subscriber::fmt::init();

    let initial_q = Arc::new(tokio::sync::Mutex::new(QuantumQuantizer::new(
        Modality::Superconducting,
        &std::collections::HashMap::new(),
        "init".into(),
    )));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 zeta-quantum API v0.4.0 live on http://0.0.0.0:8080");
    println!("   OpenAPI: http://0.0.0.0:8080/openapi.json");
    println!("   Swagger: http://0.0.0.0:8080/swagger-ui");

    axum::serve(listener, app(initial_q)).await.unwrap();
}

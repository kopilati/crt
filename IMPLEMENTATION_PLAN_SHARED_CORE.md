# Shared Core Library Implementation Plan

## Overview

Create a shared Rust library (`crt-core`) that contains:
- Request/Response type definitions
- DORA metric configurations and translation logic
- Validation logic
- Common utilities

This library will be used by:
- **Backend**: Direct Rust dependency
- **Frontend**: Compiled to WASM and imported as a JavaScript module

## Benefits

1. **Single Source of Truth**: DORA configurations, validation rules, and types defined once
2. **Type Safety**: Frontend gets compile-time guarantees matching backend
3. **Consistency**: No more drift between frontend and backend logic
4. **Maintainability**: Changes to business logic only need to be made in one place
5. **Performance**: WASM provides near-native performance for validation/translation
6. **Testing**: Shared test suite ensures both sides behave identically

## Project Structure

```
crt/
├── crates/
│   ├── crt-core/           # Shared library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs    # Request/Response types
│   │   │   ├── dora.rs     # DORA metrics config & translation
│   │   │   ├── validation.rs # Validation logic
│   │   │   └── wasm.rs     # WASM bindings
│   │   └── tests/
│   ├── crt-backend/        # Backend service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── crt-frontend/       # Frontend WASM module
│       ├── Cargo.toml
│       ├── src/
│       │   └── lib.rs
│       └── pkg/            # Generated WASM package
└── services/
    ├── backend/            # Current backend (to be migrated)
    └── agents/             # Agent services
```

## Implementation Steps

### Phase 1: Create Core Library

#### 1.1 Create `crt-core` Crate

```bash
cd /Users/rune/projects/crt
mkdir -p crates/crt-core/src
cd crates/crt-core
```

**`Cargo.toml`:**
```toml
[package]
name = "crt-core"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = "0.3"

[dependencies.wasm-bindgen-futures]
version = "0.4"

[features]
default = ["console_error_panic_hook"]

[dependencies.console_error_panic_hook]
version = "0.1.6"
optional = true
```

#### 1.2 Move Types to Core Library

**`crates/crt-core/src/types.rs`:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DoraMetrics {
    pub deployment_frequency: f32,
    pub lead_time: f32,
    pub change_failure_rate: f32,
    pub mttr: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EngineeringMetrics {
    pub commit_frequency: f32,
    pub branch_lifetime: f32,
    pub pbis_delivered_per_sprint_per_team: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeAllocation {
    pub meetings: i32,
    pub unplanned: i32,
    pub bugs: i32,
    pub feature: i32,
    pub tech_debt: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyseRequest {
    pub crt: String,
    pub dora_metrics: DoraMetrics,
    pub extended_engineering_metrics: EngineeringMetrics,
    pub westrum: f32,
    pub time_allocation: TimeAllocation,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DoraMetric {
    pub value: f32,
    pub unit: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisResult {
    pub executive_summary: String,
    pub core_systemic_issues: Vec<CoreSystemicIssue>,
    pub leverage_points: Vec<LeveragePoint>,
    pub systemic_relationships: Vec<String>,
    pub assumptions: Vec<String>,
    pub analysis_confidence: String,
    pub analysis_metadata: Option<AnalysisMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CoreSystemicIssue {
    pub issue: String,
    pub causes: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeveragePoint {
    pub constraint: String,
    pub rationale: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisMetadata {
    pub confidence_score: String,
    pub data_completeness: String,
    pub analysis_timestamp: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisResponse {
    pub run_id: String,
    pub result: AnalysisResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluateRequest {
    pub original_payload: AnalyseRequest,
    pub analysis_result: AnalysisResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EvaluationResponse {
    pub run_id: String,
    pub result: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalyseWithFeedbackRequest {
    pub original_payload: AnalyseRequest,
    pub analysis_result: AnalysisResult,
    pub evaluation: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefineRequest {
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefineResponse {
    pub run_id: Option<String>,
    // Flattened AgentRefinement fields
}
```

#### 1.3 Move DORA Configuration to Core

**`crates/crt-core/src/dora.rs`:**
```rust
use serde::{Deserialize, Serialize};
use crate::types::DoraMetric;

#[derive(Debug, Clone)]
pub struct DoraMetricConfig {
    pub min_value: f32,
    pub max_value: f32,
    pub unit: &'static str,
    pub inverted: bool,
}

impl DoraMetricConfig {
    pub fn translate(&self, slider_value: f32) -> DoraMetric {
        let translated_value = if self.inverted {
            self.max_value - (self.max_value - self.min_value) * slider_value
        } else {
            self.min_value + (self.max_value - self.min_value) * slider_value
        };

        DoraMetric {
            value: (translated_value * 1000.0).round() / 1000.0,
            unit: self.unit.to_string(),
        }
    }
}

pub const DORA_METRIC_CONFIGS: &[(&str, DoraMetricConfig)] = &[
    ("deployment_frequency", DoraMetricConfig {
        min_value: 0.025,
        max_value: 10.0,
        unit: "deployments/day",
        inverted: false,
    }),
    ("lead_time", DoraMetricConfig {
        min_value: 0.1,
        max_value: 60.0,
        unit: "days",
        inverted: true,
    }),
    ("change_failure_rate", DoraMetricConfig {
        min_value: 0.0,
        max_value: 100.0,
        unit: "%",
        inverted: true,
    }),
    ("mttr", DoraMetricConfig {
        min_value: 0.05,
        max_value: 14.0,
        unit: "days",
        inverted: true,
    }),
    ("commit_frequency", DoraMetricConfig {
        min_value: 0.25,
        max_value: 10.0,
        unit: "commits/day per developer",
        inverted: false,
    }),
    ("branch_lifetime", DoraMetricConfig {
        min_value: 0.05,
        max_value: 30.0,
        unit: "days",
        inverted: true,
    }),
];

pub fn translate_dora_metrics_for_agent(dora_metrics: &crate::types::DoraMetrics) -> serde_json::Value {
    let get_config = |metric_name: &str| -> &DoraMetricConfig {
        DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == metric_name)
            .map(|(_, config)| config)
            .expect("Unknown DORA metric")
    };

    serde_json::json!({
        "deployment_frequency": get_config("deployment_frequency").translate(dora_metrics.deployment_frequency),
        "lead_time": get_config("lead_time").translate(dora_metrics.lead_time),
        "change_failure_rate": get_config("change_failure_rate").translate(dora_metrics.change_failure_rate),
        "mttr": get_config("mttr").translate(dora_metrics.mttr),
    })
}

pub fn translate_engineering_metrics_for_agent(engineering_metrics: &crate::types::EngineeringMetrics) -> serde_json::Value {
    let get_config = |metric_name: &str| -> &DoraMetricConfig {
        DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == metric_name)
            .map(|(_, config)| config)
            .expect("Unknown DORA metric")
    };

    serde_json::json!({
        "commit_frequency": get_config("commit_frequency").translate(engineering_metrics.commit_frequency),
        "branch_lifetime": get_config("branch_lifetime").translate(engineering_metrics.branch_lifetime),
        "pbis_delivered_per_sprint_per_team": engineering_metrics.pbis_delivered_per_sprint_per_team,
    })
}
```

#### 1.4 Move Validation to Core

**`crates/crt-core/src/validation.rs`:**
```rust
use crate::types::*;

pub trait Validate {
    fn validate(&self) -> Result<(), String>;
}

impl Validate for AnalyseRequest {
    fn validate(&self) -> Result<(), String> {
        if self.crt.is_empty() {
            return Err("CRT is required".to_string());
        }
        if self.dora_metrics.deployment_frequency < 0.0 || self.dora_metrics.deployment_frequency > 1.0 {
            return Err("Deployment frequency must be between 0 and 1".to_string());
        }
        if self.dora_metrics.lead_time < 0.0 || self.dora_metrics.lead_time > 1.0 {
            return Err("Lead time must be between 0 and 1".to_string());
        }
        if self.dora_metrics.change_failure_rate < 0.0 || self.dora_metrics.change_failure_rate > 1.0 {
            return Err("Change failure rate must be between 0 and 1".to_string());
        }
        if self.dora_metrics.mttr < 0.0 || self.dora_metrics.mttr > 1.0 {
            return Err("MTTR must be between 0 and 1".to_string());
        }
        if self.extended_engineering_metrics.commit_frequency < 0.0 || self.extended_engineering_metrics.commit_frequency > 1.0 {
            return Err("Commit frequency must be between 0 and 1".to_string());
        }
        if self.extended_engineering_metrics.branch_lifetime < 0.0 || self.extended_engineering_metrics.branch_lifetime > 1.0 {
            return Err("Branch lifetime must be between 0 and 1".to_string());
        }
        if self.extended_engineering_metrics.pbis_delivered_per_sprint_per_team < 0.0 || self.extended_engineering_metrics.pbis_delivered_per_sprint_per_team > 1.0 {
            return Err("PBIs delivered per sprint per team must be between 0 and 1".to_string());
        }
        if self.westrum < 0.0 || self.westrum > 7.0 {
            return Err("Westrum must be between 0 and 7".to_string());
        }
        if self.time_allocation.meetings < 0 || self.time_allocation.unplanned < 0 || self.time_allocation.bugs < 0 || self.time_allocation.feature < 0 || self.time_allocation.tech_debt < 0 {
            return Err("Time allocation must be greater than 0".to_string());
        }
        if self.time_allocation.meetings + self.time_allocation.unplanned + self.time_allocation.bugs + self.time_allocation.feature + self.time_allocation.tech_debt != 100 {
            return Err("Time allocation must sum to 100".to_string());
        } 
        Ok(())
    }
}

impl Validate for RefineRequest {
    fn validate(&self) -> Result<(), String> {
        if self.content.trim().is_empty() {
            return Err("Content must not be empty".to_string());
        }
        if self.content.len() > 100_000 {
            return Err("Content is too large (max 100,000 characters)".to_string());
        }
        Ok(())
    }
}

impl Validate for EvaluateRequest {
    fn validate(&self) -> Result<(), String> {
        self.original_payload.validate()?;
        
        if self.analysis_result.executive_summary.is_empty() {
            return Err("Analysis result must have an executive summary".to_string());
        }
        if self.analysis_result.core_systemic_issues.is_empty() {
            return Err("Analysis result must have at least one core systemic issue".to_string());
        }
        if self.analysis_result.leverage_points.is_empty() {
            return Err("Analysis result must have at least one leverage point".to_string());
        }
        if self.analysis_result.analysis_confidence.is_empty() {
            return Err("Analysis result must have analysis confidence".to_string());
        }
        Ok(())
    }
}

impl Validate for AnalyseWithFeedbackRequest {
    fn validate(&self) -> Result<(), String> {
        self.original_payload.validate()?;
        
        if self.analysis_result.executive_summary.is_empty() {
            return Err("Analysis result must have an executive summary".to_string());
        }
        if self.analysis_result.core_systemic_issues.is_empty() {
            return Err("Analysis result must have at least one core systemic issue".to_string());
        }
        if self.analysis_result.leverage_points.is_empty() {
            return Err("Analysis result must have at least one leverage point".to_string());
        }
        if self.analysis_result.analysis_confidence.is_empty() {
            return Err("Analysis result must have analysis confidence".to_string());
        }
        
        if self.evaluation.is_null() {
            return Err("Evaluation feedback is required".to_string());
        }
        Ok(())
    }
}
```

#### 1.5 Create WASM Bindings

**`crates/crt-core/src/wasm.rs`:**
```rust
use wasm_bindgen::prelude::*;
use crate::types::*;
use crate::validation::Validate;
use crate::dora::*;

#[wasm_bindgen]
pub struct WasmAnalyseRequest {
    inner: AnalyseRequest,
}

#[wasm_bindgen]
impl WasmAnalyseRequest {
    #[wasm_bindgen(constructor)]
    pub fn new(
        crt: String,
        deployment_frequency: f32,
        lead_time: f32,
        change_failure_rate: f32,
        mttr: f32,
        commit_frequency: f32,
        branch_lifetime: f32,
        pbis_delivered_per_sprint_per_team: f32,
        westrum: f32,
        meetings: i32,
        unplanned: i32,
        bugs: i32,
        feature: i32,
        tech_debt: i32,
    ) -> WasmAnalyseRequest {
        WasmAnalyseRequest {
            inner: AnalyseRequest {
                crt,
                dora_metrics: DoraMetrics {
                    deployment_frequency,
                    lead_time,
                    change_failure_rate,
                    mttr,
                },
                extended_engineering_metrics: EngineeringMetrics {
                    commit_frequency,
                    branch_lifetime,
                    pbis_delivered_per_sprint_per_team,
                },
                westrum,
                time_allocation: TimeAllocation {
                    meetings,
                    unplanned,
                    bugs,
                    feature,
                    tech_debt,
                },
            },
        }
    }

    #[wasm_bindgen]
    pub fn validate(&self) -> Result<(), JsValue> {
        self.inner.validate().map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub fn translate_dora_metric(metric_name: &str, slider_value: f32) -> Result<JsValue, JsValue> {
    let config = DORA_METRIC_CONFIGS
        .iter()
        .find(|(name, _)| *name == metric_name)
        .map(|(_, config)| config)
        .ok_or_else(|| JsValue::from_str(&format!("Unknown metric: {}", metric_name)))?;

    let result = config.translate(slider_value);
    Ok(serde_wasm_bindgen::to_value(&result)?)
}

#[wasm_bindgen]
pub fn get_dora_metric_config(metric_name: &str) -> Result<JsValue, JsValue> {
    let config = DORA_METRIC_CONFIGS
        .iter()
        .find(|(name, _)| *name == metric_name)
        .map(|(_, config)| config)
        .ok_or_else(|| JsValue::from_str(&format!("Unknown metric: {}", metric_name)))?;

    let config_info = serde_json::json!({
        "min_value": config.min_value,
        "max_value": config.max_value,
        "unit": config.unit,
        "inverted": config.inverted,
    });

    Ok(serde_wasm_bindgen::to_value(&config_info)?)
}
```

**`crates/crt-core/src/lib.rs`:**
```rust
pub mod types;
pub mod dora;
pub mod validation;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dora::*;
    use crate::types::*;

    #[test]
    fn test_deployment_frequency_translation() {
        let config = DORA_METRIC_CONFIGS
            .iter()
            .find(|(name, _)| *name == "deployment_frequency")
            .map(|(_, config)| config)
            .unwrap();

        let result_0 = config.translate(0.0);
        assert_eq!(result_0.value, 0.025);
        assert_eq!(result_0.unit, "deployments/day");

        let result_1 = config.translate(1.0);
        assert_eq!(result_1.value, 10.0);
        assert_eq!(result_1.unit, "deployments/day");
    }

    // ... other tests
}
```

### Phase 2: Create Frontend WASM Module

#### 2.1 Create Frontend Crate

```bash
cd /Users/rune/projects/crt/crates
cargo generate --git https://github.com/rustwasm/wasm-pack-template crt-frontend
cd crt-frontend
```

**`crates/crt-frontend/Cargo.toml`:**
```toml
[package]
name = "crt-frontend"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
crt-core = { path = "../crt-core" }
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
js-sys = "0.3"
web-sys = "0.3"
```

**`crates/crt-frontend/src/lib.rs`:**
```rust
use wasm_bindgen::prelude::*;
use crt_core::*;

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

// Re-export core functionality
pub use crt_core::wasm::*;
```

#### 2.2 Build WASM Package

```bash
cd crates/crt-frontend
wasm-pack build --target web --out-dir pkg
```

### Phase 3: Update Backend to Use Core Library

#### 3.1 Update Backend Cargo.toml

**`crates/crt-backend/Cargo.toml`:**
```toml
[package]
name = "crt-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
crt-core = { path = "../crt-core" }
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
dotenvy = "0.15"
regex = "1.0"
tower-http = { version = "0.5", features = ["cors"] }
```

#### 3.2 Update Backend Main.rs

**`crates/crt-backend/src/main.rs`:**
```rust
use crt_core::*;
use crt_core::validation::Validate;
use crt_core::dora::*;

// Remove all the type definitions and use crt_core::* instead
// Remove DORA_METRIC_CONFIGS and use crt_core::dora::DORA_METRIC_CONFIGS
// Remove validation implementations and use crt_core::validation::Validate

async fn analyse(
    State(state): State<AppState>,
    Json(request): Json<AnalyseRequest>,
) -> Result<Json<AnalysisResponse>, (StatusCode, String)> {
    // Validate request first
    request.validate().map_err(|err| (StatusCode::BAD_REQUEST, err))?;
    
    // Translate metrics for agent consumption
    let agent_payload = serde_json::json!({
        "crt": request.crt,
        "dora_metrics": translate_dora_metrics_for_agent(&request.dora_metrics),
        "extended_engineering_metrics": translate_engineering_metrics_for_agent(&request.extended_engineering_metrics),
        "westrum": request.westrum,
        "time_allocation": request.time_allocation,
    });

    // ... rest of the function
}
```

### Phase 4: Update Frontend to Use WASM

#### 4.1 Add WASM Module to HTML

**`index.html`:**
```html
<script type="module">
  import init, { WasmAnalyseRequest, translate_dora_metric, get_dora_metric_config } from './crates/crt-frontend/pkg/crt_frontend.js';
  
  await init();
  
  // Replace JavaScript translation functions with WASM calls
  function translateDoraMetric(metricId, sliderValue) {
    const result = translate_dora_metric(metricId, sliderValue);
    return {
      value: result.value,
      unit: result.unit
    };
  }
  
  // Replace validation with WASM
  function validateAnalysisRequest(payload) {
    const request = new WasmAnalyseRequest(
      payload.crt,
      payload.dora_metrics.deployment_frequency,
      payload.dora_metrics.lead_time,
      payload.dora_metrics.change_failure_rate,
      payload.dora_metrics.mttr,
      payload.extended_engineering_metrics.commit_frequency,
      payload.extended_engineering_metrics.branch_lifetime,
      payload.extended_engineering_metrics.pbis_delivered_per_sprint_per_team,
      payload.westrum,
      payload.time_allocation.meetings,
      payload.time_allocation.unplanned,
      payload.time_allocation.bugs,
      payload.time_allocation.feature,
      payload.time_allocation.tech_debt
    );
    
    try {
      request.validate();
      return { valid: true, error: null };
    } catch (error) {
      return { valid: false, error: error.toString() };
    }
  }
</script>
```

#### 4.2 Update Frontend Functions

Replace all JavaScript translation and validation logic with WASM calls:

```javascript
// Before (JavaScript)
function translateDoraMetric(metricType, sliderValue) {
  // ... JavaScript implementation
}

// After (WASM)
function translateDoraMetric(metricType, sliderValue) {
  return translate_dora_metric(metricType, sliderValue);
}
```

### Phase 5: Migration Strategy

#### 5.1 Gradual Migration

1. **Create core library** with all types and logic
2. **Update backend** to use core library (keep existing endpoints working)
3. **Build WASM module** and test in isolation
4. **Gradually replace frontend functions** with WASM calls
5. **Remove old JavaScript implementations**

#### 5.2 Testing Strategy

1. **Unit tests** in core library ensure logic correctness
2. **Integration tests** verify backend uses core correctly
3. **WASM tests** verify frontend bindings work
4. **E2E tests** ensure full flow works with shared logic

### Phase 6: Build and Deployment

#### 6.1 Build Scripts

**`build.sh`:**
```bash
#!/bin/bash
set -e

# Build core library
cd crates/crt-core
cargo test

# Build backend
cd ../crt-backend
cargo build --release

# Build frontend WASM
cd ../crt-frontend
wasm-pack build --target web --out-dir pkg
```

#### 6.2 CI/CD Integration

```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      
      - name: Test core library
        run: cd crates/crt-core && cargo test
      
      - name: Test backend
        run: cd crates/crt-backend && cargo test
      
      - name: Build WASM
        run: |
          curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
          cd crates/crt-frontend && wasm-pack build --target web
```

## Benefits Summary

1. **Single Source of Truth**: All business logic in one place
2. **Type Safety**: Frontend gets Rust's type system benefits
3. **Performance**: WASM provides near-native performance
4. **Consistency**: No more frontend/backend drift
5. **Maintainability**: Changes only need to be made once
6. **Testing**: Shared test suite ensures correctness
7. **Future-Proof**: Easy to add new features to both sides

## Timeline

- **Week 1**: Create core library and move types/logic
- **Week 2**: Update backend to use core library
- **Week 3**: Create WASM module and frontend bindings
- **Week 4**: Migrate frontend to use WASM, testing and cleanup

This approach will significantly improve the maintainability and consistency of the CRT system while providing better performance and type safety.

//! Resolve — A2A Deliberative Compilation System
//! Transmutes intention into computation through multi-agent deliberation.
//! JSON payloads as first-class citizens.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core payload: every operation in Resolve is a Payload.
/// JSON-native, confidence-bearing, provenance-tracked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub op: String,
    pub inputs: Vec<String>,
    pub constraints: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub provenance: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub id: String,
}

impl Payload {
    pub fn new(op: &str) -> Self {
        Self {
            op: op.to_string(),
            inputs: vec![],
            constraints: HashMap::new(),
            confidence: 0.5,
            provenance: vec![],
            metadata: HashMap::new(),
            id: uuid_simple(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    pub fn merge_confidence(&self, other: &Self) -> f64 {
        let c1 = self.confidence.max(0.001);
        let c2 = other.confidence.max(0.001);
        1.0 / (1.0 / c1 + 1.0 / c2)
    }
}

/// Bayesian confidence combination: 1/(1/c1 + 1/c2)
pub fn bayesian_combine(c1: f64, c2: f64) -> f64 {
    1.0 / (1.0 / c1.max(0.001) + 1.0 / c2.max(0.001))
}

/// Payload chain: ordered list forming a deliberation DAG
#[derive(Debug, Clone)]
pub struct PayloadChain {
    pub payloads: Vec<Payload>,
}

impl PayloadChain {
    pub fn new() -> Self { Self { payloads: vec![] } }

    pub fn add(&mut self, p: Payload) { self.payloads.push(p); }

    pub fn aggregate_confidence(&self) -> f64 {
        if self.payloads.is_empty() { return 0.0; }
        let prod: f64 = self.payloads.iter()
            .map(|p| p.confidence.max(0.001))
            .product();
        prod.powf(1.0 / self.payloads.len() as f64)
    }

    pub fn best(&self) -> Option<&Payload> {
        self.payloads.iter().max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }
}

/// Agent trait: every agent receives payloads and returns payloads
pub trait Agent {
    fn role(&self) -> &str;
    fn receive(&mut self, payload: &Payload) -> Payload;
    fn proposals_made(&self) -> usize;
    fn confidence_threshold(&self) -> f64;
}

/// Base agent implementation
pub struct BaseAgent {
    pub role_name: String,
    pub threshold: f64,
    pub proposals: usize,
}

impl BaseAgent {
    pub fn new(role: &str, threshold: f64) -> Self {
        Self { role_name: role.to_string(), threshold, proposals: 0 }
    }

    pub fn respond(&mut self, source: &Payload, op: &str, result: serde_json::Value, confidence: f64) -> Payload {
        self.proposals += 1;
        let mut p = Payload::new(op);
        p.inputs = vec![result.to_string()];
        p.constraints = source.constraints.clone();
        p.confidence = confidence.clamp(0.0, 1.0);
        p.provenance = vec![source.id.clone()];
        p.provenance.extend(source.provenance.iter().cloned());
        p.metadata.insert("agent".to_string(), serde_json::Value::String(self.role_name.clone()));
        p.metadata.insert("result".to_string(), result);
        p
    }
}

/// Intent parser agent: extracts structured constraints from human text
pub struct IntentParser { base: BaseAgent }

impl IntentParser {
    pub fn new() -> Self { Self { base: BaseAgent::new("intent_parser", 0.9) } }

    fn extract_constraints(&self, text: &str) -> (Vec<String>, Vec<String>) {
        let lower = text.to_lowercase();
        let mut hard = vec![];
        let mut soft = vec![];
        if lower.contains("descending") { hard.push("descending".to_string()); }
        else if lower.contains("ascending") { hard.push("ascending".to_string()); }
        if lower.contains("fast") || lower.contains("efficient") { soft.push("efficient".to_string()); }
        (hard, soft)
    }
}

impl Agent for IntentParser {
    fn role(&self) -> &str { &self.base.role_name }
    fn receive(&mut self, payload: &Payload) -> Payload {
        if payload.op != "intent" { return payload.clone(); }
        let text = payload.inputs.first().unwrap_or(&String::new()).clone();
        let (hard, soft) = self.extract_constraints(&text);
        let result = serde_json::json!({"goal": text, "hard": hard, "soft": soft});
        self.base.respond(payload, "parsed_intent", result, 0.95)
    }
    fn proposals_made(&self) -> usize { self.base.proposals }
    fn confidence_threshold(&self) -> f64 { self.base.threshold }
}

/// Deliberation engine: orchestrates multi-agent deliberation
pub struct DeliberationEngine {
    pub agents: Vec<Box<dyn Agent>>,
    pub confidence_threshold: f64,
    pub max_rounds: usize,
    pub trace: Vec<TraceEntry>,
}

#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub round: usize,
    pub agent: String,
    pub op: String,
    pub confidence: f64,
    pub summary: String,
}

impl DeliberationEngine {
    pub fn new(threshold: f64, max_rounds: usize) -> Self {
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(IntentParser::new()),
        ];
        Self { agents, confidence_threshold: threshold, max_rounds, trace: vec![] }
    }

    pub fn deliberate(&mut self, intent_text: &str) -> Option<Artifact> {
        let mut current = Payload::new("intent");
        current.inputs = vec![intent_text.to_string()];
        current.confidence = 1.0;

        let mut chain = PayloadChain::new();

        for round in 1..=self.max_rounds {
            let mut best_conf = 0.0_f64;
            for agent in &mut self.agents {
                let result = agent.receive(&current);
                let summary = result.metadata.get("result")
                    .map(|r| r.to_string().chars().take(50).collect())
                    .unwrap_or_default();
                self.trace.push(TraceEntry {
                    round, agent: agent.role().to_string(),
                    op: result.op.clone(), confidence: result.confidence,
                    summary,
                });
                chain.add(result.clone());
                best_conf = best_conf.max(result.confidence);
            }
            if best_conf >= self.confidence_threshold {
                break;
            }
            if let Some(best) = chain.best() {
                current = best.clone();
            }
        }

        let best = chain.best()?;
        Some(Artifact {
            code: best.metadata.get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("fn(data) { data }")
                .to_string(),
            confidence: best.confidence,
            agents_used: self.trace.iter().map(|e| e.agent.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect(),
        })
    }
}

/// Artifact: the output of deliberation
pub struct Artifact {
    pub code: String,
    pub confidence: f64,
    pub agents_used: Vec<String>,
}

/// Simple UUID generator (no deps)
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{:016x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_creation() {
        let p = Payload::new("test");
        assert_eq!(p.op, "test");
        assert!(!p.id.is_empty());
    }

    #[test]
    fn test_confidence_merge() {
        let a = Payload::new("a"); let a = Payload { confidence: 0.5, ..a };
        let b = Payload::new("b"); let b = Payload { confidence: 0.5, ..b };
        let merged = a.merge_confidence(&b);
        assert!((merged - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_chain_aggregate() {
        let mut chain = PayloadChain::new();
        let p1 = Payload { confidence: 0.5, ..Payload::new("a") };
        let p2 = Payload { confidence: 0.8, ..Payload::new("b") };
        chain.add(p1); chain.add(p2);
        let best = chain.best().unwrap();
        assert!((best.confidence - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_intent_parser() {
        let mut parser = IntentParser::new();
        let payload = Payload { op: "intent".to_string(), inputs: vec!["sort descending".to_string()], ..Payload::new("intent") };
        let result = parser.receive(&payload);
        assert_eq!(result.op, "parsed_intent");
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_bayesian_combine() {
        assert!((bayesian_combine(0.5, 0.5) - 0.25).abs() < 0.01);
        assert!((bayesian_combine(0.9, 0.9) - 0.45).abs() < 0.01);
    }
}

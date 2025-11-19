use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryEvent {
    Text(String),
    ObjectDetection { label: String, confidence: f32 },
    // New: The scientist can publish a hypothesis or insight
    Insight { topic: String, content: String }, 
}

// --- Local Types mirroring sim_engine ---
#[derive(Debug, Clone)]
pub enum AgentAction {
    FlipCell { r: usize, c: usize },
    Perturb { which: u8, delta: f64 },
    // NEW: Allow the scientist to change simulation constants
    SetParam { name: String, val: f64 }, 
    Noop,
}

#[derive(Debug, Clone)]
pub enum AgentObservation {
    GridSummary { width: usize, height: usize },
    StateVec([f64; 3]),
    None,
}

/// An Experimenter is an agent that chooses actions AND can publish findings.
pub trait Experimenter {
    // Now returns a tuple: (The Action, An Optional Discovery/Log)
    fn act(&mut self, obs: &AgentObservation, step: u64) -> (AgentAction, Option<DiscoveryEvent>);
}

// --- UPDATED MOCK SCIENTIST ---
pub struct MockExperimenter {
    rng_seed: u64,
}

impl MockExperimenter {
    pub fn new() -> Self { Self { rng_seed: 0 } }
}

impl Experimenter for MockExperimenter {
    fn act(&mut self, obs: &AgentObservation, step: u64) -> (AgentAction, Option<DiscoveryEvent>) {
        // 1. Decide on Action
        let action = match obs {
            AgentObservation::GridSummary { width, height } => {
                if step % 60 == 0 {
                    AgentAction::FlipCell { r: height/2, c: width/2 }
                } else {
                    AgentAction::Noop
                }
            }
            AgentObservation::StateVec(_v) => {
                if step % 30 == 0 {
                    AgentAction::Perturb { which: 0, delta: 2.0 }
                } else {
                    AgentAction::Noop
                }
            }
            _ => AgentAction::Noop,
        };

        // 2. Decide on Analysis (Mocking a "realization")
        let discovery = if step > 0 && step % 120 == 0 {
            Some(DiscoveryEvent::Text(format!("Scientist: Tick {} shows interesting stability.", step)))
        } else {
            None
        };

        (action, discovery)
    }
}

// Remove `get_mock_inference` as it is now obsolete (the agent handles it).

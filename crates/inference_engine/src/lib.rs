use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryEvent {
    Text(String),
    ObjectDetection { label: String, confidence: f32 },
}

// --- Local Types mirroring sim_engine (for decoupling) ---
#[derive(Debug, Clone)]
pub enum AgentAction {
    FlipCell { r: usize, c: usize },
    Perturb { which: u8, delta: f64 },
    Noop,
}

#[derive(Debug, Clone)]
pub enum AgentObservation {
    GridSummary { width: usize, height: usize },
    StateVec([f64; 3]),
    None,
}

/// An Experimenter is an agent that chooses actions.
pub trait Experimenter {
    fn act(&mut self, obs: &AgentObservation, step: u64) -> AgentAction;
}

/// A simple random/heuristic experimenter.
pub struct MockExperimenter {
    rng_seed: u64,
}

impl MockExperimenter {
    pub fn new() -> Self { Self { rng_seed: 0 } }
}

impl Experimenter for MockExperimenter {
    fn act(&mut self, obs: &AgentObservation, step: u64) -> AgentAction {
        match obs {
            AgentObservation::GridSummary { width, height } => {
                // Every 60 ticks, flip a cell in the center
                if step % 60 == 0 {
                    AgentAction::FlipCell { r: height/2, c: width/2 }
                } else {
                    AgentAction::Noop
                }
            }
            AgentObservation::StateVec(_v) => {
                // Every 30 ticks, kick the X variable
                if step % 30 == 0 {
                    AgentAction::Perturb { which: 0, delta: 2.0 }
                } else {
                    AgentAction::Noop
                }
            }
            _ => AgentAction::Noop,
        }
    }
}

pub fn get_mock_inference(tick: u64) -> Option<DiscoveryEvent> {
    if tick == 0 || tick % 120 != 0 { return None; }
    Some(DiscoveryEvent::Text(format!("Agent analyzing... (Tick: {})", tick)))
}

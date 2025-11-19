use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use js_sys::Math;

// --- SHARED EVENTS ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryEvent {
    Text(String),
    Insight { topic: String, content: String },
}

// --- AGENT INTERFACE ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum DiscreteAction {
    Noop,
    KickXPos, KickXNeg,
    KickYPos, KickYNeg,
    KickZPos, KickZNeg,
}

#[derive(Debug, Clone)]
pub enum AgentAction {
    FlipCell { r: usize, c: usize },
    Perturb { which: u8, delta: f64 },
    SetParam { name: String, val: f64 },
    Noop,
}

#[derive(Debug, Clone)]
pub enum AgentObservation {
    GridSummary { width: usize, height: usize },
    StateVec([f64; 3]),
    None,
}

pub trait Experimenter {
    fn act(&mut self, obs: &AgentObservation, reward: f64, step: u64) -> (AgentAction, Option<DiscoveryEvent>);
}

// ---------------------------------------------------------
// INTELLIGENCE: Curious Q-Learning Agent (v2: Predictive)
// ---------------------------------------------------------
pub struct QLearningAgent {
    // Q-Table: Maps "StateHash" -> {Action: Value}
    q_table: HashMap<String, HashMap<DiscreteAction, f64>>,
    
    // NEW: World Model (Physics Engine in the Brain)
    // Maps (StateHash, Action) -> Predicted Next Continuous State [x, y, z]
    world_model: HashMap<(String, DiscreteAction), [f64; 3]>,

    last_action: DiscreteAction,
    last_state_key: String,
    last_state_vec: [f64; 3], // Keep track of exact physics state
    
    // Hyperparameters
    epsilon: f64, 
    alpha: f64,   
    gamma: f64,   
}

impl QLearningAgent {
    pub fn new() -> Self {
        Self {
            q_table: HashMap::new(),
            world_model: HashMap::new(),
            last_action: DiscreteAction::Noop,
            last_state_key: "0_0_0".to_string(),
            last_state_vec: [0.0, 0.0, 0.0],
            epsilon: 0.5, 
            alpha: 0.1,
            gamma: 0.9,
        }
    }

    // CHANGE #3: Foveated Vision (Logarithmic Discretization)
    // High resolution near 0, low resolution far away.
    fn discretize(&self, state: [f64; 3]) -> String {
        let foveate = |v: f64| -> i32 {
            // log(1 + |x|) compresses huge distances.
            // Scale factor 4.0 gives us buckets like: 0..0.25, 0.25..0.6, etc.
            let sign = v.signum();
            let val = (v.abs() + 1.0).ln(); 
            (sign * val * 4.0) as i32
        };

        format!("{}_{}_{}", foveate(state[0]), foveate(state[1]), foveate(state[2]))
    }

    fn map_action(&self, action: DiscreteAction) -> AgentAction {
        let kick = 5.0; 
        match action {
            DiscreteAction::Noop => AgentAction::Noop,
            DiscreteAction::KickXPos => AgentAction::Perturb { which: 0, delta: kick },
            DiscreteAction::KickXNeg => AgentAction::Perturb { which: 0, delta: -kick },
            DiscreteAction::KickYPos => AgentAction::Perturb { which: 1, delta: kick },
            DiscreteAction::KickYNeg => AgentAction::Perturb { which: 1, delta: -kick },
            DiscreteAction::KickZPos => AgentAction::Perturb { which: 2, delta: kick },
            DiscreteAction::KickZNeg => AgentAction::Perturb { which: 2, delta: -kick },
        }
    }

    fn get_max_q(&self, state_key: &str) -> f64 {
        if let Some(actions) = self.q_table.get(state_key) {
            actions.values().cloned().fold(f64::NEG_INFINITY, f64::max)
        } else {
            0.0 
        }
    }

    // Calculate Euclidean distance between two 3D points
    fn dist(&self, a: [f64; 3], b: [f64; 3]) -> f64 {
        ((a[0]-b[0]).powi(2) + (a[1]-b[1]).powi(2) + (a[2]-b[2]).powi(2)).sqrt()
    }
}

impl Experimenter for QLearningAgent {
    fn act(&mut self, obs: &AgentObservation, base_reward: f64, step: u64) -> (AgentAction, Option<DiscoveryEvent>) {
        let mut discovery = None;

        if let AgentObservation::StateVec(current_state) = obs {
            // 1. OBSERVE
            let current_state_key = self.discretize(*current_state);
            
            // 2. CHANGE #1: CALCULATE SURPRISE (Prediction Error)
            // Did the world behave how we thought it would given our last action?
            let mut surprise = 0.0;
            let prediction_key = (self.last_state_key.clone(), self.last_action);
            
            if let Some(predicted_state) = self.world_model.get(&prediction_key) {
                let error = self.dist(*predicted_state, *current_state);
                // If error is high, we are surprised! Reward this.
                // We cap it to prevent infinite loops of chaos.
                surprise = (error * 5.0).min(50.0); 
            } else {
                // First time trying this? Moderate curiosity boost.
                surprise = 5.0;
            }

            // 3. UPDATE WORLD MODEL (Learn Physics)
            // "Next time I am in [LastState] and do [Action], I expect [CurrentState]"
            // Use a simple moving average to smooth out noise (Learning Rate 0.5)
            let new_prediction = if let Some(prev) = self.world_model.get(&prediction_key) {
                [
                    0.5 * prev[0] + 0.5 * current_state[0],
                    0.5 * prev[1] + 0.5 * current_state[1],
                    0.5 * prev[2] + 0.5 * current_state[2]
                ]
            } else {
                *current_state
            };
            self.world_model.insert(prediction_key, new_prediction);

            // 4. TOTAL REWARD = Stability (Base) + Curiosity (Surprise)
            let total_reward = base_reward + surprise;

            // 5. LEARN (Update Q-Table)
            let max_future_q = self.get_max_q(&current_state_key);
            let action_values = self.q_table.entry(self.last_state_key.clone()).or_default();
            let current_q = action_values.entry(self.last_action).or_insert(0.0);
            
            *current_q += self.alpha * (total_reward + self.gamma * max_future_q - *current_q);

            // 6. DECIDE ACTION (Epsilon-Greedy)
            let action = if Math::random() < self.epsilon {
                match (Math::random() * 7.0) as u8 {
                    0 => DiscreteAction::KickXPos, 1 => DiscreteAction::KickXNeg,
                    2 => DiscreteAction::KickYPos, 3 => DiscreteAction::KickYNeg,
                    4 => DiscreteAction::KickZPos, 5 => DiscreteAction::KickZNeg,
                    _ => DiscreteAction::Noop,
                }
            } else {
                let values = self.q_table.entry(current_state_key.clone()).or_default();
                values.iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(k, _)| *k)
                    .unwrap_or(DiscreteAction::Noop)
            };

            // 7. LOGGING & MEMORY
            self.last_state_key = current_state_key;
            self.last_state_vec = *current_state;
            self.last_action = action;
            
            if self.epsilon > 0.05 { self.epsilon *= 0.995; }

            // Generate insights based on Surprise, not just random text
            if surprise > 25.0 && step % 60 == 0 {
                 discovery = Some(DiscoveryEvent::Insight {
                     topic: "Anomaly Detected".into(),
                     content: format!("Physics violation? Predicted vs Actual error: {:.2}. Reward spike!", surprise/5.0)
                 });
            } 

            return (self.map_action(action), discovery);
        }

        (AgentAction::Noop, None)
    }
}

// ... (GardenerAgent, MockExperimenter, Factory - Keep same) ...
pub struct GardenerAgent;
impl GardenerAgent { pub fn new() -> Self { Self } }
impl Experimenter for GardenerAgent {
    fn act(&mut self, _: &AgentObservation, _: f64, _: u64) -> (AgentAction, Option<DiscoveryEvent>) {
        (AgentAction::Noop, None)
    }
}

pub struct MockExperimenter;
impl MockExperimenter { pub fn new() -> Self { Self } }
impl Experimenter for MockExperimenter {
    fn act(&mut self, _: &AgentObservation, _: f64, _: u64) -> (AgentAction, Option<DiscoveryEvent>) {
        (AgentAction::Noop, None)
    }
}

pub enum BrainType {
    QLearner,
    Gardener,
    Mock,
}

pub fn create_brain(brain_type: BrainType) -> Box<dyn Experimenter> {
    match brain_type {
        BrainType::QLearner => Box::new(QLearningAgent::new()),
        BrainType::Gardener => Box::new(GardenerAgent::new()),
        BrainType::Mock => Box::new(MockExperimenter::new()),
    }
}

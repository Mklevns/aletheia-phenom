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
// INTELLIGENCE: Curious Q-Learning Agent
// ---------------------------------------------------------
pub struct QLearningAgent {
    // Q-Table: Maps "StateHash" -> {Action: Value}
    q_table: HashMap<String, HashMap<DiscreteAction, f64>>,
    
    // Novelty Memory: Maps "StateHash" -> Visit Count
    visit_counts: HashMap<String, u64>,

    last_action: DiscreteAction,
    last_state_key: String,
    
    // Hyperparameters
    epsilon: f64, // Curiosity (Randomness)
    alpha: f64,   // Learning Rate
    gamma: f64,   // Discount Factor
}

impl QLearningAgent {
    pub fn new() -> Self {
        Self {
            q_table: HashMap::new(),
            visit_counts: HashMap::new(),
            last_action: DiscreteAction::Noop,
            last_state_key: "0_0_0".to_string(),
            epsilon: 0.5, 
            alpha: 0.1,
            gamma: 0.9,
        }
    }

    // Quantize the continuous world into "Concepts" the AI can understand
    fn discretize(&self, state: [f64; 3]) -> String {
        // Round to nearest 5.0 to create "regions" of space
        let x = (state[0] / 5.0).round() as i32;
        let y = (state[1] / 5.0).round() as i32;
        let z = (state[2] / 5.0).round() as i32;
        format!("{}_{}_{}", x, y, z)
    }

    fn map_action(&self, action: DiscreteAction) -> AgentAction {
        let kick = 5.0; // Strong kick to allow pattern creation
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
            0.0 // Optimistic initialization could go here
        }
    }
}

impl Experimenter for QLearningAgent {
    fn act(&mut self, obs: &AgentObservation, base_reward: f64, step: u64) -> (AgentAction, Option<DiscoveryEvent>) {
        let mut discovery = None;

        if let AgentObservation::StateVec(state) = obs {
            // 1. OBSERVE & MEMORIZE
            let current_state_key = self.discretize(*state);
            
            // 2. CALCULATE NOVELTY (The "Multiplier")
            let visit_count = self.visit_counts.entry(current_state_key.clone()).or_insert(0);
            *visit_count += 1;
            
            // Novelty Bonus: High for new states, decays as we see them more
            // Multiplier = 1 + (10 / count)
            let novelty_multiplier = 1.0 + (10.0 / (*visit_count as f64).max(1.0));
            
            // 3. TOTAL REWARD = Order * Novelty
            // "I like stability, but I REALLY like STABLE NEW PLACES."
            let effective_reward = base_reward * novelty_multiplier;

            // 4. LEARN (Update Brain)
            let max_future_q = self.get_max_q(&current_state_key);
            let action_values = self.q_table.entry(self.last_state_key.clone()).or_default();
            let current_q = action_values.entry(self.last_action).or_insert(0.0);
            
            // Q-Learning Equation
            *current_q += self.alpha * (effective_reward + self.gamma * max_future_q - *current_q);

            // 5. DECIDE ACTION (Epsilon-Greedy)
            let action = if Math::random() < self.epsilon {
                // Explore
                match (Math::random() * 7.0) as u8 {
                    0 => DiscreteAction::KickXPos, 1 => DiscreteAction::KickXNeg,
                    2 => DiscreteAction::KickYPos, 3 => DiscreteAction::KickYNeg,
                    4 => DiscreteAction::KickZPos, 5 => DiscreteAction::KickZNeg,
                    _ => DiscreteAction::Noop,
                }
            } else {
                // Exploit
                let values = self.q_table.entry(current_state_key.clone()).or_default();
                values.iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(k, _)| *k)
                    .unwrap_or(DiscreteAction::Noop)
            };

            self.last_state_key = current_state_key;
            self.last_action = action;
            
            // Decay exploration
            if self.epsilon > 0.05 { self.epsilon *= 0.995; }

            // 6. REPORT FINDINGS
            // If we found a highly rewarding state (Order) that is also Rare (Novelty)
            if effective_reward > 50.0 && step % 60 == 0 {
                 discovery = Some(DiscoveryEvent::Insight {
                     topic: "Pattern Discovered".into(),
                     content: format!("Found a highly ordered region! Base Reward: {:.1}, Novelty Mult: {:.1}x", base_reward, novelty_multiplier)
                 });
            } 
            else if step % 120 == 0 {
                discovery = Some(DiscoveryEvent::Text(
                    format!("Analysis: Explored {} regions. Current focus: Stability.", self.visit_counts.len())
                ));
            }

            return (self.map_action(action), discovery);
        }

        (AgentAction::Noop, None)
    }
}

// --- DUMMY GARDENER ---
pub struct GardenerAgent;
impl GardenerAgent { pub fn new() -> Self { Self } }
impl Experimenter for GardenerAgent {
    fn act(&mut self, _: &AgentObservation, _: f64, _: u64) -> (AgentAction, Option<DiscoveryEvent>) {
        (AgentAction::Noop, None)
    }
}

// --- MOCK ---
pub struct MockExperimenter;
impl MockExperimenter { pub fn new() -> Self { Self } }
impl Experimenter for MockExperimenter {
    fn act(&mut self, _: &AgentObservation, _: f64, _: u64) -> (AgentAction, Option<DiscoveryEvent>) {
        (AgentAction::Noop, None)
    }
}

// --- FACTORY ---
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

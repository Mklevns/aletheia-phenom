use sim_engine::{Simulation, Experimentable, SimState, Action, Observation};
use inference_engine::{Experimenter, AgentAction, AgentObservation, DiscoveryEvent};

/// A Session holds the World (Simulation) and the Scientist (Experimenter).
pub struct Session {
    pub sim: Box<dyn Simulation>,
    pub agent: Box<dyn Experimenter>,
    pub step_count: u64,
}

impl Session {
    pub fn new(sim: Box<dyn Simulation>, agent: Box<dyn Experimenter>) -> Self {
        Self { sim, agent, step_count: 0 }
    }

    /// The main loop: Observe -> Think -> Act -> Step
    /// Returns a DiscoveryEvent if the scientist had an epiphany.
    pub fn tick(&mut self) -> Option<DiscoveryEvent> {
        let mut discovery = None;

        // 1. Allow Agent to Observe and Act (if Sim is experimentable)
        if let Some(exp_sim) = self.sim.as_experimentable() {
            let obs = exp_sim.observe();
            let agent_obs = self.map_obs(obs);

            // The Scientist thinks...
            let (agent_action, event) = self.agent.act(&agent_obs, self.step_count);
            discovery = event;

            // Apply the Scientist's will
            let sim_action = self.map_act(agent_action);
            exp_sim.apply_action(sim_action);
        }

        // 2. Advance Physics
        self.sim.step();
        self.step_count += 1;

        discovery
    }

    pub fn get_state(&self) -> SimState {
        self.sim.get_state()
    }

    // --- Mapping Helpers (The Bridge) ---
    fn map_obs(&self, obs: Observation) -> AgentObservation {
        match obs {
            Observation::GridSummary { width, height, .. } => AgentObservation::GridSummary { width, height },
            Observation::StateVec(v) => AgentObservation::StateVec(v),
            _ => AgentObservation::None,
        }
    }

    fn map_act(&self, act: AgentAction) -> Action {
        match act {
            AgentAction::FlipCell { r, c } => Action::FlipCell { r, c },
            AgentAction::Perturb { which, delta } => Action::Perturb { which, delta },
            AgentAction::SetParam { name, val } => Action::SetParam { name, value: val },
            AgentAction::Noop => Action::Noop,
        }
    }
}

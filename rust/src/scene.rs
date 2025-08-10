use crate::fluid::Fluid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneType {
    Tank,
    WindTunnel,
    Paint,
    Default,
}

/// Simulation scene configuration
pub struct Scene {
    pub gravity: f64,
    pub dt: f64,
    pub num_iters: usize,
    pub over_relaxation: f64,
    pub obstacle_x: f64,
    pub obstacle_y: f64,
    pub obstacle_radius: f64,
    pub paused: bool,
    pub scene_nr: usize,
    pub show_obstacle: bool,
    pub show_streamlines: bool,
    pub scene_type: SceneType,
    pub show_velocities: bool,
    pub show_pressure: bool,
    pub show_smoke: bool,
    pub inflow_velocity: f64,  // Add inflow velocity tracking
    pub fluid: Option<Fluid>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            gravity: 0.0,  // No gravity for horizontal wind tunnel
            dt: 1.0 / 60.0,
            num_iters: 100,
            over_relaxation: 1.9,
            obstacle_x: 0.0,
            obstacle_y: 0.0,
            obstacle_radius: 0.15,
            paused: false,
            scene_nr: 0,
            show_obstacle: false,
            show_streamlines: false,
            scene_type: SceneType::Default,
            show_velocities: false,
            show_pressure: false,
            show_smoke: true,
            inflow_velocity: 5.0,  // Default inflow velocity
            fluid: None,
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Setup different simulation scenarios
    pub fn setup_scene(&mut self, scene_nr: usize, sim_width: f64, sim_height: f64) {
        self.scene_nr = scene_nr;
        self.obstacle_radius = 0.15;
        self.over_relaxation = 1.9;
        self.dt = 1.0 / 60.0;
        self.num_iters = 40;
        
        let res = match scene_nr {
            0 => 80,   // Tank - higher resolution
            1 => 200,  // Wind tunnel - much higher resolution  
            3 => 300,  // High-res tunnel - very high resolution
            4 => 250,  // Radiator testing - high resolution for accuracy
            _ => 150,  // Default resolution
        };
        
        // Make domain much larger for radiator testing
        let domain_height = if scene_nr == 4 { 2.0 } else { 1.0 }; // Double height for radiator
        let domain_width = if scene_nr == 4 { 4.0 } else { domain_height / sim_height * sim_width }; // Much wider for radiator
        
        let h = domain_height / (res as f64);
        
        let num_x = (domain_width / h) as usize;
        let num_y = (domain_height / h) as usize;
        
        let density = 1000.0;
        
        let mut fluid = Fluid::new(density, num_x, num_y, h);
        
        // Setup specific scenarios
        match scene_nr {
            0 => {
                // Tank scenario
                self.scene_type = SceneType::Tank;
                self.setup_tank(&mut fluid);
            }
            1 | 3 => {
                // Wind tunnel with vortex shedding
                self.scene_type = SceneType::WindTunnel;
                self.setup_wind_tunnel(&mut fluid);
            }
            2 => {
                // Paint scenario
                self.scene_type = SceneType::Paint;
                self.setup_paint(&mut fluid);
            }
            4 => {
                // Clean wind tunnel for radiator testing
                self.scene_type = SceneType::WindTunnel;
                self.setup_clean_wind_tunnel(&mut fluid);
            }
            _ => {
                // Default scenario
                self.scene_type = SceneType::Default;
                self.setup_default(&mut fluid);
            }
        }
        
        self.fluid = Some(fluid);
    }
    
    fn setup_tank(&mut self, fluid: &mut Fluid) {
        // Simple tank with no inflow
        // Fluid is initially at rest
        println!("Setting up tank scenario");
        
        // Add some initial smoke for visualization
        for i in 1..fluid.num_x / 4 {
            for j in 1..fluid.num_y - 1 {
                fluid.m[[i, j]] = 1.0;
            }
        }
    }
    
    fn setup_wind_tunnel(&mut self, fluid: &mut Fluid) {
        println!("Setting up advanced wind tunnel with boundary layer control");
        
        self.inflow_velocity = 5.0; // Store the inflow velocity
        
        // Set boundary conditions for wind tunnel
        for j in 1..fluid.num_y - 1 {
            // Inflow on left boundary (first two columns for stability)
            fluid.u[[0, j]] = self.inflow_velocity;
            fluid.u[[1, j]] = self.inflow_velocity;
            
            // Add smoke at inflow for visualization
            fluid.m[[0, j]] = 1.0;
            fluid.m[[1, j]] = 1.0;
        }
        
        // Advanced boundary layer management
        // Top and bottom walls with boundary layer suction simulation
        for i in 0..fluid.num_x {
            // Top wall with boundary layer control
            fluid.u[[i, fluid.num_y - 1]] = 0.0;
            fluid.v[[i, fluid.num_y - 1]] = 0.0;
            fluid.s[[i, fluid.num_y - 1]] = 0.0; // Solid
            
            // Boundary layer suction simulation - remove near-wall velocity gradients
            if fluid.num_y > 3 {
                // Suction near top wall (simulate boundary layer removal)
                let suction_strength = -0.1; // Negative = suction
                fluid.v[[i, fluid.num_y - 2]] = suction_strength;
                fluid.v[[i, fluid.num_y - 3]] = suction_strength * 0.5;
            }
            
            // Bottom wall with boundary layer control
            fluid.u[[i, 0]] = 0.0;
            fluid.v[[i, 0]] = 0.0;
            fluid.s[[i, 0]] = 0.0; // Solid
            
            // Suction near bottom wall
            if fluid.num_y > 3 {
                let suction_strength = 0.1; // Positive = suction downward
                fluid.v[[i, 1]] = suction_strength;
                fluid.v[[i, 2]] = suction_strength * 0.5;
            }
        }
        
        // Set obstacle in the middle of the domain for vortex shedding
        let obs_x = fluid.num_x as f64 * fluid.h * 0.4;
        let obs_y = fluid.num_y as f64 * fluid.h * 0.5;
        fluid.set_obstacle(obs_x, obs_y, self.obstacle_radius);
        self.obstacle_x = obs_x;
        self.obstacle_y = obs_y;
        self.show_obstacle = true;
    }
    
    fn setup_paint(&mut self, _fluid: &mut Fluid) {
        println!("Setting up paint scenario");
        
        // Paint scenario - no specific setup needed
        // User interaction will add smoke/dye
    }
    
    fn setup_default(&mut self, fluid: &mut Fluid) {
        println!("Setting up default scenario");
        
        // Default scenario
        for i in 1..fluid.num_x / 3 {
            for j in 1..fluid.num_y - 1 {
                fluid.m[[i, j]] = 1.0;
            }
        }
    }
    
    /// Setup a clean wind tunnel with optimal boundary layer control
    fn setup_clean_wind_tunnel(&mut self, fluid: &mut Fluid) {
        println!("Setting up clean wind tunnel with boundary layer control");
        
        self.inflow_velocity = 10.0; // Realistic automotive testing speed
        
        // Initialize uniform flow field
        for i in 0..fluid.num_x {
            for j in 1..fluid.num_y - 1 {
                fluid.u[[i, j]] = self.inflow_velocity;
                fluid.v[[i, j]] = 0.0;
                fluid.s[[i, j]] = 1.0; // Fluid
            }
        }
        
        // Set boundary conditions
        for j in 1..fluid.num_y - 1 {
            // Strong inflow enforcement
            fluid.u[[0, j]] = self.inflow_velocity;
            fluid.u[[1, j]] = self.inflow_velocity;
            fluid.u[[2, j]] = self.inflow_velocity; // Extra columns for stability
            
            // Add smoke for visualization
            fluid.m[[0, j]] = 1.0;
            fluid.m[[1, j]] = 1.0;
            fluid.m[[2, j]] = 0.8;
        }
        
        // Wall boundaries with special treatment
        for i in 0..fluid.num_x {
            // Top wall
            fluid.u[[i, fluid.num_y - 1]] = 0.0;
            fluid.v[[i, fluid.num_y - 1]] = 0.0;
            fluid.s[[i, fluid.num_y - 1]] = 0.0; // Solid
            
            // Bottom wall
            fluid.u[[i, 0]] = 0.0;
            fluid.v[[i, 0]] = 0.0;
            fluid.s[[i, 0]] = 0.0; // Solid
        }
        
        // No obstacle in clean tunnel (radiator will be added later)
        self.show_obstacle = false;
    }
    
    /// Add obstacle at specified position
    pub fn set_obstacle(&mut self, x: f64, y: f64, reset: bool) {
        if let Some(ref mut fluid) = self.fluid {
            if !reset {
                // Set velocity based on mouse movement (for future interactive features)
            }
            
            self.obstacle_x = x;
            self.obstacle_y = y;
            fluid.set_obstacle(x, y, self.obstacle_radius);
            self.show_obstacle = true;
        }
    }
    
    /// Run simulation step
    pub fn simulate(&mut self) {
        if !self.paused {
            if let Some(ref mut fluid) = self.fluid {
                // Use enhanced simulation with boundary enforcement
                fluid.simulate_with_boundaries(
                    self.dt, 
                    self.gravity, 
                    self.num_iters, 
                    self.over_relaxation,
                    self.inflow_velocity
                );
                
                // Apply boundary layer control for wind tunnel scenarios
                if matches!(self.scene_type, SceneType::WindTunnel) {
                    fluid.apply_boundary_layer_control(self.inflow_velocity);
                }
            }
        }
    }
}

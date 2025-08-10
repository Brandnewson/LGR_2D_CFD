use crate::fluid::Fluid;
use crate::obstacle::{ObstacleManager, Obstacle};

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
    pub scene_type: SceneType,
    pub inflow_velocity: f64,  // Add inflow velocity tracking
    pub fluid: Option<Fluid>,
    pub obstacle_manager: ObstacleManager, // New obstacle system
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
            scene_type: SceneType::Default,
            inflow_velocity: 5.0,  // Default inflow velocity
            fluid: None,
            obstacle_manager: ObstacleManager::new(),
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
            0 => 60,   // Tank - reduced resolution
            1 => 120,  // Wind tunnel - balanced resolution
            3 => 180,  // High-res tunnel - moderate high resolution
            4 => 150,  // Radiator testing - optimized resolution for speed
            _ => 100,  // Default resolution
        };
        
        // Reduce iterations for better performance
        self.num_iters = match scene_nr {
            4 => 20,   // Radiator - fewer iterations for speed
            _ => 30,   // Other scenes
        };
        
        // Optimize domain size for radiator testing
        let domain_height = if scene_nr == 4 { 1.5 } else { 1.0 }; // Smaller height for radiator
        let domain_width = if scene_nr == 4 { 3.0 } else { domain_height / sim_height * sim_width }; // Smaller width for radiator
        
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
        
        // Add obstacle using new system
        self.obstacle_manager.clear();
        let cylinder = Obstacle::new_circle(obs_x, obs_y, self.obstacle_radius);
        self.obstacle_manager.add_obstacle(cylinder);
        
        // Apply obstacles to fluid
        self.obstacle_manager.apply_to_fluid(
            &mut fluid.s, &mut fluid.u, &mut fluid.v, &mut fluid.m,
            fluid.num_x, fluid.num_y, fluid.h
        );
        
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
    
    /// Add obstacle at specified position using the new obstacle system
    pub fn add_obstacle(&mut self, obstacle: Obstacle) {
        self.obstacle_manager.add_obstacle(obstacle);
        if let Some(ref mut fluid) = self.fluid {
            self.obstacle_manager.apply_to_fluid(
                &mut fluid.s, &mut fluid.u, &mut fluid.v, &mut fluid.m,
                fluid.num_x, fluid.num_y, fluid.h
            );
        }
        self.show_obstacle = true;
    }
    
    /// Clear all obstacles
    pub fn clear_obstacles(&mut self) {
        self.obstacle_manager.clear();
        self.show_obstacle = false;
        
        // Reset fluid to open state (this requires reinitializing the scene)
        if let Some(ref mut fluid) = self.fluid {
            for i in 0..fluid.num_x {
                for j in 0..fluid.num_y {
                    fluid.s[[i, j]] = 1.0; // Reset to fluid
                }
            }
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
                
                // Apply boundary layer control for wind tunnel scenarios (less frequently)
                if matches!(self.scene_type, SceneType::WindTunnel) {
                    // Only apply every 5th step for performance
                    static mut STEP_COUNTER: usize = 0;
                    unsafe {
                        STEP_COUNTER += 1;
                        if STEP_COUNTER % 5 == 0 {
                            fluid.apply_boundary_layer_control(self.inflow_velocity);
                        }
                    }
                }
            }
        }
    }
}

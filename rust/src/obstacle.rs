use ndarray::Array2;

/// Types of obstacles that can be placed in the flow
#[derive(Debug, Clone, PartialEq)]
pub enum ObstacleShape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Airfoil { chord: f64, thickness: f64, angle: f64 },
    Cylinder { radius: f64 },
}

/// Obstacle representation for CFD simulation
#[derive(Debug, Clone)]
pub struct Obstacle {
    pub x: f64,           // Center x position
    pub y: f64,           // Center y position
    pub angle: f64,       // Rotation angle in radians
    pub shape: ObstacleShape,
    pub is_porous: bool,  // Whether it's porous (like a radiator)
    pub porosity: f64,    // Porosity factor (0-1)
    pub resistance: f64,  // Flow resistance coefficient
}

impl Obstacle {
    /// Create a new circular obstacle
    pub fn new_circle(x: f64, y: f64, radius: f64) -> Self {
        Self {
            x,
            y,
            angle: 0.0,
            shape: ObstacleShape::Circle { radius },
            is_porous: false,
            porosity: 0.0,
            resistance: 0.0,
        }
    }
    
    /// Create a new rectangular obstacle
    pub fn new_rectangle(x: f64, y: f64, width: f64, height: f64, angle: f64) -> Self {
        Self {
            x,
            y,
            angle,
            shape: ObstacleShape::Rectangle { width, height },
            is_porous: false,
            porosity: 0.0,
            resistance: 0.0,
        }
    }
    
    /// Create a new airfoil obstacle
    pub fn new_airfoil(x: f64, y: f64, chord: f64, thickness: f64, angle: f64) -> Self {
        Self {
            x,
            y,
            angle,
            shape: ObstacleShape::Airfoil { chord, thickness, angle },
            is_porous: false,
            porosity: 0.0,
            resistance: 0.0,
        }
    }
    
    /// Create a porous obstacle (like a radiator)
    pub fn new_porous_rectangle(x: f64, y: f64, width: f64, height: f64, angle: f64, porosity: f64, resistance: f64) -> Self {
        Self {
            x,
            y,
            angle,
            shape: ObstacleShape::Rectangle { width, height },
            is_porous: true,
            porosity,
            resistance,
        }
    }
    
    /// Check if a point is inside the obstacle
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        // Transform point to obstacle-local coordinates
        let cos_angle = self.angle.cos();
        let sin_angle = self.angle.sin();
        
        let dx = px - self.x;
        let dy = py - self.y;
        
        let local_x = dx * cos_angle + dy * sin_angle;
        let local_y = -dx * sin_angle + dy * cos_angle;
        
        match &self.shape {
            ObstacleShape::Circle { radius } => {
                local_x * local_x + local_y * local_y <= radius * radius
            }
            ObstacleShape::Cylinder { radius } => {
                local_x * local_x + local_y * local_y <= radius * radius
            }
            ObstacleShape::Rectangle { width, height } => {
                local_x.abs() <= width * 0.5 && local_y.abs() <= height * 0.5
            }
            ObstacleShape::Airfoil { chord, thickness, angle: _ } => {
                // Simple symmetric airfoil approximation (NACA 4-digit style)
                if local_x < 0.0 || local_x > *chord {
                    return false;
                }
                
                let x_norm = local_x / chord;
                let half_thickness = thickness * 0.5 * chord * 
                    (0.2969 * x_norm.sqrt() - 
                     0.1260 * x_norm - 
                     0.3516 * x_norm * x_norm + 
                     0.2843 * x_norm * x_norm * x_norm - 
                     0.1015 * x_norm * x_norm * x_norm * x_norm);
                
                local_y.abs() <= half_thickness
            }
        }
    }
    
    /// Get the distance to the nearest surface of the obstacle
    pub fn distance_to_surface(&self, px: f64, py: f64) -> f64 {
        let cos_angle = self.angle.cos();
        let sin_angle = self.angle.sin();
        
        let dx = px - self.x;
        let dy = py - self.y;
        
        let local_x = dx * cos_angle + dy * sin_angle;
        let local_y = -dx * sin_angle + dy * cos_angle;
        
        match &self.shape {
            ObstacleShape::Circle { radius } | ObstacleShape::Cylinder { radius } => {
                let dist = (local_x * local_x + local_y * local_y).sqrt();
                (dist - radius).abs()
            }
            ObstacleShape::Rectangle { width, height } => {
                let half_w = width * 0.5;
                let half_h = height * 0.5;
                
                let dx = (local_x.abs() - half_w).max(0.0);
                let dy = (local_y.abs() - half_h).max(0.0);
                (dx * dx + dy * dy).sqrt()
            }
            ObstacleShape::Airfoil { chord, thickness, angle: _ } => {
                // Simplified distance calculation for airfoil
                if local_x < 0.0 || local_x > *chord {
                    return f64::INFINITY;
                }
                
                let x_norm = local_x / chord;
                let half_thickness = thickness * 0.5 * chord * 
                    (0.2969 * x_norm.sqrt() - 
                     0.1260 * x_norm - 
                     0.3516 * x_norm * x_norm + 
                     0.2843 * x_norm * x_norm * x_norm - 
                     0.1015 * x_norm * x_norm * x_norm * x_norm);
                
                (local_y.abs() - half_thickness).abs()
            }
        }
    }
}

/// Manager for multiple obstacles in the simulation
pub struct ObstacleManager {
    pub obstacles: Vec<Obstacle>,
}

impl ObstacleManager {
    /// Create a new obstacle manager
    pub fn new() -> Self {
        Self {
            obstacles: Vec::new(),
        }
    }
    
    /// Add an obstacle to the simulation
    pub fn add_obstacle(&mut self, obstacle: Obstacle) {
        self.obstacles.push(obstacle);
    }
    
    /// Remove all obstacles
    pub fn clear(&mut self) {
        self.obstacles.clear();
    }
    
    /// Apply obstacles to the fluid simulation arrays
    pub fn apply_to_fluid(&self, 
        fluid_s: &mut Array2<f64>, 
        fluid_u: &mut Array2<f64>, 
        fluid_v: &mut Array2<f64>, 
        fluid_m: &mut Array2<f64>,
        num_x: usize, 
        num_y: usize, 
        h: f64
    ) {
        for obstacle in &self.obstacles {
            self.apply_single_obstacle(obstacle, fluid_s, fluid_u, fluid_v, fluid_m, num_x, num_y, h);
        }
    }
    
    /// Apply a single obstacle to the fluid arrays with improved boundary conditions
    fn apply_single_obstacle(&self,
        obstacle: &Obstacle,
        fluid_s: &mut Array2<f64>, 
        fluid_u: &mut Array2<f64>, 
        fluid_v: &mut Array2<f64>, 
        fluid_m: &mut Array2<f64>,
        num_x: usize, 
        num_y: usize, 
        h: f64
    ) {
        // First pass: mark solid cells
        for i in 1..num_x - 2 {
            for j in 1..num_y - 2 {
                let px = (i as f64 + 0.5) * h;
                let py = (j as f64 + 0.5) * h;
                
                if obstacle.contains_point(px, py) {
                    if obstacle.is_porous {
                        // Porous obstacle - reduce velocity but don't make solid
                        fluid_s[[i, j]] = obstacle.porosity;
                        fluid_u[[i, j]] *= obstacle.porosity;
                        fluid_u[[i + 1, j]] *= obstacle.porosity;
                        fluid_v[[i, j]] *= obstacle.porosity;
                        fluid_v[[i, j + 1]] *= obstacle.porosity;
                    } else {
                        // Solid obstacle
                        fluid_s[[i, j]] = 0.0;
                        fluid_m[[i, j]] = 0.0;
                    }
                }
            }
        }
        
        // Second pass: enforce no-slip boundary conditions on velocity
        for i in 1..num_x - 1 {
            for j in 1..num_y - 1 {
                // Check u-velocity points (face-centered)
                let u_px = (i as f64) * h;
                let u_py = (j as f64 + 0.5) * h;
                
                // Set u-velocity to zero if either adjacent cell is solid
                let left_solid = i > 0 && fluid_s[[i - 1, j]] == 0.0;
                let right_solid = i < num_x - 1 && fluid_s[[i, j]] == 0.0;
                if left_solid || right_solid || obstacle.contains_point(u_px, u_py) {
                    fluid_u[[i, j]] = 0.0;
                }
                
                // Check v-velocity points (face-centered)
                let v_px = (i as f64 + 0.5) * h;
                let v_py = (j as f64) * h;
                
                // Set v-velocity to zero if either adjacent cell is solid
                let bottom_solid = j > 0 && fluid_s[[i, j - 1]] == 0.0;
                let top_solid = j < num_y - 1 && fluid_s[[i, j]] == 0.0;
                if bottom_solid || top_solid || obstacle.contains_point(v_px, v_py) {
                    fluid_v[[i, j]] = 0.0;
                }
            }
        }
    }
    
    /// Check if any obstacle contains a point
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        self.obstacles.iter().any(|obs| obs.contains_point(x, y))
    }
    
    /// Get all obstacles as reference
    pub fn get_obstacles(&self) -> &Vec<Obstacle> {
        &self.obstacles
    }
}

impl Default for ObstacleManager {
    fn default() -> Self {
        Self::new()
    }
}

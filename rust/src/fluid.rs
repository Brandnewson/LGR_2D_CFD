use ndarray::Array2;

/// Field types for different physical quantities
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    U,      // Horizontal velocity
    V,      // Vertical velocity
    Smoke,  // Scalar field (smoke/dye concentration)
}

/// 2D Incompressible Fluid Simulator
/// Based on the Eulerian grid-based approach from the JavaScript implementation
pub struct Fluid {
    pub density: f64,
    pub num_x: usize,
    pub num_y: usize,
    pub h: f64,  // Grid spacing
    
    // Velocity fields (staggered grid)
    pub u: Array2<f64>,      // Horizontal velocity
    pub v: Array2<f64>,      // Vertical velocity
    pub new_u: Array2<f64>,  // Temporary storage for advection
    pub new_v: Array2<f64>,  // Temporary storage for advection
    
    // Pressure field
    pub p: Array2<f64>,
    
    // Solid field (0 = solid, 1 = fluid)
    pub s: Array2<f64>,
    
    // Scalar field (smoke/dye)
    pub m: Array2<f64>,      // Current concentration
    pub new_m: Array2<f64>,  // Temporary storage for advection
}

impl Fluid {
    /// Create a new fluid simulation with given parameters
    pub fn new(density: f64, num_x: usize, num_y: usize, h: f64) -> Self {
        let total_x = num_x + 2; // Add ghost cells
        let total_y = num_y + 2;
        
        let mut fluid = Self {
            density,
            num_x: total_x,
            num_y: total_y,
            h,
            u: Array2::zeros((total_x, total_y)),
            v: Array2::zeros((total_x, total_y)),
            new_u: Array2::zeros((total_x, total_y)),
            new_v: Array2::zeros((total_x, total_y)),
            p: Array2::zeros((total_x, total_y)),
            s: Array2::zeros((total_x, total_y)),
            m: Array2::zeros((total_x, total_y)),
            new_m: Array2::zeros((total_x, total_y)),
        };
        
        // Initialize solid field (1.0 = fluid, 0.0 = solid)
        fluid.s.fill(1.0);
        
        fluid
    }
    
    /// Apply gravity to the velocity field
    pub fn integrate(&mut self, dt: f64, gravity: f64) {
        for i in 1..self.num_x {
            for j in 1..self.num_y - 1 {
                if self.s[[i, j]] != 0.0 && self.s[[i, j - 1]] != 0.0 {
                    self.v[[i, j]] += gravity * dt;
                }
            }
        }
    }
    
    /// Solve for incompressibility using Gauss-Seidel iteration with early convergence
    pub fn solve_incompressibility(&mut self, num_iters: usize, dt: f64, over_relaxation: f64) {
        let cp = self.density * self.h / dt;
        let convergence_threshold = 1e-6;
        
        for iter in 0..num_iters {
            let mut max_change: f64 = 0.0;
            
            for i in 1..self.num_x - 1 {
                for j in 1..self.num_y - 1 {
                    if self.s[[i, j]] == 0.0 {
                        continue;
                    }
                    
                    let sx0 = self.s[[i - 1, j]];
                    let sx1 = self.s[[i + 1, j]];
                    let sy0 = self.s[[i, j - 1]];
                    let sy1 = self.s[[i, j + 1]];
                    let s_sum = sx0 + sx1 + sy0 + sy1;
                    
                    if s_sum == 0.0 {
                        continue;
                    }
                    
                    // Calculate divergence
                    let div = self.u[[i + 1, j]] - self.u[[i, j]] 
                            + self.v[[i, j + 1]] - self.v[[i, j]];
                    
                    let mut p = -div / s_sum;
                    p *= over_relaxation;
                    
                    let p_change = cp * p;
                    self.p[[i, j]] += p_change;
                    max_change = max_change.max(p_change.abs());
                    
                    // Update velocities
                    self.u[[i, j]]     -= sx0 * p;
                    self.u[[i + 1, j]] += sx1 * p;
                    self.v[[i, j]]     -= sy0 * p;
                    self.v[[i, j + 1]] += sy1 * p;
                }
            }
            
            // Early convergence check (skip for first few iterations)
            if iter > 3 && max_change < convergence_threshold {
                break;
            }
        }
    }
    
    /// Extrapolate velocities to boundaries and enforce boundary conditions
    pub fn extrapolate(&mut self) {
        // Horizontal boundaries (top and bottom)
        for i in 0..self.num_x {
            // Bottom boundary (j=0)
            self.u[[i, 0]] = self.u[[i, 1]];
            self.v[[i, 0]] = 0.0; // No penetration
            
            // Top boundary (j=num_y-1)
            self.u[[i, self.num_y - 1]] = self.u[[i, self.num_y - 2]];
            self.v[[i, self.num_y - 1]] = 0.0; // No penetration
        }
        
        // Vertical boundaries (left and right)
        for j in 0..self.num_y {
            // Left boundary (i=0) - maintain inflow if set
            if self.u[[0, j]] == 0.0 {
                self.u[[0, j]] = self.u[[1, j]];
            }
            self.v[[0, j]] = self.v[[1, j]];
            
            // Right boundary (i=num_x-1) - outflow
            self.u[[self.num_x - 1, j]] = self.u[[self.num_x - 2, j]];
            self.v[[self.num_x - 1, j]] = self.v[[self.num_x - 2, j]];
        }
    }
    
    /// Sample a field at arbitrary coordinates using bilinear interpolation
    pub fn sample_field(&self, x: f64, y: f64, field: FieldType) -> f64 {
        let h1 = 1.0 / self.h;
        let h2 = 0.5 * self.h;
        
        // Clamp coordinates to valid domain
        let x_clamped = x.max(self.h).min((self.num_x as f64 - 1.0) * self.h);
        let y_clamped = y.max(self.h).min((self.num_y as f64 - 1.0) * self.h);
        
        let (dx, dy, field_data) = match field {
            FieldType::U => (0.0, h2, &self.u),
            FieldType::V => (h2, 0.0, &self.v),
            FieldType::Smoke => (h2, h2, &self.m),
        };
        
        let x_sample = x_clamped - dx;
        let y_sample = y_clamped - dy;
        
        let x0 = (x_sample * h1).floor() as usize;
        let x0 = x0.min(self.num_x - 1);
        let tx = (x_sample * h1) - (x0 as f64);
        let x1 = (x0 + 1).min(self.num_x - 1);
        
        let y0 = (y_sample * h1).floor() as usize;
        let y0 = y0.min(self.num_y - 1);
        let ty = (y_sample * h1) - (y0 as f64);
        let y1 = (y0 + 1).min(self.num_y - 1);
        
        let sx = 1.0 - tx;
        let sy = 1.0 - ty;
        
        sx * sy * field_data[[x0, y0]] +
        tx * sy * field_data[[x1, y0]] +
        tx * ty * field_data[[x1, y1]] +
        sx * ty * field_data[[x0, y1]]
    }
    
    /// Get averaged horizontal velocity at cell center
    pub fn avg_u(&self, i: usize, j: usize) -> f64 {
        if j == 0 { return self.u[[i, j]]; }
        (self.u[[i, j - 1]] + self.u[[i, j]] +
         self.u[[i + 1, j - 1]] + self.u[[i + 1, j]]) * 0.25
    }
    
    /// Get averaged vertical velocity at cell center
    pub fn avg_v(&self, i: usize, j: usize) -> f64 {
        if i == 0 { return self.v[[i, j]]; }
        (self.v[[i - 1, j]] + self.v[[i, j]] +
         self.v[[i - 1, j + 1]] + self.v[[i, j + 1]]) * 0.25
    }
    
    /// Advect velocity field using semi-Lagrangian method
    pub fn advect_velocity(&mut self, dt: f64) {
        self.new_u.assign(&self.u);
        self.new_v.assign(&self.v);
        
        let h2 = 0.5 * self.h;
        
        for i in 1..self.num_x {
            for j in 1..self.num_y {
                // Advect u component
                if self.s[[i, j]] != 0.0 && self.s[[i - 1, j]] != 0.0 && j < self.num_y - 1 {
                    let x = (i as f64) * self.h;
                    let y = (j as f64) * self.h + h2;
                    let u = self.u[[i, j]];
                    let v = self.avg_v(i, j);
                    
                    let x_prev = x - dt * u;
                    let y_prev = y - dt * v;
                    let u_new = self.sample_field(x_prev, y_prev, FieldType::U);
                    self.new_u[[i, j]] = u_new;
                }
                
                // Advect v component
                if self.s[[i, j]] != 0.0 && j > 0 && self.s[[i, j - 1]] != 0.0 && i < self.num_x - 1 {
                    let x = (i as f64) * self.h + h2;
                    let y = (j as f64) * self.h;
                    let u = self.avg_u(i, j);
                    let v = self.v[[i, j]];
                    
                    let x_prev = x - dt * u;
                    let y_prev = y - dt * v;
                    let v_new = self.sample_field(x_prev, y_prev, FieldType::V);
                    self.new_v[[i, j]] = v_new;
                }
            }
        }
        
        self.u.assign(&self.new_u);
        self.v.assign(&self.new_v);
    }
    
    /// Advect scalar field (smoke/dye)
    pub fn advect_smoke(&mut self, dt: f64) {
        self.new_m.assign(&self.m);
        
        let h2 = 0.5 * self.h;
        
        for i in 1..self.num_x - 1 {
            for j in 1..self.num_y - 1 {
                if self.s[[i, j]] != 0.0 {
                    let x = (i as f64) * self.h + h2;
                    let y = (j as f64) * self.h + h2;
                    let u = (self.u[[i, j]] + self.u[[i + 1, j]]) * 0.5;
                    let v = (self.v[[i, j]] + self.v[[i, j + 1]]) * 0.5;
                    
                    let x_prev = x - dt * u;
                    let y_prev = y - dt * v;
                    let m_new = self.sample_field(x_prev, y_prev, FieldType::Smoke);
                    self.new_m[[i, j]] = m_new;
                }
            }
        }
        
        self.m.assign(&self.new_m);
    }
    
    /// Enhanced simulation step with boundary condition enforcement
    pub fn simulate_with_boundaries(&mut self, dt: f64, gravity: f64, num_iters: usize, over_relaxation: f64, inflow_velocity: f64) {
        self.integrate(dt, gravity);
        
        self.p.fill(0.0);
        self.solve_incompressibility(num_iters, dt, over_relaxation);
        
        self.extrapolate();
        
        self.advect_velocity(dt);
        self.advect_smoke(dt);
        
        // Enforce boundary conditions only once at the end
        self.enforce_boundary_conditions(inflow_velocity);
    }
    
    /// Enforce specific boundary conditions (call after each simulation step)
    pub fn enforce_boundary_conditions(&mut self, inflow_velocity: f64) {
        // Maintain inflow velocity on left boundary
        for j in 1..self.num_y - 1 {
            self.u[[0, j]] = inflow_velocity;
            self.u[[1, j]] = inflow_velocity; // Also set second column for stability
        }
        
        // Ensure no-slip on walls
        for i in 0..self.num_x {
            // Bottom wall
            self.u[[i, 0]] = 0.0;
            self.v[[i, 0]] = 0.0;
            
            // Top wall
            self.u[[i, self.num_y - 1]] = 0.0;
            self.v[[i, self.num_y - 1]] = 0.0;
        }
        
        // Outflow boundary (right side) - zero gradient
        for j in 0..self.num_y {
            self.u[[self.num_x - 1, j]] = self.u[[self.num_x - 2, j]];
            self.v[[self.num_x - 1, j]] = self.v[[self.num_x - 2, j]];
        }
    }
    
    /// Advanced boundary layer control for realistic wind tunnel simulation
    pub fn apply_boundary_layer_control(&mut self, inflow_velocity: f64) {
        // Apply realistic boundary layer suction (like real wind tunnels)
        
        // Top wall boundary layer control
        if self.num_y > 4 {
            for i in 2..self.num_x - 2 {
                // Gradual velocity profile recovery near top wall
                let wall_distance = 3;
                for offset in 1..=wall_distance {
                    let j = self.num_y - 1 - offset;
                    let blend_factor = 1.0 - (offset as f64 / wall_distance as f64);
                    
                    // Gradually restore uniform flow profile
                    let target_u = inflow_velocity * (1.0 - 0.1 * blend_factor);
                    self.u[[i, j]] = self.u[[i, j]] * 0.8 + target_u * 0.2;
                    
                    // Small suction velocity to simulate boundary layer removal
                    self.v[[i, j]] = -0.05 * blend_factor; // Gentle suction
                }
            }
        }
        
        // Bottom wall boundary layer control
        if self.num_y > 4 {
            for i in 2..self.num_x - 2 {
                let wall_distance = 3;
                for offset in 1..=wall_distance {
                    let j = offset;
                    let blend_factor = 1.0 - (offset as f64 / wall_distance as f64);
                    
                    // Gradually restore uniform flow profile
                    let target_u = inflow_velocity * (1.0 - 0.1 * blend_factor);
                    self.u[[i, j]] = self.u[[i, j]] * 0.8 + target_u * 0.2;
                    
                    // Small suction velocity to simulate boundary layer removal
                    self.v[[i, j]] = 0.05 * blend_factor; // Gentle suction
                }
            }
        }
        
        // Maintain core flow velocity in the center region
        let core_start = self.num_y / 4;
        let core_end = 3 * self.num_y / 4;
        for i in 2..self.num_x - 10 { // Leave some development length
            for j in core_start..core_end {
                // Gently nudge core flow back to uniform velocity
                self.u[[i, j]] = self.u[[i, j]] * 0.95 + inflow_velocity * 0.05;
            }
        }
    }
}

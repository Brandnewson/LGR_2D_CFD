use crate::fluid::Fluid;
use serde::{Deserialize, Serialize};

/// Radiator geometry and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Radiator {
    pub x: f64,         // X position (center)
    pub y: f64,         // Y position (center)
    pub width: f64,     // Width of radiator
    pub height: f64,    // Height of radiator
    pub angle: f64,     // Angle in radians (0 = vertical)
    pub porosity: f64,  // Porosity (0-1, 1 = fully open)
    pub resistance: f64, // Flow resistance coefficient
}

/// Metrics for radiator performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiatorMetrics {
    pub angle_degrees: f64,
    pub mass_flow_rate: f64,      // kg/s
    pub pressure_drop: f64,       // Pa
    pub inlet_velocity: f64,      // m/s
    pub outlet_velocity: f64,     // m/s
    pub drag_force: f64,          // N
    pub lift_force: f64,          // N
    pub cooling_efficiency: f64,  // Dimensionless
    pub fan_power_required: f64,  // W
}

/// Radiator analysis utilities
pub struct RadiatorAnalyzer {
    pub radiators: Vec<Radiator>,
    pub metrics_history: Vec<RadiatorMetrics>,
}

impl RadiatorAnalyzer {
    pub fn new() -> Self {
        Self {
            radiators: Vec::new(),
            metrics_history: Vec::new(),
        }
    }
    
    /// Add a radiator to the analysis
    pub fn add_radiator(&mut self, radiator: Radiator) {
        self.radiators.push(radiator);
    }
    
    /// Set radiator as porous medium in the fluid domain
    pub fn apply_radiator_to_fluid(&self, fluid: &mut Fluid, radiator: &Radiator) {
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
        // Transform radiator corners
        let half_width = radiator.width * 0.5;
        let half_height = radiator.height * 0.5;
        
        for i in 1..fluid.num_x - 1 {
            for j in 1..fluid.num_y - 1 {
                let x = (i as f64 + 0.5) * fluid.h;
                let y = (j as f64 + 0.5) * fluid.h;
                
                // Transform to radiator local coordinates
                let dx = x - radiator.x;
                let dy = y - radiator.y;
                let local_x = dx * cos_angle + dy * sin_angle;
                let local_y = -dx * sin_angle + dy * cos_angle;
                
                // Check if point is inside radiator bounds
                if local_x.abs() <= half_width && local_y.abs() <= half_height {
                    // Apply porous medium model
                    self.apply_porous_medium_resistance(fluid, i, j, radiator);
                }
            }
        }
    }
    
    /// Apply porous medium resistance (Darcy-Forchheimer model)
    fn apply_porous_medium_resistance(&self, fluid: &mut Fluid, i: usize, j: usize, radiator: &Radiator) {
        let u = fluid.u[[i, j]];
        let v = fluid.v[[i, j]];
        let velocity_magnitude = (u * u + v * v).sqrt();
        
        if velocity_magnitude < 1e-6 {
            return; // Skip if velocity is too small
        }
        
        // Darcy-Forchheimer equation: F = -α*μ*v - β*ρ*|v|*v
        // Where α is inverse permeability, β is inertial resistance
        let permeability = radiator.porosity / radiator.resistance; // m²
        let alpha = 1.0 / permeability.max(1e-10);
        let beta = (1.0 - radiator.porosity) / (radiator.porosity.powi(3)) * 1.75; // Ergun equation
        
        // Viscous resistance (Darcy term)
        let mu = 1.8e-5; // Air viscosity at room temperature
        let viscous_resistance = alpha * mu;
        
        // Inertial resistance (Forchheimer term)
        let inertial_resistance = beta * fluid.density * velocity_magnitude;
        
        let total_resistance = viscous_resistance + inertial_resistance;
        let resistance_factor = total_resistance * fluid.h; // Scale by cell size
        
        // Apply resistance as force per unit volume
        let damping = 1.0 / (1.0 + resistance_factor);
        
        fluid.u[[i, j]] *= damping;
        fluid.v[[i, j]] *= damping;
        
        // Modify solid field to represent porous medium
        fluid.s[[i, j]] = radiator.porosity;
    }
    
    /// Calculate mass flow rate through radiator
    pub fn calculate_mass_flow(&self, fluid: &Fluid, radiator: &Radiator) -> f64 {
        let mut mass_flow = 0.0;
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
        // Calculate flow through radiator face (perpendicular to radiator)
        // Sample along the radiator height
        let samples = 20;
        let area_per_sample = radiator.height / samples as f64;
        
        for i in 0..samples {
            let t = (i as f64 + 0.5) / samples as f64 - 0.5; // Center of each segment
            let sample_y_local = t * radiator.height;
            
            // Transform to global coordinates (point on radiator face)
            let face_x = radiator.x - sample_y_local * sin_angle;
            let face_y = radiator.y + sample_y_local * cos_angle;
            
            // Ensure we're sampling within the domain
            if face_x < 0.0 || face_x >= (fluid.num_x as f64) * fluid.h ||
               face_y < 0.0 || face_y >= (fluid.num_y as f64) * fluid.h {
                continue;
            }
            
            // Sample velocity at this point
            let u = fluid.sample_field(face_x, face_y, crate::fluid::FieldType::U);
            let v = fluid.sample_field(face_x, face_y, crate::fluid::FieldType::V);
            
            // Component normal to radiator face (positive = flow through radiator)
            let normal_velocity = u * cos_angle + v * sin_angle;
            
            // Add to mass flow (area_per_sample * thickness * density * velocity)
            mass_flow += normal_velocity * area_per_sample * fluid.h * fluid.density;
        }
        
        mass_flow.abs()
    }
    
    /// Calculate pressure drop across radiator
    pub fn calculate_pressure_drop(&self, fluid: &Fluid, radiator: &Radiator) -> f64 {
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
        // Sample pressure upstream and downstream of radiator
        let probe_distance = radiator.width * 0.5 + 2.0 * fluid.h; // Go outside radiator bounds
        
        // Upstream point (before radiator in flow direction)
        let upstream_x = radiator.x - probe_distance * cos_angle;
        let upstream_y = radiator.y - probe_distance * sin_angle;
        
        // Downstream point (after radiator in flow direction)
        let downstream_x = radiator.x + probe_distance * cos_angle;
        let downstream_y = radiator.y + probe_distance * sin_angle;
        
        let p_upstream = self.sample_pressure_line_average(fluid, upstream_x, upstream_y, radiator.height, radiator.angle);
        let p_downstream = self.sample_pressure_line_average(fluid, downstream_x, downstream_y, radiator.height, radiator.angle);
        
        // Pressure drop should be positive when flow is slowed down
        p_upstream - p_downstream
    }
    
    /// Sample average pressure along a line perpendicular to flow
    fn sample_pressure_line_average(&self, fluid: &Fluid, x: f64, y: f64, length: f64, angle: f64) -> f64 {
        let samples = 10;
        let mut pressure_sum = 0.0;
        let mut valid_samples = 0;
        
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();
        
        for i in 0..samples {
            let t = (i as f64 + 0.5) / samples as f64 - 0.5; // -0.5 to 0.5
            let sample_x = x - t * length * sin_angle;
            let sample_y = y + t * length * cos_angle;
            
            // Check bounds
            if sample_x >= fluid.h && sample_x < (fluid.num_x as f64 - 1.0) * fluid.h &&
               sample_y >= fluid.h && sample_y < (fluid.num_y as f64 - 1.0) * fluid.h {
                
                let pressure = self.sample_pressure_at_point(fluid, sample_x, sample_y);
                pressure_sum += pressure;
                valid_samples += 1;
            }
        }
        
        if valid_samples > 0 {
            pressure_sum / valid_samples as f64
        } else {
            0.0
        }
    }
    
    /// Calculate forces on radiator
    pub fn calculate_forces(&self, fluid: &Fluid, radiator: &Radiator) -> (f64, f64) {
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
        let mut drag_force = 0.0;
        let mut lift_force = 0.0;
        
        // Calculate pressure and viscous forces around radiator perimeter
        let samples = 100;
        for i in 0..samples {
            let t = (i as f64) / (samples as f64) * 2.0 * std::f64::consts::PI;
            
            let dx = radiator.width * 0.5 * t.cos();
            let dy = radiator.height * 0.5 * t.sin();
            
            // Transform to global coordinates
            let x = radiator.x + dx * cos_angle - dy * sin_angle;
            let y = radiator.y + dx * sin_angle + dy * cos_angle;
            
            // Sample pressure at this point
            let pressure = self.sample_pressure_at_point(fluid, x, y);
            
            // Normal vector (outward from radiator)
            let normal_x = dx.cos() * cos_angle - dy.sin() * sin_angle;
            let normal_y = dx.cos() * sin_angle + dy.sin() * cos_angle;
            
            // Force contribution
            let ds = 2.0 * std::f64::consts::PI / samples as f64;
            let force_x = pressure * normal_x * ds;
            let force_y = pressure * normal_y * ds;
            
            // Project to drag (x-direction) and lift (y-direction)
            drag_force += force_x;
            lift_force += force_y;
        }
        
        (drag_force, lift_force)
    }
    
    /// Sample pressure at a specific point
    fn sample_pressure_at_point(&self, fluid: &Fluid, x: f64, y: f64) -> f64 {
        let grid_x = (x / fluid.h).max(0.0).min((fluid.num_x - 1) as f64);
        let grid_y = (y / fluid.h).max(0.0).min((fluid.num_y - 1) as f64);
        
        let i0 = grid_x.floor() as usize;
        let j0 = grid_y.floor() as usize;
        let i1 = (i0 + 1).min(fluid.num_x - 1);
        let j1 = (j0 + 1).min(fluid.num_y - 1);
        
        let tx = grid_x - i0 as f64;
        let ty = grid_y - j0 as f64;
        
        (1.0 - tx) * (1.0 - ty) * fluid.p[[i0, j0]] +
        tx * (1.0 - ty) * fluid.p[[i1, j0]] +
        tx * ty * fluid.p[[i1, j1]] +
        (1.0 - tx) * ty * fluid.p[[i0, j1]]
    }
    
    /// Analyze radiator performance at given angle
    pub fn analyze_performance(&mut self, fluid: &Fluid, radiator: &Radiator) -> RadiatorMetrics {
        let mass_flow = self.calculate_mass_flow(fluid, radiator);
        let pressure_drop = self.calculate_pressure_drop(fluid, radiator);
        let (drag_force, lift_force) = self.calculate_forces(fluid, radiator);
        
        // Calculate inlet/outlet velocities
        let inlet_velocity = self.calculate_inlet_velocity(fluid, radiator);
        let outlet_velocity = self.calculate_outlet_velocity(fluid, radiator);
        
        // Estimate cooling efficiency (simplified)
        let cooling_efficiency = (mass_flow / (mass_flow + 0.1)).min(1.0);
        
        // Estimate fan power required
        let volumetric_flow = mass_flow / fluid.density;
        let fan_power = pressure_drop * volumetric_flow;
        
        let metrics = RadiatorMetrics {
            angle_degrees: radiator.angle.to_degrees(),
            mass_flow_rate: mass_flow,
            pressure_drop,
            inlet_velocity,
            outlet_velocity,
            drag_force,
            lift_force,
            cooling_efficiency,
            fan_power_required: fan_power,
        };
        
        self.metrics_history.push(metrics.clone());
        metrics
    }
    
    fn calculate_inlet_velocity(&self, fluid: &Fluid, radiator: &Radiator) -> f64 {
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
        let inlet_x = radiator.x - 0.05 * cos_angle;
        let inlet_y = radiator.y - 0.05 * sin_angle;
        
        let u = fluid.sample_field(inlet_x, inlet_y, crate::fluid::FieldType::U);
        let v = fluid.sample_field(inlet_x, inlet_y, crate::fluid::FieldType::V);
        
        (u * u + v * v).sqrt()
    }
    
    fn calculate_outlet_velocity(&self, fluid: &Fluid, radiator: &Radiator) -> f64 {
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
        let outlet_x = radiator.x + 0.05 * cos_angle;
        let outlet_y = radiator.y + 0.05 * sin_angle;
        
        let u = fluid.sample_field(outlet_x, outlet_y, crate::fluid::FieldType::U);
        let v = fluid.sample_field(outlet_x, outlet_y, crate::fluid::FieldType::V);
        
        (u * u + v * v).sqrt()
    }
    
    /// Save metrics to JSON file
    pub fn save_metrics<P: AsRef<std::path::Path>>(&self, filename: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.metrics_history)?;
        std::fs::write(filename, json)?;
        Ok(())
    }
    
    /// Print performance summary
    pub fn print_summary(&self) {
        println!("\n📊 Radiator Performance Summary");
        println!("================================");
        
        for metrics in &self.metrics_history {
            println!("Angle: {:.1}°", metrics.angle_degrees);
            println!("  Mass flow rate: {:.4} kg/s", metrics.mass_flow_rate);
            println!("  Pressure drop: {:.2} Pa", metrics.pressure_drop);
            println!("  Drag force: {:.3} N", metrics.drag_force);
            println!("  Lift force: {:.3} N", metrics.lift_force);
            println!("  Cooling efficiency: {:.3}", metrics.cooling_efficiency);
            println!("  Fan power required: {:.2} W", metrics.fan_power_required);
            println!();
        }
    }
}

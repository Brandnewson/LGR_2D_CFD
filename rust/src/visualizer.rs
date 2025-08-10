use crate::fluid::{Fluid, FieldType};
use crate::radiator::Radiator;
use crate::obstacle::{Obstacle, ObstacleShape, ObstacleManager};
use plotters::prelude::*;
use std::path::Path;

/// Visualization utilities for the CFD simulation
pub struct Visualizer {
    width: u32,
    height: u32,
}

impl Visualizer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    
    /// Generate scientific colormap for field visualization
    pub fn get_sci_color(val: f64, min_val: f64, max_val: f64) -> (u8, u8, u8) {
        let val_clamped = val.max(min_val).min(max_val - 0.0001);
        let d = max_val - min_val;
        let normalized = if d == 0.0 { 0.5 } else { (val_clamped - min_val) / d };
        
        let m = 0.25;
        let num = (normalized / m).floor() as i32;
        let s = (normalized - (num as f64) * m) / m;
        
        let (r, g, b) = match num {
            0 => (0.0, s, 1.0),
            1 => (0.0, 1.0, 1.0 - s),
            2 => (s, 1.0, 0.0),
            3 => (1.0, 1.0 - s, 0.0),
            _ => (1.0, 0.0, 0.0),
        };
        
        ((255.0 * r) as u8, (255.0 * g) as u8, (255.0 * b) as u8)
    }
    
    /// Save pressure field visualization with optional radiator overlay
    pub fn save_pressure_field_with_radiator<P: AsRef<Path>>(
        &self,
        fluid: &Fluid,
        radiator: Option<&Radiator>,
        filename: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&filename, (self.width, self.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        // Find pressure range
        let mut min_p = fluid.p[[0, 0]];
        let mut max_p = fluid.p[[0, 0]];
        
        for i in 0..fluid.num_x {
            for j in 0..fluid.num_y {
                min_p = min_p.min(fluid.p[[i, j]]);
                max_p = max_p.max(fluid.p[[i, j]]);
            }
        }
        
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        // Draw pressure field
        for i in 0..fluid.num_x {
            for j in 0..fluid.num_y {
                let pressure = fluid.p[[i, j]];
                let (r, g, b) = Self::get_sci_color(pressure, min_p, max_p);
                
                let x1 = (i as f64 * cell_width) as i32;
                let y1 = ((fluid.num_y - j - 1) as f64 * cell_height) as i32;
                let x2 = ((i + 1) as f64 * cell_width) as i32;
                let y2 = ((fluid.num_y - j) as f64 * cell_height) as i32;
                
                root.draw(&Rectangle::new(
                    [(x1, y1), (x2, y2)],
                    RGBColor(r, g, b).filled(),
                ))?;
            }
        }
        
        // Draw radiator if provided
        if let Some(rad) = radiator {
            self.draw_radiator(&root, rad, fluid, &BLACK)?;
        }
        
        root.present()?;
        println!("Pressure field saved to {:?}", filename.as_ref());
        Ok(())
    }
    
    /// Save velocity field visualization
    pub fn save_velocity_field<P: AsRef<Path>>(
        &self,
        fluid: &Fluid,
        filename: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&filename, (self.width, self.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        let scale = 50.0; // Velocity arrow scaling factor
        
        for i in 1..fluid.num_x - 1 {
            for j in 1..fluid.num_y - 1 {
                if fluid.s[[i, j]] != 0.0 {
                    let u = (fluid.u[[i, j]] + fluid.u[[i + 1, j]]) * 0.5;
                    let v = (fluid.v[[i, j]] + fluid.v[[i, j + 1]]) * 0.5;
                    
                    let x = (i as f64 + 0.5) * cell_width;
                    let y = (fluid.num_y as f64 - j as f64 - 0.5) * cell_height;
                    
                    let dx = u * scale;
                    let dy = -v * scale; // Flip y-axis for display
                    
                    if u.abs() > 0.01 || v.abs() > 0.01 {
                        root.draw(&PathElement::new(
                            vec![(x as i32, y as i32), ((x + dx) as i32, (y + dy) as i32)],
                            &RED,
                        ))?;
                    }
                }
            }
        }
        
        root.present()?;
        println!("Velocity field saved to {:?}", filename.as_ref());
        Ok(())
    }
    
    /// Save smoke/dye concentration field
    pub fn save_smoke_field<P: AsRef<Path>>(
        &self,
        fluid: &Fluid,
        filename: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&filename, (self.width, self.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        for i in 0..fluid.num_x {
            for j in 0..fluid.num_y {
                let smoke = fluid.m[[i, j]];
                let intensity = (255.0 * (1.0 - smoke).max(0.0).min(1.0)) as u8;
                
                let x1 = (i as f64 * cell_width) as i32;
                let y1 = ((fluid.num_y - j - 1) as f64 * cell_height) as i32;
                let x2 = ((i + 1) as f64 * cell_width) as i32;
                let y2 = ((fluid.num_y - j) as f64 * cell_height) as i32;
                
                root.draw(&Rectangle::new(
                    [(x1, y1), (x2, y2)],
                    RGBColor(intensity, intensity, intensity).filled(),
                ))?;
            }
        }
        
        root.present()?;
        println!("Smoke field saved to {:?}", filename.as_ref());
        Ok(())
    }
    
    /// Draw smoke field on an existing drawing area
    pub fn draw_smoke_field<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        fluid: &Fluid,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        for i in 0..fluid.num_x {
            for j in 0..fluid.num_y {
                let smoke = fluid.m[[i, j]];
                let intensity = (255.0 * (1.0 - smoke).max(0.0).min(1.0)) as u8;
                
                let x1 = (i as f64 * cell_width) as i32;
                let y1 = ((fluid.num_y - j - 1) as f64 * cell_height) as i32;
                let x2 = ((i + 1) as f64 * cell_width) as i32;
                let y2 = ((fluid.num_y - j) as f64 * cell_height) as i32;
                
                area.draw(&Rectangle::new(
                    [(x1, y1), (x2, y2)],
                    RGBColor(intensity, intensity, intensity).filled(),
                ))?;
            }
        }
        
        Ok(())
    }
    
    /// Save streamlines visualization with optional radiator overlay
    pub fn save_streamlines_with_radiator<P: AsRef<Path>>(
        &self,
        fluid: &Fluid,
        radiator: Option<&Radiator>,
        filename: P,
        num_streamlines: usize,
        max_steps: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&filename, (self.width, self.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        // Generate streamlines starting from left boundary
        for stream_i in 0..num_streamlines {
            let start_y = (stream_i as f64 + 1.0) * fluid.num_y as f64 / (num_streamlines as f64 + 1.0);
            let mut x = 0.1; // Start slightly inside domain
            let mut y = start_y * fluid.h;
            
            let mut points = Vec::new();
            
            for _step in 0..max_steps {
                let u = fluid.sample_field(x, y, FieldType::U);
                let v = fluid.sample_field(x, y, FieldType::V);
                
                if u.abs() < 0.001 && v.abs() < 0.001 {
                    break; // Stop if velocity is too small
                }
                
                let screen_x = (x / fluid.h * cell_width) as i32;
                let screen_y = ((fluid.num_y as f64 - y / fluid.h) * cell_height) as i32;
                
                if screen_x < 0 || screen_x >= self.width as i32 || 
                   screen_y < 0 || screen_y >= self.height as i32 {
                    break; // Stop if outside domain
                }
                
                points.push((screen_x, screen_y));
                
                // Advance position
                let dt = 0.01;
                x += u * dt;
                y += v * dt;
                
                if x >= (fluid.num_x - 1) as f64 * fluid.h {
                    break; // Stop at right boundary
                }
            }
            
            if points.len() > 1 {
                root.draw(&PathElement::new(points, &BLUE))?;
            }
        }
        
        // Draw radiator if provided
        if let Some(rad) = radiator {
            self.draw_radiator(&root, rad, fluid, &RED)?;
        }
        
        root.present()?;
        println!("Streamlines saved to {:?}", filename.as_ref());
        Ok(())
    }
    
    /// Draw radiator geometry on the plot
    fn draw_radiator<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        radiator: &Radiator,
        fluid: &Fluid,
        color: &RGBColor,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        // Convert radiator coordinates to screen coordinates
        let center_x = (radiator.x / fluid.h * cell_width) as i32;
        let center_y = ((fluid.num_y as f64 - radiator.y / fluid.h) * cell_height) as i32;
        
        let half_width = (radiator.width / fluid.h * cell_width * 0.5) as i32;
        let half_height = (radiator.height / fluid.h * cell_height * 0.5) as i32;
        
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
        // Calculate rotated corner points
        let corners = [
            (-half_width, -half_height),
            (half_width, -half_height),
            (half_width, half_height),
            (-half_width, half_height),
        ];
        
        let mut rotated_corners = Vec::new();
        for (dx, dy) in corners.iter() {
            let rot_x = center_x + ((*dx as f64) * cos_angle - (*dy as f64) * sin_angle) as i32;
            let rot_y = center_y + ((*dx as f64) * sin_angle + (*dy as f64) * cos_angle) as i32;
            rotated_corners.push((rot_x, rot_y));
        }
        
        // Draw radiator outline
        for i in 0..4 {
            let start = rotated_corners[i];
            let end = rotated_corners[(i + 1) % 4];
            root.draw(&PathElement::new(vec![start, end], color.stroke_width(3)))?;
        }
        
        // Draw radiator fins/matrix pattern
        let num_fins = 8;
        for i in 0..num_fins {
            let t = (i as f64) / (num_fins as f64 - 1.0) * 2.0 - 1.0;
            let fin_x = (t * half_width as f64) as i32;
            
            let start_x = center_x + (fin_x as f64 * cos_angle) as i32;
            let start_y = center_y + (fin_x as f64 * sin_angle) as i32;
            let end_x = center_x + (fin_x as f64 * cos_angle - half_height as f64 * sin_angle) as i32;
            let end_y = center_y + (fin_x as f64 * sin_angle + half_height as f64 * cos_angle) as i32;
            
            root.draw(&PathElement::new(
                vec![(start_x, start_y), (end_x, end_y)], 
                &color.mix(0.5)
            ))?;
        }
        
        Ok(())
    }
    
    /// Keep the original method for backward compatibility
    pub fn save_pressure_field<P: AsRef<Path>>(
        &self,
        fluid: &Fluid,
        filename: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.save_pressure_field_with_radiator(fluid, None, filename)
    }
    
    /// Keep the original method for backward compatibility
    pub fn save_streamlines<P: AsRef<Path>>(
        &self,
        fluid: &Fluid,
        filename: P,
        num_streamlines: usize,
        max_steps: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.save_streamlines_with_radiator(fluid, None, filename, num_streamlines, max_steps)
    }
    
    /// Draw obstacles on the given drawing area
    pub fn draw_obstacles<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        obstacle_manager: &ObstacleManager,
        fluid: &Fluid,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        for obstacle in obstacle_manager.get_obstacles() {
            self.draw_single_obstacle(area, obstacle, fluid, cell_width, cell_height)?;
        }
        
        Ok(())
    }
    
    /// Draw a single obstacle
    fn draw_single_obstacle<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        obstacle: &Obstacle,
        fluid: &Fluid,
        cell_width: f64,
        cell_height: f64,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        let center_x = (obstacle.x / fluid.h * cell_width) as i32;
        let center_y = ((fluid.num_y as f64 - obstacle.y / fluid.h) * cell_height) as i32;
        
        let color = if obstacle.is_porous { &BLUE } else { &RED };
        let line_width = 3;
        
        match &obstacle.shape {
            ObstacleShape::Circle { radius } | ObstacleShape::Cylinder { radius } => {
                let screen_radius = (radius / fluid.h * cell_width.min(cell_height)) as i32;
                
                // Draw circle outline
                area.draw(&Circle::new((center_x, center_y), screen_radius, color.stroke_width(line_width)))?;
                
                // Fill if solid
                if !obstacle.is_porous {
                    area.draw(&Circle::new((center_x, center_y), screen_radius, color.filled()))?;
                }
            }
            ObstacleShape::Rectangle { width, height } => {
                let half_width = (width / fluid.h * cell_width * 0.5) as i32;
                let half_height = (height / fluid.h * cell_height * 0.5) as i32;
                
                self.draw_rotated_rectangle(
                    area, center_x, center_y, half_width, half_height, 
                    obstacle.angle, color, line_width, !obstacle.is_porous
                )?;
            }
            ObstacleShape::Airfoil { chord, thickness: _, angle: _ } => {
                // Simplified airfoil drawing as an elongated shape
                let chord_pixels = (chord / fluid.h * cell_width) as i32;
                let thickness_pixels = (chord * 0.12 / fluid.h * cell_height) as i32; // 12% thickness
                
                self.draw_rotated_rectangle(
                    area, center_x, center_y, chord_pixels / 2, thickness_pixels / 2,
                    obstacle.angle, color, line_width, !obstacle.is_porous
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Draw a rotated rectangle
    fn draw_rotated_rectangle<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        center_x: i32,
        center_y: i32,
        half_width: i32,
        half_height: i32,
        angle: f64,
        color: &RGBColor,
        line_width: u32,
        filled: bool,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        
        // Calculate rotated corners
        let corners = [
            (-half_width, -half_height),
            (half_width, -half_height),
            (half_width, half_height),
            (-half_width, half_height),
        ];
        
        let mut rotated_corners = Vec::new();
        for (dx, dy) in corners.iter() {
            let rot_x = center_x + ((*dx as f64) * cos_angle - (*dy as f64) * sin_angle) as i32;
            let rot_y = center_y + ((*dx as f64) * sin_angle + (*dy as f64) * cos_angle) as i32;
            rotated_corners.push((rot_x, rot_y));
        }
        
        if filled {
            // Draw filled polygon
            area.draw(&Polygon::new(rotated_corners.clone(), color.filled()))?;
        }
        
        // Draw outline
        for i in 0..4 {
            let start = rotated_corners[i];
            let end = rotated_corners[(i + 1) % 4];
            area.draw(&PathElement::new(vec![start, end], color.stroke_width(line_width)))?;
        }
        
        Ok(())
    }
    
    /// Save visualization with obstacles
    pub fn save_field_with_obstacles<P: AsRef<Path>>(
        &self,
        fluid: &Fluid,
        obstacle_manager: &ObstacleManager,
        filename: P,
        field_type: FieldType,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(&filename, (self.width, self.height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        // Draw the field
        match field_type {
            FieldType::Smoke => {
                self.draw_smoke_field(&root, fluid)?;
            }
            _ => {
                // For other fields, use existing methods
                return Err("Field type not implemented for obstacle visualization".into());
            }
        }
        
        // Draw obstacles on top
        self.draw_obstacles(&root, obstacle_manager, fluid)?;
        
        root.present()?;
        println!("Visualization with obstacles saved to {}", filename.as_ref().display());
        
        Ok(())
    }
}

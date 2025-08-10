use crate::fluid::{Fluid, FieldType};
use crate::radiator::Radiator;
use plotters::prelude::*;
use std::process::Command;

/// Animation system for CFD visualization
pub struct Animator {
    width: u32,
    height: u32,
    frame_counter: usize,
    output_dir: String,
}

impl Animator {
    pub fn new(width: u32, height: u32, output_dir: String) -> Self {
        // Create output directory
        std::fs::create_dir_all(&output_dir).unwrap_or_default();
        
        Self {
            width,
            height,
            frame_counter: 0,
            output_dir,
        }
    }
    
    /// Save a combined frame showing multiple visualizations
    pub fn save_combined_frame(
        &mut self,
        fluid: &Fluid,
        radiator: Option<&Radiator>,
        show_pressure: bool,
        show_velocity: bool,
        show_streamlines: bool,
        show_smoke: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frame_filename = format!("{}/frame_{:04}.png", self.output_dir, self.frame_counter);
        
        // Create a 2x2 grid layout
        let root = BitMapBackend::new(&frame_filename, (self.width * 2, self.height * 2)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let areas = root.split_evenly((2, 2));
        let pressure_area = &areas[0];
        let velocity_area = &areas[1];
        let streamlines_area = &areas[2];
        let smoke_area = &areas[3];
        
        // Draw pressure field
        if show_pressure {
            self.draw_pressure_field(pressure_area, fluid, radiator)?;
            pressure_area.titled("Pressure Field", ("sans-serif", 30))?;
        }
        
        // Draw velocity field
        if show_velocity {
            self.draw_velocity_field(velocity_area, fluid, radiator)?;
            velocity_area.titled("Velocity Field", ("sans-serif", 30))?;
        }
        
        // Draw streamlines
        if show_streamlines {
            self.draw_streamlines(streamlines_area, fluid, radiator)?;
            streamlines_area.titled("Streamlines", ("sans-serif", 30))?;
        }
        
        // Draw smoke
        if show_smoke {
            self.draw_smoke_field(smoke_area, fluid)?;
            smoke_area.titled("Smoke/Dye", ("sans-serif", 30))?;
        }
        
        root.present()?;
        
        println!("Animation frame {} saved", self.frame_counter);
        self.frame_counter += 1;
        
        Ok(())
    }
    
    /// Draw pressure field in a given area
    fn draw_pressure_field<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        fluid: &Fluid,
        radiator: Option<&Radiator>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
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
                let (r, g, b) = self.get_sci_color(pressure, min_p, max_p);
                
                let x1 = (i as f64 * cell_width) as i32;
                let y1 = ((fluid.num_y - j - 1) as f64 * cell_height) as i32;
                let x2 = ((i + 1) as f64 * cell_width) as i32;
                let y2 = ((fluid.num_y - j) as f64 * cell_height) as i32;
                
                area.draw(&Rectangle::new(
                    [(x1, y1), (x2, y2)],
                    RGBColor(r, g, b).filled(),
                ))?;
            }
        }
        
        // Draw radiator if provided
        if let Some(rad) = radiator {
            self.draw_radiator_overlay(area, rad, fluid)?;
        }
        
        Ok(())
    }
    
    /// Draw velocity field with arrows
    fn draw_velocity_field<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        fluid: &Fluid,
        radiator: Option<&Radiator>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        area.fill(&WHITE)?;
        
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        let scale = 20.0; // Velocity arrow scaling
        let skip = 4; // Skip cells for clarity
        
        for i in (1..fluid.num_x - 1).step_by(skip) {
            for j in (1..fluid.num_y - 1).step_by(skip) {
                if fluid.s[[i, j]] != 0.0 {
                    let u = (fluid.u[[i, j]] + fluid.u[[i + 1, j]]) * 0.5;
                    let v = (fluid.v[[i, j]] + fluid.v[[i, j + 1]]) * 0.5;
                    
                    let x = (i as f64 + 0.5) * cell_width;
                    let y = (fluid.num_y as f64 - j as f64 - 0.5) * cell_height;
                    
                    let dx = u * scale;
                    let dy = -v * scale; // Flip y-axis for display
                    
                    if u.abs() > 0.01 || v.abs() > 0.01 {
                        let velocity_mag = (u * u + v * v).sqrt();
                        let color = if velocity_mag > 3.0 { &RED } else { &BLUE };
                        
                        area.draw(&PathElement::new(
                            vec![(x as i32, y as i32), ((x + dx) as i32, (y + dy) as i32)],
                            (*color).stroke_width(2),
                        ))?;
                        
                        // Add arrowhead
                        let arrow_size = 3.0;
                        let angle = dy.atan2(dx);
                        let x_tip = x + dx;
                        let y_tip = y + dy;
                        
                        let x1 = x_tip - arrow_size * (angle - 0.5).cos();
                        let y1 = y_tip - arrow_size * (angle - 0.5).sin();
                        let x2 = x_tip - arrow_size * (angle + 0.5).cos();
                        let y2 = y_tip - arrow_size * (angle + 0.5).sin();
                        
                        area.draw(&PathElement::new(
                            vec![(x_tip as i32, y_tip as i32), (x1 as i32, y1 as i32)],
                            (*color).stroke_width(2),
                        ))?;
                        area.draw(&PathElement::new(
                            vec![(x_tip as i32, y_tip as i32), (x2 as i32, y2 as i32)],
                            (*color).stroke_width(2),
                        ))?;
                    }
                }
            }
        }
        
        // Draw radiator if provided
        if let Some(rad) = radiator {
            self.draw_radiator_overlay(area, rad, fluid)?;
        }
        
        Ok(())
    }
    
    /// Draw streamlines
    fn draw_streamlines<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        fluid: &Fluid,
        radiator: Option<&Radiator>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        area.fill(&WHITE)?;
        
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        // Generate streamlines
        let num_streamlines = 15;
        for stream_i in 0..num_streamlines {
            let start_y = (stream_i as f64 + 1.0) * fluid.num_y as f64 / (num_streamlines as f64 + 1.0);
            let mut x = 0.1;
            let mut y = start_y * fluid.h;
            
            let mut points = Vec::new();
            
            for _step in 0..800 {
                let u = fluid.sample_field(x, y, FieldType::U);
                let v = fluid.sample_field(x, y, FieldType::V);
                
                if u.abs() < 0.001 && v.abs() < 0.001 {
                    break;
                }
                
                let screen_x = (x / fluid.h * cell_width) as i32;
                let screen_y = ((fluid.num_y as f64 - y / fluid.h) * cell_height) as i32;
                
                if screen_x < 0 || screen_x >= self.width as i32 || 
                   screen_y < 0 || screen_y >= self.height as i32 {
                    break;
                }
                
                points.push((screen_x, screen_y));
                
                let dt = 0.01;
                x += u * dt;
                y += v * dt;
                
                if x >= (fluid.num_x - 1) as f64 * fluid.h {
                    break;
                }
            }
            
            if points.len() > 1 {
                        area.draw(&PathElement::new(points, BLUE.stroke_width(2)))?;
            }
        }
        
        // Draw radiator if provided
        if let Some(rad) = radiator {
            self.draw_radiator_overlay(area, rad, fluid)?;
        }
        
        Ok(())
    }
    
    /// Draw smoke/dye field
    fn draw_smoke_field<DB: DrawingBackend>(
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
    
    /// Draw radiator overlay
    fn draw_radiator_overlay<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        radiator: &Radiator,
        fluid: &Fluid,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        <DB as DrawingBackend>::ErrorType: 'static,
    {
        let cell_width = self.width as f64 / fluid.num_x as f64;
        let cell_height = self.height as f64 / fluid.num_y as f64;
        
        let center_x = (radiator.x / fluid.h * cell_width) as i32;
        let center_y = ((fluid.num_y as f64 - radiator.y / fluid.h) * cell_height) as i32;
        
        let half_width = (radiator.width / fluid.h * cell_width * 0.5) as i32;
        let half_height = (radiator.height / fluid.h * cell_height * 0.5) as i32;
        
        let cos_angle = radiator.angle.cos();
        let sin_angle = radiator.angle.sin();
        
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
        
        // Draw radiator outline
        for i in 0..4 {
            let start = rotated_corners[i];
            let end = rotated_corners[(i + 1) % 4];
            area.draw(&PathElement::new(vec![start, end], RED.stroke_width(4)))?;
        }
        
        Ok(())
    }
    
    /// Generate scientific colormap
    fn get_sci_color(&self, val: f64, min_val: f64, max_val: f64) -> (u8, u8, u8) {
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
    
    /// Create GIF animation from saved frames
    pub fn create_gif_animation(&self, fps: u32) -> Result<(), Box<dyn std::error::Error>> {
        println!("Creating GIF animation with {} frames at {} FPS...", self.frame_counter, fps);
        
        // Use imagemagick to create GIF (if available)
        let delay = 100 / fps; // centiseconds
        let output = Command::new("magick")
            .args(&[
                "-delay", &delay.to_string(),
                "-loop", "0",
                &format!("{}/*.png", self.output_dir),
                &format!("{}/animation.gif", self.output_dir)
            ])
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ GIF animation created: {}/animation.gif", self.output_dir);
                } else {
                    println!("⚠️  ImageMagick failed. Individual frames available in {}/", self.output_dir);
                }
            }
            Err(_) => {
                println!("⚠️  ImageMagick not found. Individual frames available in {}/", self.output_dir);
                println!("   Install ImageMagick to generate GIF animations automatically.");
            }
        }
        
        Ok(())
    }
    
    /// Get frame count
    pub fn frame_count(&self) -> usize {
        self.frame_counter
    }
}

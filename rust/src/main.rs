mod fluid;
mod scene;
mod visualizer;
mod radiator;
mod animator;

use crate::scene::Scene;
use crate::visualizer::Visualizer;
use crate::radiator::{Radiator, RadiatorAnalyzer};
use crate::animator::Animator;
use std::time::Instant;
use clap::{Arg, Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("CFD Simulator")
        .version("0.1.0")
        .author("Your Name")
        .about("2D Incompressible CFD Simulator for Formula Student Radiator Analysis")
        .arg(
            Arg::new("scene")
                .short('s')
                .long("scene")
                .value_name("SCENE_NUMBER")
                .help("Scene to simulate (0=Tank, 1=Wind Tunnel, 2=Paint, 3=Hires Tunnel, 4=Radiator Analysis)")
                .default_value("4")
        )
        .arg(
            Arg::new("steps")
                .short('n')
                .long("steps")
                .value_name("STEPS")
                .help("Number of simulation steps to run")
                .default_value("1000")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT_DIR")
                .help("Output directory for visualization files")
                .default_value("output")
        )
        .arg(
            Arg::new("radiator-sweep")
                .long("radiator-sweep")
                .help("Perform radiator angle sweep analysis")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("animate")
                .long("animate")
                .help("Generate animated visualization during simulation")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    let scene_nr: usize = matches.get_one::<String>("scene").unwrap().parse()?;
    let num_steps: usize = matches.get_one::<String>("steps").unwrap().parse()?;
    let output_dir = matches.get_one::<String>("output").unwrap();
    let radiator_sweep = matches.get_flag("radiator-sweep");
    let animate = matches.get_flag("animate");

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    println!("🌊 Starting 2D CFD Simulator for Formula Student Radiator Analysis");
    println!("Scene: {}", scene_nr);
    println!("Steps: {}", num_steps);
    println!("Output: {}", output_dir);
    println!("Animation: {}", if animate { "Yes" } else { "No" });

    if radiator_sweep {
        run_radiator_angle_sweep(output_dir, num_steps)?;
    } else {
        run_single_simulation(scene_nr, num_steps, output_dir, animate)?;
    }

    Ok(())
}

fn run_single_simulation(scene_nr: usize, num_steps: usize, output_dir: &str, animate: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize simulation
    let sim_width = 1600.0;
    let sim_height = 900.0;
    let mut scene = Scene::new();
    scene.setup_scene(scene_nr, sim_width, sim_height);

    let visualizer = Visualizer::new(800, 600);

    println!("✅ Simulation initialized");
    
    if let Some(ref fluid) = scene.fluid {
        println!("Grid size: {}x{}", fluid.num_x, fluid.num_y);
        println!("Grid spacing: {:.4}", fluid.h);
        println!("Time step: {:.6}", scene.dt);
    }

    // Add radiator if scene 4
    let mut radiator_analyzer = RadiatorAnalyzer::new();
    if scene_nr == 4 {
        if let Some(ref mut fluid) = scene.fluid {
            // Set up realistic inflow for radiator testing FIRST
            scene.inflow_velocity = 10.0; // 10 m/s typical for automotive
            
            for j in 1..fluid.num_y - 1 {
                fluid.u[[0, j]] = scene.inflow_velocity;
                fluid.u[[1, j]] = scene.inflow_velocity;
                fluid.m[[0, j]] = 1.0; // Add smoke for visualization
                fluid.m[[1, j]] = 1.0;
            }
            
            // Set walls
            for i in 0..fluid.num_x {
                // Top and bottom walls
                fluid.s[[i, 0]] = 0.0;
                fluid.s[[i, fluid.num_y - 1]] = 0.0;
                fluid.u[[i, 0]] = 0.0;
                fluid.u[[i, fluid.num_y - 1]] = 0.0;
                fluid.v[[i, 0]] = 0.0;
                fluid.v[[i, fluid.num_y - 1]] = 0.0;
            }
            
            // Add radiator in the middle of the domain
            let radiator = Radiator {
                x: fluid.num_x as f64 * fluid.h * 0.5,
                y: fluid.num_y as f64 * fluid.h * 0.5,
                width: 0.05,  // Thinner radiator
                height: 0.3,  // Reasonable height
                angle: 15.0_f64.to_radians(), // 15 degree angle
                porosity: 0.9,  // High porosity (typical for radiators)
                resistance: 1000.0,  // Moderate resistance
            };
            
            radiator_analyzer.add_radiator(radiator.clone());
            radiator_analyzer.apply_radiator_to_fluid(fluid, &radiator);
            
            println!("🏎️  Radiator added at {:.2}, {:.2} with {:.1}° angle", 
                     radiator.x, radiator.y, radiator.angle.to_degrees());
            println!("   Porosity: {:.1}%, Resistance: {:.0}", radiator.porosity * 100.0, radiator.resistance);
            println!("   Inflow velocity: {:.1} m/s", scene.inflow_velocity);
        }
    }

    // Run simulation
    let start_time = Instant::now();
    let mut frame_count = 0;
    
    // Initialize animator if animation is requested
    let mut animator = if animate {
        Some(Animator::new(800, 600, format!("{}/animation", output_dir)))
    } else {
        None
    };
    
    if animate {
        println!("🎬 Animation mode enabled - saving frames for GIF generation");
    }

    for step in 0..num_steps {
        scene.simulate();
        frame_count += 1;

        // Save visualizations every 100 steps
        if step % 100 == 0 {
            if let Some(ref fluid) = scene.fluid {
                let step_str = format!("{:04}", step);
                
                // Generate animation frame if requested
                if let Some(ref mut anim) = animator {
                    let radiator = if scene_nr == 4 && !radiator_analyzer.radiators.is_empty() {
                        Some(&radiator_analyzer.radiators[0])
                    } else {
                        None
                    };
                    
                    anim.save_combined_frame(
                        fluid, 
                        radiator, 
                        true,  // show pressure
                        true,  // show velocity
                        true,  // show streamlines
                        true,  // show smoke
                    )?;
                }
                
                // Debug: Check fluid state and physics validation
                if step % 200 == 0 {
                    let max_u = fluid.u.iter().fold(0.0f64, |acc, &x| acc.max(x.abs()));
                    let max_v = fluid.v.iter().fold(0.0f64, |acc, &x| acc.max(x.abs()));
                    let max_p = fluid.p.iter().fold(0.0f64, |acc, &x| acc.max(x.abs()));
                    
                    // Check mass conservation (divergence should be ~0)
                    let mut max_div: f64 = 0.0;
                    for i in 1..fluid.num_x-1 {
                        for j in 1..fluid.num_y-1 {
                            if fluid.s[[i,j]] != 0.0 {
                                let div = (fluid.u[[i+1,j]] - fluid.u[[i,j]]) + 
                                         (fluid.v[[i,j+1]] - fluid.v[[i,j]]);
                                max_div = max_div.max(div.abs());
                            }
                        }
                    }
                    
                    // Check Reynolds number (for reference)
                    let characteristic_length = fluid.num_y as f64 * fluid.h;
                    let nu = 1.5e-5; // Air kinematic viscosity
                    let reynolds = scene.inflow_velocity * characteristic_length / nu;
                    
                    println!("  Physics Check - Max |u|: {:.3}, Max |v|: {:.3}, Max |p|: {:.3}", max_u, max_v, max_p);
                    println!("  Mass Conservation - Max |∇·u|: {:.6} (should be ~0)", max_div);
                    println!("  Reynolds Number: {:.0} (Re > 2300 = turbulent)", reynolds);
                }
                
                // Always save smoke field for visualization
                visualizer.save_smoke_field(
                    fluid,
                    format!("{}/smoke_{}.png", output_dir, step_str),
                )?;
                
                // Save pressure field for radiator analysis
                if scene_nr == 4 {
                    if let Some(ref radiator) = radiator_analyzer.radiators.first() {
                        visualizer.save_pressure_field_with_radiator(
                            fluid,
                            Some(radiator),
                            format!("{}/pressure_{}.png", output_dir, step_str),
                        )?;
                        
                        visualizer.save_streamlines_with_radiator(
                            fluid,
                            Some(radiator),
                            format!("{}/streamlines_{}.png", output_dir, step_str),
                            20,
                            1000,
                        )?;
                    } else {
                        visualizer.save_pressure_field(
                            fluid,
                            format!("{}/pressure_{}.png", output_dir, step_str),
                        )?;
                        
                        visualizer.save_streamlines(
                            fluid,
                            format!("{}/streamlines_{}.png", output_dir, step_str),
                            20,
                            1000,
                        )?;
                    }
                    
                    visualizer.save_velocity_field(
                        fluid,
                        format!("{}/velocity_{}.png", output_dir, step_str),
                    )?;
                }
            }
            
            let elapsed = start_time.elapsed().as_secs_f64();
            let fps = frame_count as f64 / elapsed;
            println!("Step {}: {:.1} FPS", step, fps);
        }
    }

    // Analyze radiator performance if applicable
    if scene_nr == 4 && !radiator_analyzer.radiators.is_empty() {
        if let Some(ref fluid) = scene.fluid {
            let radiator = radiator_analyzer.radiators[0].clone();
            let metrics = radiator_analyzer.analyze_performance(fluid, &radiator);
            
            println!("\n🏎️  Radiator Performance Analysis:");
            println!("   Angle: {:.1}°", metrics.angle_degrees);
            println!("   Mass flow rate: {:.4} kg/s", metrics.mass_flow_rate);
            println!("   Pressure drop: {:.2} Pa", metrics.pressure_drop);
            println!("   Drag force: {:.3} N", metrics.drag_force);
            println!("   Lift force: {:.3} N", metrics.lift_force);
            println!("   Cooling efficiency: {:.3}", metrics.cooling_efficiency);
            println!("   Fan power required: {:.2} W", metrics.fan_power_required);
            
            radiator_analyzer.save_metrics(format!("{}/radiator_metrics.json", output_dir))?;
        }
    }

    let total_time = start_time.elapsed();
    let final_fps = frame_count as f64 / total_time.as_secs_f64();
    
    // Generate GIF animation if frames were captured
    if let Some(ref animator) = animator {
        println!("🎬 Generating GIF animation from {} frames...", animator.frame_count());
        animator.create_gif_animation(5)?; // 5 FPS for smooth viewing
    }
    
    println!("🎉 Simulation completed!");
    println!("Total time: {:.2} seconds", total_time.as_secs_f64());
    println!("Average FPS: {:.1}", final_fps);
    println!("Output saved to: {}", output_dir);

    Ok(())
}

fn run_radiator_angle_sweep(output_dir: &str, steps_per_angle: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Running radiator angle sweep analysis...");
    
    let angles: Vec<f64> = vec![0.0, 15.0, 30.0, 45.0, 60.0, 75.0, 90.0]; // Degrees
    let mut radiator_analyzer = RadiatorAnalyzer::new();
    
    for angle_deg in angles {
        println!("\n📐 Analyzing angle: {:.1}°", angle_deg);
        
        // Setup fresh simulation for each angle
        let sim_width = 1600.0;
        let sim_height = 900.0;
        let mut scene = Scene::new();
        scene.setup_scene(1, sim_width, sim_height); // Wind tunnel
        
        if let Some(ref mut fluid) = scene.fluid {
            // Create radiator at this angle
            let radiator = Radiator {
                x: fluid.num_x as f64 * fluid.h * 0.4,
                y: fluid.num_y as f64 * fluid.h * 0.5,
                width: 0.15,
                height: 0.3,
                angle: angle_deg.to_radians(),
                porosity: 0.8,
                resistance: 100.0,
            };
            
            radiator_analyzer.apply_radiator_to_fluid(fluid, &radiator);
            
            // Set up controlled inflow
            for j in 1..fluid.num_y - 1 {
                fluid.u[[0, j]] = 5.0; // 5 m/s inflow
                fluid.u[[1, j]] = 5.0;
                fluid.m[[0, j]] = 1.0;
            }
            
            // Run simulation to steady state
            let start_time = Instant::now();
            for step in 0..steps_per_angle {
                scene.simulate();
                
                if step % 200 == 0 {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    println!("  Step {}/{} ({:.1}s)", step, steps_per_angle, elapsed);
                }
            }
            
            // Analyze performance at this angle (borrow fluid separately)
            let metrics = if let Some(ref fluid) = scene.fluid {
                radiator_analyzer.analyze_performance(fluid, &radiator)
            } else {
                continue; // Skip if no fluid
            };
            
            println!("  Results:");
            println!("    Mass flow: {:.4} kg/s", metrics.mass_flow_rate);
            println!("    Pressure drop: {:.2} Pa", metrics.pressure_drop);
            println!("    Drag: {:.3} N, Lift: {:.3} N", metrics.drag_force, metrics.lift_force);
            println!("    Fan power: {:.2} W", metrics.fan_power_required);
            
            // Save visualization for this angle
            let visualizer = Visualizer::new(800, 600);
            let angle_str = format!("{:02.0}", angle_deg);
            
            if let Some(ref fluid) = scene.fluid {
                visualizer.save_smoke_field(
                    fluid,
                    format!("{}/sweep_smoke_{}.png", output_dir, angle_str),
                )?;
                
                visualizer.save_pressure_field_with_radiator(
                    fluid,
                    Some(&radiator),
                    format!("{}/sweep_pressure_{}.png", output_dir, angle_str),
                )?;
                
                visualizer.save_streamlines_with_radiator(
                    fluid,
                    Some(&radiator),
                    format!("{}/sweep_streamlines_{}.png", output_dir, angle_str),
                    25,
                    1000,
                )?;
            }
        }
    }
    
    // Save comprehensive results
    radiator_analyzer.save_metrics(format!("{}/radiator_sweep_results.json", output_dir))?;
    radiator_analyzer.print_summary();
    
    // Find optimal angle
    let mut best_angle = 0.0;
    let mut best_efficiency = 0.0;
    
    for metrics in &radiator_analyzer.metrics_history {
        let efficiency = metrics.cooling_efficiency / (1.0 + metrics.fan_power_required / 1000.0);
        if efficiency > best_efficiency {
            best_efficiency = efficiency;
            best_angle = metrics.angle_degrees;
        }
    }
    
    println!("\n🏆 Optimal radiator angle: {:.1}° (efficiency: {:.3})", best_angle, best_efficiency);
    println!("📊 Complete results saved to: {}/radiator_sweep_results.json", output_dir);
    
    Ok(())
}

# 2D CFD Simulator for Formula Student Radiator Analysis

A high-performance 2D incompressible computational fluid dynamics (CFD) simulator written in Rust, specifically designed for Formula Student car radiator optimization.

## Features

- **2D Incompressible Navier-Stokes Solver**: Based on the projection method with Gauss-Seidel iteration
- **Radiator Performance Analysis**: Comprehensive analysis of radiator angle effects on:
  - Mass flow rate through the radiator (kg/s)
  - Pressure drop across the matrix (Pa)
  - Drag and lift forces on the radiator (N)
  - Fan power requirements (W)
  - Cooling efficiency metrics
- **Multiple Simulation Scenarios**: Tank, wind tunnel, paint, and radiator-specific scenarios
- **Real-time Visualization**: Pressure fields, velocity fields, streamlines, and smoke/dye concentration
- **Batch Analysis**: Automated angle sweep analysis for radiator optimization
- **High Performance**: Leverages Rust's performance with parallel processing capabilities

## Physics Implementation

The simulator implements:
- **Eulerian Grid-based Approach**: Staggered MAC grid for velocity components
- **Semi-Lagrangian Advection**: For velocity and scalar field transport
- **Porous Medium Model**: Darcy-Forchheimer equation for radiator flow resistance
- **Incompressibility Constraint**: Projection method ensuring ∇·u = 0
- **Boundary Conditions**: Inflow, outflow, and no-slip wall conditions

## Usage

### Basic Simulation

```bash
# Run wind tunnel simulation with default settings
cargo run -- --scene 1 --steps 1000

# Run radiator analysis scenario
cargo run -- --scene 4 --steps 2000 --output radiator_results

# Custom output directory
cargo run -- --scene 1 --steps 500 --output my_results
```

### Radiator Angle Sweep Analysis

```bash
# Perform comprehensive radiator angle analysis
cargo run -- --radiator-sweep --output sweep_results

# Custom steps per angle
cargo run -- --radiator-sweep --steps 1500 --output detailed_sweep
```

### Command Line Options

- `--scene, -s`: Simulation scenario
  - `0`: Tank (fluid at rest)
  - `1`: Wind tunnel (default)
  - `2`: Paint (interactive dye injection)
  - `3`: High-resolution wind tunnel
  - `4`: Radiator analysis
- `--steps, -n`: Number of simulation time steps
- `--output, -o`: Output directory for results
- `--radiator-sweep`: Run angle sweep analysis (0° to 90°)

## Scenarios

### Wind Tunnel (Scene 1)
- Steady inflow at 2 m/s
- Obstacle for vortex shedding demonstration
- Ideal for studying flow patterns around objects

### Radiator Analysis (Scene 4)
- Controlled inflow conditions
- Radiator positioned in flow field
- Comprehensive performance metrics calculation
- Suitable for single-angle analysis

### Radiator Angle Sweep
- Automated analysis across multiple angles (0°, 15°, 30°, 45°, 60°, 75°, 90°)
- Generates comparative performance data
- Identifies optimal radiator orientation

## Output Files

The simulator generates several types of output:

### Visualization Files (PNG)
- `smoke_XXXX.png`: Smoke/dye concentration field
- `pressure_XXXX.png`: Pressure field with scientific colormap
- `velocity_XXXX.png`: Velocity vector field
- `streamlines_XXXX.png`: Flow streamlines

### Analysis Files (JSON)
- `radiator_metrics.json`: Performance metrics for single analysis
- `radiator_sweep_results.json`: Comprehensive sweep results

### Sample Metrics Output
```json
{
  "angle_degrees": 15.0,
  "mass_flow_rate": 0.0234,
  "pressure_drop": 45.2,
  "inlet_velocity": 5.0,
  "outlet_velocity": 4.1,
  "drag_force": 2.3,
  "lift_force": 0.8,
  "cooling_efficiency": 0.89,
  "fan_power_required": 1.06
}
```

## Radiator Performance Metrics

### Mass Flow Rate (kg/s)
- Calculated by integrating velocity normal to radiator face
- Higher values indicate better cooling potential
- Affected by radiator angle and porosity

### Pressure Drop (Pa)
- Difference between upstream and downstream pressures
- Critical for fan sizing and power requirements
- Lower values reduce parasitic power losses

### Forces (N)
- **Drag**: Force component opposing flow direction
- **Lift**: Force component perpendicular to flow
- Important for radiator mounting design

### Cooling Efficiency
- Dimensionless metric combining flow rate and effectiveness
- Higher values indicate better heat transfer potential

### Fan Power (W)
- Power required to overcome pressure drop
- P = ΔP × Q (pressure drop × volumetric flow rate)
- Critical for electrical system design

## Building and Dependencies

### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo package manager

### Dependencies
- `ndarray`: Multi-dimensional arrays for fluid fields
- `plotters`: Visualization and plotting
- `rayon`: Parallel processing
- `serde`: Data serialization
- `clap`: Command-line argument parsing

### Build Instructions
```bash
# Clone the repository
git clone <repository-url>
cd cfd_simulator

# Build the project
cargo build --release

# Run with optimizations
cargo run --release -- --radiator-sweep
```

## Formula Student Applications

This simulator is specifically designed for Formula Student car radiator optimization:

### Typical Analysis Workflow
1. **Initial Setup**: Define radiator geometry and flow conditions
2. **Angle Sweep**: Run analysis across multiple angles (0° to 90°)
3. **Performance Comparison**: Analyze mass flow vs. pressure drop trade-offs
4. **Optimization**: Identify angle providing best cooling/drag compromise
5. **Validation**: Verify results with CFD best practices

### Design Considerations
- **Cooling Requirements**: Minimum mass flow for heat rejection
- **Aerodynamic Impact**: Drag penalty vs. cooling benefit
- **Packaging Constraints**: Physical space limitations
- **Fan Selection**: Power requirements and efficiency
- **Safety Margins**: Conservative design factors

### Expected Results
Typical findings for Formula Student radiators:
- **Optimal Angles**: Usually between 15° and 45°
- **Trade-offs**: Steeper angles increase flow but also drag
- **Efficiency**: Peak cooling-to-drag ratio varies by design
- **Flow Patterns**: Visualization helps identify separation and recirculation

## Validation and Accuracy

### Grid Independence
- Minimum 100x100 grid for basic analysis
- 200x200+ recommended for detailed studies
- Ensure radiator is at least 2x grid spacings from boundaries

### Convergence Criteria
- Run sufficient time steps for steady-state (typically 1000+)
- Monitor force and flow rate convergence
- Verify mass conservation (∇·u ≈ 0)

### Physical Validity
- Reynolds numbers appropriate for automotive applications
- Boundary layer resolution considerations
- Compressibility limits (Ma < 0.3)

## Future Enhancements

- [ ] Heat transfer modeling
- [ ] Turbulence models (RANS)
- [ ] 3D capabilities
- [ ] Real-time interactive visualization
- [ ] Optimization algorithms
- [ ] Experimental validation data

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Based on the JavaScript fluid simulator from Ten Minute Physics by Matthias Müller.
Adapted and extended for Formula Student radiator analysis applications.

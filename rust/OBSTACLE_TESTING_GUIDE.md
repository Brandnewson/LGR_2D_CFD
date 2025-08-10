# Obstacle Testing Guide

## 🎯 Problem Solved
The persistent circle in Scene 1 was being added in **two places**:
1. `src/main.rs` - User-controlled obstacles (which you commented out)
2. `src/scene.rs` in `setup_wind_tunnel()` - Default obstacle (hidden from user)

## ✅ Fixes Applied

### 1. **Removed Default Obstacle**
- Removed automatic circle creation in `setup_wind_tunnel()`
- Now only `main.rs` controls obstacle placement
- You have full control over what obstacles appear

### 2. **Enhanced Visualization Frequency**
- **First 100 steps**: Save every 10 steps (10 images)
- **After step 100**: Save every 50 steps 
- **Animation mode**: Save every 20 steps
- **Performance mode**: Only final frame

### 3. **Improved Obstacle Sizing**
- **Circle**: 12 grid cells radius (24 cells diameter)
- **Airfoil**: 40 grid cells chord length
- **Better separation**: Circle at 30%, airfoil at 70% downstream

## 🎮 Testing Different Configurations

### Test 1: Only Airfoil
```rust
// In main.rs, comment out the circle lines:
// let circle = Obstacle::new_circle(circle_x, circle_y, circle_radius);
// scene.add_obstacle(circle);
```

### Test 2: Only Circle
```rust
// In main.rs, comment out the airfoil lines:
// let airfoil = Obstacle::new_airfoil(airfoil_x, airfoil_y, airfoil_chord, airfoil_thickness, airfoil_angle);
// scene.add_obstacle(airfoil);
```

### Test 3: Both Obstacles (Default)
```rust
// Keep both uncommented for interaction study
```

### Test 4: No Obstacles
```rust
// Comment out the entire Scene 1 obstacle block
if scene_nr == 1 {
    // println!("🔶 Adding demonstration obstacles to Scene 1:");
    // ... comment out everything inside ...
}
```

## 🚀 Run Commands

```bash
# Test with detailed frame capture (every 10 steps for first 100)
cargo run --release -- --scene 1 --steps 200 --output obstacle_test

# Quick test with only final frame
cargo run --release -- --scene 1 --steps 100 --output quick_test --performance

# Animation mode for GIF creation
cargo run --release -- --scene 1 --steps 150 --output animation_test --animate
```

## 📊 Expected Output Files

### Standard Mode (First 100 Steps):
- `smoke_0000.png` - Initial state
- `smoke_0010.png` - Early flow development
- `smoke_0020.png` - Smoke reaching first obstacle
- `smoke_0030.png` - Wake formation starting
- `smoke_0040.png` - Interaction development
- `smoke_0050.png` - Wake propagation
- `smoke_0060.png` - Downstream effects
- `smoke_0070.png` - Flow stabilization
- `smoke_0080.png` - Steady wake patterns
- `smoke_0090.png` - Full interaction visible
- Then every 50 steps until completion

### Key Visualization Points:
- **Steps 0-20**: Smoke injection and initial flow
- **Steps 20-40**: First obstacle interaction
- **Steps 40-80**: Wake development and propagation  
- **Steps 80-150**: Second obstacle interaction (if present)
- **Steps 150+**: Steady-state flow patterns

## 🔧 Customization Options

### Obstacle Positioning:
```rust
let circle_x = domain_width * 0.3;  // 30% downstream (adjust 0.1-0.8)
let circle_y = domain_height * 0.4; // 40% height (adjust 0.2-0.8)
```

### Obstacle Sizing:
```rust
let circle_radius = grid_spacing * 12.0;  // 12 cells radius (adjust 8-20)
let airfoil_chord = grid_spacing * 40.0;  // 40 cells chord (adjust 20-60)
```

### Airfoil Parameters:
```rust
let airfoil_thickness = 0.15;  // 15% thickness (adjust 0.08-0.25)
let airfoil_angle = 10.0_f64.to_radians();  // 10° attack angle (adjust 0-20°)
```

This gives you complete control over the obstacle configuration and much better visualization of the smoke-obstacle interaction!

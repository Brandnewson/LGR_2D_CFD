#!/bin/bash
# Radiator Angle Sweep Analysis Script
# This script runs CFD simulations at different radiator angles
# to determine optimal positioning for Formula Student cars

echo "🏎️  Formula Student Radiator Analysis Suite"
echo "============================================="
echo ""

# Create results directory with timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_DIR="radiator_analysis_$TIMESTAMP"
mkdir -p $RESULTS_DIR

echo "📁 Results will be saved to: $RESULTS_DIR"
echo ""

# Test different radiator angles
ANGLES=(0 15 30 45 60 75 90)
STEPS=100  # Adjust based on desired accuracy vs speed

echo "🔄 Running CFD simulations for ${#ANGLES[@]} different angles..."
echo "   Steps per angle: $STEPS"
echo "   Estimated time: ~2-3 minutes per angle"
echo ""

for angle in "${ANGLES[@]}"; do
    echo "📐 Analyzing radiator at ${angle}° angle..."
    
    # Create angle-specific output directory
    ANGLE_DIR="$RESULTS_DIR/angle_${angle}deg"
    
    # Run simulation for this angle
    # Note: You'll need to modify the code to accept angle parameter
    cargo run --release -- --scene 4 --steps $STEPS --output "$ANGLE_DIR" --performance
    
    if [ $? -eq 0 ]; then
        echo "   ✅ Completed ${angle}° analysis"
    else
        echo "   ❌ Failed ${angle}° analysis"
    fi
    echo ""
done

echo "🎉 Radiator angle sweep analysis completed!"
echo ""
echo "📊 Results summary:"
echo "   • Check individual angle directories for detailed results"
echo "   • Compare pressure drops across angles"
echo "   • Analyze mass flow rates for cooling efficiency"
echo "   • Review drag/lift forces for aerodynamic impact"
echo ""
echo "📈 For optimal radiator positioning:"
echo "   1. High mass flow rate (better cooling)"
echo "   2. Low pressure drop (less fan power required)"
echo "   3. Minimal drag increase (better lap times)"
echo "   4. Consider packaging constraints in your car"

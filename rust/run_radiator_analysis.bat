@echo off
REM Radiator Angle Sweep Analysis Script for Windows
REM This script runs CFD simulations at different radiator angles
REM to determine optimal positioning for Formula Student cars

echo 🏎️  Formula Student Radiator Analysis Suite
echo =============================================
echo.

REM Create results directory with timestamp
for /f "tokens=2 delims==" %%i in ('wmic os get localdatetime /value') do set datetime=%%i
set TIMESTAMP=%datetime:~0,8%_%datetime:~8,6%
set RESULTS_DIR=radiator_analysis_%TIMESTAMP%
mkdir %RESULTS_DIR%

echo 📁 Results will be saved to: %RESULTS_DIR%
echo.

REM Test different radiator angles
set STEPS=100

echo 🔄 Running CFD simulations for different angles...
echo    Steps per angle: %STEPS%
echo    Estimated time: ~2-3 minutes per angle
echo.

REM Loop through angles
for %%a in (0 15 30 45 60 75 90) do (
    echo 📐 Analyzing radiator at %%a° angle...
    
    REM Create angle-specific output directory
    set ANGLE_DIR=%RESULTS_DIR%/angle_%%adeg
    
    REM Run simulation for this angle
    cargo run --release -- --scene 4 --steps %STEPS% --output !ANGLE_DIR! --performance
    
    if !errorlevel! equ 0 (
        echo    ✅ Completed %%a° analysis
    ) else (
        echo    ❌ Failed %%a° analysis
    )
    echo.
)

echo 🎉 Radiator angle sweep analysis completed!
echo.
echo 📊 Results summary:
echo    • Check individual angle directories for detailed results
echo    • Compare pressure drops across angles
echo    • Analyze mass flow rates for cooling efficiency
echo    • Review drag/lift forces for aerodynamic impact
echo.
echo 📈 For optimal radiator positioning:
echo    1. High mass flow rate (better cooling)
echo    2. Low pressure drop (less fan power required)
echo    3. Minimal drag increase (better lap times)
echo    4. Consider packaging constraints in your car

pause

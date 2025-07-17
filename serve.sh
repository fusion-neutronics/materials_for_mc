#!/bin/bash

# Simple HTTP server for testing the WASM application
echo "Starting HTTP server for WASM testing..."

# Check if Python 3 is available
if command -v python3 &> /dev/null; then
    python3 -m http.server 8000
elif command -v python &> /dev/null; then
    python -m http.server 8000
else
    echo "Error: Python is not installed. Please install Python 3 to run the server."
    exit 1
fi

echo "Server is running at http://localhost:8000"
echo "Navigate to http://localhost:8000/macroscopic_xs_plotter.html to view the application"
echo "Press Ctrl+C to stop the server"

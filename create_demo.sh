#!/bin/bash
# Create a demo GIF using asciinema + agg

# Record with proper terminal settings
asciinema rec \
  --cols 100 \
  --rows 30 \
  --command "./demo_script.sh" \
  --overwrite \
  demo.cast

# Convert to GIF
agg \
  --font-size 16 \
  --theme monokai \
  --speed 1.2 \
  demo.cast \
  demo.gif

echo "Demo created: demo.gif"

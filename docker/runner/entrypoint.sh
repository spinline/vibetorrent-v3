#!/bin/bash
set -e

# If GITEA_INSTANCE_URL and GITEA_RUNNER_TOKEN are provided, try to register
if [ -n "$GITEA_INSTANCE_URL" ] && [ -n "$GITEA_RUNNER_TOKEN" ]; then
    if [ ! -f ".runner" ]; then
        echo "Registering runner..."
        act_runner register \
          --instance "$GITEA_INSTANCE_URL" \
          --token "$GITEA_RUNNER_TOKEN" \
          --name "custom-mips-runner-$(hostname)" \
          --no-interactive
    else
        echo "Runner already registered."
    fi
fi

# Run the daemon
echo "Starting act_runner daemon..."
exec act_runner daemon

#!/bin/bash

set -e

echo "Copying credentials & token files to raspi ..."
scp calendar_credentials.json calendar_token.json calendars.txt pi:~

echo "Done!"

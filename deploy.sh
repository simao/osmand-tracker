#!/bin/bash

set -x

cargo build --release

docker build . -t simaom/osmand-tracker:latest

docker push simaom/osmand-tracker:latest

ssh root@0io.eu podman pull simaom/osmand-tracker:latest

ssh root@0io.eu podman rm -f osmand-tracker

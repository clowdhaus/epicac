#!/usr/bin/env bash

aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin <UPDATE>

TAG=epicac-$(date +%Y%m%d_%H%M%S)

docker build --platform amd64 -t <UPDATE>:${TAG} .
docker push <UPDATE>:${TAG}

# Update the pod manifest with the new image tag
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  sed -i "s|image:.*|image:<UPDATE>:${TAG}|g" infra/pod.yaml
elif [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s|image:.*|image: <UPDATE>:${TAG}|g" infra/pod.yaml
else
  echo "Unsupported OS: $OSTYPE"
  exit 1
fi

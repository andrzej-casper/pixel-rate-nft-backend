#!/usr/bin/env bash
set -e

# determine script path
pushd . > /dev/null
SCRIPT_PATH="${BASH_SOURCE[0]}"
if ([ -h "${SCRIPT_PATH}" ]); then
  while([ -h "${SCRIPT_PATH}" ]); do cd `dirname "$SCRIPT_PATH"`;
  SCRIPT_PATH=`readlink "${SCRIPT_PATH}"`; done
fi
cd `dirname ${SCRIPT_PATH}` > /dev/null
SCRIPT_PATH=`pwd`;
popd  > /dev/null

IMAGE_NAME="pixel-rate-nft-backend:0.1"
CONTAINER_NAME="pixelrate-nft"

echo "> Checking requirements"
if which docker >/dev/null 2>&1; then
  :
else
  echo "Error: docker not installed."
  echo "You can get it with:"
  echo " $ sudo pacman -S docker"
  echo " $ sudo systemctl enable --now docker.service"
  echo " $ sudo usermod -aG docker \$USER && newgrp docker"
  exit 1
fi

echo "> Building image"
(
  cd ${SCRIPT_PATH}
  docker build -t ${IMAGE_NAME} .
)

echo "> Making sure no old container is running"
if [ "$(docker ps -qa -f name=${CONTAINER_NAME})" ]; then
  if [ "$(docker ps -q -f name=${CONTAINER_NAME})" ]; then
    docker stop ${CONTAINER_NAME};
  fi
  docker rm ${CONTAINER_NAME};
fi

echo "> Launching new NFT backend container"
docker run \
  --detach \
  --name ${CONTAINER_NAME} \
  --restart=unless-stopped \
  -v ${SCRIPT_PATH}/wallet:/app/wallet \
  -p 8035:8035 \
  ${IMAGE_NAME}

echo "Done! NFT backend is running in background, listening at 0.0.0.0:8035."
echo "You might close this console ;)"

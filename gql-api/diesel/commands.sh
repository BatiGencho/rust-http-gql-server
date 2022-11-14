# build and push migrator image to aws
DOCKER_BUILDKIT=1 docker build --file Dockerfile --tag db-migrator .
docker tag db-migrator:latest 382480309488.dkr.ecr.eu-west-2.amazonaws.com/db-migrator:latest
docker push 382480309488.dkr.ecr.eu-west-2.amazonaws.com/db-migrator:latest 
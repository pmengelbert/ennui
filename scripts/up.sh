#!/usr/bin/env bash
set -euo pipefail

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $DIR/../

docker-compose up -d
docker-compose logs -f &
trap "docker-compose down --remove-orphans" EXIT

( sleep 2 && PGPASSWORD=password123 psql -U postgres -h localhost -f ./populate.sql -x )

while :
do
    sleep 1
done

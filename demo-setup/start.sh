# Script assumes that host system has go-lang installed and polkadex.tar file is present in the same directory as this script
# The script assumes that openfinex opens it's port on 8001


git clone https://github.com/openware/barong-jwt.git
cd barong-jwt || exit
go run main.go -role admin
# Load the polkadex.tar to your docker:
docker load --input polkadex.tar
# start docker compose
docker-compose -f compose/upstream_compose.yml up -Vd
# wait for ~30s and then start the services again - api and engine can only be started when the other services are running already
sleep 30s
docker-compose -f compose/upstream_compose.yml start
docker-compose up

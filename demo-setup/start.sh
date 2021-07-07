# Script assumes that host system has go-lang installed and polkadex.tar is present in the same directory and config.yaml
# The script assumes that openfinex opens it's port on 8001
# It also assumes that config.yaml is inside config folder.
# We also need docker-compose 1.29 or newer 


git clone https://github.com/openware/barong-jwt.git
cd barong-jwt || exit
go build main.go
cd ..
./barong-jwt/main -role admin
# Load the polkadex.tar to your docker:
docker load --input polkadex.tar || exit
# start docker compose
docker-compose -f ./compose/upstream_compose.yml up -Vd || exit
# wait for ~30s and then start the services again - api and engine can only be started when the other services are running already
echo "Waiting for 30s to ensure all openfinex services are running"
sleep 30s
echo "Starting the api and engine services"
docker-compose -f compose/upstream_compose.yml start
echo "Starting Polkadex Node and TEE Gateway"
docker-compose up --build
